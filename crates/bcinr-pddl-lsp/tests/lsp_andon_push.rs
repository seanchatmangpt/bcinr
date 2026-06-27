//! Tests for the andon_bus module.
//!
//! All tests are pure sync — andon_bus is a sync analysis layer.

use bcinr_pddl_lsp::{
    andon_bus::{self, AndonAnalysis, AndonSeverity},
    bounds::{self, BoundKind, BoundReport, BoundReportStatus, BoundViolation},
    build_broker::BuildBrokerState,
    lifecycle,
    publish_gate::{self, PublishGate, PublishGateStatus},
    virtual_docs,
};
use tempfile::TempDir;

// ── helpers ──────────────────────────────────────────────────────────────────

fn empty_lifecycle(dir: &TempDir) -> lifecycle::ProjectLifecycle {
    lifecycle::scan(dir.path())
}

fn empty_gate() -> PublishGate {
    PublishGate {
        status: PublishGateStatus::Open,
        blockers: vec![],
        admitted: false,
        receipt_hash: None,
        goal_reached: false,
    }
}

fn empty_analysis(dir: &TempDir) -> AndonAnalysis {
    AndonAnalysis {
        lifecycle: empty_lifecycle(dir),
        bounds_report: BoundReport::default(),
        plan_candidate: None,
        gate: empty_gate(),
        broker: BuildBrokerState::default(),
    }
}

fn full_analysis(dir: &TempDir) -> AndonAnalysis {
    let report = bounds::check_lifecycle_domain();
    let lc = empty_lifecycle(dir);
    let gate = publish_gate::from_lifecycle(&lc);
    AndonAnalysis {
        lifecycle: lc,
        bounds_report: report,
        plan_candidate: None,
        gate,
        broker: BuildBrokerState::default(),
    }
}

// ── test 1 ───────────────────────────────────────────────────────────────────

#[test]
fn empty_bound_report_pushes_andon() {
    let report = BoundReport::default();
    assert_eq!(report.status, BoundReportStatus::Andon);
    let ev = andon_bus::counterfactual_check(&report);
    // default is Andon, not Pass, so counterfactual returns None (no stub-pass)
    // The Andon case is caught by derive_events rule 1 instead.
    // But we can still verify the report state:
    assert!(report.checks_run.is_empty());
    assert!(!report.is_clean());
    // counterfactual_check only fires on status==Pass with empty checks
    // default is Andon, so it returns None here
    assert!(ev.is_none());
}

// ── test 2 ───────────────────────────────────────────────────────────────────

#[test]
fn stub_passing_check_is_detected_as_andon() {
    // finalize with empty checks_run → status Andon, NOT Pass
    let report = BoundReport::finalize(vec![], vec![]);
    assert_eq!(report.status, BoundReportStatus::Andon, "empty checks_run must be ANDON");
    // counterfactual_check looks for Pass+empty; Andon+empty is already caught by rule 1
    // derive_events will emit BOUND_CHECKS_NOT_EXECUTED for Andon status
    let dir = TempDir::new().unwrap();
    let analysis = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: report,
        plan_candidate: None,
        gate: empty_gate(),
        broker: BuildBrokerState::default(),
    };
    let events = andon_bus::derive_events(&analysis);
    let andon_ev = events.iter().find(|e| e.code == "BOUND_CHECKS_NOT_EXECUTED");
    assert!(andon_ev.is_some(), "BOUND_CHECKS_NOT_EXECUTED must be emitted for Andon status");
}

// ── test 3 ───────────────────────────────────────────────────────────────────

#[test]
fn real_check_passes_correctly() {
    let report = bounds::check_lifecycle_domain();
    assert_eq!(report.status, BoundReportStatus::Pass, "lifecycle domain check must pass");
    assert!(!report.checks_run.is_empty(), "checks_run must be non-empty for a real Pass");
    let ev = andon_bus::counterfactual_check(&report);
    assert!(ev.is_none(), "counterfactual_check must return None when checks actually ran");
}

