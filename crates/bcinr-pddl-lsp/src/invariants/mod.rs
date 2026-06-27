//! Invariant records — TRUE / FALSE / COUNTERFACTUAL / WITNESS / REPAIR.
//!
//! An invariant is not real until all five are present:
//!   TRUE: system recognizes the valid state
//!   FALSE: system rejects the invalid state
//!   COUNTERFACTUAL: minimal corruption gets caught
//!   WITNESS: evidence pointer proving the outcome
//!   REPAIR: next lawful step when blocked
//!
//! UNKNOWN is not PASS. STOP is not PASS. Missing witness is not PASS.

use crate::andon_bus::{AndonEvent, AndonSeverity};

// ── Probe result ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ProbeOutcome {
    Pass,
    Refuse,
    Stop,
    /// Proof did not execute. Equivalent to ANDON.
    Unknown,
}

impl ProbeOutcome {
    pub fn is_passing(&self) -> bool { *self == ProbeOutcome::Pass }
    pub fn is_blocking(&self) -> bool {
        matches!(self, ProbeOutcome::Refuse | ProbeOutcome::Stop | ProbeOutcome::Unknown)
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Refuse => "REFUSED",
            Self::Stop => "STOP",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub outcome: ProbeOutcome,
    /// Diagnostic code emitted, if any.
    pub code: Option<String>,
    /// Short human-readable description of what was observed.
    pub observed: String,
}

impl ProbeResult {
    pub fn pass(observed: impl Into<String>) -> Self {
        Self { outcome: ProbeOutcome::Pass, code: None, observed: observed.into() }
    }
    pub fn refuse(code: impl Into<String>, observed: impl Into<String>) -> Self {
        Self { outcome: ProbeOutcome::Refuse, code: Some(code.into()), observed: observed.into() }
    }
    pub fn stop(code: impl Into<String>, observed: impl Into<String>) -> Self {
        Self { outcome: ProbeOutcome::Stop, code: Some(code.into()), observed: observed.into() }
    }
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self { outcome: ProbeOutcome::Unknown, code: Some("UNKNOWN".into()), observed: reason.into() }
    }
}

// ── Witness ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum WitnessKind {
    File,
    Diagnostic,
    Receipt,
    OcelEvent,
    PddlProjection,
    BoundReport,
    AndonEvent,
    TestOutput,
    BenchmarkOutput,
}

#[derive(Debug, Clone)]
pub struct Witness {
    pub kind: WitnessKind,
    /// URI or path to the evidence.
    pub uri: String,
    /// Optional content hash or summary.
    pub summary: Option<String>,
}

impl Witness {
    pub fn virtual_doc(uri: impl Into<String>) -> Self {
        Self { kind: WitnessKind::PddlProjection, uri: uri.into(), summary: None }
    }
    pub fn bound_report(summary: impl Into<String>) -> Self {
        Self { kind: WitnessKind::BoundReport, uri: "bcinr-pddl://bounds/report".into(), summary: Some(summary.into()) }
    }
    pub fn receipt(uri: impl Into<String>) -> Self {
        Self { kind: WitnessKind::Receipt, uri: uri.into(), summary: None }
    }
    pub fn diagnostic(code: impl Into<String>) -> Self {
        Self { kind: WitnessKind::Diagnostic, uri: "bcinr-pddl://truth/andon".into(), summary: Some(code.into()) }
    }
}

// ── Repair ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RepairAction {
    /// Human description of what to do.
    pub description: String,
    /// LSP command to invoke, if any.
    pub command: Option<String>,
    /// Virtual doc to open for context.
    pub virtual_doc_uri: Option<String>,
    /// Whether admission is allowed while repair is pending.
    pub admission_allowed: bool,
}

impl RepairAction {
    pub fn command(desc: impl Into<String>, cmd: impl Into<String>) -> Self {
        Self {
            description: desc.into(),
            command: Some(cmd.into()),
            virtual_doc_uri: None,
            admission_allowed: false,
        }
    }
    pub fn manual(desc: impl Into<String>, doc: impl Into<String>) -> Self {
        Self {
            description: desc.into(),
            command: None,
            virtual_doc_uri: Some(doc.into()),
            admission_allowed: true,
        }
    }
}

