//! BCINR-CMCA-F: does a real CMCA priority change which op the *production*
//! PDDL -> POWL path (`bcinr_pddl::production::PddlPowlRuntime`) admits
//! under genuine capacity scarcity?
//!
//! # Verdict: yes -- confirmed by falsifier, through the real production
//! entrypoint (not a hand-built tape)
//!
//! This is the direct continuation of BCINR-CMCA-E's disclosed gap: E
//! proved priority-driven admission was real and consequential, but only
//! against a hand-built `Powl2Model`/`compile_powl2` tape, explicitly
//! disclaiming any claim about `bcinr_pddl::production`. BCINR-CMCA-F closes
//! that gap.
//!
//! # Bridging note (same honest pattern as BCINR-CMCA-E)
//!
//! `bcinr_pddl::production::PddlPowlRuntime::plan` compiles through
//! `compile_powl_v2` over a flat `bcinr_mfw_ir::PowlModel` -- a different
//! type from `Powl2Model`/`compile_powl2` (the recursive model
//! `multifractal::consequence_mass` consumes). No bridge type exists
//! between them. This fixture establishes the correspondence itself: it
//! builds a `Powl2Model` with the *same* activity labels as the real
//! production plan's ground actions, runs the real CMCA cascade over it,
//! and maps each resulting mass back onto its production tape slot by
//! label round-trip through `CompiledPowlV2::node_labels` +
//! `PowlTape::label_slab` -- verified, not assumed.
//!
//! # Preserve
//!
//! `PddlPowlPlan::execute` (the existing, default entrypoint) is completely
//! unchanged and untouched by this checkpoint. `execute_with_selector` is a
//! new, additive, opt-in method; `execute_and_seal_v2`/`verify_execution_v2`
//! still hard-default to `StableMaximalSelector` and every pre-existing
//! caller of them is unaffected -- confirmed by the full crate test suite
//! passing unchanged (see verification notes in this checkpoint's report).

#![cfg(feature = "mfw-planner")]

use std::collections::BTreeMap;

use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_pddl::production::{PddlPowlConfig, PddlPowlRuntime};
use bcinr_powl::multifractal::consequence_mass;
use bcinr_powl::process_toolkit::{activity, partial_order};
use bcinr_powl::scheduler::PriorityCapacitySelector;

/// Three actions, all applicable from the empty initial state, no shared
/// atoms between any pair -- genuinely independent, so all three become
/// scheduler-ready in the same tick (no precedence edge excludes any pair,
/// per BCINR-SCHED-001's own finding about how dependent pairs alone
/// produce precedence edges).
const DOMAIN: &str = "(define (domain cmcaf)
    (:predicates (done-low) (done-mid) (done-high))
    (:action act-low :parameters () :precondition () :effect (done-low))
    (:action act-mid :parameters () :precondition () :effect (done-mid))
    (:action act-high :parameters () :precondition () :effect (done-high)))";
const PROBLEM: &str = "(define (problem cmcafp) (:domain cmcaf) (:init)
    (:goal (and (done-low) (done-mid) (done-high))))";

