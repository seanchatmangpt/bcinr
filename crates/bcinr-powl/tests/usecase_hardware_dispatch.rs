//! Hardware Instruction Dispatch Scheduler
//!
//! Demonstrates how POWL's compiled partial-order tape gives CPU/GPU
//! dispatch formal guarantees on hazard-freedom and parallelism that
//! ad-hoc heuristic schedulers cannot prove.
//!
//! ## The Problem
//!
//! CPU and GPU schedulers must:
//! - Respect data dependencies (no read-before-write, no write-after-read)
//! - Avoid write-after-write hazards
//! - Maximize parallelism (execute independent instructions concurrently)
//!
//! Conventional schedulers use heuristics without formal guarantees.
//!
//! ## The Solution
//!
//! POWL's `compile_powl` builds an explicit precedence graph (pred_mask /
//! succ_mask per op) from declared dependency edges. We inspect that
//! compiled graph directly and run the real scheduler to verify hazard
//! freedom and parallel admission, not merely assert `true`.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};

/// Test 1: Data-dependency precedence prevents read-before-write /
/// write-after-read hazards
///
/// Load -> Compute -> Store, with compute depending on load and store
/// depending on compute. Verify the compiled tape encodes both edges in
/// pred_mask.
#[test]
fn test_data_dependency_precedence_prevents_war_hazards() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("load_r0"),
            PowlAstNode::Atom("compute_r1_add_r0"),
            PowlAstNode::Atom("store_r1"),
        ],
        edges: vec![(0, 1), (1, 2)],
    };

    let tape = compile_powl(&ast).expect("must compile");
    let ops = &tape.ops[..tape.len as usize];

    assert_eq!(ops[0].pred_mask, 0, "load has no predecessor");
    assert_eq!(
        ops[1].pred_mask & (1u64 << 0),
        1u64 << 0,
        "compute must depend on load (read-before-write)"
    );
    assert_eq!(
        ops[2].pred_mask & (1u64 << 1),
        1u64 << 1,
        "store must depend on compute (write-after-read)"
    );
}

/// Test 2: Forward progress guarantee — all independent ops eventually execute
///
/// 3 fully independent instructions must all fire when run through the real
/// scheduler loop, proving no op is left behind.
#[test]
fn test_dispatch_forward_progress_guaranteed() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("add_r1_r2"),
            PowlAstNode::Atom("mul_r3_r4"),
            PowlAstNode::Atom("load_r5_mem"),
        ],
        edges: vec![],
    };

    let tape = compile_powl(&ast).expect("must compile");
    let mut state = PowlRunState::new(&tape);
    let mut fired = std::collections::HashSet::new();

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            fired.insert(op_idx);
        }
    }

    assert_eq!(state.check_mask, 0, "scheduler must terminate");
    for op_idx in 0..3u32 {
        assert!(
            fired.contains(&op_idx),
            "instruction {} must eventually execute",
            op_idx
        );
    }
}

/// Test 3: Independent instructions are admitted in a single tick (parallel dispatch)
///
/// If op0 and op1 have no dependency between them, they must both be
/// members of the scheduler's ready set (check_mask) simultaneously at
/// tick 0 — proof the scheduler doesn't serialize independent work.
#[test]
fn test_concurrent_independent_instructions_parallelized() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("add_r1_r2"),
            PowlAstNode::Atom("mul_r3_r4"),
        ],
        edges: vec![],
    };

    let tape = compile_powl(&ast).expect("must compile");
    let state = PowlRunState::new(&tape);

    // Both op 0 and op 1 must be ready before any tick runs.
    assert_eq!(
        state.check_mask & 0b11,
        0b11,
        "independent instructions must be simultaneously ready for dispatch"
    );
}

/// Test 4: No false dependencies are introduced by the compiler
///
/// Given op0 -> op1 and an independent op2, op2's pred_mask must be empty:
/// the compiler must not serialize op2 behind op0/op1 just because they
/// appear earlier in source order.
#[test]
fn test_no_false_dependencies_from_scheduler() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("op0"),
            PowlAstNode::Atom("op1"),
            PowlAstNode::Atom("op2"),
        ],
        edges: vec![(0, 1)],
    };

    let tape = compile_powl(&ast).expect("must compile");
    let ops = &tape.ops[..tape.len as usize];

    assert_eq!(
        ops[2].pred_mask, 0,
        "op2 must have no false dependency on op0 or op1"
    );
}

/// Test 5: Write-after-write hazards are encoded as precedence, not silently allowed
///
/// Two writes to the same register must be ordered: mov r0,5 before mov
/// r0,10. A Sequence composition (not an unordered PartialOrder) is the
/// correct construct — verify it produces a strict precedence edge.
#[test]
fn test_write_after_write_hazard_prevented() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("mov_r0_5"),
        PowlAstNode::Atom("mov_r0_10"),
    ]);

    let tape = compile_powl(&ast).expect("must compile");
    let ops = &tape.ops[..tape.len as usize];

    assert_eq!(
        ops[1].pred_mask & (1u64 << 0),
        1u64 << 0,
        "second write must be ordered after the first (WAW hazard prevented)"
    );
}

/// Test 6: Full schedule verification — mixed dependent + independent instructions
///
/// Load -> Compute -> Store, running in parallel with an independent op.
/// Verify: (a) the dependent chain preserves precedence, (b) the
/// independent op is ready immediately (parallel), (c) the whole schedule
/// terminates with all 4 ops fired.
#[test]
fn test_dispatch_schedule_formally_verified() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("load"),
            PowlAstNode::Atom("compute"),
            PowlAstNode::Atom("store"),
            PowlAstNode::Atom("independent"),
        ],
        edges: vec![(0, 1), (1, 2)],
    };

    let tape = compile_powl(&ast).expect("must compile");
    let ops = &tape.ops[..tape.len as usize];

    // Dependent chain precedence intact.
    assert_eq!(ops[1].pred_mask & (1u64 << 0), 1u64 << 0);
    assert_eq!(ops[2].pred_mask & (1u64 << 1), 1u64 << 1);

    // independent (op 3) has no predecessor and is ready at tick 0
    // alongside load (op 0).
    let mut state = PowlRunState::new(&tape);
    assert_eq!(ops[3].pred_mask, 0, "independent op has no dependency");
    assert_eq!(
        state.check_mask & ((1u64 << 0) | (1u64 << 3)),
        (1u64 << 0) | (1u64 << 3),
        "load and independent must both be ready in parallel at tick 0"
    );

    // Full schedule terminates with all 4 declared ops fired.
    let mut fired = std::collections::HashSet::new();
    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(ops, &mut state).0;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            fired.insert(op_idx);
        }
    }
    assert_eq!(
        state.check_mask, 0,
        "schedule must terminate (deadlock-free)"
    );
    for op_idx in 0..4u32 {
        assert!(fired.contains(&op_idx), "op {} must fire", op_idx);
    }
}
