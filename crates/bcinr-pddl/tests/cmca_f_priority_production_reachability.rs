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
//! # BCINR-CMCA-G note
//!
//! This fixture originally hand-rolled the process-to-priority mapping
//! (build a matching `Powl2Model`, run the cascade, walk
//! `workflow.powl_model.provenance` -> `causal_plan.occurrences` ->
//! `epoch.actions[..].label` by hand). BCINR-CMCA-G promoted that exact
//! chain into the one canonical production function,
//! `bcinr_pddl::cmca_execution::allocate_pddl_powl_plan`, and this fixture
//! now calls it instead of duplicating the mapping -- per BCINR-CMCA-G
//! acceptance criterion #4 ("fixture-local mapping logic is removed or
//! reduced to calls into that production function"). The verdict and
//! assertions below are unchanged from the original checkpoint.

#![cfg(feature = "mfw-planner")]

use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_pddl::cmca_execution::{
    AllocationSemantics, CmcaExecutionProfile, CmcaExecutionRequest, LensSchedule,
    ProcessMassField, ProfileIdentity,
};
use bcinr_pddl::production::{PddlPowlConfig, PddlPowlRuntime};
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

fn request(capacity: u32) -> CmcaExecutionRequest {
    let mut masses = std::collections::BTreeMap::new();
    masses.insert("act-low".to_string(), NonNegativeFixed::from_bits(1));
    masses.insert("act-mid".to_string(), NonNegativeFixed::from_bits(10));
    masses.insert("act-high".to_string(), NonNegativeFixed::from_bits(100));
    CmcaExecutionRequest {
        profile: CmcaExecutionProfile {
            identity: ProfileIdentity("BCINR_CMCA_PROFILE_V0_1".to_string()),
            lens_schedule: LensSchedule(vec![1]),
            allocation_semantics: AllocationSemantics::UniformSiblingCoverageQ0,
            // No Lean manifest binding required for BCINR-CMCA-F's scope --
            // this field was introduced later, by ECOSYSTEM-JOIN-001 Rail B.
            lean_manifest_digest: bcinr_mfw_ir::Digest::ZERO,
        },
        capacity,
        masses: ProcessMassField(masses),
    }
}

fn plan() -> bcinr_pddl::production::PddlPowlPlan {
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
    plan
}

#[test]
fn real_cmca_priority_determines_deferral_through_the_production_entrypoint() {
    let plan = plan();
    let request = request(2);

    let allocation = bcinr_pddl::cmca_execution::allocate_pddl_powl_plan(&plan, &request)
        .expect("complete mass field over 3 real production actions must allocate");
    assert_eq!(allocation.priority_map.len(), 3);

    let low_slot = *allocation
        .priority_map
        .iter()
        .min_by_key(|(_, mass)| **mass)
        .map(|(id, _)| id)
        .expect("non-empty priority map");

    let mut seal_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: allocation.priority_map.clone(),
    };
    let mut verify_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: allocation.priority_map.clone(),
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
    let _ = low_slot; // documents which slot was lowest, for the assertion above
}

/// Hostile falsifier: invert the priority so `act-low` has the highest
/// mass, and confirm the DEFERRED action changes accordingly, through the
/// same real production entrypoint. If it doesn't, priority is not
/// consequential here and this checkpoint's headline claim is false.
#[test]
fn inverting_real_priority_changes_which_action_the_production_path_defers() {
    let plan = plan();
    let mut masses = std::collections::BTreeMap::new();
    masses.insert("act-low".to_string(), NonNegativeFixed::from_bits(1000));
    masses.insert("act-mid".to_string(), NonNegativeFixed::from_bits(1));
    masses.insert("act-high".to_string(), NonNegativeFixed::from_bits(1));
    let request = CmcaExecutionRequest {
        profile: CmcaExecutionProfile {
            identity: ProfileIdentity("BCINR_CMCA_PROFILE_V0_1".to_string()),
            lens_schedule: LensSchedule(vec![1]),
            allocation_semantics: AllocationSemantics::UniformSiblingCoverageQ0,
            // No Lean manifest binding required for BCINR-CMCA-F's scope --
            // this field was introduced later, by ECOSYSTEM-JOIN-001 Rail B.
            lean_manifest_digest: bcinr_mfw_ir::Digest::ZERO,
        },
        capacity: 2,
        masses: ProcessMassField(masses),
    };

    let allocation = bcinr_pddl::cmca_execution::allocate_pddl_powl_plan(&plan, &request)
        .expect("complete mass field over 3 real production actions must allocate");

    let mut seal_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: allocation.priority_map.clone(),
    };
    let mut verify_selector = PriorityCapacitySelector {
        capacity: 2,
        priority: allocation.priority_map,
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