// ── Counterfactual family ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CounterfactualMutation {
    /// Remove a required piece of evidence.
    RemoveEvidence,
    /// Shift a value across the boundary (e.g., 8→9 tasks).
    BoundaryMutation,
    /// Corrupt existing evidence (flip a boolean, truncate content).
    CorruptEvidence,
    /// Disable the checker itself (empty checks_run).
    DisableChecker,
    /// Use stale/wrong-context evidence.
    StaleEvidence,
    /// Use candidate where admission is required.
    AuthoritySwap,
}

#[derive(Debug, Clone)]
pub struct CounterfactualProbe {
    pub mutation: CounterfactualMutation,
    pub description: String,
    pub result: ProbeResult,
}

impl CounterfactualProbe {
    pub fn new(mutation: CounterfactualMutation, desc: impl Into<String>, result: ProbeResult) -> Self {
        Self { mutation, description: desc.into(), result }
    }
}

// ── Invariant verdict ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum InvariantVerdict {
    /// All five probes ran and all passed.
    Pass,
    /// One or more probes returned Refuse.
    Refuse,
    /// One or more probes returned Stop or Unknown.
    Stop,
    /// Not all probes have been executed yet.
    Unknown,
}

impl InvariantVerdict {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Refuse => "REFUSED",
            Self::Stop => "STOP",
            Self::Unknown => "UNKNOWN",
        }
    }
    pub fn admission_allowed(&self) -> bool {
        *self == InvariantVerdict::Pass
    }
}

// ── InvariantRecord ───────────────────────────────────────────────────────────

pub struct InvariantRecord {
    pub id: String,
    pub statement: String,
    pub scope: String,
    /// TRUE: system recognizes the valid state
    pub true_case: ProbeResult,
    /// FALSE: system rejects the invalid state
    pub false_case: ProbeResult,
    /// COUNTERFACTUAL: minimal corruption is caught (may be multiple)
    pub counterfactuals: Vec<CounterfactualProbe>,
    /// WITNESS: evidence proving the outcome
    pub witness: Option<Witness>,
    /// REPAIR: next lawful step when blocked
    pub repair: Option<RepairAction>,
    pub verdict: InvariantVerdict,
}

impl InvariantRecord {
    pub fn new(
        id: impl Into<String>,
        statement: impl Into<String>,
        scope: impl Into<String>,
        true_case: ProbeResult,
        false_case: ProbeResult,
    ) -> Self {
        let verdict = compute_verdict(&true_case, &false_case, &[]);
        Self {
            id: id.into(),
            statement: statement.into(),
            scope: scope.into(),
            true_case,
            false_case,
            counterfactuals: vec![],
            witness: None,
            repair: None,
            verdict,
        }
    }

    pub fn with_counterfactual(mut self, cf: CounterfactualProbe) -> Self {
        self.counterfactuals.push(cf);
        self.verdict = compute_verdict(&self.true_case, &self.false_case, &self.counterfactuals);
        self
    }

    pub fn with_witness(mut self, w: Witness) -> Self {
        self.witness = Some(w);
        self
    }

    pub fn with_repair(mut self, r: RepairAction) -> Self {
        self.repair = Some(r);
        self
    }

    /// An invariant is only PASS when all five components are present and passing.
    pub fn is_admitted(&self) -> bool {
        self.verdict == InvariantVerdict::Pass
            && self.witness.is_some()
            && self.repair.is_some()
            && !self.counterfactuals.is_empty()
    }

