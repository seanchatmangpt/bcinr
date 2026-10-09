//! Typed-refusal coverage at the public API boundary (constitution §18).
//!
//! Every refused authoritative operation must be asserted against its EXACT
//! typed error variant; `is_err()` alone is prohibited (`assert_ne!`-only
//! mutant detection is CHEAT-009 mutant theater). Integration-level (tests/)
//! exact-variant assertions were audited MISSING for every variant exercised
//! below except `CompileError::EmptySequence` (already exact in
//! `usecase_realtime_safety.rs:221`).
//!
//! # Dead variants (audit finding, not testable)
//!
//! Grep over `src/` shows these variants have NO construction site anywhere,
//! so no public-API path can reach them and no exact-refusal test can exist:
//!
//! - `ValidationError::CyclicDependency` (declared src/typestate.rs:264)
//! - `ValidationError::InvalidPredecessorIndex` (declared src/typestate.rs:266)
//! - `ExecutionDefect::TokenMismatch` (declared src/typestate.rs:309;
//!   `PowlRunner::complete` binds no run identity into the token)
//!
//! # Non-vacuity method
//!
//! Each test carries a `// non-vacuity:` line naming the exact guarded source
//! path (file:line) and the plausible mutation that would flip the assertion
//! to failure. A passing test is only evidence together with that argument
//! (anti-vacuity law: a gate that never refuses is vacuous).

// The caller of a const-generic `TopologyKind` API mirrors the defining
// crate's typestate features; the current toolchain no longer requires an
// explicit `#![feature(adt_const_params)]` at the call site — re-add only if
// a toolchain pin demands it.

use bcinr_powl::compiler::{
    check_all_ops_reachable, check_full_graph_acyclic, compile_powl, CompileError, PowlAstNode,
};
use bcinr_powl::tape::{OpKind, PowlTape};
use bcinr_powl::typestate::{
    ExecutionDefect, HasPowlTape, PowlRunner, TopologyKind, ValidationError,
};

// ---------------------------------------------------------------------------
// CompileError — via the public compiler entry point
// ---------------------------------------------------------------------------

/// `CompileError::TapeFull`: a 65-atom sequence exceeds the 64-slot tape.
///
/// This variant had NO assertion anywhere (src-unit or tests/) before this
/// test.
///
/// non-vacuity: guards src/compiler.rs:190 (`tape.alloc(..).ok_or(CompileError::TapeFull)`)
/// reached at capacity; if `PowlTape::alloc`'s capacity check (src/tape.rs
/// `if self.len >= 64`) were relaxed by one (`> 64`), slot 65 writes out of
/// bounds and this test fails (panic); if the error were remapped to any
/// other variant, the `Err(TapeFull)` equality fails.
#[test]
fn tape_full_refused_when_sequence_exceeds_64_slots() {
    let labels: &'static [String] = Box::leak(
        (0..65)
            .map(|i| format!("a{i}"))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let atoms: Vec<PowlAstNode> = (0..65)
        .map(|i| PowlAstNode::Atom(labels[i].as_str()))
        .collect();
    let ast = PowlAstNode::Sequence(atoms);
    assert_eq!(
        compile_powl(&ast),
        Err(CompileError::TapeFull),
        "65 atoms must be refused as TapeFull at the 64-slot capacity boundary"
    );
}

/// `CompileError::EmptyChoice`: an XOR choice with no branches.
///
/// non-vacuity: guards src/compiler.rs:288-289 (`if branches.is_empty()`);
/// dropping that guard falls through to allocating dispatch/join ops and
/// returns `Ok`, so the `Err(EmptyChoice)` equality fails.
#[test]
fn empty_choice_refused() {
    let ast = PowlAstNode::XorChoice(vec![]);
    assert_eq!(
        compile_powl(&ast),
        Err(CompileError::EmptyChoice),
        "an XOR choice with zero branches must be refused as EmptyChoice"
    );
}

/// `CompileError::EmptyPartialOrder` at the public API.
///
/// non-vacuity: guards src/compiler.rs:231-232 (`if children.is_empty()`);
/// dropping the guard wires no edges and returns `Ok`, failing the equality.
/// src/compiler.rs:1405 (crate-internal unit test) asserts the same variant;
/// this is the integration-level binding of the public surface.
#[test]
fn empty_partial_order_refused_at_public_api() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![],
        edges: vec![],
    };
    assert_eq!(
        compile_powl(&ast),
        Err(CompileError::EmptyPartialOrder),
        "a partial order with zero children must be refused as EmptyPartialOrder"
    );
}

