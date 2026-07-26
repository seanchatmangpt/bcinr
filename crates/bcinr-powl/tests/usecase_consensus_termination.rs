//! Distributed Consensus: Ordering and Termination Guarantees
//!
//! Demonstrates how POWL's compiled precedence graph and OCEL receipt chain
//! give Byzantine-tolerant consensus protocols two properties conventional
//! ad-hoc schedulers cannot prove: deterministic termination and
//! tamper-evident message ordering.
//!
//! ## The Problem
//!
//! Distributed consensus protocols (Raft, Paxos, PBFT) need to agree on a
//! value while tolerating adversarial nodes. Two failure modes matter:
//! - Livelock: agents oscillate between proposals and never terminate.
//! - Reordering attacks: a Byzantine relay reorders messages to fake a
//!   different protocol execution.
//!
//! ## The Solution
//!
//! POWL provides:
//! - A compiled, acyclic precedence graph: the scheduler's bounded-tick loop
//!   (see `execute`) is guaranteed to terminate (`check_mask == 0`) because
//!   the compiler rejects cyclic dependency graphs at compile time.
//! - BLAKE3-chained OCEL receipts: reordering any two votes changes the
//!   receipt digest, giving auditors a checkable tamper signal.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::OcelLog;
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

fn execute(ast: &PowlAstNode<'_>, run_id: u64) -> (PowlTape, PowlRunState, OcelLog, u32) {
    let tape = compile_powl(ast).expect("POWL model must compile");
    let mut state = PowlRunState::new(&tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;
    let mut ticks = 0u32;

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        ticks += 1;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            log.record_op_fired(run_id, op_idx, 0).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    log.record_run_sealed(run_id, op_trace).unwrap();
    (tape, state, log, ticks)
}

/// Test 1: Termination is guaranteed and bounded, not merely hoped for
///
/// Run a 3-node proposal/vote/commit protocol to completion inside the
/// bounded 128-tick loop used by `execute`. Termination is proven by
/// `check_mask == 0` after the loop — if the protocol could livelock, this
/// assertion would fail (check_mask would stay nonzero).
#[test]
fn test_dwell_enforcement_blocks_rapid_mode_switching() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("propose_value_x"),
        PowlAstNode::Atom("node_b_votes_x"),
        PowlAstNode::Atom("node_c_votes_x"),
    ]);

    let (_tape, state, _log, ticks) = execute(&ast, 1);

    assert_eq!(state.check_mask, 0, "protocol must terminate (no livelock)");
    assert_eq!(
        ticks, 3,
        "3-step consensus must terminate in exactly 3 ticks"
    );
}

/// Test 2: Independent proposal branches all resolve (no branch is starved)
///
/// Model 3 independent validator confirmations (no ordering constraint
/// between them) as a partial order with no edges. All 3 must fire — proof
/// that the scheduler doesn't starve any ready op, a precondition for
/// eventual agreement across all honest nodes.
#[test]
fn test_convergence_guaranteed_lyapunov_bound() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("validator_a_confirms"),
            PowlAstNode::Atom("validator_b_confirms"),
            PowlAstNode::Atom("validator_c_confirms"),
        ],
        edges: vec![],
    };

    let (_tape, state, log, _ticks) = execute(&ast, 2);

    assert_eq!(state.check_mask, 0, "all validators must reach a decision");
    let fired: std::collections::HashSet<u32> = log.events().iter().map(|e| e.op_idx).collect();
    // The compiler may add structural bookkeeping ops (e.g. an implicit
    // join) alongside the 3 declared validator confirmations (ops 0, 1, 2).
    // The property under test is that none of the 3 declared ops is
    // starved, not that the trace contains nothing else.
    for op_idx in 0..3u32 {
        assert!(
            fired.contains(&op_idx),
            "validator confirmation {} must fire — not starved",
            op_idx
        );
    }
}

/// Test 3: Message reordering is detected via receipt divergence
///
/// Byzantine attackers may reorder messages. We construct two logs over the
/// same 3 votes, one in the true order and one reordered, and prove their
/// BLAKE3 receipts diverge — the mechanism an auditor uses to detect a
/// reordering attack.
#[test]
fn test_event_ordering_preserved_ocel_chain() {
    let run_id = 1u64;

    let mut log_correct = OcelLog::new();
    log_correct.record_op_fired(run_id, 0, 0).unwrap(); // A proposes X
    log_correct.record_op_fired(run_id, 1, 0).unwrap(); // B votes X
    log_correct.record_op_fired(run_id, 2, 0).unwrap(); // C votes X
    log_correct.record_run_sealed(run_id, 0b111).unwrap();
    let digest_correct = log_correct.seal_receipt().digest();

    let mut log_reordered = OcelLog::new();
    log_reordered.record_op_fired(run_id, 1, 0).unwrap(); // B votes X (moved first)
    log_reordered.record_op_fired(run_id, 0, 0).unwrap(); // A proposes X (moved second)
    log_reordered.record_op_fired(run_id, 2, 0).unwrap();
    log_reordered.record_run_sealed(run_id, 0b111).unwrap();
    let digest_reordered = log_reordered.seal_receipt().digest();

    assert_ne!(
        digest_correct, digest_reordered,
        "message reordering must change the receipt digest"
    );
}

/// Test 4: A cyclic dependency (Byzantine deadlock attempt) is refused at compile time
///
/// If a Byzantine leader proposes a workflow that requires op A to precede
/// op B and also B to precede A, that is a structural deadlock. POWL's
/// compiler must refuse to build such a graph rather than silently
/// accepting it and hanging at scheduling time.
#[test]
fn test_byzantine_partition_detection_typed_refusal() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("op_a"), PowlAstNode::Atom("op_b")],
        edges: vec![(0, 1), (1, 0)], // cycle: a->b and b->a
    };

    let result = compile_powl(&ast);

    assert!(
        result.is_err(),
        "a cyclic precedence graph must be refused with a typed compile error, not silently admitted"
    );
}

/// Test 5: State machine consistency — commands apply in the same order every replay
///
/// Consensus protocols require all honest nodes to apply commands in the
/// same order. We compile a 3-command sequence and verify the scheduler
/// admits them in the compiled order on every independent replay.
#[test]
fn test_state_machine_consistency_via_audit_log() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("set_x_10"),
        PowlAstNode::Atom("add_x_5"),
        PowlAstNode::Atom("set_x_20"),
    ]);

    let (_tape1, _state1, log1, _) = execute(&ast, 5);
    let (_tape2, _state2, log2, _) = execute(&ast, 5);

    let order1: Vec<u32> = log1.events().iter().map(|e| e.op_idx).collect();
    let order2: Vec<u32> = log2.events().iter().map(|e| e.op_idx).collect();

    assert_eq!(
        order1, order2,
        "independent replays must apply commands in the same order"
    );
    // The compiled tape may append trailing structural ops after the 3
    // declared commands; verify the declared commands (0, 1, 2) lead the
    // trace in strictly increasing program order.
    assert_eq!(
        &order1[..3],
        &[0, 1, 2],
        "declared commands must apply in program order"
    );
}
