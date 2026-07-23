//! M3/M5: bcinr-pddl invocation — two-phase: plan (safe) vs execute (admitted).
//!
//! Phase 1 — plan(): parse → ground → BFS → PlanCandidate  (always safe, no receipt)
//! Phase 2 — admit(): execute tape → receipt + OCEL + ADMITTED  (explicit only)

use bcinr_pddl::{
    domain_from_pddl, execute_tape as bcinr_execute_tape, problem_from_pddl, GroundProblem,
    Pddl8ExecutionLog, Pddl8ExecutionReceipt,
};
use std::collections::BTreeSet;
use wasm4pm_compat::{
    ocel::OCEL,
    pddl::{Pddl8GroundAtom, Pddl8Tape},
};

use crate::projection::Pddl8Projection;

#[derive(Debug)]
pub enum PlannerError {
    ParseError(String),
    GroundingError(String),
    NoAdmittedPlan,
    ExecutionError(String),
}

impl std::fmt::Display for PlannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(s) => write!(f, "PDDL parse error: {s}"),
            Self::GroundingError(s) => write!(f, "Grounding error: {s}"),
            Self::NoAdmittedPlan => write!(f, "NO_ADMITTED_PLAN: BFS exhausted reachable states"),
            Self::ExecutionError(s) => write!(f, "Execution error: {s}"),
        }
    }
}

/// Phase 1 output: a candidate plan with no receipt and no OCEL.
/// CANDIDATE ≠ ADMITTED. Receipt requires explicit executeTape command.
#[derive(Debug, Clone)]
pub struct PlanCandidate {
    pub plan_steps: Vec<String>,
    pub tape: Pddl8Tape,
    pub initial_state: BTreeSet<Pddl8GroundAtom>,
    pub goal: Vec<Pddl8GroundAtom>,
}

/// Phase 2 output: admitted plan with receipt and OCEL.
#[derive(Debug)]
pub struct PlanResult {
    pub plan_steps: Vec<String>,
    pub log: Pddl8ExecutionLog,
    pub receipt: Pddl8ExecutionReceipt,
    pub ocel: OCEL,
}

/// Phase 1: parse → ground → BFS → PlanCandidate.
///
/// Safe to call on every didOpen/didChange/didSave.
/// Does NOT execute the tape, does NOT emit receipts, does NOT produce OCEL.
pub fn plan(projection: &Pddl8Projection) -> Result<PlanCandidate, PlannerError> {
    let domain = domain_from_pddl(&projection.domain_text)
        .map_err(|e| PlannerError::ParseError(format!("{e:?}")))?;

    let problem = problem_from_pddl(&projection.problem_text)
        .map_err(|e| PlannerError::ParseError(format!("{e:?}")))?;

    let gp = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| PlannerError::GroundingError(format!("{e:?}")))?;

    let tape = gp.find_plan().map_err(|_| PlannerError::NoAdmittedPlan)?;

    let plan_steps: Vec<String> = tape.ops.iter().map(|op| op.label.clone()).collect();

    let initial_state: BTreeSet<Pddl8GroundAtom> = problem
        .init
        .iter()
        .map(|a| Pddl8GroundAtom {
            pred: a.pred.clone(),
            args: a.args.clone(),
        })
        .collect();
    let goal: Vec<Pddl8GroundAtom> = problem
        .goal
        .iter()
        .map(|a| Pddl8GroundAtom {
            pred: a.pred.clone(),
            args: a.args.clone(),
        })
        .collect();

    Ok(PlanCandidate {
        plan_steps,
        tape,
        initial_state,
        goal,
    })
}

/// Phase 2: execute a candidate tape under Prolog8 admission → receipt + OCEL.
///
/// ONLY called by bcinrPddl.executeTape (explicit command).
/// Elevates gate from PARTIAL → ADMITTED iff goal_reached = true.
/// Optionally persists receipt + OCEL to `output_dir` (`.bcinr/`).
pub fn admit(candidate: &PlanCandidate, case_id: &str) -> Result<PlanResult, PlannerError> {
    let (log, receipt, ocel) = bcinr_execute_tape(
        &candidate.tape,
        &candidate.initial_state,
        &candidate.goal,
        case_id,
        &[],
    )
    .map_err(|e| PlannerError::ExecutionError(format!("{e:?}")))?;

    Ok(PlanResult {
        plan_steps: candidate.plan_steps.clone(),
        log,
        receipt,
        ocel,
    })
}

/// Persist receipt and OCEL to `.bcinr/` after admission.
///
/// Creates `.bcinr/receipts/latest.json` and `.bcinr/ocel/latest.json`.
pub fn persist_admission(root: &std::path::Path, result: &PlanResult) -> std::io::Result<()> {
    let bcinr = root.join(".bcinr");
    let receipts_dir = bcinr.join("receipts");
    let ocel_dir = bcinr.join("ocel");
    std::fs::create_dir_all(&receipts_dir)?;
    std::fs::create_dir_all(&ocel_dir)?;

    let receipt_json = serde_json::json!({
        "plan_root": result.receipt.plan_root,
        "state_root": result.receipt.state_root,
        "goal_root": result.receipt.goal_root,
        "chain_hash": result.receipt.chain_hash,
        "goal_reached": result.receipt.goal_reached,
        "step_count": result.receipt.step_count,
    });
    std::fs::write(
        receipts_dir.join("latest.json"),
        serde_json::to_string_pretty(&receipt_json).unwrap_or_default(),
    )?;

    let ocel_json = serde_json::json!({
        "event_count": result.ocel.events.len(),
        "events": result.ocel.events.iter().map(|e| serde_json::json!({
            "id": e.id,
            "type": e.event_type,
        })).collect::<Vec<_>>(),
    });
    std::fs::write(
        ocel_dir.join("latest.json"),
        serde_json::to_string_pretty(&ocel_json).unwrap_or_default(),
    )?;

    Ok(())
}

/// Convenience: plan + execute in one call (for tests and explicit runPlan commands).
pub fn plan_and_execute(
    projection: &Pddl8Projection,
    case_id: &str,
) -> Result<PlanResult, PlannerError> {
    let candidate = plan(projection)?;
    admit(&candidate, case_id)
}