/// `CompileError::InvalidEdge` carries the exact offending indices.
///
/// This variant had NO exact-field assertion anywhere before this test.
///
/// non-vacuity: guards src/compiler.rs:241-248; the equality asserts all
/// three fields (`from: 0, to: 5, len: 2`), so a swap of `from`/`to`, a
/// `>=` -> `>` off-by-one in the bound check, or a dropped bound check all
/// fail the equality.
#[test]
fn invalid_edge_refused_with_exact_indices() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
        edges: vec![(0, 5)],
    };
    assert_eq!(
        compile_powl(&ast),
        Err(CompileError::InvalidEdge {
            from: 0,
            to: 5,
            len: 2
        }),
        "edge (0,5) over 2 children must be refused with the exact offending indices"
    );
}

/// `CompileError::Cycle` through the public compile path — the exact typed
/// refusal behind `usecase_swarm_19_reordered_evidence.rs::test_cyclic_causal_dependencies_rejected`,
/// which asserts only `is_err()`.
///
/// non-vacuity: guards src/compiler.rs:627 (`check_full_graph_acyclic`) ->
/// src/compiler.rs:444 (`Err(CompileError::Cycle)` from the Kahn walk);
/// a mutation counting the cyclic pair as visited or skipping one back-edge
/// direction returns `Ok` and the equality fails.
#[test]
fn cycle_refused_through_public_compile() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("event_a"), PowlAstNode::Atom("event_b")],
        edges: vec![(0, 1), (1, 0)],
    };
    assert_eq!(
        compile_powl(&ast),
        Err(CompileError::Cycle),
        "mutual dependency A->B->A must be refused as Cycle, not just non-Ok"
    );
}

/// Narrow gate: `check_full_graph_acyclic` on a hand-built cyclic tape.
///
/// non-vacuity: guards src/compiler.rs:482-486 -> 419-444 (Kahn walk over
/// non-LoopRedo slots). The fixture wires op0->op1->op0 exactly as Kahn's
/// in-degrees see them; a mutation dropping one direction while building
/// `in_deg` makes `visited == n` and the `Err(Cycle)` equality fails.
#[test]
fn narrow_acyclic_gate_refuses_hand_built_cycle() {
    let mut tape = PowlTape::new();
    let a = tape.alloc(OpKind::Atom).expect("tape not full");
    let b = tape.alloc(OpKind::Atom).expect("tape not full");
    tape.ops[a as usize].succ_mask = 1 << b;
    tape.ops[b as usize].pred_mask = 1 << a;
    tape.ops[b as usize].succ_mask = 1 << a;
    tape.ops[a as usize].pred_mask = 1 << b;
    assert_eq!(
        check_full_graph_acyclic(&tape),
        Err(CompileError::Cycle),
        "the narrow acyclicity gate must refuse a two-op mutual cycle"
    );
}

/// `CompileError::Unreachable`: an op declared reachable only through a
/// predecessor that no entry reaches (via the public narrow checker).
///
/// non-vacuity: guards src/compiler.rs:590-593 (violation =
/// `must_be_reachable & !reachable_from_entry`) surfaced at 617-624. The
/// fixture gives op0 a `pred_mask` naming op1 while op1's `succ_mask` omits
/// op0, so forward reachability from the entry (op1) never covers op0; a
/// mutation replacing `!reachable_from_entry` with `reachable_from_entry`
/// yields violation == 0 -> `Ok`, failing the equality.
#[test]
fn unreachable_refused_for_orphaned_node() {
    let mut tape = PowlTape::new();
    let orphan = tape.alloc(OpKind::Atom).expect("tape not full");
    let entry = tape.alloc(OpKind::Atom).expect("tape not full");
    // orphan declares op1 as its predecessor; op1 declares no successors.
    tape.ops[orphan as usize].pred_mask = 1 << entry;
    tape.ops[entry as usize].pred_mask = 0;
    tape.ops[entry as usize].succ_mask = 0;
    tape.entry_mask = 1 << entry;
    assert_eq!(
        check_all_ops_reachable(&tape),
        Err(CompileError::Unreachable),
        "a node whose only declared predecessor edge does not forward-reach it must be Unreachable"
    );
}

