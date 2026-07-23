//! Andon bus — derive and classify lifecycle ANDON events.
//!
//! Pure sync. All derived from `AndonAnalysis` — no I/O, no async.

use lsp_types_max::{Diagnostic, DiagnosticSeverity, MessageType, Position, Range};

use crate::bounds::{BoundReport, BoundReportStatus};
use crate::build_broker::BuildBrokerState;
use crate::lifecycle::{LifecycleStage, ProjectLifecycle};
use crate::planner_client::{PlanCandidate, PlannerError};
use crate::publish_gate::{PublishGate, PublishGateStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum AndonSeverity {
    Info,
    Warning,
    Stop,
    Refuse,
}

#[derive(Debug, Clone)]
pub struct AndonEvent {
    pub id: String,
    pub severity: AndonSeverity,
    pub code: String,
    pub title: String,
    pub message: String,
    pub invariant_id: String,
    pub observed_state: String,
    pub expected_state: String,
    pub blocking: bool,
    pub requires_ack: bool,
    pub next_lawful_step: Option<String>,
    pub required_command: Option<String>,
    pub evidence_uri: Option<String>,
    pub virtual_doc_uri: Option<String>,
    pub receipt_required: bool,
    pub admission_allowed: bool,
}

impl AndonEvent {
    fn new(code: &str, severity: AndonSeverity, title: &str, message: &str) -> Self {
        Self {
            id: format!("andon:{}", code.to_lowercase().replace('_', "-")),
            severity,
            code: code.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            invariant_id: code.to_string(),
            observed_state: String::new(),
            expected_state: String::new(),
            blocking: false,
            requires_ack: false,
            next_lawful_step: None,
            required_command: None,
            evidence_uri: None,
            virtual_doc_uri: None,
            receipt_required: false,
            admission_allowed: true,
        }
    }
}

pub struct AndonAnalysis {
    pub lifecycle: ProjectLifecycle,
    pub bounds_report: BoundReport,
    pub plan_candidate: Option<Result<PlanCandidate, PlannerError>>,
    pub gate: PublishGate,
    pub broker: BuildBrokerState,
}

/// Derive all ANDON events from an analysis snapshot, in priority order.
pub fn derive_events(analysis: &AndonAnalysis) -> Vec<AndonEvent> {
    let mut events = Vec::new();

    // 1. Bounds ANDON (no checks ran)
    if analysis.bounds_report.status == BoundReportStatus::Andon {
        let mut ev = AndonEvent::new(
            "BOUND_CHECKS_NOT_EXECUTED",
            AndonSeverity::Stop,
            "Bound checks never ran",
            "No bound checks have executed. Status is ANDON — system cannot admit this state.",
        );
        ev.blocking = true;
        ev.requires_ack = true;
        ev.admission_allowed = false;
        ev.next_lawful_step = Some("implement_check_lifecycle_domain".into());
        ev.virtual_doc_uri = Some("bcinr-pddl://truth/andon".into());
        events.push(ev);
    }

    // 2. Bounds REFUSED — one event per violation
    if analysis.bounds_report.status == BoundReportStatus::Refused {
        for violation in &analysis.bounds_report.violations {
            let code = violation.diagnostic_code();
            let mut ev = AndonEvent::new(code, AndonSeverity::Refuse, code, &violation.message());
            ev.blocking = true;
            ev.admission_allowed = false;
            ev.next_lawful_step = Some("bcinrPddl.splitNeed9".into());
            events.push(ev);
        }
    }

    // 3. Publish gate REFUSED
    if analysis.gate.status == PublishGateStatus::Refused {
        let mut ev = AndonEvent::new(
            "GOAL_REACHED_FALSE",
            AndonSeverity::Refuse,
            "Goal not reached",
            "Receipt on disk has goal_reached=false. Plan must be re-executed after fixing blockers.",
        );
        ev.blocking = true;
        ev.admission_allowed = false;
        ev.required_command = Some("bcinrPddl.verifyReceipt".into());
        events.push(ev);
    }

    // 4. Build broker denials
    if analysis.broker.denial_count > 0 {
        let mut ev = AndonEvent::new(
            "BUILD_SLOT_DENIED",
            AndonSeverity::Refuse,
            "Build slot denied",
            &format!(
                "Build slot has been denied {} time(s). Release the slot before proceeding.",
                analysis.broker.denial_count
            ),
        );
        ev.blocking = false;
        ev.admission_allowed = false;
        ev.required_command = Some("bcinrPddl.releaseBuildSlot".into());
        events.push(ev);
    }

    // 5. Missing required lifecycle stages
    const REQUIRED_FOR_PUBLISH: &[LifecycleStage] = &[
        LifecycleStage::PrdAdmitted,
        LifecycleStage::ArdAdmitted,
        LifecycleStage::AdrRecorded,
        LifecycleStage::TestsPassed,
    ];
    for stage in REQUIRED_FOR_PUBLISH {
        if !analysis.lifecycle.has(stage) {
            let pred = stage.predicate_name();
            let code = format!("{}_MISSING", pred.to_uppercase());
            let stage_label = stage_to_label(stage);
            let mut ev = AndonEvent::new(
                &code,
                AndonSeverity::Warning,
                &format!("Lifecycle stage missing: {pred}"),
                &format!("Required stage '{pred}' is not present in the project lifecycle."),
            );
            ev.blocking = false;
            ev.admission_allowed = true;
            ev.next_lawful_step = Some(format!("bcinrPddl.create{stage_label}"));
            events.push(ev);
        }
    }

    // 6. No stages at all — INTENT_MISSING
    if analysis.lifecycle.true_stages.is_empty() && analysis.lifecycle.evidence.is_empty() {
        let mut ev = AndonEvent::new(
            "INTENT_MISSING",
            AndonSeverity::Info,
            "No intent captured",
            "Project has no lifecycle evidence. Create a README.md or docs/prd.md to begin.",
        );
        ev.blocking = false;
        ev.admission_allowed = true;
        ev.next_lawful_step = Some("Create README.md or docs/prd.md".into());
        events.push(ev);
    }

    events
}

/// Counterfactual check: was the PASS status earned honestly?
///
/// A PASS with empty checks_run is a stub-pass — emit ANDON.
pub fn counterfactual_check(report: &BoundReport) -> Option<AndonEvent> {
    if report.status == BoundReportStatus::Pass && report.checks_run.is_empty() {
        let mut ev = AndonEvent::new(
            "BOUND_CHECKS_NOT_EXECUTED",
            AndonSeverity::Stop,
            "Counterfactual: PASS with no checks",
            "Report shows PASS but checks_run is empty. This is a stub-pass — ANDON.",
        );
        ev.blocking = true;
        ev.requires_ack = true;
        ev.admission_allowed = false;
        ev.next_lawful_step = Some("implement_check_lifecycle_domain".into());
        ev.virtual_doc_uri = Some("bcinr-pddl://truth/andon".into());
        return Some(ev);
    }
    None
}

fn stage_to_label(stage: &LifecycleStage) -> &'static str {
    match stage {
        LifecycleStage::PrdAdmitted => "PrdAdmitted",
        LifecycleStage::ArdAdmitted => "ArdAdmitted",
        LifecycleStage::AdrRecorded => "AdrRecorded",
        LifecycleStage::TestsPassed => "TestsPassed",
        _ => "Stage",
    }
}

