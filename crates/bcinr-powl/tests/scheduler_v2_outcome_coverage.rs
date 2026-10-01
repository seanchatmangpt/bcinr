//! Outcome-variant coverage for `scheduler_tick_v2` at the public API.
//!
//! `PowlV2TickOutcome` has three variants — `Fired(u64)`, `Complete`,
//! `Deadlock { remaining_mask }` — and before this file only crate-internal
//! unit tests (src/scheduler_v2.rs:218/240/258) asserted them; tests/ had no
//! exact-variant assertion at all. This file binds the public
//! `scheduler_tick_v2` surface: exact fired masks, exact completion, and the
//! exact `remaining_mask` of a deadlocked run.
//!
//! # Non-vacuity method
//!
//! Each test carries a `// non-vacuity:` line naming the exact guarded source
//! path (file:line) and the plausible mutation that would flip the assertion
//! to failure (anti-vacuity law; CHEAT-009 prohibits assert-only-difference
//! mutant theater).

use bcinr_powl::scheduler::StableMaximalSelector;
use bcinr_powl::scheduler_v2::{execute_v2, scheduler_tick_v2, PowlV2RunState, PowlV2TickOutcome};
use bcinr_powl::tape::v2::{ConcurrencyGuardTable, OpKind as V2OpKind, Powl64Op, PowlTape};

/// An `Activity` op with the given pred/succ masks.
fn activity(pred_mask: u64, succ_mask: u64) -> Powl64Op {
    let mut op = Powl64Op::silent();
    op.op_kind = V2OpKind::Activity;
    op.pred_mask = pred_mask;
    op.succ_mask = succ_mask;
    op
}

/// Sequential chain op0 -> op1 -> op2: each tick fires exactly one op, in
/// Kahn order, with the exact single-bit mask.
///
/// non-vacuity: guards src/scheduler_v2.rs:93-95 (`state.done_mask |= fired;
/// ... PowlV2TickOutcome::Fired(fired)`) plus the ready-set computation at
/// src/scheduler_v2.rs:38-49 (`unfinished && predecessors_complete`). An
/// off-by-one in the ready bit (`1 << (index+1)`), a dropped `& !done_mask`
/// (line 43), or a stale `pred_mask` acceptance (`op.pred_mask & done_mask
/// == 0` instead of `& !done_mask == 0`) changes WHICH mask fires first or
/// fires too much, and the first `Fired(0b01)` equality fails.
#[test]
fn sequential_chain_fires_exact_single_bit_masks_in_order() {
    let mut tape = PowlTape::new();
    tape.push(activity(0, 0b010)).expect("tape not full");
    tape.push(activity(0b001, 0b100)).expect("tape not full");
    tape.push(activity(0b010, 0)).expect("tape not full");

    let mut state = PowlV2RunState::new();
    let mut selector = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();

    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b001),
        "tick 1 must fire exactly op 0"
    );
    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b010),
        "tick 2 must fire exactly op 1"
    );
    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b100),
        "tick 3 must fire exactly op 2"
    );
    assert_eq!(state.done_mask, 0b111, "all three ops must be done");
    assert_eq!(
        state.tick, 3,
        "exactly three firing ticks must have elapsed"
    );
}

/// Two independent ready ops fire TOGETHER in one tick with the combined
/// mask — the concurrent-admission case.
///
/// non-vacuity: guards the event-set <-> mask round trip feeding
/// src/scheduler_v2.rs:93-95 (`Fired(fired)`): `mask_to_event_set` losing a
/// bit or `event_set_to_mask` dropping the high op would make the first tick
/// `Fired(0b011)` (or any proper subset) instead of `Fired(0b111)`, failing
/// the equality. Also guards `ready_mask`'s union over simultaneous ready ops
/// (src/scheduler_v2.rs:38-49).
#[test]
fn parallel_ready_ops_fire_in_one_tick_with_combined_mask() {
    let mut tape = PowlTape::new();
    tape.push(activity(0, 0)).expect("tape not full");
    tape.push(activity(0, 0)).expect("tape not full");
    tape.push(activity(0, 0)).expect("tape not full");

    let mut state = PowlV2RunState::new();
    let mut selector = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();

    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b111),
        "three unguarded ready ops must all fire in one tick"
    );
    assert_eq!(state.done_mask, 0b111);
}