/// `CompileError::XorInsideLoop` with the EXACT slot fields (src/compiler.rs:1783
/// asserts only `matches! { .. }`; this asserts both fields).
///
/// Fixture: `Sequence[Atom, Loop{ body: Sequence[Atom, XorChoice[Atom]], redo: Atom }]`
/// compiles to slots: a=0, b=1, dispatch=2, join=3, x=4, r=5 -> the scan at
/// src/compiler.rs:335-347 finds the XorDispatch at slot 2 and the body entry
/// (bit 1) at slot 1.
///
/// non-vacuity: guards src/compiler.rs:340-346; an equality mutation that
/// swaps `xor_slot`/`loop_body_entry`, reports the redo segment's entry
/// instead of `body_seg.entries`, or drops the `.skip(pre_len)` scan bound
/// changes the reported fields and fails the equality.
#[test]
fn xor_inside_loop_refused_with_exact_slots() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Sequence(vec![
                PowlAstNode::Atom("b"),
                PowlAstNode::XorChoice(vec![PowlAstNode::Atom("x")]),
            ])),
            redo: Box::new(PowlAstNode::Atom("r")),
            max_iters: 3,
        },
    ]);
    assert_eq!(
        compile_powl(&ast),
        Err(CompileError::XorInsideLoop {
            xor_slot: 2,
            loop_body_entry: 1,
        }),
        "XorDispatch at slot 2 inside the loop must be refused naming slot 2 and body entry 1"
    );
}

// ---------------------------------------------------------------------------
// ValidationError — via the public PowlRunner::validate gate
// ---------------------------------------------------------------------------

/// A test tape with a caller-chosen size, for the `TapeTooLarge` bound that
/// the real 64-slot tape type cannot express.
struct OversizeTape;

impl HasPowlTape for OversizeTape {
    fn op_count(&self) -> usize {
        65
    }
    fn entry_mask(&self) -> u64 {
        1
    }
}

/// `ValidationError::TapeTooLarge { len: 65 }`.
///
/// This variant had NO assertion anywhere (src-unit or tests/) before this
/// test: the real `tape::PowlTape` cannot exceed 64 ops, so only a custom
/// `HasPowlTape` can reach the bound.
///
/// non-vacuity: guards src/typestate.rs:687-688 (`if n > 64`); an off-by-one
/// (`n > 65`) falls through to the entry check, returns `Ok`, and the
/// `unwrap_err` panics; a dropped `len` field binding fails the equality.
#[test]
fn oversize_tape_refused_with_exact_len() {
    let err = PowlRunner::new(OversizeTape).validate().unwrap_err();
    assert_eq!(
        err,
        ValidationError::TapeTooLarge { len: 65 },
        "a 65-op tape must be refused as TapeTooLarge naming len 65"
    );
}

/// `ValidationError::EmptyTape` at the public API (integration binding;
/// crate-internal unit test src/typestate.rs:1058 and the doctest cover the
/// crate-local path).
///
/// non-vacuity: guards src/typestate.rs:684-685 (`if n == 0`); removing the
/// check proceeds to the entry-mask check and returns `NoEntryOp`, failing
/// the equality (the gate still refuses, but with the wrong law — §18
/// requires the exact typed refusal).
#[test]
fn empty_tape_refused() {
    let err = PowlRunner::new(PowlTape::new()).validate().unwrap_err();
    assert_eq!(
        err,
        ValidationError::EmptyTape,
        "empty tape must be refused as EmptyTape"
    );
}

/// `ValidationError::NoEntryOp` (integration binding; src/typestate.rs:1076
/// covers the crate-local path).
///
/// non-vacuity: guards src/typestate.rs:690-691 (`entry_mask() == 0`); a
/// mutation inverting the comparison (`!= 0`) admits the runner and
/// `unwrap_err` panics on `Ok`.
#[test]
fn no_entry_op_refused() {
    let mut tape = PowlTape::new();
    tape.alloc(OpKind::Atom).expect("tape not full");
    tape.entry_mask = 0;
    let err = PowlRunner::new(tape).validate().unwrap_err();
    assert_eq!(
        err,
        ValidationError::NoEntryOp,
        "a tape whose entry mask is empty must be refused as NoEntryOp"
    );
}

// ---------------------------------------------------------------------------
// ExecutionDefect — via the public typestate execution flow
// (PowlRunner::begin_execution -> ExecutionToken::consume_op -> complete).
// `ExecutionToken::new_for_test` is gated behind feature `testing`, which the
// default test profile does not enable, so every defect below is injected
// through the public `consume_op` path on a real runner.
// ---------------------------------------------------------------------------

/// Two-op tape fixture for the typestate execution flow.
fn two_op_runner() -> PowlRunner<bcinr_powl::typestate::Unvalidated, PowlTape> {
    let mut tape = PowlTape::new();
    tape.alloc(OpKind::Atom).expect("tape not full");
    tape.alloc(OpKind::Atom).expect("tape not full");
    tape.entry_mask = 0b01;
    PowlRunner::new(tape)
}

