use std::collections::BTreeSet;

use bcinr_pddl::{
    run_all_manufactured_scenarios, run_manufactured_scenario, SwarmStanding,
    SwarmValidationError, MANUFACTURED_SCENARIOS,
};

#[test]
fn all_ten_ggen_manufactured_scenarios_execute_and_verify() {
    let receipts = run_all_manufactured_scenarios().expect("all manufactured scenarios must run");
    assert_eq!(receipts.len(), 10);
    assert_eq!(receipts.len(), MANUFACTURED_SCENARIOS.len());

    let ids: BTreeSet<_> = receipts
        .iter()
        .map(|receipt| receipt.descriptor.id)
        .collect();
    assert_eq!(ids.len(), 10);

    for receipt in &receipts {
        receipt.verify().expect("scenario receipt must replay");
        assert!(!receipt.receipt_root.is_empty());
    }
}

#[test]
fn manufactured_expected_standing_is_preserved() {
    let deadline = run_manufactured_scenario("deadline_aware_verification")
        .expect("deadline scenario must execute");
    let adversarial =
        run_manufactured_scenario("adversarial_worker").expect("adversarial scenario must execute");
    let parallel = run_manufactured_scenario("parallel_software_delivery")
        .expect("parallel scenario must execute");

    assert_eq!(deadline.standing, SwarmStanding::Blocked);
    assert_eq!(adversarial.standing, SwarmStanding::Refused);
    assert_eq!(parallel.standing, SwarmStanding::Alive);
}

#[test]
fn receipt_tampering_is_refused() {
    let mut receipt = run_manufactured_scenario("long_running_supervision")
        .expect("supervision scenario must execute");
    receipt.events[0].logical_time_ms += 1;

    assert!(matches!(
        receipt.verify(),
        Err(SwarmValidationError::EventOrder { .. })
            | Err(SwarmValidationError::ReceiptMismatch)
    ));
}

#[test]
fn ggen_json_vectors_match_the_compiled_scenario_inventory() {
    let raw = include_str!("../../../contracts/v26.7.28/conformance_vectors.json");
    let vectors: serde_json::Value = serde_json::from_str(raw).expect("vectors must be valid JSON");

    assert_eq!(vectors["runtime_version"], "26.7.28");
    assert_eq!(vectors["llm_calls"], 0);
    let scenarios = vectors["scenarios"]
        .as_array()
        .expect("scenarios must be an array");
    assert_eq!(scenarios.len(), MANUFACTURED_SCENARIOS.len());

    for descriptor in MANUFACTURED_SCENARIOS {
        let vector = scenarios
            .iter()
            .find(|value| value["id"] == descriptor.id)
            .expect("every compiled descriptor must have a JSON vector");
        assert_eq!(vector["workers"], descriptor.workers);
        assert_eq!(
            vector["requires_concurrency"],
            descriptor.requires_concurrency
        );
        assert_eq!(
            vector["requires_substitution"],
            descriptor.requires_substitution
        );
        assert_eq!(
            vector["requires_speculation"],
            descriptor.requires_speculation
        );
        assert_eq!(
            vector["requires_human_approval"],
            descriptor.requires_human_approval
        );
        assert_eq!(vector["expected_standing"], descriptor.expected_standing);
    }
}
