//! Integration tests for bcinr-compliance.

use bcinr_compliance::{generate_report, ComplianceStandard};

/// A minimal WorldManufactureReceipt JSON that exercises every field we map.
fn mock_receipt(admitted: bool, step_count: u64, goal_reached: bool) -> String {
    // Compute a valid manufacture_chain for the given witnesses so chain tests pass.
    let domain_w = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";
    let problem_w = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let plan_chain = "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe";
    let mut h = blake3::Hasher::new();
    h.update(domain_w.as_bytes());
    h.update(problem_w.as_bytes());
    h.update(plan_chain.as_bytes());
    let manufacture_chain: String = h
        .finalize()
        .as_bytes()
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect();

    // Build plan_steps array with `step_count` entries.
    let steps: Vec<String> = (0..step_count)
        .map(|i| {
            format!(
                r#"{{"action_name":"move_{i}","start_time":{i}.0,"duration":1.0}}"#,
                i = i
            )
        })
        .collect();
    let steps_json = steps.join(",");

    format!(
        r#"{{
            "ok": true,
            "domain_name": "logistics",
            "problem_name": "p1",
            "domain_witness": "{domain_w}",
            "problem_witness": "{problem_w}",
            "manufacture_chain": "{manufacture_chain}",
            "plan_chain_hash": "{plan_chain}",
            "admitted": {admitted},
            "step_count": {step_count},
            "makespan": {makespan},
            "goal_reached": {goal_reached},
            "refusal_reason": null,
            "plan_steps": [{steps_json}]
        }}"#,
        admitted = admitted,
        step_count = step_count,
        makespan = step_count as f64,
        goal_reached = goal_reached,
        steps_json = steps_json,
        domain_w = domain_w,
        problem_w = problem_w,
        manufacture_chain = manufacture_chain,
        plan_chain = plan_chain,
    )
}

// ── EU AI Act ────────────────────────────────────────────────────────────────

#[test]
fn eu_ai_act_report_is_populated() {
    let receipt = mock_receipt(true, 3, true);
    let report = generate_report(&receipt, ComplianceStandard::EuAiAct, &[]).unwrap();

    let eu = report.eu_ai_act.expect("EuAiActReport must be present");
    let t = &eu.article_13_transparency;

    assert!(!t.audit_trail_hash.is_empty(), "audit_trail_hash must not be empty");
    assert_eq!(t.step_count, 3);
    assert!(t.human_oversight_gate_passed);
    assert!(t.goal_achieved);
    assert!(t.chain_integrity_valid, "BLAKE3 chain must be valid for mock receipt");
    assert!(t.refusal_reason.is_none());
    assert!(t.system_description.contains("logistics"));
    assert!(!eu.determinism_evidence.is_empty());
}

#[test]
fn eu_ai_act_report_nist_and_soc2_absent() {
    let receipt = mock_receipt(true, 1, true);
    let report = generate_report(&receipt, ComplianceStandard::EuAiAct, &[]).unwrap();
    assert!(report.nist_ai_rmf.is_none());
    assert!(report.soc2.is_none());
}

// ── NIST AI RMF ──────────────────────────────────────────────────────────────

#[test]
fn nist_rmf_report_is_populated() {
    let receipt = mock_receipt(true, 5, true);
    let report = generate_report(&receipt, ComplianceStandard::NistAiRmf, &[]).unwrap();

    let nist = report.nist_ai_rmf.expect("NistAiRmfReport must be present");

    assert_eq!(nist.map_.domain_name, "logistics");
    assert_eq!(nist.map_.step_count, 5);
    assert!(nist.map_.makespan > 0.0);
    assert!(nist.measure.goal_reached);
    assert!(!nist.manage.domain_witness.is_empty());
    assert!(!nist.manage.manufacture_chain.is_empty());
}

// ── SOC2 CC6 ─────────────────────────────────────────────────────────────────

#[test]
fn soc2_report_is_populated() {
    let receipt = mock_receipt(true, 4, true);
    let report = generate_report(&receipt, ComplianceStandard::Soc2, &[]).unwrap();

    let soc2 = report.soc2.expect("Soc2Report must be present");

    assert!(soc2.cc6_1_logical_access);
    assert!(!soc2.cc6_2_domain_credential.is_empty());
    assert_eq!(soc2.cc6_3_action_count, 4);
    assert!(soc2.cc6_6_refusal_code.is_none());
    assert!(soc2.cc6_7_chain_valid);
    assert!(!soc2.cc6_8_step_chain_hash.is_empty());
}

