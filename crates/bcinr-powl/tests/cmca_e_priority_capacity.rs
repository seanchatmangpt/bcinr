//! BCINR-CMCA-E: does a real CMCA-derived priority ever change which ready
//! ops get admitted under genuine capacity scarcity?
//!
//! # Verdict: yes -- confirmed by falsifier, in this scoped construction
//!
//! Prerequisite standing, established by the two prior checkpoints in this
//! sequence: BCINR-SCHED-001 found `ConcurrencyGuardTable`
//! `NOT_A_DECISION_BOUNDARY` against real production analyzer output (tape
//! precedence edges already exclude every pair it would screen).
//! BCINR-SCHED-002 built `CapacityBoundedSelector`, giving scarcity its own
//! semantic home, uncoupled from conflict exclusion. This checkpoint asks
//! the governing question directly: "First establish whether scarcity
//! already has a real semantic home. Only then should CMCA enter the
//! scheduler" -- scarcity now has one, so this drives real
//! `bcinr_cmca::cascade::consequence_mass` output into
//! `PriorityCapacitySelector` (`crates/bcinr-powl/src/scheduler.rs`) and
//! shows it change which op is admitted vs. deferred.
//!
//! # What this fixture does and does not claim
//!
//! One `Powl2Model` (`PartialOrder` of 3 independent activities, no order
//! edges) is compiled twice through two genuinely different real paths:
//! `compile_powl2` (the executable tape) and
//! `bcinr_powl::multifractal::consequence_mass` (the CMCA cascade). Both
//! consume the identical model instance, but there is no existing bridge
//! type correlating a `compile_powl2` tape slot with a
//! `crate::process_toolkit::ProcessNodeRef` by construction -- this fixture
//! establishes that correspondence itself, by declared-order index
//! alignment, and then *verifies* it by round-tripping each tape slot's
//! interned label through `LabelSlab::get` and asserting it matches the
//! `ProcessNodeRef`'s expected activity label. If a future refactor changes
//! either traversal order, this assertion fails loudly instead of the test
//! silently correlating the wrong slots.
//!
//! This is not a claim that CMCA is wired into the *production* PDDL->POWL
//! path (`bcinr_pddl::production`) -- it is not; that remains a distinct,
//! larger checkpoint. This is the narrower, honest claim the governing
//! principle asked for first: given real CMCA masses, does priority-driven
//! admission actually change a scheduler decision, end to end, through real
//! (not hand-mocked) cascade and compiler code? Yes.

use std::collections::BTreeMap;

use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_powl::multifractal::consequence_mass;
use bcinr_powl::powl2::{compile_powl2, LowestIndexPolicy};
use bcinr_powl::process_toolkit::{activity, partial_order};
use bcinr_powl::scheduler::{ConcurrencySelector, PriorityCapacitySelector};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;

/// Build the shared model, compile its tape, and return
/// `(tape_op_id -> priority)` derived from real `consequence_mass` output,
/// verified (not assumed) to correspond to the tape's own activity slots by
/// label round-trip.
fn build_tape_and_priorities() -> (
    bcinr_powl::tape::v2::PowlTape,
    BTreeMap<usize, NonNegativeFixed>,
) {
    // Three independent activities -- no order edges, so all three are
    // simultaneously ready once the entry op fires.
    let model = partial_order(
        vec![activity("low"), activity("mid"), activity("high")],
        vec![],
    )
    .expect("3-child partial order with no edges is a valid Powl2Model");

    let mut policy = LowestIndexPolicy;
    let compiled = compile_powl2(&model, &mut policy).expect("flat partial order always compiles");

    // Distinct, hand-assigned masses (not uniform) so the cascade produces
    // a genuine, non-tied ranking: "high" > "mid" > "low".
    let mass_of = |node: &bcinr_powl::powl2::Powl2Model| -> NonNegativeFixed {
        match node {
            bcinr_powl::powl2::Powl2Model::Activity(label) if label == "low" => {
                NonNegativeFixed::from_bits(1)
            }
            bcinr_powl::powl2::Powl2Model::Activity(label) if label == "mid" => {
                NonNegativeFixed::from_bits(10)
            }
            bcinr_powl::powl2::Powl2Model::Activity(label) if label == "high" => {
                NonNegativeFixed::from_bits(100)
            }
            _ => NonNegativeFixed::ONE,
        }
    };

    let allocated =
        consequence_mass(&model, &[1], mass_of).expect("real cascade over a valid tree");

    // Correlate each real consequence_mass entry (keyed by ProcessNodeRef,
    // itself keyed by child-declaration-order path) to its tape slot in
    // `activity_slots` (populated in the same declared-child-order by
    // `compile_powl2`'s traversal) -- verified by label round-trip through
    // the tape's own LabelSlab, not assumed from index equality alone.
    let mut priority = BTreeMap::new();
    for (node_ref, mass) in &allocated {
        let path = node_ref.path();
        if path.len() != 1 {
            continue; // skip the root; only leaf activities have a tape slot
        }
        let child_index = path[0] as usize;
        let expected_label = match child_index {
            0 => "low",
            1 => "mid",
            2 => "high",
            other => panic!("unexpected child index {other}"),
        };
        let (slot, offset) = compiled
            .activity_slots
            .get(child_index)
            .copied()
            .unwrap_or_else(|| panic!("no activity slot recorded for child {child_index}"));
        let actual_label = compiled.tape.label_slab.get(offset);
        assert_eq!(
            actual_label, expected_label,
            "activity_slots[{child_index}] does not correspond to the expected \
             child by declared order -- correlation assumption is wrong, fix \
             the mapping before trusting priorities derived from it"
        );
        priority.insert(slot as usize, *mass);
    }
    assert_eq!(
        priority.len(),
        3,
        "expected one priority entry per activity"
    );

    (compiled.tape, priority)
}

