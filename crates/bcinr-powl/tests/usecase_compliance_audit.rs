//! Compliance & Auditability
//!
//! Demonstrates how POWL's BLAKE3 receipt chaining and OCEL logging
//! provide immutable audit trails for regulatory systems.
//!
//! ## The Problem
//!
//! Healthcare, finance, and government systems need to prove every decision
//! was made correctly for audits. Conventional systems log decisions but
//! auditors have no cryptographic proof that logs weren't tampered with.
//!
//! ## The Solution
//!
//! POWL provides:
//! - BLAKE3 receipt chains: tamper-evident hash of each step
//! - Conformance checking against the compiled precedence graph
//! - Replay determinism: same inputs -> identical receipt (no randomness)

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::{ConformanceResult, OcelLog};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

fn execute(ast: &PowlAstNode<'_>, run_id: u64) -> (PowlTape, PowlRunState, OcelLog) {
    let tape = compile_powl(ast).expect("POWL model must compile");
    let mut state = PowlRunState::new(&tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            log.record_op_fired(run_id, op_idx, 0, 0).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    log.record_run_sealed(run_id, op_trace, 0).unwrap();
    (tape, state, log)
}

/// Test 1: BLAKE3 receipt chain is tamper-evident
///
/// Run the same regulatory decision workflow twice from independently
/// constructed logs — once faithfully, once with a real tampering attempt
/// (an injected event with a different op_idx spliced into the sequence) —
/// and prove the receipt digests diverge.
#[test]
fn test_blake3_receipt_chain_tamper_evident() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("check_kyc"),
        PowlAstNode::Atom("approve_transaction"),
    ]);
    let run_id = 1u64;

    let (_tape, _state, log_honest) = execute(&ast, run_id);
    let receipt_honest = log_honest.seal_receipt();
    let digest_honest = receipt_honest.digest();

    // Tampering attempt: fabricate a log claiming a third, unrecorded op
    // fired (a forged approval step that never actually ran through the
    // scheduler).
    let mut log_tampered = OcelLog::new();
    log_tampered.record_op_fired(run_id, 0, 0, 0).unwrap();
    log_tampered.record_op_fired(run_id, 1, 0, 0).unwrap();
    log_tampered.record_op_fired(run_id, 99, 0, 0).unwrap(); // forged op
    log_tampered.record_run_sealed(run_id, 0b111, 0).unwrap();

    let digest_tampered = log_tampered.seal_receipt().digest();

    assert_ne!(
        digest_honest, digest_tampered,
        "forged event must change the receipt digest"
    );
}

/// Test 2: Replay determinism — same workflow, same run_id -> identical receipt
///
/// Regulatory auditors replay a decision through the real compiler and
/// scheduler and must get back the exact same BLAKE3 digest.
#[test]
fn test_replay_determinism_same_input_same_receipt() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("verify_identity"),
        PowlAstNode::Atom("apply_sanctions_check"),
        PowlAstNode::Atom("release_funds"),
    ]);
    let run_id = 42u64;

    let (_tape1, _state1, log1) = execute(&ast, run_id);
    let (_tape2, _state2, log2) = execute(&ast, run_id);

    let digest1 = log1.seal_receipt().digest();
    let digest2 = log2.seal_receipt().digest();

    assert_eq!(
        digest1, digest2,
        "replaying an identical workflow must produce an identical receipt"
    );
    assert_eq!(
        log1.events(),
        log2.events(),
        "replay must reproduce the exact same event trace"
    );
}

/// Test 3: Log conforms to the compiled precedence graph
///
/// A regulatory audit trail is worthless if it can diverge from the actual
/// program that generated it. We verify the recorded trace conforms to the
/// compiled tape structurally, using the real conformance checker
/// (`validate_against_tape`), not a placeholder.
#[test]
fn test_ocel_export_independent_validation() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("intake"),
        PowlAstNode::Atom("review"),
    ]);
    let (tape, _state, log) = execute(&ast, 1);

    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Conforms,
        "recorded audit trail must conform to the compiled decision graph"
    );
}

/// Test 4: Audit trail — every decision recorded, none dropped
///
/// For a 3-step regulatory decision, the log must contain exactly 3
/// op-fired events plus a sealed-run event.
#[test]
fn test_audit_trail_every_decision_recorded() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("allocate_candidate_0"),
        PowlAstNode::Atom("allocate_candidate_1"),
        PowlAstNode::Atom("allocate_candidate_2"),
    ]);
    let (_tape, _state, log) = execute(&ast, 7);

    let receipt = log.seal_receipt();
    // 3 op-fired events + 1 run-sealed event.
    assert_eq!(
        receipt.event_count(),
        4,
        "every decision plus the seal must be recorded"
    );
}

/// Test 5: A log missing a required decision fails conformance
///
/// This is the negative case: if a decision that the compiled workflow
/// requires is missing from the trace, `validate_against_tape` must reject
/// it — proving the checker actually enforces completeness rather than
/// trivially accepting any input.
#[test]
fn test_precondition_recording_for_audit() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("check_budget"),
        PowlAstNode::Atom("commit_allocation"),
    ]);
    let tape = compile_powl(&ast).expect("must compile");

    // Fabricate an incomplete log: only the second op is recorded, its
    // predecessor (check_budget) never fired. This must NOT conform.
    let mut incomplete_log = OcelLog::new();
    incomplete_log.record_op_fired(1, 1, 0, 0).unwrap();
    incomplete_log.record_run_sealed(1, 0b10, 0).unwrap();

    assert_ne!(
        incomplete_log.validate_against_tape(&tape),
        ConformanceResult::Conforms,
        "a trace missing a required predecessor must fail conformance"
    );
}