#[test]
fn soc2_chain_invalid_when_manufacture_chain_tampered() {
    // Use a receipt with a deliberately wrong manufacture_chain.
    let receipt = r#"{
        "ok": true, "domain_name": "d", "problem_name": "p",
        "domain_witness": "aa", "problem_witness": "bb",
        "manufacture_chain": "0000000000000000000000000000000000000000000000000000000000000000",
        "plan_chain_hash": "cc",
        "admitted": true, "step_count": 1, "makespan": 1.0,
        "goal_reached": true, "refusal_reason": null, "plan_steps": []
    }"#;
    let report = generate_report(receipt, ComplianceStandard::Soc2, &[]).unwrap();
    let soc2 = report.soc2.unwrap();
    assert!(!soc2.cc6_7_chain_valid, "tampered chain must be detected");
}

// ── Policy bundles ────────────────────────────────────────────────────────────

#[test]
fn gdpr_policy_passes_for_small_plan() {
    let receipt = mock_receipt(true, 5, true);
    let report =
        generate_report(&receipt, ComplianceStandard::EuAiAct, &["gdpr_data_minimisation"])
            .unwrap();
    assert_eq!(report.policy_verdicts.len(), 1);
    let v = &report.policy_verdicts[0];
    assert_eq!(v.policy_name, "gdpr_data_minimisation");
    assert!(v.passed, "GDPR policy must pass for a 5-step admitted plan: {}", v.evidence);
}

#[test]
fn gdpr_policy_fails_for_large_plan() {
    let receipt = mock_receipt(true, 25, true);
    let report =
        generate_report(&receipt, ComplianceStandard::EuAiAct, &["gdpr_data_minimisation"])
            .unwrap();
    let v = &report.policy_verdicts[0];
    assert!(!v.passed, "25-step plan exceeds GDPR minimisation limit");
}

#[test]
fn hipaa_policy_passes_for_valid_receipt() {
    let receipt = mock_receipt(true, 3, true);
    let report =
        generate_report(&receipt, ComplianceStandard::NistAiRmf, &["hipaa_phi_handling"])
            .unwrap();
    let v = &report.policy_verdicts[0];
    assert!(v.passed, "HIPAA policy must pass for admitted+intact receipt: {}", v.evidence);
}

#[test]
fn financial_policy_passes_for_valid_receipt() {
    let receipt = mock_receipt(true, 3, true);
    let report = generate_report(
        &receipt,
        ComplianceStandard::Soc2,
        &["financial_best_execution"],
    )
    .unwrap();
    let v = &report.policy_verdicts[0];
    assert!(v.passed, "financial policy must pass: {}", v.evidence);
}

#[test]
fn unknown_policy_returns_error() {
    let receipt = mock_receipt(true, 1, true);
    let err = generate_report(&receipt, ComplianceStandard::EuAiAct, &["nonexistent_policy"]);
    assert!(err.is_err());
}

#[test]
fn multiple_policies_evaluated_independently() {
    let receipt = mock_receipt(true, 3, true);
    let report = generate_report(
        &receipt,
        ComplianceStandard::EuAiAct,
        &["gdpr_data_minimisation", "hipaa_phi_handling"],
    )
    .unwrap();
    assert_eq!(report.policy_verdicts.len(), 2);
    assert_eq!(report.policy_verdicts[0].policy_name, "gdpr_data_minimisation");
    assert_eq!(report.policy_verdicts[1].policy_name, "hipaa_phi_handling");
}

// ── receipt_hash and generated_at ───────────────────────────────────────────

#[test]
fn receipt_hash_is_manufacture_chain() {
    let receipt = mock_receipt(true, 2, true);
    let data: serde_json::Value = serde_json::from_str(&receipt).unwrap();
    let expected = data["manufacture_chain"].as_str().unwrap();
    let report = generate_report(&receipt, ComplianceStandard::EuAiAct, &[]).unwrap();
    assert_eq!(report.receipt_hash, expected);
}

#[test]
fn generated_at_is_non_empty_iso8601() {
    let receipt = mock_receipt(true, 1, true);
    let report = generate_report(&receipt, ComplianceStandard::EuAiAct, &[]).unwrap();
    assert!(!report.generated_at.is_empty());
    assert!(report.generated_at.contains('T'), "should look like ISO-8601: {}", report.generated_at);
}

// ── Invalid JSON ─────────────────────────────────────────────────────────────

#[test]
fn invalid_json_returns_error() {
    let err = generate_report("not-json", ComplianceStandard::EuAiAct, &[]);
    assert!(err.is_err());
    assert!(matches!(err, Err(bcinr_compliance::ComplianceError::InvalidJson(_))));
}