/// Map an `AndonEvent` to an LSP `Diagnostic`.
pub fn to_lsp_diagnostic(event: &AndonEvent) -> Diagnostic {
    let severity = match event.severity {
        AndonSeverity::Stop | AndonSeverity::Refuse => DiagnosticSeverity::ERROR,
        AndonSeverity::Warning => DiagnosticSeverity::WARNING,
        AndonSeverity::Info => DiagnosticSeverity::INFORMATION,
    };
    let zero = Position {
        line: 0,
        character: 0,
    };
    Diagnostic {
        range: Range {
            start: zero,
            end: zero,
        },
        severity: Some(severity),
        code: Some(lsp_types_max::NumberOrString::String(event.code.clone())),
        source: Some("bcinr-pddl-lsp".to_string()),
        message: event.message.clone(),
        related_information: None,
        tags: None,
        code_description: None,
        data: None,
    }
}

/// Map severity to LSP `MessageType`.
pub fn to_show_message_type(severity: &AndonSeverity) -> MessageType {
    match severity {
        AndonSeverity::Stop | AndonSeverity::Refuse => MessageType::ERROR,
        AndonSeverity::Warning => MessageType::WARNING,
        AndonSeverity::Info => MessageType::INFO,
    }
}

/// Notification method for the given event.
pub fn notification_method(event: &AndonEvent) -> &'static str {
    if event.code.contains("COUNTERFACTUAL") {
        return "bcinrPddl/counterfactualFailed";
    }
    match event.code.as_str() {
        "GOAL_REACHED_FALSE" => "bcinrPddl/publishGateBlocked",
        "BUILD_SLOT_DENIED" => "bcinrPddl/buildSlotDenied",
        _ if matches!(event.severity, AndonSeverity::Stop | AndonSeverity::Refuse)
            && event.blocking =>
        {
            "bcinrPddl/andonRaised"
        }
        _ if matches!(event.severity, AndonSeverity::Info) => "bcinrPddl/truthTableChanged",
        _ => "bcinrPddl/andonRaised",
    }
}
