//! BCINR-CMCA-G: does the execution receipt identify WHICH allocation
//! (masses, profile, capacity, priority map) governed execution, not just
//! WHAT fired?
//!
//! # Verdict: yes, for the governing law this checkpoint names --
//! `fired trace equal` does NOT imply `allocation receipt equal` (Fixture
//! B / falsifier 6), and every governing input is independently
//! recomputed and compared by `verify_cmca_execution`, never trusted from a
//! caller-supplied selector (falsifiers 1-3).
//!
//! All allocation/mapping goes through the one canonical production
//! surface, `bcinr_pddl::cmca_execution::allocate_pddl_powl_plan` /
//! `PddlPowlPlan::execute_with_cmca` / `verify_cmca_execution` -- no
//! fixture-local mapping logic here (BCINR-CMCA-F's fixture was rewritten
//! to route through the same production function, per this checkpoint's
//! acceptance criterion #4).

#![cfg(feature = "mfw-planner")]

use std::collections::BTreeMap;

use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_pddl::cmca_execution::{
    allocate_pddl_powl_plan, verify_cmca_execution, AllocationSemantics, CmcaAllocationRefusal,
    CmcaExecutionProfile, CmcaExecutionRequest, LensSchedule, ProcessMassField, ProfileIdentity,
};
use bcinr_pddl::production::{PddlPowlConfig, PddlPowlPlan, PddlPowlRuntime};

const DOMAIN: &str = "(define (domain cmcag)
    (:predicates (done-low) (done-mid) (done-high))
    (:action act-low :parameters () :precondition () :effect (done-low))
    (:action act-mid :parameters () :precondition () :effect (done-mid))
    (:action act-high :parameters () :precondition () :effect (done-high)))";
const PROBLEM: &str = "(define (problem cmcagp) (:domain cmcag) (:init)
    (:goal (and (done-low) (done-mid) (done-high))))";

/// A second, topologically identical process with different action
/// identities -- used by Fixture C.
const DOMAIN_2: &str = "(define (domain cmcag2)
    (:predicates (done-x) (done-y) (done-z))
    (:action act-x :parameters () :precondition () :effect (done-x))
    (:action act-y :parameters () :precondition () :effect (done-y))
    (:action act-z :parameters () :precondition () :effect (done-z)))";
const PROBLEM_2: &str = "(define (problem cmcag2p) (:domain cmcag2) (:init)
    (:goal (and (done-x) (done-y) (done-z))))";

fn plan() -> PddlPowlPlan {
    PddlPowlRuntime::new(PddlPowlConfig::default())
        .plan(DOMAIN, PROBLEM)
        .expect("3 independent actions must plan")
}

fn plan_2() -> PddlPowlPlan {
    PddlPowlRuntime::new(PddlPowlConfig::default())
        .plan(DOMAIN_2, PROBLEM_2)
        .expect("second, topologically identical process must plan")
}

fn profile(identity: &str, lenses: Vec<i32>) -> CmcaExecutionProfile {
    CmcaExecutionProfile {
        identity: ProfileIdentity(identity.to_string()),
        lens_schedule: LensSchedule(lenses),
        allocation_semantics: AllocationSemantics::UniformSiblingCoverageQ0,
        // No Lean manifest binding required for BCINR-CMCA-G's scope --
        // this field was introduced later, by ECOSYSTEM-JOIN-001 Rail B.
        lean_manifest_digest: bcinr_mfw_ir::Digest::ZERO,
    }
}

fn masses(low: u32, mid: u32, high: u32) -> ProcessMassField {
    let mut m = BTreeMap::new();
    m.insert("act-low".to_string(), NonNegativeFixed::from_bits(low));
    m.insert("act-mid".to_string(), NonNegativeFixed::from_bits(mid));
    m.insert("act-high".to_string(), NonNegativeFixed::from_bits(high));
    ProcessMassField(m)
}

fn request(capacity: u32, low: u32, mid: u32, high: u32) -> CmcaExecutionRequest {
    CmcaExecutionRequest {
        profile: profile("BCINR_CMCA_PROFILE_V0_1", vec![1]),
        capacity,
        masses: masses(low, mid, high),
    }
}

// ---------------------------------------------------------------------
// Fixture A -- F's three-action scarcity case, through the canonical API.
// ---------------------------------------------------------------------