/// `PowlV2TickOutcome::Complete`: a finished tape is recognized on the next
/// tick without firing anything.
///
/// non-vacuity: guards src/scheduler_v2.rs:74-76 (`if state.is_complete(tape)
/// { return Complete }`) and `is_complete` (src/scheduler_v2.rs:33-35,
/// `done_mask & valid_mask == valid_mask`). A mutation comparing against a
/// partial mask (e.g. `done_mask != 0`) would return `Complete` early here
/// but break the chain test; a mutation returning `Deadlock` instead fails
/// this equality directly.
#[test]
fn completed_tape_reports_complete_with_no_further_firing() {
    let mut tape = PowlTape::new();
    tape.push(activity(0, 0)).expect("tape not full");
    tape.push(activity(0, 0)).expect("tape not full");

    let mut state = PowlV2RunState::new();
    let mut selector = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();

    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b11)
    );
    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Complete,
        "a tape whose every op is done must report Complete"
    );
    assert_eq!(
        state.done_mask, 0b11,
        "Complete must not mutate the done mask"
    );
}

/// `PowlV2TickOutcome::Deadlock { remaining_mask }`: a self-dependent op can
/// never be ready; the refusal names exactly the unfinished set.
///
/// non-vacuity: guards src/scheduler_v2.rs:86-90 (`if ready_mask == 0 ||
/// fired == 0 { return Deadlock { remaining_mask: valid_mask(tape.len) &
/// !state.done_mask } }`). A sign flip (`valid_mask & state.done_mask`)
/// reports 0 instead of 0b01; dropping the `!` in the ready computation
/// (src/scheduler_v2.rs:44) makes the self-dependent op look ready and fires
/// it instead — both fail the equality.
#[test]
fn self_dependent_tape_deadlocks_with_exact_remaining_mask() {
    let mut tape = PowlTape::new();
    tape.push(activity(0b01, 0)).expect("tape not full"); // op 0 waits on itself

    let mut state = PowlV2RunState::new();
    let mut selector = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();

    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Deadlock {
            remaining_mask: 0b01
        },
        "a self-dependent op must deadlock naming itself as remaining"
    );
    assert_eq!(
        state.done_mask, 0,
        "a deadlocked tick must leave persistent state unchanged"
    );
}

/// Deadlock after a partial prefix: `remaining_mask` is the UNFINISHED set
/// only (done ops excluded) — the exact `valid_mask & !done_mask` contract.
///
/// non-vacuity: guards src/scheduler_v2.rs:88 (`remaining_mask: valid_mask(tape.len)
/// & !state.done_mask`). Reporting `valid_mask` alone (0b111), or masking
/// with `done_mask` (0b000), or computing `!done_mask` without `valid_mask`
/// (high garbage bits) each fail the exact `0b100` equality.
#[test]
fn deadlocked_prefix_reports_only_unfinished_ops() {
    let mut tape = PowlTape::new();
    tape.push(activity(0, 0b010)).expect("tape not full");
    tape.push(activity(0b001, 0)).expect("tape not full");
    tape.push(activity(0b100, 0)).expect("tape not full"); // op 2 waits on itself

    let mut state = PowlV2RunState::new();
    let mut selector = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();

    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b001)
    );
    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Fired(0b010)
    );
    assert_eq!(
        scheduler_tick_v2(&tape, &mut state, &mut selector, &guards),
        PowlV2TickOutcome::Deadlock {
            remaining_mask: 0b100
        },
        "after firing ops 0 and 1, only the stuck op 2 may be reported remaining"
    );
    assert_eq!(state.done_mask, 0b011);
    assert_eq!(state.tick, 2, "only firing ticks advance the tick counter");
}

// ---------------------------------------------------------------------------
// Falsifiers — witnessed non-vacuity for the assertions above
//
// The existing mutation infrastructure of this suite
// (mutant_kill_g4_powl.rs) simulates mutants by mutating runtime values and
// asserting the exact typed kill. The same method is used here: the plausible
// mutations of the readiness law are APPLIED to the fixture tape (mutated
// masks), the real scheduler is executed on both lawful and mutated subjects,
// and the test proves the asserted outcome discriminates the two. A
// Deadlock/Fired assertion that could not tell these tapes apart would be
// vacuous; these tests witness that it can.
// ---------------------------------------------------------------------------

