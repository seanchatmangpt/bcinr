//! Diagnostics — lifecycle, bounds, planner, process violations → LSP diagnostic codes.

use crate::bounds::{BoundReport, BoundViolation};
use crate::build_broker::{BrokerDenial, DirectCommandViolation};
use crate::lifecycle::{LifecycleStage, ProjectLifecycle};
use lsp_types_max::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

fn make_diag(code: &str, message: String, severity: DiagnosticSeverity) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        },
        severity: Some(severity),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some("bcinr-pddl-lsp".to_string()),
        message,
        ..Diagnostic::default()
    }
}

/// Lifecycle diagnostics from missing stages.
pub fn lifecycle_diagnostics(lifecycle: &ProjectLifecycle) -> Vec<Diagnostic> {
    lifecycle
        .missing
        .iter()
        .map(|stage| {
            let (code, msg, severity) = stage_diagnostic(stage);
            make_diag(code, msg, severity)
        })
        .collect()
}

fn stage_diagnostic(stage: &LifecycleStage) -> (&'static str, String, DiagnosticSeverity) {
    match stage {
        LifecycleStage::IntentCaptured => (
            "INTENT_MISSING",
            "No intent file found (README.md, CLAUDE.md, intent.md). Create one to start.".into(),
            DiagnosticSeverity::ERROR,
        ),
        LifecycleStage::PrdExists => (
            "PRD_MISSING",
            "No PRD found (docs/prd.md). Create a Product Requirements Document.".into(),
            DiagnosticSeverity::ERROR,
        ),
        LifecycleStage::PrdAdmitted => (
            "PRD_NOT_ADMITTED",
            "PRD exists but is not admitted. Add 'ADMITTED' status marker.".into(),
            DiagnosticSeverity::WARNING,
        ),
        LifecycleStage::ArdExists => (
            "ARD_MISSING",
            "No ARD found (docs/ard.md). Derive Architecture Requirements Document from admitted PRD.".into(),
            DiagnosticSeverity::ERROR,
        ),
        LifecycleStage::ArdAdmitted => (
            "ARD_NOT_ADMITTED",
            "ARD exists but is not admitted — next lawful action is admit_ard, not derive_ard. Add 'ADMITTED' marker.".into(),
            DiagnosticSeverity::WARNING,
        ),
        LifecycleStage::AdrRecorded => (
            "ADR_MISSING",
            "No Architecture Decision Records found (docs/adr/*.md). Record at least one decision.".into(),
            DiagnosticSeverity::WARNING,
        ),
        LifecycleStage::WorkUnitsGenerated => (
            "WORK_UNITS_MISSING",
            "Work units not generated (docs/work-units.md). Generate from admitted ARD + ADR.".into(),
            DiagnosticSeverity::INFORMATION,
        ),
        LifecycleStage::ImplementationComplete => (
            "IMPLEMENTATION_INCOMPLETE",
            "No source files found under src/ or crates/. Implement work units.".into(),
            DiagnosticSeverity::INFORMATION,
        ),
        LifecycleStage::TestsPassed => (
            "TESTS_NOT_PASSED",
            "No passing test report found (.bcinr/test-report.json). Run tests through build broker.".into(),
            DiagnosticSeverity::WARNING,
        ),
        LifecycleStage::DocsProjected => (
            "DOCS_NOT_PROJECTED",
            "No projected docs found beyond PRD/ARD. Add architectural or user documentation.".into(),
            DiagnosticSeverity::INFORMATION,
        ),
        LifecycleStage::ReleaseReady => (
            "RELEASE_NOT_READY",
            "No release artifact found (.bcinr/release.json or docs/publish.md).".into(),
            DiagnosticSeverity::INFORMATION,
        ),
        LifecycleStage::Published => (
            "PUBLISH_BLOCKED",
            "Published stage not reached. Run bcinrPddl.executeTape to admit and emit receipt.".into(),
            DiagnosticSeverity::WARNING,
        ),
    }
}

/// Bounds diagnostics from BoundReport.
pub fn bound_diagnostics(report: &BoundReport) -> Vec<Diagnostic> {
    report
        .violations
        .iter()
        .map(|v| {
            let severity = bound_severity(v);
            make_diag(v.diagnostic_code(), v.message(), severity)
        })
        .collect()
}

fn bound_severity(v: &BoundViolation) -> DiagnosticSeverity {
    use crate::bounds::BoundKind;
    match v.kind {
        BoundKind::WorkUnitTasks | BoundKind::BuildConcurrency | BoundKind::ResourceEnvelope => {
            DiagnosticSeverity::ERROR
        }
        BoundKind::ActionParameters
        | BoundKind::ActionPreconditions
        | BoundKind::ActionEffects
        | BoundKind::GoalAtoms
        | BoundKind::PredicateArity
        | BoundKind::PlanDepth
        | BoundKind::GroundActions => DiagnosticSeverity::WARNING,
    }
}

/// Planner diagnostics from error strings.
pub fn planner_error_diagnostic(code: &str, msg: &str) -> Diagnostic {
    make_diag(code, msg.to_string(), DiagnosticSeverity::ERROR)
}

/// Build broker denial diagnostic.
pub fn broker_denial_diagnostic(denial: &BrokerDenial) -> Diagnostic {
    make_diag(
        "BUILD_SLOT_DENIED",
        format!(
            "Build slot denied for '{}': {}",
            denial.command, denial.reason
        ),
        DiagnosticSeverity::ERROR,
    )
}

/// Direct heavy command blocked diagnostic.
pub fn direct_command_diagnostic(violation: &DirectCommandViolation) -> Diagnostic {
    make_diag(
        violation.diagnostic_code(),
        violation.message(),
        DiagnosticSeverity::ERROR,
    )
}

/// OCEL trace missing after admission.
pub fn ocel_missing_diagnostic() -> Diagnostic {
    make_diag(
        "OCEL_TRACE_MISSING",
        "OCEL trace not found after admission. Run bcinrPddl.executeTape and emit OCEL.".into(),
        DiagnosticSeverity::WARNING,
    )
}

/// Receipt integrity error — file present but content invalid.
pub fn receipt_integrity_diagnostic(detail: &str) -> Diagnostic {
    make_diag(
        "RECEIPT_INTEGRITY_ERROR",
        format!("Receipt integrity error: {detail}. Publish gate is REFUSED."),
        DiagnosticSeverity::ERROR,
    )
}

/// Convenience: check lifecycle and emit diagnostics for all missing lifecycle stages.
pub fn check_lifecycle(report: &BoundReport) -> Vec<Diagnostic> {
    bound_diagnostics(report)
}