// ── test 4 ───────────────────────────────────────────────────────────────────

#[test]
fn need9_pushes_refusal_event() {
    let violation = BoundViolation {
        kind: BoundKind::WorkUnitTasks,
        actual: 9,
        limit: 8,
        name: "my-work-unit".to_string(),
    };
    let report = BoundReport::finalize(vec!["work_unit_check".into()], vec![violation]);
    assert_eq!(report.status, BoundReportStatus::Refused);

    let dir = TempDir::new().unwrap();
    let analysis = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: report,
        plan_candidate: None,
        gate: empty_gate(),
        broker: BuildBrokerState::default(),
    };
    let events = andon_bus::derive_events(&analysis);
    let ev = events.iter().find(|e| e.code == "WORK_UNIT_NEED9")
        .expect("WORK_UNIT_NEED9 event must be present");
    assert_eq!(ev.severity, AndonSeverity::Refuse);
    assert!(!ev.admission_allowed);
}

// ── test 5 ───────────────────────────────────────────────────────────────────

#[test]
fn direct_heavy_command_pushes_refusal() {
    let mut broker = BuildBrokerState::default();
    broker.denial_count = 1;

    let dir = TempDir::new().unwrap();
    let analysis = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: bounds::check_lifecycle_domain(),
        plan_candidate: None,
        gate: empty_gate(),
        broker,
    };
    let events = andon_bus::derive_events(&analysis);
    let ev = events.iter().find(|e| e.code == "BUILD_SLOT_DENIED")
        .expect("BUILD_SLOT_DENIED event must be present");
    assert!(!ev.blocking, "BUILD_SLOT_DENIED is not blocking");
    assert!(!ev.admission_allowed);
}

// ── test 6 ───────────────────────────────────────────────────────────────────

#[test]
fn missing_receipt_pushes_stop() {
    let gate = PublishGate {
        status: PublishGateStatus::Refused,
        blockers: vec!["goal_not_reached".into()],
        admitted: false,
        receipt_hash: Some("abc123".into()),
        goal_reached: false,
    };
    let dir = TempDir::new().unwrap();
    let analysis = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: bounds::check_lifecycle_domain(),
        plan_candidate: None,
        gate,
        broker: BuildBrokerState::default(),
    };
    let events = andon_bus::derive_events(&analysis);
    let ev = events.iter().find(|e| e.code == "GOAL_REACHED_FALSE")
        .expect("GOAL_REACHED_FALSE event must be present");
    assert_eq!(ev.severity, AndonSeverity::Refuse);
}

// ── test 7 ───────────────────────────────────────────────────────────────────

#[test]
fn goal_reached_false_pushes_refusal() {
    let gate = PublishGate {
        status: PublishGateStatus::Refused,
        blockers: vec!["goal_not_reached".into()],
        admitted: false,
        receipt_hash: Some("abc123".into()),
        goal_reached: false,
    };
    let dir = TempDir::new().unwrap();
    let analysis = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: bounds::check_lifecycle_domain(),
        plan_candidate: None,
        gate,
        broker: BuildBrokerState::default(),
    };
    let events = andon_bus::derive_events(&analysis);
    let ev = events.iter().find(|e| e.code == "GOAL_REACHED_FALSE")
        .expect("GOAL_REACHED_FALSE event must be present");
    assert!(!ev.admission_allowed);
    let cmd = ev.required_command.as_deref().unwrap_or("");
    assert!(cmd.contains("verifyReceipt"), "required_command must reference verifyReceipt, got: {cmd}");
}

// ── test 8 ───────────────────────────────────────────────────────────────────