/// Positive control: a fully consumed token is ADMITTED.
///
/// non-vacuity: proves the harness itself is not a gate that always refuses
/// (admission non-vacuity): if the defect accumulators (src/typestate.rs:472/479/490)
/// spuriously flagged a clean run, or `complete`'s trace computation
/// (src/typestate.rs:821 `op_trace = !remaining & valid_mask`) mis-masked,
/// `Ok` / `op_trace == 0b11` fail.
#[test]
fn fully_consumed_token_is_admitted_with_exact_trace() {
    let executing = two_op_runner()
        .validate()
        .expect("fixture tape is valid")
        .schedule::<{ TopologyKind::Standard }>();
    let (executing, mut token) = executing.begin_execution();
    token.record_fire(0);
    token.consume_op(0b01);
    token.record_fire(1);
    token.consume_op(0b10);
    let (_receipted, receipt) = executing.complete(token).expect("clean run must complete");
    assert_eq!(receipt.op_trace, 0b11, "both ops must appear in the trace");
    assert_eq!(receipt.topology, TopologyKind::Standard);
}

/// `ExecutionDefect::MalformedFires { bits: 0b11 }` for a multi-bit consume.
///
/// non-vacuity: guards src/typestate.rs:482-490 (is_multi detection) surfaced
/// at src/typestate.rs:832-836; dropping the `is_multi` flag leaves
/// `defect_malformed == 0` and `complete` falls through to `UnexhaustedOps`,
/// failing the equality with the WRONG variant — exactly what §18 forbids.
#[test]
fn multi_bit_consume_refused_as_malformed() {
    let (executing, mut token) = two_op_runner()
        .validate()
        .expect("fixture tape is valid")
        .schedule::<{ TopologyKind::Standard }>()
        .begin_execution();
    token.consume_op(0b11); // two bits at once — malformed
    assert_eq!(
        executing.complete(token).unwrap_err(),
        ExecutionDefect::MalformedFires { bits: 0b11 },
        "a two-bit consume must be refused as MalformedFires naming both bits"
    );
}

/// `ExecutionDefect::InvalidFires { bits: 0b100 }` for an out-of-bounds bit.
///
/// non-vacuity: guards src/typestate.rs:470-472 (`op_bit & !self.valid_mask`)
/// surfaced at src/typestate.rs:837-841; inverting the complement
/// (`op_bit & self.valid_mask`) yields invalid == 0 and the run surfaces as
/// `UnexhaustedOps` — wrong variant, equality fails.
#[test]
fn out_of_bounds_consume_refused_as_invalid_fires() {
    let (executing, mut token) = two_op_runner()
        .validate()
        .expect("fixture tape is valid")
        .schedule::<{ TopologyKind::Standard }>()
        .begin_execution();
    token.consume_op(0b100); // bit 2 does not exist on a 2-op tape
    assert_eq!(
        executing.complete(token).unwrap_err(),
        ExecutionDefect::InvalidFires { bits: 0b100 },
        "an out-of-bounds consume must be refused as InvalidFires naming bit 2"
    );
}

/// `ExecutionDefect::OpAlreadyConsumed { bit: 0b01 }` for a double-fire.
///
/// non-vacuity: guards src/typestate.rs:474-479 (`target_valid ^ present`
/// double-fire detection) surfaced at src/typestate.rs:842-846; a mutation
/// computing `present` against `valid_mask` instead of `remaining` yields
/// `defect_double_fire == 0` -> `UnexhaustedOps`, wrong variant, fails.
#[test]
fn double_fire_refused_as_op_already_consumed() {
    let (executing, mut token) = two_op_runner()
        .validate()
        .expect("fixture tape is valid")
        .schedule::<{ TopologyKind::Standard }>()
        .begin_execution();
    token.record_fire(0);
    token.consume_op(0b01);
    token.record_fire(0); // same op recorded twice
    token.consume_op(0b01); // same op consumed twice — double-fire
    assert_eq!(
        executing.complete(token).unwrap_err(),
        ExecutionDefect::OpAlreadyConsumed { bit: 0b01 },
        "consuming op 0 twice must be refused as OpAlreadyConsumed naming bit 0"
    );
}

/// `ExecutionDefect::UnexhaustedOps { remaining: 0b10 }` when op 1 never
/// fires.
///
/// non-vacuity: guards src/typestate.rs:847-848 (`if remaining != 0`); a
/// mutation dropping the check admits an incomplete run as `Ok` and the
/// equality fails.
#[test]
fn unexhausted_token_refused_with_exact_remaining() {
    let (executing, mut token) = two_op_runner()
        .validate()
        .expect("fixture tape is valid")
        .schedule::<{ TopologyKind::Standard }>()
        .begin_execution();
    token.record_fire(0);
    token.consume_op(0b01); // op 1 left unfired
    assert_eq!(
        executing.complete(token).unwrap_err(),
        ExecutionDefect::UnexhaustedOps { remaining: 0b10 },
        "a token with op 1 unfired must be refused as UnexhaustedOps naming 0b10"
    );
}
