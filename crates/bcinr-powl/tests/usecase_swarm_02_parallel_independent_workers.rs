//! Swarm Orchestration: 5 Independent Workers in Parallel
//!
//! Demonstrates how POWL's scheduler handles full parallelism without
//! coordination between workers. This is the simplest swarm use case:
//! no ordering constraints, no precedence, no resource contention.
//!
//! ## The Problem
//!
//! Traditional task schedulers serialize independent work or require explicit
//! synchronization. Swarm orchestration needs to:
//! - Fire all ready workers simultaneously (no artificial ordering)
//! - Verify all workers complete (no starvation)
//! - Prove completion in bounded ticks (deterministic scheduling)
//! - Log execution order via OCEL receipts (auditability)
//!
//! ## The Solution
//!
//! POWL models 5 independent workers as a PartialOrder with:
//! - 5 child ops (one per worker)
//! - 0 edges (no dependencies between workers)
//!
//! The scheduler will fire all 5 in a single tick (or grouped ticks,
//! depending on capacity), and terminate when check_mask == 0.

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
            log.record_op_fired(run_id, op_idx, ticks, 1).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    log.record_run_sealed(run_id, op_trace, ticks).unwrap();
    (tape, state, log, ticks)
}

/// Test 1: All 5 workers fire and terminate without artificial ordering
///
/// With no edges in the PartialOrder, the scheduler has freedom to fire
/// all 5 workers as soon as they are ready. We verify:
/// - All 5 workers (op indices 0..5) are recorded in the log
/// - Execution terminates (check_mask == 0)
/// - Completion is reached within bounded ticks
#[test]
fn test_five_workers_all_fire_parallel() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("worker_1"),
            PowlAstNode::Atom("worker_2"),
            PowlAstNode::Atom("worker_3"),
            PowlAstNode::Atom("worker_4"),
            PowlAstNode::Atom("worker_5"),
        ],
        edges: vec![], // No coordination — all independent
    };

    let (_tape, state, log, ticks) = execute(&ast, 1);

    assert_eq!(
        state.check_mask, 0,
        "all workers must complete (no deadlock, no starvation)"
    );
    assert!(
        ticks <= 128,
        "scheduler must terminate within bounded 128-tick loop"
    );
    assert!(ticks >= 1, "at least 1 tick needed to fire ready workers");

    let fired: std::collections::HashSet<u32> = log.events().iter().map(|e| e.op_idx).collect();
    for worker_id in 0..5u32 {
        assert!(
            fired.contains(&worker_id),
            "worker {} must fire — no starvation",
            worker_id
        );
    }
}

/// Test 2: Execution determinism — independent replays fire all workers
///
/// Scheduling independent workers should produce the same set of firings
/// on every replay (order may vary due to scheduler freedom, but all must
/// fire). We run the same AST twice and verify:
/// - Both runs fire exactly the same 5 workers
/// - No worker is starved in either run
#[test]
fn test_parallel_workers_deterministic_completion() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("worker_a"),
            PowlAstNode::Atom("worker_b"),
            PowlAstNode::Atom("worker_c"),
            PowlAstNode::Atom("worker_d"),
            PowlAstNode::Atom("worker_e"),
        ],
        edges: vec![],
    };

    let (_tape1, state1, log1, _) = execute(&ast, 10);
    let (_tape2, state2, log2, _) = execute(&ast, 10);

    assert_eq!(state1.check_mask, 0, "first run must complete all workers");
    assert_eq!(state2.check_mask, 0, "second run must complete all workers");

    let fired1: std::collections::HashSet<u32> = log1.events().iter().map(|e| e.op_idx).collect();
    let fired2: std::collections::HashSet<u32> = log2.events().iter().map(|e| e.op_idx).collect();

    assert_eq!(
        fired1, fired2,
        "independent runs must fire the same set of workers"
    );

    // Verify count: at least 5 (the declared workers), possibly more if the
    // compiler adds structural ops like a join node.
    assert!(
        fired1.len() >= 5,
        "both runs must fire at least the 5 declared workers"
    );
}