/// Falsifier for `sequential_chain_fires_exact_single_bit_masks_in_order`:
/// the mutation "stale pred_mask acceptance" (op 1's predecessor edge
/// dropped — the `pred_mask & !done_mask == 0` check at src/scheduler_v2.rs:43
/// accepting an incomplete predecessor set) produces a DIFFERENT observable:
/// op 1 fires together with op 0 in tick one.
///
/// If both this mutated tape and the lawful chain produced `Fired(0b001)`
/// first, the chain test's assertion would carry no information about the
/// predecessor-completeness law. They do not: the mutation is witnessed to
/// change the outcome to `Fired(0b011)`.
#[test]
fn falsifier_stale_pred_acceptance_changes_fired_mask() {
    // Lawful subject: op 1 cannot fire before op 0.
    let mut lawful = PowlTape::new();
    lawful.push(activity(0, 0b010)).expect("tape not full");
    lawful.push(activity(0b001, 0)).expect("tape not full");

    // Mutated subject: the pred_mask edge is LOST (stale acceptance).
    let mut mutated = PowlTape::new();
    mutated.push(activity(0, 0b010)).expect("tape not full");
    mutated.push(activity(0, 0)).expect("tape not full"); // pred dropped

    let mut sel_l = StableMaximalSelector;
    let mut sel_m = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();
    let mut state_l = PowlV2RunState::new();
    let mut state_m = PowlV2RunState::new();

    let lawful_out = scheduler_tick_v2(&lawful, &mut state_l, &mut sel_l, &guards);
    let mutated_out = scheduler_tick_v2(&mutated, &mut state_m, &mut sel_m, &guards);

    assert_eq!(lawful_out, PowlV2TickOutcome::Fired(0b001));
    assert_eq!(
        mutated_out,
        PowlV2TickOutcome::Fired(0b011),
        "witnessed: dropping op 1's pred_mask lets it co-fire in tick one"
    );
    assert_ne!(
        lawful_out, mutated_out,
        "the Fired-mask assertion must discriminate the mutation; it does"
    );
}

/// Falsifier for the Deadlock assertions: the mutation "deadlock reported on
/// a completable tape" (e.g. the `unfinished` term at src/scheduler_v2.rs:42
/// inverted, or a `pred_mask` bug that empties the ready set) would turn a
/// completable chain into a spurious `Deadlock`. Driven through the public
/// `execute_v2` loop, the lawful tape is witnessed to reach `Complete` while
/// a one-bit pred_mask difference (op 2 self-edge) yields
/// `Deadlock { remaining_mask: 0b100 }` — the Deadlock assertions in this
/// file are sensitive to exactly the readiness law, not vacuously
/// satisfiable by any outcome.
#[test]
fn falsifier_spurious_deadlock_discriminated_by_completable_tape() {
    // Lawful completable chain: deadlock here would be a scheduler defect.
    let mut lawful = PowlTape::new();
    lawful.push(activity(0, 0b010)).expect("tape not full");
    lawful.push(activity(0b001, 0b100)).expect("tape not full");
    lawful.push(activity(0b010, 0)).expect("tape not full");

    // Genuinely deadlocked subject: op 2 waits on itself. Same shape except
    // that one bit of pred_mask.
    let mut deadlocked = PowlTape::new();
    deadlocked.push(activity(0, 0b010)).expect("tape not full");
    deadlocked
        .push(activity(0b001, 0b100))
        .expect("tape not full");
    deadlocked.push(activity(0b100, 0)).expect("tape not full");

    let mut sel_l = StableMaximalSelector;
    let mut sel_d = StableMaximalSelector;
    let guards = ConcurrencyGuardTable::empty();

    let (_, lawful_out) = execute_v2(&lawful, &mut sel_l, &guards, 8);
    let (_, deadlocked_out) = execute_v2(&deadlocked, &mut sel_d, &guards, 8);

    assert_eq!(
        lawful_out,
        PowlV2TickOutcome::Complete,
        "witnessed: a lawful chain reaches Complete under execute_v2"
    );
    assert_eq!(
        deadlocked_out,
        PowlV2TickOutcome::Deadlock {
            remaining_mask: 0b100
        },
        "witnessed: the single self-edge bit drives Deadlock naming the stuck op"
    );
    assert_ne!(
        lawful_out, deadlocked_out,
        "the Complete/Deadlock assertions must discriminate the one-bit mutation; they do"
    );
}
