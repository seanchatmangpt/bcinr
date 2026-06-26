//! M3: bcinr-pddl invocation — parse → ground → plan → execute → receipt + OCEL.

use bcinr_pddl::{
    domain_from_pddl, execute_tape, problem_from_pddl, GroundProblem,
    Pddl8ExecutionLog, Pddl8ExecutionReceipt,
};
use std::collections::BTreeSet;
use wasm4pm_compat::{
    ocel::OCEL,
    pddl::Pddl8GroundAtom,
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

#[derive(Debug)]
pub struct PlanResult {
    pub plan_steps: Vec<String>,
    pub log: Pddl8ExecutionLog,
    pub receipt: Pddl8ExecutionReceipt,
    pub ocel: OCEL,
}

/// Parse, ground, plan, and execute the lifecycle projection.
/// Returns the plan steps, execution log, receipt, and OCEL trace.
pub fn plan_and_execute(
    projection: &Pddl8Projection,
    case_id: &str,
) -> Result<PlanResult, PlannerError> {
    let domain = domain_from_pddl(&projection.domain_text)
        .map_err(|e| PlannerError::ParseError(format!("{e:?}")))?;

    let problem = problem_from_pddl(&projection.problem_text)
        .map_err(|e| PlannerError::ParseError(format!("{e:?}")))?;

    let gp = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| PlannerError::GroundingError(format!("{e:?}")))?;

    let tape = gp.find_plan()
        .map_err(|_| PlannerError::NoAdmittedPlan)?;

    let plan_steps: Vec<String> = tape.ops.iter().map(|op| op.label.clone()).collect();

    let initial_state: BTreeSet<Pddl8GroundAtom> = problem
        .init
        .iter()
        .map(|a| Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
        .collect();
    let goal: Vec<Pddl8GroundAtom> = problem
        .goal
        .iter()
        .map(|a| Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
        .collect();

    let (log, receipt, ocel) = execute_tape(&tape, &initial_state, &goal, case_id, &[])
        .map_err(|e| PlannerError::ExecutionError(format!("{e:?}")))?;

    Ok(PlanResult { plan_steps, log, receipt, ocel })
}