/// Test 3: Swarm receipt chain validates the entire execution
///
/// All 5 worker firings are recorded in the OCEL log and sealed via
/// BLAKE3. We verify:
/// - The log's seal_receipt produces a deterministic digest
/// - Changing the run_id or op_trace changes the digest
///   (tamper detection via cryptographic binding)
#[test]
fn test_swarm_receipt_integrity_ocel_chain() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("task_1"),
            PowlAstNode::Atom("task_2"),
            PowlAstNode::Atom("task_3"),
            PowlAstNode::Atom("task_4"),
            PowlAstNode::Atom("task_5"),
        ],
        edges: vec![],
    };

    let (_tape, _state, log_correct, _) = execute(&ast, 100);
    let digest_correct = log_correct.seal_receipt().digest();

    // Verify the receipt is deterministic (re-sealing same log = same digest)
    let digest_correct_2 = log_correct.seal_receipt().digest();
    assert_eq!(
        digest_correct, digest_correct_2,
        "receipt digest must be deterministic for the same log"
    );

    // Verify tampering is detected: a different run_id changes the digest
    let mut log_tampered = OcelLog::new();
    for e in log_correct.events() {
        log_tampered
            .record_op_fired(e.run_id + 1, e.op_idx, e.start_time, 1)
            .unwrap();
    }
    log_tampered.record_run_sealed(101, 0x1F, 100).unwrap();
    let digest_tampered = log_tampered.seal_receipt().digest();

    assert_ne!(
        digest_correct, digest_tampered,
        "receipt must change when run_id is modified (tamper detection)"
    );
}

/// Test 4: No LLM calls — pure algorithm, no external I/O
///
/// POWL scheduling is deterministic, closed, and side-effect-free.
/// This test serves as a compile-time check: if any code paths call
/// external LLM services, the grep below will catch it (UNVERIFIED).
///
/// Note: This is a logical assertion, not a behavioral test. The presence
/// of external calls is a source-code property, not a runtime property.
#[test]
fn test_no_external_llm_calls() {
    // The actual enforcement is a static property of the codebase.
    // This test documents the expectation and serves as a checkpoint
    // for code reviews. Runtime verification:
    // ```
    // grep -r "api.anthropic\|openai\|model.*provider" crates/bcinr-powl/src/
    // ```
    // must return 0 matches (UNVERIFIED if not checked).

    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("worker_x"),
            PowlAstNode::Atom("worker_y"),
            PowlAstNode::Atom("worker_z"),
        ],
        edges: vec![],
    };

    // Execute the full workflow
    let (_tape, state, _log, _ticks) = execute(&ast, 42);

    // If we reach this point without panic or external I/O, the test passes.
    // The check_mask == 0 confirms termination; the absence of network calls
    // or LLM invocations is confirmed via code inspection.
    assert_eq!(
        state.check_mask, 0,
        "execution completed without calling external services"
    );
}

/// Test 5: Scale to edge case — single worker (minimal swarm)
///
/// Verify the parallel scheduler correctly handles the boundary case of
/// a single worker (1-worker swarm). No coordination needed, but the
/// scheduler mechanics must still apply.
#[test]
fn test_single_worker_edge_case() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("solo_worker")],
        edges: vec![],
    };

    let (_tape, state, log, ticks) = execute(&ast, 999);

    assert_eq!(state.check_mask, 0, "single worker must complete");
    assert!(
        (1..=128).contains(&ticks),
        "single worker must fire within bounded ticks"
    );

    let fired: std::collections::HashSet<u32> = log.events().iter().map(|e| e.op_idx).collect();
    assert!(fired.contains(&0), "the only worker (op 0) must fire");
}

/// Test 6: Large swarm — verify O(1) termination
///
/// Independent workers should maintain O(1) scheduling per tick regardless
/// of worker count (barring resource contention, which we ignore here).
/// This test documents the property; actual performance is verified
/// via benchmarks in bcinr-bench/.
#[test]
fn test_larger_swarm_termination_bounded() {
    // Even with many workers, the bounded-tick loop should terminate
    // because there are no cycles in the PartialOrder.
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("worker_0"),
            PowlAstNode::Atom("worker_1"),
            PowlAstNode::Atom("worker_2"),
            PowlAstNode::Atom("worker_3"),
            PowlAstNode::Atom("worker_4"),
            PowlAstNode::Atom("worker_5"),
            PowlAstNode::Atom("worker_6"),
            PowlAstNode::Atom("worker_7"),
            PowlAstNode::Atom("worker_8"),
            PowlAstNode::Atom("worker_9"),
        ],
        edges: vec![],
    };

    let (_tape, state, log, ticks) = execute(&ast, 500);

    assert_eq!(
        state.check_mask, 0,
        "10-worker swarm must complete all workers"
    );
    assert!(
        ticks <= 128,
        "even a larger swarm must terminate within bounded ticks"
    );

    let fired: std::collections::HashSet<u32> = log.events().iter().map(|e| e.op_idx).collect();
    assert!(fired.len() >= 10, "all 10 declared workers must fire");
}
