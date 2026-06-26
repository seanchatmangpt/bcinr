//! LSP diagnostic emission for lifecycle and planning blockers.

use lsp_types_max::{Diagnostic, DiagnosticSeverity, Position, Range};
use crate::lifecycle::{LifecycleStage, ProjectLifecycle};
use crate::bounds::BoundViolation;
use crate::planner_client::PlannerError;

fn zero_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 0))
}

fn diag(code: &str, message: String, sev: DiagnosticSeverity) -> Diagnostic {
    Diagnostic {
        range: zero_range(),
        severity: Some(sev),
        code: Some(lsp_types_max::NumberOrString::String(code.to_string())),
        source: Some("bcinr-pddl-lsp".to_string()),
        message,
        ..Diagnostic::default()
    }
}

/// Convert missing lifecycle stages into LSP diagnostics.
pub fn lifecycle_diagnostics(lifecycle: &ProjectLifecycle) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for missing in &lifecycle.missing {
        let (code, msg, sev) = match missing {
            LifecycleStage::IntentCaptured => (
                "INTENT_MISSING",
                "No intent file found (README.md, CLAUDE.md, intent.md).".into(),
                DiagnosticSeverity::ERROR,
            ),
            LifecycleStage::PrdExists => (
                "PRD_MISSING",
                "No PRD file found (docs/prd.md). Create one to proceed.".into(),
                DiagnosticSeverity::ERROR,
            ),
            LifecycleStage::PrdAdmitted => (
                "PRD_NOT_ADMITTED",
                "PRD exists but is not admitted. Add 'ADMITTED' status marker.".into(),
                DiagnosticSeverity::WARNING,
            ),
            LifecycleStage::ArdExists => (
                "ARD_MISSING",
                "No ARD file found (docs/ard.md). Derive ARD from admitted PRD.".into(),
                DiagnosticSeverity::ERROR,
            ),
            LifecycleStage::ArdAdmitted => (
                "ARD_NOT_ADMITTED",
                "ARD exists but is not admitted. Add 'ADMITTED' status marker.".into(),
                DiagnosticSeverity::WARNING,
            ),
            LifecycleStage::WorkUnitsGenerated => (
                "WORK_UNITS_MISSING",
                "Work units not generated (docs/work-units.md). Generate from ARD.".into(),
                DiagnosticSeverity::INFORMATION,
            ),
            LifecycleStage::ImplementationComplete => (
                "IMPLEMENTATION_INCOMPLETE",
                "No source files found under src/ or crates/.".into(),
                DiagnosticSeverity::INFORMATION,
            ),
            LifecycleStage::TestsPassed => (
                "TESTS_NOT_PASSED",
                "No passing test report found (.bcinr/test-report.json).".into(),
                DiagnosticSeverity::WARNING,
            ),
            LifecycleStage::DocsProjected => (
                "DOCS_NOT_PROJECTED",
                "No projected docs found beyond PRD/ARD. Add documentation.".into(),
                DiagnosticSeverity::INFORMATION,
            ),
            LifecycleStage::ReleaseReady => (
                "RELEASE_NOT_READY",
                "No release artifact found (.bcinr/release.json, docs/publish.md).".into(),
                DiagnosticSeverity::INFORMATION,
            ),
            LifecycleStage::Published => (
                "PUBLISH_BLOCKED",
                "Publish goal not reached. Execute tape through bcinr-pddl to obtain receipt.".into(),
                DiagnosticSeverity::WARNING,
            ),
        };
        out.push(diag(code, msg, sev));
    }
    out
}

/// Convert bound violations into diagnostics.
pub fn bound_diagnostics(violations: &[BoundViolation]) -> Vec<Diagnostic> {
    violations.iter().map(|v| {
        let sev = if v.is_need9() {
            DiagnosticSeverity::ERROR
        } else {
            DiagnosticSeverity::WARNING
        };
        diag(v.diagnostic_code(), v.message(), sev)
    }).collect()
}

/// Convert planner errors into diagnostics.
pub fn planner_diagnostics(err: &PlannerError) -> Vec<Diagnostic> {
    let (code, sev) = match err {
        PlannerError::ParseError(_) => ("PDDL_PARSE_ERROR", DiagnosticSeverity::ERROR),
        PlannerError::GroundingError(_) => ("EMPTY_GROUNDING", DiagnosticSeverity::ERROR),
        PlannerError::NoAdmittedPlan => ("NO_ADMITTED_PLAN", DiagnosticSeverity::ERROR),
        PlannerError::ExecutionError(_) => ("STEP_DENIED", DiagnosticSeverity::ERROR),
    };
    vec![diag(code, err.to_string(), sev)]
}