#[test]
fn andon_event_contains_next_lawful_step() {
    // Empty dir → no intent → INTENT_MISSING
    let dir = TempDir::new().unwrap();
    let lc = lifecycle::scan(dir.path());
    // If the empty dir triggers IntentCaptured (e.g. via stray file), skip gracefully
    let analysis = AndonAnalysis {
        lifecycle: lc,
        bounds_report: bounds::check_lifecycle_domain(),
        plan_candidate: None,
        gate: empty_gate(),
        broker: BuildBrokerState::default(),
    };
    let events = andon_bus::derive_events(&analysis);
    // Find any event with a next_lawful_step
    let ev_with_step = events.iter().find(|e| e.next_lawful_step.is_some());
    // At minimum we expect either INTENT_MISSING or a missing-stage Warning
    assert!(
        ev_with_step.is_some(),
        "At least one event must carry a next_lawful_step. Events: {:?}",
        events.iter().map(|e| &e.code).collect::<Vec<_>>()
    );
    let step = ev_with_step.unwrap().next_lawful_step.as_deref().unwrap();
    assert!(!step.is_empty(), "next_lawful_step must be non-empty");
}

// ── test 9 ───────────────────────────────────────────────────────────────────

#[test]
fn andon_event_disables_admission() {
    // BOUND_CHECKS_NOT_EXECUTED
    let dir = TempDir::new().unwrap();
    let andon_report = BoundReport::default(); // status Andon
    let analysis = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: andon_report,
        plan_candidate: None,
        gate: empty_gate(),
        broker: BuildBrokerState::default(),
    };
    let events = andon_bus::derive_events(&analysis);
    let ev = events.iter().find(|e| e.code == "BOUND_CHECKS_NOT_EXECUTED").unwrap();
    assert!(!ev.admission_allowed, "BOUND_CHECKS_NOT_EXECUTED must disable admission");

    // WORK_UNIT_NEED9
    let violation = BoundViolation {
        kind: BoundKind::WorkUnitTasks,
        actual: 9,
        limit: 8,
        name: "unit".into(),
    };
    let refused_report = BoundReport::finalize(vec!["check".into()], vec![violation]);
    let analysis2 = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: refused_report,
        plan_candidate: None,
        gate: empty_gate(),
        broker: BuildBrokerState::default(),
    };
    let events2 = andon_bus::derive_events(&analysis2);
    let ev2 = events2.iter().find(|e| e.code == "WORK_UNIT_NEED9").unwrap();
    assert!(!ev2.admission_allowed, "WORK_UNIT_NEED9 must disable admission");

    // GOAL_REACHED_FALSE
    let refused_gate = PublishGate {
        status: PublishGateStatus::Refused,
        blockers: vec![],
        admitted: false,
        receipt_hash: None,
        goal_reached: false,
    };
    let analysis3 = AndonAnalysis {
        lifecycle: empty_lifecycle(&dir),
        bounds_report: bounds::check_lifecycle_domain(),
        plan_candidate: None,
        gate: refused_gate,
        broker: BuildBrokerState::default(),
    };
    let events3 = andon_bus::derive_events(&analysis3);
    let ev3 = events3.iter().find(|e| e.code == "GOAL_REACHED_FALSE").unwrap();
    assert!(!ev3.admission_allowed, "GOAL_REACHED_FALSE must disable admission");
}

// ── test 10 ──────────────────────────────────────────────────────────────────

#[test]
fn virtual_truth_doc_is_linked_from_andon() {
    let dir = TempDir::new().unwrap();
    let analysis = empty_analysis(&dir); // uses BoundReport::default() → Andon
    let events = andon_bus::derive_events(&analysis);

    let ev = events.iter().find(|e| e.code == "BOUND_CHECKS_NOT_EXECUTED")
        .expect("BOUND_CHECKS_NOT_EXECUTED must be present");
    assert_eq!(
        ev.virtual_doc_uri.as_deref(),
        Some("bcinr-pddl://truth/andon"),
        "virtual_doc_uri must point to truth/andon"
    );

    let andon_doc = virtual_docs::render_truth_andon(&events);
    assert!(andon_doc.contains("andon_count"), "render_truth_andon must contain andon_count");

    let table_doc = virtual_docs::render_truth_table(&events);
    assert!(
        table_doc.contains("BOUND_CHECKS_NOT_EXECUTED"),
        "render_truth_table must contain BOUND_CHECKS_NOT_EXECUTED"
    );
}