#[test]
fn fixture_a_scarcity_case_binds_the_priority_map_and_is_deterministic() {
    let request = request(2, 1, 10, 100);

    let allocation = allocate_pddl_powl_plan(&plan(), &request)
        .expect("complete mass field over 3 real actions must allocate");
    let low_slot = *allocation
        .priority_map
        .iter()
        .min_by_key(|(_, m)| **m)
        .map(|(id, _)| id)
        .unwrap();
    let high_slot = *allocation
        .priority_map
        .iter()
        .max_by_key(|(_, m)| **m)
        .map(|(id, _)| id)
        .unwrap();
    assert_ne!(low_slot, high_slot);

    // Highest-priority activities execute first, lowest remains pending.
    let execution = plan()
        .execute_with_cmca(&request)
        .expect("canonical execution must succeed and self-verify");
    let batches = execution
        .execution
        .execution_batches()
        .expect("labels resolve");
    assert_eq!(batches.first().map(Vec::len), Some(2));
    assert!(!batches[0].contains(&"act-low".to_string()));

    // The receipt binds the exact priority map (via its digest).
    assert_eq!(
        execution.receipt.allocation.priority_digest,
        execution.allocation.priority_digest
    );

    // Repeated allocation is byte-identical (deterministic).
    let allocation_again = allocate_pddl_powl_plan(&plan(), &request).unwrap();
    assert_eq!(allocation.priority_digest, allocation_again.priority_digest);
    assert_eq!(allocation.process_digest, allocation_again.process_digest);
    assert_eq!(
        allocation.allocation_trace_digest,
        allocation_again.allocation_trace_digest
    );

    // Independent verification succeeds against the same plan/request.
    verify_cmca_execution(&execution.receipt, &plan(), &request)
        .expect("verification must succeed against the plan/request that governed execution");
}

// ---------------------------------------------------------------------
// Fixture B / falsifier 6 -- REQUIRED LAW: matching fired trace does not
// imply matching allocation receipt.
// ---------------------------------------------------------------------

#[test]
fn fixture_b_same_fired_trace_different_priorities_different_receipt_and_refused_cross_verify() {
    // capacity >= ready.len() (3): every ranking admits all 3 actions in
    // one tick, so the fired trace is the same regardless of priority order.
    let request_a = request(3, 1, 10, 100);
    let request_b = request(3, 100, 10, 1); // a different, non-equal priority map

    let plan_a = plan();
    let allocation_a = allocate_pddl_powl_plan(&plan_a, &request_a).unwrap();
    let allocation_b = allocate_pddl_powl_plan(&plan_a, &request_b).unwrap();
    assert_ne!(
        allocation_a.priority_digest, allocation_b.priority_digest,
        "the two requests must actually induce different priority maps"
    );

    let execution_a = plan().execute_with_cmca(&request_a).unwrap();
    let execution_b = plan().execute_with_cmca(&request_b).unwrap();

    // fired trace: equal
    assert_eq!(
        execution_a.execution.powl_receipt.fired_masks,
        execution_b.execution.powl_receipt.fired_masks,
        "capacity 3 over 3 ready actions must admit all 3 in tick one \
         regardless of priority order -- fired traces should match"
    );

    // allocation receipt identity: unequal
    assert_ne!(
        execution_a.receipt.allocation.priority_digest,
        execution_b.receipt.allocation.priority_digest
    );
    assert_ne!(execution_a.receipt.root, execution_b.receipt.root);

    // cross-verification: refuses
    let cross = verify_cmca_execution(&execution_a.receipt, &plan(), &request_b);
    assert_eq!(cross, Err(CmcaAllocationRefusal::PriorityDigestMismatch));
}

// ---------------------------------------------------------------------
// Fixture C -- process substitution must not permit receipt reuse.
// ---------------------------------------------------------------------