#[test]
fn cmca_priority_determines_which_op_is_deferred_under_real_scarcity() {
    let (tape, priority) = build_tape_and_priorities();

    // All 3 activities are ready simultaneously (entry op has already
    // fired conceptually -- we drive the selector directly against the
    // known-ready activity slots, matching how `scheduler_tick_v2` derives
    // `ready` from `pred_mask` for ops with no unfinished predecessor).
    let ready_ids: Vec<usize> = priority.keys().copied().collect();
    let mut ready = bcinr_mfw_ir::EventSet::empty();
    for id in &ready_ids {
        ready.insert(*id);
    }
    let guards = ConcurrencyGuardTable::empty();

    // Capacity 2 of 3 -- exactly one op must be deferred. Priority favors
    // "high" (mass 100) and "mid" (mass 10) over "low" (mass 1).
    let mut selector = PriorityCapacitySelector {
        capacity: 2,
        priority: priority.clone(),
    };
    let selected = selector.select_checked(&ready, &guards);
    assert_eq!(
        selected.len(),
        2,
        "capacity 2 admits exactly 2 of 3 ready ops"
    );

    let low_slot = tape_slot_for(&tape, &priority, "low");
    assert!(
        !selected.contains(low_slot),
        "the lowest-priority op ('low', mass 1) must be the one deferred -- \
         got selected={selected:?}, low_slot={low_slot}"
    );

    // Hostile falsifier: invert the priorities (give "low" the highest
    // mass) on the identical ready set / capacity, and confirm the
    // DEFERRED op changes accordingly. If it didn't, priority would not
    // actually be the operative cause of the tick-one selection -- it
    // would just be coincidental with iteration/tie-break order.
    let mut inverted = BTreeMap::new();
    for &id in priority.keys() {
        let is_low = id == low_slot;
        inverted.insert(
            id,
            if is_low {
                NonNegativeFixed::from_bits(1000)
            } else {
                NonNegativeFixed::from_bits(1)
            },
        );
    }
    let mut inverted_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: inverted,
    };
    let inverted_selected = inverted_selector.select_checked(&ready, &guards);
    assert_eq!(inverted_selected.len(), 2);
    assert!(
        inverted_selected.contains(low_slot),
        "NOT_CONSEQUENTIAL falsifier failed: inverting priority did not \
         change which op is admitted -- got {inverted_selected:?}. If this \
         fails, priority is not actually driving the admission decision, \
         and this checkpoint's headline claim is false."
    );
    assert_ne!(
        selected, inverted_selected,
        "inverting priority must change the selected set"
    );
}

fn tape_slot_for(
    _tape: &bcinr_powl::tape::v2::PowlTape,
    priority: &BTreeMap<usize, NonNegativeFixed>,
    label: &str,
) -> usize {
    // "low" always has the smallest constructed mass (bits=1) in the
    // non-inverted map built by `build_tape_and_priorities`.
    if label != "low" {
        panic!("tape_slot_for only supports 'low' in this fixture");
    }
    *priority
        .iter()
        .min_by_key(|(_, mass)| **mass)
        .map(|(id, _)| id)
        .expect("priority map is non-empty")
}