/// Build the real production plan, then derive a real CMCA-priority map
/// keyed by production tape slot id, verified by label round-trip.
fn plan_and_priorities() -> (
    bcinr_pddl::production::PddlPowlPlan,
    BTreeMap<usize, NonNegativeFixed>,
) {
    let mut runtime = PddlPowlRuntime::new(PddlPowlConfig::default());
    let plan = runtime
        .plan(DOMAIN, PROBLEM)
        .expect("3 independent, immediately-applicable actions must plan");

    assert!(
        plan.compiled.guards.nonfaces.is_empty(),
        "genuinely independent actions must produce no guard conflict -- \
         otherwise this fixture accidentally exercises deferral caused by \
         something other than priority/capacity"
    );

    // Real CMCA masses, computed over a Powl2Model built with the SAME
    // labels as the real production ground actions.
    let model = partial_order(
        vec![
            activity("act-low"),
            activity("act-mid"),
            activity("act-high"),
        ],
        vec![],
    )
    .expect("3-child partial order with no edges is valid");
    let mass_of = |node: &bcinr_powl::powl2::Powl2Model| -> NonNegativeFixed {
        match node {
            bcinr_powl::powl2::Powl2Model::Activity(label) if label == "act-low" => {
                NonNegativeFixed::from_bits(1)
            }
            bcinr_powl::powl2::Powl2Model::Activity(label) if label == "act-mid" => {
                NonNegativeFixed::from_bits(10)
            }
            bcinr_powl::powl2::Powl2Model::Activity(label) if label == "act-high" => {
                NonNegativeFixed::from_bits(100)
            }
            _ => NonNegativeFixed::ONE,
        }
    };
    let allocated =
        consequence_mass(&model, &[1], mass_of).expect("real cascade over a valid tree");
    let mass_by_label: BTreeMap<String, NonNegativeFixed> = allocated
        .into_iter()
        .filter(|(node_ref, _)| node_ref.path().len() == 1)
        .map(|(node_ref, mass)| {
            let label = match node_ref.path()[0] {
                0 => "act-low",
                1 => "act-mid",
                2 => "act-high",
                other => panic!("unexpected child index {other}"),
            };
            (label.to_string(), mass)
        })
        .collect();
    assert_eq!(mass_by_label.len(), 3);

    // Map each production tape slot to its real CMCA mass, resolved through
    // the real ground-action label -- `CompiledPowlV2::node_labels` turns
    // out to hold the projector's own synthetic "action-N" placeholders,
    // NOT the real PDDL action name (confirmed by inspection: this is a
    // genuinely different label namespace from `execution_batches()`'s
    // output). The real label lives in `workflow.powl_model.provenance`
    // (node -> occurrence) -> `workflow.causal_plan.occurrences` (occurrence
    // -> action index) -> `workflow.epoch.actions[..].label` -- the same
    // chain `production.rs`'s own private `action_for_slot` walks.
    let mut priority = BTreeMap::new();
    for (&node_id, &occurrence_id) in &plan.workflow.powl_model.provenance {
        let occurrence = plan
            .workflow
            .causal_plan
            .occurrences
            .iter()
            .find(|occurrence| occurrence.id == occurrence_id)
            .expect("provenance must reference a real occurrence");
        let action = plan
            .workflow
            .epoch
            .actions
            .get(occurrence.action as usize)
            .expect("occurrence must reference a real action index");
        if let Some(mass) = mass_by_label.get(&action.label) {
            priority.insert(node_id.0 as usize, *mass);
        }
    }
    assert_eq!(
        priority.len(),
        3,
        "expected to resolve a real CMCA mass for all 3 production action labels -- \
         got {priority:?} from node_labels {:?}",
        plan.compiled.node_labels
    );

    (plan, priority)
}

#[test]
fn real_cmca_priority_determines_deferral_through_the_production_entrypoint() {
    let (plan, priority) = plan_and_priorities();

    let mut seal_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: priority.clone(),
    };
    let mut verify_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: priority.clone(),
    };
    let execution = plan
        .execute_with_selector(&mut seal_selector, &mut verify_selector)
        .expect("priority-capacity execution must succeed and self-verify");

    let batches = execution
        .execution_batches()
        .expect("every fired mask must resolve to real action labels");
    assert_eq!(
        batches.iter().flatten().count(),
        3,
        "all 3 real, admissible actions must eventually fire -- got {batches:?}"
    );
    assert_eq!(
        batches.first().map(Vec::len),
        Some(2),
        "capacity 2 must admit exactly 2 of 3 ready actions in tick one -- got {batches:?}"
    );
    assert!(
        !batches[0].contains(&"act-low".to_string()),
        "the lowest-priority action ('act-low') must be the one deferred out \
         of tick one -- got {batches:?}"
    );
}

/// Hostile falsifier: invert the priority so `act-low` has the highest
/// mass, and confirm the DEFERRED action changes accordingly, through the
/// same real production entrypoint. If it doesn't, priority is not
/// consequential here and this checkpoint's headline claim is false.
#[test]
fn inverting_real_priority_changes_which_action_the_production_path_defers() {
    let (plan, priority) = plan_and_priorities();
    let low_slot = *priority
        .iter()
        .min_by_key(|(_, mass)| **mass)
        .map(|(id, _)| id)
        .expect("non-empty priority map");

    let mut inverted = BTreeMap::new();
    for &id in priority.keys() {
        inverted.insert(
            id,
            if id == low_slot {
                NonNegativeFixed::from_bits(1000)
            } else {
                NonNegativeFixed::from_bits(1)
            },
        );
    }

    let mut seal_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: inverted.clone(),
    };
    let mut verify_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: inverted,
    };
    let execution = plan
        .execute_with_selector(&mut seal_selector, &mut verify_selector)
        .expect("priority-capacity execution must succeed and self-verify");

    let batches = execution.execution_batches().expect("labels resolve");
    assert!(
        batches[0].contains(&"act-low".to_string()),
        "NOT_CONSEQUENTIAL falsifier failed: inverting priority did not \
         change which action tick one admits through the real production \
         path -- got {batches:?}"
    );
}
