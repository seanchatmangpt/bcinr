//! BCINR-SCHED-002: characterizes `CapacityBoundedSelector` as a genuine
//! decrementable-resource concurrency admission mechanism, distinct from
//! `ConcurrencyGuardTable` (pairwise conflict exclusion, found
//! `NOT_A_DECISION_BOUNDARY` against real production input by
//! BCINR-SCHED-001) and from `max_ticks` (a total-tick completion bound,
//! not a per-tick concurrent-slot budget).
//!
//! # Verdict: `CAPACITY_BOUNDARY` (by construction, confirmed by falsifier)
//!
//! Three ops, no recorded conflict between any of them (empty guard table),
//! all simultaneously ready. With `capacity: 2`, exactly two fire in tick
//! one and the third is deferred to tick two -- a genuine "admitted vs.
//! still-pending, not refused" boundary, the same distinction BCINR-SCHED-001
//! required and found `ConcurrencyGuardTable` incapable of providing against
//! real analyzer output. Unlike SCHED-001's subject, this selector's
//! capacity is not derived from -- or redundant with -- tape precedence
//! edges: nothing else in this fixture's tape excludes any pair from
//! co-readiness, so the deferral observed here is caused only by `capacity`.
//!
//! This checkpoint is additive only: `CapacityBoundedSelector` is a new
//! `ConcurrencySelector` implementation (`crates/bcinr-powl/src/scheduler.rs`)
//! alongside the existing `StableMaximalSelector`, selected explicitly by
//! callers via the generic `S: ConcurrencySelector` parameter already
//! present on `scheduler_tick_v2`/`execute_v2`. No existing call site,
//! production or test, is changed -- they keep using `StableMaximalSelector`
//! unless they opt in. CMCA is not wired into this selector; that remains
//! future work (BCINR-CMCA-E), now that scarcity has a real semantic home
//! to attach to.

use bcinr_mfw_ir::EventSet;
use bcinr_powl::scheduler::{CapacityBoundedSelector, ConcurrencySelector};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;

#[test]
fn capacity_defers_a_third_ready_op_with_no_recorded_conflict() {
    let ready = EventSet::empty().with(0).with(1).with(2);
    let guards = ConcurrencyGuardTable::empty();

    let mut selector = CapacityBoundedSelector { capacity: 2 };
    let tick_one = selector.select_checked(&ready, &guards);
    assert_eq!(
        tick_one.len(),
        2,
        "capacity 2 must admit exactly 2 of the 3 ready ops in one tick"
    );
    assert!(
        tick_one.is_subset_of(&ready),
        "selected ops must come from the ready set"
    );

    // The deferred op is real, admissible work (not refused/impossible):
    // once it is the only one left ready, it fires on its own.
    let remaining: Vec<usize> = ready
        .iter_stable()
        .filter(|id| !tick_one.contains(*id))
        .collect();
    assert_eq!(remaining.len(), 1, "exactly one op must be deferred");
    let deferred = EventSet::empty().with(remaining[0]);
    let mut selector_tick_two = CapacityBoundedSelector { capacity: 2 };
    let tick_two = selector_tick_two.select_checked(&deferred, &guards);
    assert_eq!(
        tick_two, deferred,
        "the deferred op must be free to fire once it is the only ready op"
    );
}

/// Hostile falsifier: raise `capacity` to 3 (>= ready.len()) on the exact
/// same ready set and guard table. If the cap were not the operative cause
/// of the tick-one deferral above, this would not change anything. It does:
/// all 3 fire together, proving `capacity` -- not some other artifact of
/// `select`'s loop or `EventSet` iteration order -- was the actual
/// constraint.
#[test]
fn raising_capacity_to_cover_all_ready_ops_removes_the_deferral() {
    let ready = EventSet::empty().with(0).with(1).with(2);
    let guards = ConcurrencyGuardTable::empty();

    let mut selector = CapacityBoundedSelector { capacity: 3 };
    let selected = selector.select_checked(&ready, &guards);
    assert_eq!(
        selected, ready,
        "capacity >= ready.len() must admit every ready op in one tick -- \
         if this fails, the capacity-2 deferral above was not actually \
         caused by `capacity`"
    );
}

/// Sanity companion: a real guard-table conflict still applies underneath
/// the capacity check -- `CapacityBoundedSelector` narrows admission, it
/// does not bypass `ConcurrencyGuardTable::admits`.
#[test]
fn capacity_selector_still_honors_a_real_guard_conflict() {
    use bcinr_mfw_ir::Digest;
    use bcinr_powl::tape::v2::CompiledNonFace;

    let ready = EventSet::empty().with(0).with(1);
    let nonface = EventSet::empty().with(0).with(1);
    let guards = ConcurrencyGuardTable {
        nonfaces: vec![CompiledNonFace {
            members: nonface,
            witness_digest: Digest([0u8; 32]),
        }],
    };

    // Capacity is generous (2) -- if the guard were bypassed, both would fire.
    let mut selector = CapacityBoundedSelector { capacity: 2 };
    let selected = selector.select_checked(&ready, &guards);
    assert_eq!(
        selected.len(),
        1,
        "the recorded conflict must still block co-selection even though \
         capacity alone would have allowed both -- got {selected:?}"
    );
}
