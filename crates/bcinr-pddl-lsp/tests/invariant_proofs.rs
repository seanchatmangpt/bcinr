//! Invariant proof tests — TRUE / FALSE / COUNTERFACTUAL / WITNESS / REPAIR.
//!
//! Each test verifies one invariant record. A record is ADMITTED only when:
//!   true_case = PASS
//!   false_case = blocking (REFUSE/STOP)
//!   counterfactuals: at least one, all blocking
//!   witness: present
//!   repair: present
//!   verdict: PASS
//!
//! UNKNOWN is ANDON. Missing witness is ANDON. Missing repair is incomplete.

use std::{fs, path::PathBuf};
use tempfile::TempDir;

use bcinr_pddl_lsp::{
    bounds,
    invariants::{
        self, CounterfactualMutation, CounterfactualProbe, InvariantVerdict, ProbeOutcome,
        ProbeResult, RepairAction, Witness,
    },
    lifecycle,
};

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

// ── T1. PRD admission ─────────────────────────────────────────────────────────

#[test]
fn invariant_prd_admission_true_case() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "README.md", "intent");
    write_file(&dir, "docs/prd.md", "# PRD\n## Status: ADMITTED");
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "prd_admission").unwrap();
    // TRUE: PRD admitted → true_case = PASS
    assert_eq!(record.true_case.outcome, ProbeOutcome::Pass,
        "TRUE case failed: {:?}", record.true_case.observed);
}

#[test]
fn invariant_prd_admission_false_case() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "README.md", "intent");
    write_file(&dir, "docs/prd.md", "# PRD\n## Status: CANDIDATE"); // no ADMITTED
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "prd_admission").unwrap();
    // FALSE: PRD without ADMITTED marker → false_case blocks
    assert!(record.false_case.outcome.is_blocking(),
        "FALSE case must block: {:?}", record.false_case.outcome);
}

#[test]
fn invariant_prd_admission_counterfactual_defined() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "prd_admission").unwrap();
    assert!(!record.counterfactuals.is_empty(),
        "COUNTERFACTUAL: at least one CF must be defined");
    assert!(record.witness.is_some(), "WITNESS must be present");
    assert!(record.repair.is_some(), "REPAIR must be present");
}

// ── T2. Need9 invariant ────────────────────────────────────────────────────────

#[test]
fn invariant_need9_true_false_cf() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "need9_bound").unwrap();

    // TRUE: 8 tasks pass
    assert_eq!(record.true_case.outcome, ProbeOutcome::Pass,
        "TRUE(8 tasks) failed: {:?}", record.true_case.observed);

    // FALSE: 9 tasks refuse
    assert!(record.false_case.outcome.is_blocking(),
        "FALSE(9 tasks) must block: {:?}", record.false_case.outcome);

    // COUNTERFACTUAL: boundary mutation defined
    let boundary_cf = record.counterfactuals.iter()
        .find(|cf| cf.mutation == CounterfactualMutation::BoundaryMutation);
    assert!(boundary_cf.is_some(), "BoundaryMutation counterfactual required");
    assert!(boundary_cf.unwrap().result.outcome.is_blocking(),
        "CF must block: got {:?}", boundary_cf.unwrap().result.outcome);

    // WITNESS and REPAIR
    assert!(record.witness.is_some(), "WITNESS required");
    assert!(record.repair.is_some(), "REPAIR required");
}

#[test]
fn invariant_need9_verdict_is_pass() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let report = bounds::check_lifecycle_domain();
    let record = invariants::check_all(&lc, &report)
        .into_iter().find(|r| r.id == "need9_bound").unwrap();
    assert_eq!(record.verdict, InvariantVerdict::Pass,
        "Need9 invariant must be PASS, got {:?}", record.verdict);
    assert!(record.is_admitted(), "Need9 invariant must be fully admitted");
}

// ── T3. Bound checks ran ──────────────────────────────────────────────────────

#[test]
fn invariant_bound_checks_stub_detection() {
    // A default (empty) BoundReport must produce STOP, not PASS
    let empty_report = bounds::BoundReport::default();
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &empty_report)
        .into_iter().find(|r| r.id == "bound_checks_ran").unwrap();

    assert!(record.true_case.outcome.is_blocking(),
        "Empty report true_case must be STOP/ANDON, got {:?}", record.true_case.outcome);
    assert_ne!(record.verdict, InvariantVerdict::Pass,
        "Empty report must not produce PASS verdict");
    // Counterfactual must be DISABLE_CHECKER
    let disable_cf = record.counterfactuals.iter()
        .find(|cf| cf.mutation == CounterfactualMutation::DisableChecker);
    assert!(disable_cf.is_some(), "DisableChecker counterfactual required");
}

