//! BCINR-SCHED-001: characterizes `ConcurrencyGuardTable` end-to-end through
//! the real production path (`bcinr_pddl::production::execute_pddl_to_powl`),
//! not a hand-constructed table driven directly at the scheduler.
//!
//! # Verdict: `NOT_A_DECISION_BOUNDARY`
//!
//! This corrects an earlier `CONFLICT_ONLY` draft verdict, which this file's
//! own falsifier disproved. `ConcurrencyGuardTable::admits`
//! (`crates/bcinr-powl/src/scheduler.rs:601-610`) is real, reachable
//! subset-avoidance logic -- but it is never load-bearing against real
//! `PddlConcurrencyAnalyzer` output, because a different mechanism already
//! excludes every pair it would ever screen, before the guard is consulted.
//!
//! Root cause, read end to end:
//! - `PddlCausalAnalyzerV2::analyze` (`crates/bcinr-pddl/src/causal_v2.rs:41`)
//!   builds `CausalPlan::precedes` from `plan.independence.dependent.keys()`.
//! - `PddlConcurrencyAnalyzer::analyze` (`crates/bcinr-pddl/src/concurrency.rs`)
//!   builds one 2-element nonface per pair in that exact same `dependent` map.
//!   Both consumers read the identical source set.
//! - `compile_powl_v2` (`crates/bcinr-powl/src/compiler.rs:892-906`) wires
//!   `model.order.edges` (derived from `precedes`) directly into each op's
//!   `pred_mask`/`succ_mask` at compile time.
//! - `SchedulerState::ready_mask` (`crates/bcinr-powl/src/scheduler_v2.rs:38-43`)
//!   computes readiness from `pred_mask` alone: an op with an unfinished
//!   predecessor is never in the ready set.
//! - Consequence: for every pair the real analyzer emits a nonface for, a
//!   precedence edge has already removed that pair from ever being
//!   simultaneously ready. `ConcurrencyGuardTable::admits` only screens
//!   *already-ready* candidates (`StableMaximalSelector::select`,
//!   `scheduler.rs:601-610`) -- so it never has a live decision to make
//!   against this analyzer's output.
//!
//! Confirmed by falsifier below: replacing the real guard table with
//! `ConcurrencyGuardTable::empty()` and replaying the *same compiled tape*
//! produces byte-identical `fired_masks` -- the guard changed nothing.
//!
//! Per this verdict, `BCINR-CMCA-E` must not resume by feeding CMCA
//! priority into `ConcurrencyGuardTable` -- it is not merely "the wrong kind
//! of mechanism" (conflict vs. capacity), it is not a decision point at all
//! against real production input. See `BCINR-SCHED-002` (proposed, not
//! built) for introducing real scarcity as its own concept before CMCA
//! enters the scheduler.

#![cfg(feature = "mfw-planner")]

use bcinr_pddl::production::execute_pddl_to_powl;
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use bcinr_powl_receipt::execution_v2::execute_and_seal_v2;

/// Two actions, both applicable from the empty initial state (so both
/// become scheduler-ready in the same tick), forced into the same minimal
/// plan by a goal that only their respective effects satisfy, and causally
/// `Dependent` via delete-interference: `a1` deletes `p`, `a2` adds `p`.
const DOMAIN: &str = "(define (domain sched001)
    (:predicates (p) (done1) (done2))
    (:action a1 :parameters () :precondition () :effect (and (done1) (not (p))))
    (:action a2 :parameters () :precondition () :effect (and (done2) (p))))";
const PROBLEM: &str =
    "(define (problem sched001p) (:domain sched001) (:init) (:goal (and (done1) (done2))))";

#[test]
fn concurrency_guard_is_reachable_but_not_load_bearing_for_a_real_conflicting_pair() {
    let execution = execute_pddl_to_powl(DOMAIN, PROBLEM)
        .expect("a1/a2 forced into the same plan by the goal, both immediately applicable");

    // The real analyzer must have found the delete-interference conflict --
    // otherwise this fixture is accidentally exercising the empty-table
    // path and proves nothing.
    assert!(
        !execution.compiled.guards.nonfaces.is_empty(),
        "expected PddlConcurrencyAnalyzer to emit a real nonface for a1/a2's \
         delete-interference on `p` -- got an empty guard table, which means \
         this fixture's domain doesn't actually produce the conflict it claims to"
    );

    // Both actions are real, admissible work -- neither is impossible --
    // so both must eventually fire, and not in the same tick.
    let batches = execution
        .execution_batches()
        .expect("every fired mask must resolve to real action labels");
    let all_fired: Vec<&String> = batches.iter().flatten().collect();
    assert_eq!(
        all_fired.len(),
        2,
        "both a1 and a2 must fire (pending, not refused/impossible) -- got {batches:?}"
    );
    let same_tick_batch_has_both = batches.iter().any(|batch| batch.len() == 2);
    assert!(
        !same_tick_batch_has_both,
        "a1 and a2 must not fire in the same POWL tick -- got {batches:?}"
    );

    // Falsifier: replay the SAME compiled tape with an EMPTY guard table.
    // If the guard table were load-bearing, this should let both actions
    // fire in the same tick (nothing would block them). It does not --
    // `pred_mask` alone already keeps them apart, proving the guard table
    // never had a live decision to make here.
    let real_receipt = &execution.powl_receipt;
    let empty_guards = ConcurrencyGuardTable::empty();
    let max_ticks = execution.compiled.tape.len as u32 + 1;
    let replayed = execute_and_seal_v2(&execution.compiled.tape, &empty_guards, max_ticks)
        .expect("replay with an empty guard table must still succeed without deadlock");

    assert_eq!(
        replayed.fired_masks, real_receipt.fired_masks,
        "NOT_A_DECISION_BOUNDARY falsifier failed: emptying the guard table \
         changed the fired-mask sequence, which would mean the guard *was* \
         load-bearing here -- re-examine the verdict in this file's module doc"
    );
}

/// Sanity companion: an independent pair (no shared atom) produces no
/// nonface at all and is free to fire in the same tick -- confirms the
/// deferral above is caused by real precedence structure, not some
/// unrelated one-action-per-tick scheduling artifact.
#[test]
fn independent_pair_fires_in_the_same_tick() {
    let domain = "(define (domain sched001b)
        (:predicates (done1) (done2))
        (:action a1 :parameters () :precondition () :effect (done1))
        (:action a2 :parameters () :precondition () :effect (done2)))";
    let problem =
        "(define (problem sched001bp) (:domain sched001b) (:init) (:goal (and (done1) (done2))))";

    let execution =
        execute_pddl_to_powl(domain, problem).expect("two independent actions, both applicable");

    assert!(
        execution.compiled.guards.nonfaces.is_empty(),
        "an independent pair must produce no nonface"
    );

    let batches = execution.execution_batches().unwrap();
    let same_tick_batch_has_both = batches.iter().any(|batch| batch.len() == 2);
    assert!(
        same_tick_batch_has_both,
        "independent actions should be free to fire in the same tick -- got {batches:?}"
    );
}