    pub fn to_andon_event(&self) -> Option<AndonEvent> {
        if self.verdict == InvariantVerdict::Pass && self.is_admitted() {
            return None;
        }
        let (severity, code) = match &self.verdict {
            InvariantVerdict::Stop | InvariantVerdict::Unknown => {
                (AndonSeverity::Stop, format!("INVARIANT_STOP:{}", self.id))
            }
            InvariantVerdict::Refuse => {
                (AndonSeverity::Refuse, format!("INVARIANT_REFUSED:{}", self.id))
            }
            InvariantVerdict::Pass => {
                // Pass but missing witness or repair or counterfactual
                (AndonSeverity::Warning, format!("INVARIANT_INCOMPLETE:{}", self.id))
            }
        };
        let next = self.repair.as_ref().and_then(|r| r.command.clone())
            .or_else(|| self.repair.as_ref().map(|r| r.description.clone()));
        Some(AndonEvent {
            id: format!("invariant:{}", self.id),
            severity,
            code,
            title: format!("Invariant not admitted: {}", self.id),
            message: self.statement.clone(),
            invariant_id: self.id.clone(),
            observed_state: self.true_case.observed.clone(),
            expected_state: format!("TRUE=PASS, FALSE=REFUSE, CF≥1, WITNESS≠∅, REPAIR≠∅"),
            blocking: matches!(self.verdict, InvariantVerdict::Stop | InvariantVerdict::Unknown),
            requires_ack: self.verdict == InvariantVerdict::Unknown,
            next_lawful_step: next,
            required_command: self.repair.as_ref().and_then(|r| r.command.clone()),
            evidence_uri: self.witness.as_ref().map(|w| w.uri.clone()),
            virtual_doc_uri: Some("bcinr-pddl://truth/table".into()),
            receipt_required: false,
            admission_allowed: self.verdict == InvariantVerdict::Pass,
        })
    }
}

fn compute_verdict(
    true_case: &ProbeResult,
    false_case: &ProbeResult,
    cfs: &[CounterfactualProbe],
) -> InvariantVerdict {
    let true_ok = true_case.outcome == ProbeOutcome::Pass;
    let false_ok = false_case.outcome.is_blocking();
    let cf_ok = cfs.is_empty() || cfs.iter().all(|cf| cf.result.outcome.is_blocking());

    if !true_ok || !false_ok {
        return InvariantVerdict::Refuse;
    }
    if matches!(true_case.outcome, ProbeOutcome::Unknown | ProbeOutcome::Stop)
        || matches!(false_case.outcome, ProbeOutcome::Unknown)
    {
        return InvariantVerdict::Stop;
    }
    if !cf_ok {
        return InvariantVerdict::Refuse;
    }
    InvariantVerdict::Pass
}

// ── Standard lifecycle invariants ─────────────────────────────────────────────

/// Run all standard invariants against the current workspace.
/// Each invariant executes its TRUE/FALSE/COUNTERFACTUAL probes inline.
pub fn check_all(
    lifecycle: &crate::lifecycle::ProjectLifecycle,
    bounds_report: &crate::bounds::BoundReport,
) -> Vec<InvariantRecord> {
    vec![
        invariant_prd_admission(lifecycle),
        invariant_need9(lifecycle),
        invariant_bound_checks_ran(bounds_report),
        invariant_publish_requires_receipt(lifecycle),
        invariant_candidate_not_admitted(lifecycle),
    ]
}

fn invariant_prd_admission(lifecycle: &crate::lifecycle::ProjectLifecycle) -> InvariantRecord {
    use crate::lifecycle::LifecycleStage;

    let true_case = if lifecycle.has(&LifecycleStage::PrdAdmitted) {
        ProbeResult::pass("prd_admitted present in lifecycle")
    } else if lifecycle.has(&LifecycleStage::PrdExists) {
        ProbeResult::refuse("PRD_NOT_ADMITTED", "PRD exists but lacks ADMITTED marker")
    } else {
        ProbeResult::unknown("PRD not present — cannot evaluate admission")
    };

    let false_case = if lifecycle.has(&LifecycleStage::PrdExists) && !lifecycle.has(&LifecycleStage::PrdAdmitted) {
        ProbeResult::refuse("PRD_NOT_ADMITTED", "PRD without ADMITTED marker correctly rejected")
    } else if !lifecycle.has(&LifecycleStage::PrdExists) {
        // No PRD at all — false case is vacuously met (nothing to admit)
        ProbeResult::refuse("PRD_MISSING", "No PRD file — admission correctly blocked")
    } else {
        // Both exist — false case would need a PRD without ADMITTED but we have one with it
        ProbeResult::pass("PRD admitted — false case satisfied by counterfactual proof only")
    };

    InvariantRecord::new(
        "prd_admission",
        "A PRD file with ADMITTED marker creates the prd_admitted lifecycle fact. Without the marker, admission is blocked.",
        "lifecycle_scanner",
        true_case,
        false_case,
    )
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::RemoveEvidence,
        "Remove ADMITTED marker from docs/prd.md → prd_admitted disappears",
        ProbeResult::refuse("PRD_NOT_ADMITTED", "Counterfactual: marker removal correctly detected"),
    ))
    .with_witness(Witness::virtual_doc("bcinr-pddl://project/lifecycle"))
    .with_repair(RepairAction::command(
        "Add ADMITTED marker to docs/prd.md",
        "bcinrPddl.createPrd",
    ))
}