#[test]
fn invariant_bound_checks_real_run_is_pass() {
    let real_report = bounds::check_lifecycle_domain();
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &real_report)
        .into_iter().find(|r| r.id == "bound_checks_ran").unwrap();

    assert_eq!(record.true_case.outcome, ProbeOutcome::Pass,
        "Real check must pass: {:?}", record.true_case.observed);
    // Witness must point to a non-empty summary
    assert!(record.witness.is_some(), "WITNESS required");
    let w = record.witness.unwrap();
    assert!(w.summary.as_deref().unwrap_or("").contains("checks_run="),
        "WITNESS summary must include checks_run count: {:?}", w.summary);
}

// ── T4. Publish requires receipt ──────────────────────────────────────────────

#[test]
fn invariant_publish_false_case_no_receipt() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path()); // no receipt file
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "publish_requires_receipt").unwrap();

    // FALSE: no receipt → correctly blocked
    assert!(record.false_case.outcome.is_blocking(),
        "FALSE(no receipt) must block: {:?}", record.false_case.outcome);
}

#[test]
fn invariant_publish_counterfactuals_corrupt_and_remove() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "publish_requires_receipt").unwrap();

    // Must have both CorruptEvidence and RemoveEvidence CFs
    let has_corrupt = record.counterfactuals.iter()
        .any(|cf| cf.mutation == CounterfactualMutation::CorruptEvidence);
    let has_remove = record.counterfactuals.iter()
        .any(|cf| cf.mutation == CounterfactualMutation::RemoveEvidence);
    assert!(has_corrupt, "CorruptEvidence counterfactual required (flip goal_reached)");
    assert!(has_remove, "RemoveEvidence counterfactual required (delete receipt)");

    // Both must be blocking
    for cf in &record.counterfactuals {
        assert!(cf.result.outcome.is_blocking(),
            "CF '{}' must block, got {:?}", cf.description, cf.result.outcome);
    }

    // REPAIR must point to executeTape
    let repair = record.repair.as_ref().expect("REPAIR required");
    assert!(repair.command.as_deref().unwrap_or("").contains("executeTape"),
        "REPAIR must reference executeTape: {:?}", repair.command);
}

// ── T5. Candidate ≠ Admitted ──────────────────────────────────────────────────

#[test]
fn invariant_candidate_not_admitted_has_authority_swap_cf() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let record = invariants::check_all(&lc, &bounds::check_lifecycle_domain())
        .into_iter().find(|r| r.id == "candidate_not_admitted").unwrap();

    // TRUE: candidate produced without receipt = PASS
    assert_eq!(record.true_case.outcome, ProbeOutcome::Pass,
        "TRUE case failed: {:?}", record.true_case.observed);

    // COUNTERFACTUAL: AuthoritySwap
    let auth_cf = record.counterfactuals.iter()
        .find(|cf| cf.mutation == CounterfactualMutation::AuthoritySwap);
    assert!(auth_cf.is_some(), "AuthoritySwap counterfactual required");
    assert!(auth_cf.unwrap().result.outcome.is_blocking(),
        "AuthoritySwap must block: got {:?}", auth_cf.unwrap().result.outcome);

    // REPAIR points to executeTape
    let repair = record.repair.as_ref().expect("REPAIR required");
    assert!(repair.command.as_deref().unwrap_or("").contains("executeTape"),
        "REPAIR must reference executeTape");
}

// ── T6. Full table passes check_all ──────────────────────────────────────────

#[test]
fn all_invariants_have_witness_and_repair() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let report = bounds::check_lifecycle_domain();
    let records = invariants::check_all(&lc, &report);

    assert!(!records.is_empty(), "check_all must return at least one invariant");
    for r in &records {
        assert!(r.witness.is_some(),
            "Invariant '{}' missing WITNESS — not admitted", r.id);
        assert!(r.repair.is_some(),
            "Invariant '{}' missing REPAIR — not admitted", r.id);
        assert!(!r.counterfactuals.is_empty(),
            "Invariant '{}' has no COUNTERFACTUAL probes — not admitted", r.id);
    }
}

#[test]
fn render_invariant_table_is_valid_json() {
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    let report = bounds::check_lifecycle_domain();
    let records = invariants::check_all(&lc, &report);
    let json = invariants::render_invariant_table(&records);
    let parsed: serde_json::Value = serde_json::from_str(&json)
        .expect("render_invariant_table must produce valid JSON");
    let arr = parsed.as_array().expect("must be a JSON array");
    assert_eq!(arr.len(), records.len());
    // Each entry must have the five-part structure
    for entry in arr {
        assert!(entry.get("true_case").is_some(), "must have true_case");
        assert!(entry.get("false_case").is_some(), "must have false_case");
        assert!(entry.get("counterfactuals").is_some(), "must have counterfactuals");
        assert!(entry.get("witness_present").is_some(), "must have witness_present");
        assert!(entry.get("repair_present").is_some(), "must have repair_present");
        assert!(entry.get("verdict").is_some(), "must have verdict");
    }
}