#[test]
fn fixture_c_process_substitution_refuses() {
    let request_1 = request(2, 1, 10, 100);
    let mut masses_2 = BTreeMap::new();
    masses_2.insert("act-x".to_string(), NonNegativeFixed::from_bits(1));
    masses_2.insert("act-y".to_string(), NonNegativeFixed::from_bits(10));
    masses_2.insert("act-z".to_string(), NonNegativeFixed::from_bits(100));
    let request_2 = CmcaExecutionRequest {
        profile: profile("BCINR_CMCA_PROFILE_V0_1", vec![1]),
        capacity: 2,
        masses: ProcessMassField(masses_2),
    };

    let allocation_1 = allocate_pddl_powl_plan(&plan(), &request_1).unwrap();
    let allocation_2 = allocate_pddl_powl_plan(&plan_2(), &request_2).unwrap();
    assert_ne!(
        allocation_1.process_digest, allocation_2.process_digest,
        "two processes with different action identities must digest differently, \
         even though both have the same arity (3) and topology (fully independent)"
    );

    let execution_1 = plan().execute_with_cmca(&request_1).unwrap();

    // Attempt to reuse process 1's receipt against process 2. Note: since
    // `request_1`'s masses are keyed by process 1's real labels
    // ("act-low"/"act-mid"/"act-high"), allocating against `plan_2()` with
    // `request_1` fails at the mass-mapping gate (`MissingActionMass`)
    // before ever reaching the digest-comparison step -- an equally valid
    // refusal of process substitution, just structurally earlier than
    // `ProcessDigestMismatch`. Arity/topology equality alone does not
    // permit reuse either way.
    let cross = verify_cmca_execution(&execution_1.receipt, &plan_2(), &request_1);
    assert!(
        cross.is_err(),
        "reusing process 1's allocation against process 2 must refuse -- got {cross:?}"
    );
    match cross {
        Err(CmcaAllocationRefusal::MissingActionMass { .. }) => {}
        other => panic!("expected MissingActionMass (request_1's labels don't exist on process 2), got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// Fixture D -- complete mapping: every production action requires exactly
// one admitted mass.
// ---------------------------------------------------------------------

#[test]
fn fixture_d_missing_mass_refuses_with_no_zero_fallback() {
    let complete = request(2, 1, 10, 100);
    allocate_pddl_powl_plan(&plan(), &complete)
        .expect("a complete mass field over 3 real actions must allocate");

    let mut incomplete_masses = complete.masses.0.clone();
    incomplete_masses.remove("act-mid");
    let incomplete = CmcaExecutionRequest {
        profile: complete.profile.clone(),
        capacity: complete.capacity,
        masses: ProcessMassField(incomplete_masses),
    };

    let result = allocate_pddl_powl_plan(&plan(), &incomplete);
    match result {
        Err(CmcaAllocationRefusal::MissingActionMass { action }) => {
            assert_eq!(action, "act-mid");
        }
        other => panic!(
            "a missing mass must refuse by name, never silently fall back to ZERO -- got {other:?}"
        ),
    }
}

// ---------------------------------------------------------------------
// Hostile falsifiers (1-5; falsifier 6 is Fixture B above).
// ---------------------------------------------------------------------

/// Falsifier 1 -- priority mutation: verify against a request whose masses
/// changed after the receipt was sealed. Must refuse even when the fired
/// trace happens not to change (capacity 3 admits everyone regardless).
#[test]
fn falsifier_priority_mutation_refuses_even_without_a_fired_trace_change() {
    let original = request(3, 1, 10, 100);
    let mutated = request(3, 1, 10, 999); // only act-high's mass changed

    let execution = plan().execute_with_cmca(&original).unwrap();
    let result = verify_cmca_execution(&execution.receipt, &plan(), &mutated);
    assert_eq!(result, Err(CmcaAllocationRefusal::PriorityDigestMismatch));
}

/// Falsifier 2 -- capacity mutation: seal with capacity 2, verify with
/// capacity 3. Must refuse before scheduler replay receives standing (the
/// combined digest check in `verify_cmca_execution` catches this before
/// any replay call is made).
#[test]
fn falsifier_capacity_mutation_refuses() {
    let sealed_with = request(2, 1, 10, 100);
    let verify_with = request(3, 1, 10, 100);

    let execution = plan().execute_with_cmca(&sealed_with).unwrap();
    let result = verify_cmca_execution(&execution.receipt, &plan(), &verify_with);
    assert!(
        result.is_err(),
        "capacity mutation must refuse -- got {result:?}"
    );
}

/// Falsifier 3 -- profile mutation: change only the profile identity while
/// preserving the numeric priority map (same masses, same lens schedule).
/// Semantic identity must not collapse into coincidentally equal output.
#[test]
fn falsifier_profile_identity_mutation_refuses() {
    let sealed_with = CmcaExecutionRequest {
        profile: profile("BCINR_CMCA_PROFILE_V0_1", vec![1]),
        capacity: 2,
        masses: masses(1, 10, 100),
    };
    let verify_with = CmcaExecutionRequest {
        profile: profile("BCINR_CMCA_PROFILE_V0_2", vec![1]), // identity only differs
        capacity: 2,
        masses: masses(1, 10, 100),
    };

    let execution = plan().execute_with_cmca(&sealed_with).unwrap();
    let result = verify_cmca_execution(&execution.receipt, &plan(), &verify_with);
    assert_eq!(result, Err(CmcaAllocationRefusal::ProfileDigestMismatch));
}

/// Falsifier 4 -- provenance mutation: redirect one POWL node to a
/// different (but still real) causal occurrence, leaving topology
/// unchanged. Per this checkpoint's spec, either a mapping refusal or a
/// process-digest failure is an acceptable outcome -- this test observes
/// and asserts the real one rather than presupposing which.
#[test]
fn falsifier_provenance_mutation_is_caught() {
    let mut mutated_plan = plan();
    let request = request(2, 1, 10, 100);

    let mut occurrence_ids: Vec<_> = mutated_plan
        .workflow
        .causal_plan
        .occurrences
        .iter()
        .map(|occurrence| occurrence.id)
        .collect();
    occurrence_ids.sort_by_key(|id| id.0);
    assert!(
        occurrence_ids.len() >= 2,
        "need at least 2 occurrences to swap"
    );

    // Redirect the first provenance entry (in node-id order) to a
    // DIFFERENT real occurrence than the one it originally pointed to.
    let mut provenance: Vec<_> = mutated_plan
        .workflow
        .powl_model
        .provenance
        .iter()
        .map(|(&node, &occurrence)| (node, occurrence))
        .collect();
    provenance.sort_by_key(|(node, _)| node.0);
    let (target_node, original_occurrence) = provenance[0];
    let redirect_to = occurrence_ids
        .iter()
        .copied()
        .find(|&id| id != original_occurrence)
        .expect("at least one other real occurrence exists");
    mutated_plan
        .workflow
        .powl_model
        .provenance
        .insert(target_node, redirect_to);

    let baseline = allocate_pddl_powl_plan(&plan(), &request);
    let mutated = allocate_pddl_powl_plan(&mutated_plan, &request);

    match (baseline, mutated) {
        (Ok(base), Ok(after)) => assert_ne!(
            base.process_digest, after.process_digest,
            "provenance mutation left the allocation observationally identical -- \
             the process digest must be sensitive to which occurrence each node maps to"
        ),
        (Ok(_), Err(_)) => {} // a mapping refusal is an equally acceptable outcome
        (Err(_), _) => panic!("baseline allocation must succeed on the unmutated plan"),
    }
}

/// Falsifier 5 -- synthetic-label substitution: a mass field keyed by
/// `CompiledPowlV2::node_labels`'s synthetic `"action-N"` placeholders
/// (confirmed synthetic by BCINR-CMCA-F) cannot satisfy the real-label
/// admitted-mass mapping.
#[test]
fn falsifier_synthetic_label_substitution_is_killed() {
    let plan = plan();
    let mut synthetic_masses = BTreeMap::new();
    for (index, &offset) in plan.compiled.node_labels.values().enumerate() {
        let label = plan.compiled.tape.label_slab.get(offset).to_string();
        synthetic_masses.insert(label, NonNegativeFixed::from_bits(10 + index as u32));
    }
    // Confirms the fixture is really using the synthetic namespace, not
    // accidentally the real one.
    assert!(synthetic_masses
        .keys()
        .any(|label| label.starts_with("action-")));

    let request = CmcaExecutionRequest {
        profile: profile("BCINR_CMCA_PROFILE_V0_1", vec![1]),
        capacity: 2,
        masses: ProcessMassField(synthetic_masses),
    };

    let result = allocate_pddl_powl_plan(&plan, &request);
    assert!(
        matches!(
            result,
            Err(CmcaAllocationRefusal::MissingActionMass { .. })
                | Err(CmcaAllocationRefusal::UnknownActionMass { .. })
        ),
        "synthetic node_labels must not satisfy the real-label mass mapping -- got {result:?}"
    );
}