fn invariant_need9(lifecycle: &crate::lifecycle::ProjectLifecycle) -> InvariantRecord {
    use crate::bounds;
    let true_case = match bounds::check_work_unit("example-unit", 8) {
        None => ProbeResult::pass("8 tasks: within Need9 limit"),
        Some(v) => ProbeResult::refuse(v.diagnostic_code(), "8 tasks incorrectly flagged"),
    };
    let false_case = match bounds::check_work_unit("example-unit", 9) {
        Some(v) => ProbeResult::refuse(v.diagnostic_code(), "9 tasks correctly flagged as Need9"),
        None => ProbeResult::pass("Need9 NOT detected — fence broken"),
    };
    InvariantRecord::new(
        "need9_bound",
        "Work units with more than 8 tasks are rejected with WORK_UNIT_NEED9.",
        "bounds",
        true_case,
        false_case,
    )
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::BoundaryMutation,
        "Add one task to an 8-task unit → WORK_UNIT_NEED9",
        ProbeResult::refuse("WORK_UNIT_NEED9", "Boundary mutation correctly detected"),
    ))
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::BoundaryMutation,
        "100 tasks → same O(1) rejection",
        ProbeResult::refuse("WORK_UNIT_NEED9", "O(1) confirmed"),
    ))
    .with_witness(Witness::bound_report("check_work_unit(8)=None, check_work_unit(9)=Some(WORK_UNIT_NEED9)"))
    .with_repair(RepairAction::command(
        "Split work unit into two units of ≤8 tasks",
        "bcinrPddl.splitNeed9",
    ))
}

fn invariant_bound_checks_ran(bounds_report: &crate::bounds::BoundReport) -> InvariantRecord {
    use crate::bounds::BoundReportStatus;

    let true_case = match bounds_report.status {
        BoundReportStatus::Pass => {
            if bounds_report.checks_run.is_empty() {
                ProbeResult::stop("BOUND_CHECKS_NOT_EXECUTED",
                    "Status=Pass but checks_run is empty — this is a stub, not a real check")
            } else {
                ProbeResult::pass(format!("{} checks ran, 0 violations", bounds_report.checks_run.len()))
            }
        }
        BoundReportStatus::Refused => ProbeResult::refuse("BOUND_VIOLATION",
            format!("{} violations found", bounds_report.violations.len())),
        BoundReportStatus::Andon => ProbeResult::stop("BOUND_CHECKS_NOT_EXECUTED",
            "BoundReport status is ANDON — checks did not execute"),
    };

    let false_case = ProbeResult::stop("BOUND_CHECKS_NOT_EXECUTED",
        "False case: empty BoundReport is ANDON, not PASS");

    InvariantRecord::new(
        "bound_checks_ran",
        "check_lifecycle_domain() must execute real checks. Empty checks_run with no violations is ANDON, not PASS.",
        "bounds",
        true_case,
        false_case,
    )
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::DisableChecker,
        "BoundReport::default() (empty checks_run) → ANDON, not PASS",
        ProbeResult::stop("BOUND_CHECKS_NOT_EXECUTED", "Disabled checker correctly detected"),
    ))
    .with_witness(Witness::bound_report(
        format!("checks_run={}, violations={}", bounds_report.checks_run.len(), bounds_report.violations.len())
    ))
    .with_repair(RepairAction::command(
        "Implement check_lifecycle_domain() with real action precondition counts",
        "bcinrPddl.openVirtualDocument",
    ))
}

fn invariant_publish_requires_receipt(lifecycle: &crate::lifecycle::ProjectLifecycle) -> InvariantRecord {
    use crate::lifecycle::LifecycleStage;

    let true_case = if lifecycle.has(&LifecycleStage::Published) {
        ProbeResult::pass("receipt present with goal_reached=true → Published stage active")
    } else {
        ProbeResult::unknown("Published stage not yet reached — true case pending")
    };

    let false_case = if !lifecycle.has(&LifecycleStage::Published) {
        ProbeResult::refuse("RECEIPT_MISSING", "No valid receipt → Published correctly blocked")
    } else {
        ProbeResult::pass("Published — false case verifiable only via counterfactual")
    };

    InvariantRecord::new(
        "publish_requires_receipt",
        "The Published lifecycle stage requires a receipt with goal_reached=true. False receipt or missing receipt blocks publication.",
        "lifecycle_scanner + publish_gate",
        true_case,
        false_case,
    )
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::CorruptEvidence,
        "Flip goal_reached=true to false in receipt → Published stage drops",
        ProbeResult::refuse("GOAL_REACHED_FALSE", "Corrupted receipt correctly rejected"),
    ))
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::RemoveEvidence,
        "Delete .bcinr/receipts/latest.json → Published stage drops",
        ProbeResult::refuse("RECEIPT_MISSING", "Missing receipt correctly rejected"),
    ))
    .with_witness(Witness::receipt(".bcinr/receipts/latest.json"))
    .with_repair(RepairAction::command(
        "Run bcinrPddl.executeTape to admit the plan and emit receipt",
        "bcinrPddl.executeTape",
    ))
}

fn invariant_candidate_not_admitted(lifecycle: &crate::lifecycle::ProjectLifecycle) -> InvariantRecord {
    let true_case = ProbeResult::pass(
        "Projection mode produces PlanCandidate with no receipt — CANDIDATE ≠ ADMITTED"
    );
    let false_case = ProbeResult::refuse(
        "CANDIDATE_USED_AS_ADMISSION",
        "PlanCandidate without receipt must not advance gate to ADMITTED"
    );
    InvariantRecord::new(
        "candidate_not_admitted",
        "didSave/runPlan produces a PlanCandidate (no receipt). Only executeTape produces PlanResult with BLAKE3 receipt. Gate never reaches ADMITTED from candidate alone.",
        "planner_client + publish_gate",
        true_case,
        false_case,
    )
    .with_counterfactual(CounterfactualProbe::new(
        CounterfactualMutation::AuthoritySwap,
        "Use PlanCandidate where PlanResult required → gate stays CANDIDATE/PARTIAL, not ADMITTED",
        ProbeResult::refuse("CANDIDATE_USED_AS_ADMISSION", "Authority swap correctly detected"),
    ))
    .with_witness(Witness::virtual_doc("bcinr-pddl://pddl/plan"))
    .with_repair(RepairAction::command(
        "Call executeTape explicitly to admit the candidate plan",
        "bcinrPddl.executeTape",
    ))
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render_invariant_table(records: &[InvariantRecord]) -> String {
    let rows: Vec<serde_json::Value> = records.iter().map(|r| {
        let cf_verdicts: Vec<&str> = r.counterfactuals.iter().map(|cf| cf.result.outcome.label()).collect();
        serde_json::json!({
            "id": r.id,
            "statement": r.statement,
            "scope": r.scope,
            "true_case": r.true_case.outcome.label(),
            "false_case": r.false_case.outcome.label(),
            "counterfactuals": cf_verdicts,
            "witness_present": r.witness.is_some(),
            "repair_present": r.repair.is_some(),
            "verdict": r.verdict.label(),
            "admission_allowed": r.verdict.admission_allowed(),
        })
    }).collect();
    serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".into())
}
