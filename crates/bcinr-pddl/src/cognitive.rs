//! Exact PDDL → executable POWL 2.0 cognitive-composition rail.
//!
//! This is the production entry point for the full-semantics classical PDDL
//! fragment. It parses and admits PDDL, runs exact bounded search, projects the
//! witnessed plan as a sequential POWL 2.0 model, executes the compiled v2
//! tape, and seals a replayable whole-run receipt.
//!
//! The sequential projection is intentional: order is removed only by the
//! separate STRIPS MFW rail when `PddlCausalAnalyzerV2` has produced explicit
//! independence witnesses. This rail never guesses concurrency from a linear
//! plan containing quantified, conditional, or numeric semantics.

#![cfg(feature = "mfw-planner")]

use bcinr_mfw_ir::PlannerFailure;
use bcinr_powl::powl2::{compile_powl2, CompiledPowl2, LowestIndexPolicy, Powl2Error, Powl2Model};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use bcinr_powl_receipt::execution_v2::{
    execute_and_seal_v2, PowlV2ExecutionReceipt, PowlV2ReceiptError,
};

use crate::capability::{admit_planning_task, AdmittedPlanningTask};
use crate::error::Pddl8Error;
use crate::ground_v2::{
    ExactClassicalCapabilityProfile, ExactClassicalError, ExactClassicalProblem,
    EXACT_MAX_GROUND_ACTIONS, EXACT_MAX_PLAN_DEPTH, EXACT_MAX_SEARCH_STATES,
};
use crate::parse::{domain31_from_pddl, problem31_from_pddl};
use crate::Pddl8Tape;

/// Standing of the PDDL-to-POWL projection emitted by this rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveProjectionStanding {
    /// Every plan action is preserved in its witnessed linear order. No
    /// concurrency claim is made without an independence witness.
    ExactSequential,
}

/// Complete output of one exact cognitive-composition request.
#[derive(Debug)]
pub struct ExactCognitiveWorkflow {
    pub admitted: AdmittedPlanningTask,
    pub plan: Pddl8Tape,
    pub powl: CompiledPowl2,
    pub execution_receipt: PowlV2ExecutionReceipt,
    pub projection_standing: CognitiveProjectionStanding,
}

#[derive(Debug)]
pub enum ExactCognitiveError {
    Parse(Pddl8Error),
    Admission(PlannerFailure),
    Planning(ExactClassicalError),
    Powl(Powl2Error),
    Receipt(PowlV2ReceiptError),
}

impl std::fmt::Display for ExactCognitiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "PDDL parse failed: {error}"),
            Self::Admission(error) => write!(f, "PDDL admission refused: {error}"),
            Self::Planning(error) => write!(f, "exact classical planning failed: {error}"),
            Self::Powl(error) => write!(f, "POWL 2.0 compilation failed: {error}"),
            Self::Receipt(error) => write!(f, "POWL v2 receipt failed: {error}"),
        }
    }
}

impl std::error::Error for ExactCognitiveError {}

/// Parse, admit, plan, compile, execute, and receipt one classical PDDL task.
pub fn plan_exact_cognitive_workflow(
    domain_text: &str,
    problem_text: &str,
) -> Result<ExactCognitiveWorkflow, ExactCognitiveError> {
    plan_exact_cognitive_workflow_bounded(
        domain_text,
        problem_text,
        EXACT_MAX_GROUND_ACTIONS,
        EXACT_MAX_PLAN_DEPTH,
        EXACT_MAX_SEARCH_STATES,
    )
}

/// Bounded form of [`plan_exact_cognitive_workflow`].
pub fn plan_exact_cognitive_workflow_bounded(
    domain_text: &str,
    problem_text: &str,
    max_ground_actions: usize,
    max_plan_depth: usize,
    max_search_states: usize,
) -> Result<ExactCognitiveWorkflow, ExactCognitiveError> {
    let domain = domain31_from_pddl(domain_text).map_err(ExactCognitiveError::Parse)?;
    let problem = problem31_from_pddl(problem_text).map_err(ExactCognitiveError::Parse)?;
    let admitted = admit_planning_task(&domain, &problem, &ExactClassicalCapabilityProfile)
        .into_result()
        .map_err(ExactCognitiveError::Admission)?;
    let grounded = ExactClassicalProblem::build(&domain, &problem, max_ground_actions)
        .map_err(ExactCognitiveError::Planning)?;
    let plan = grounded
        .find_plan(max_plan_depth, max_search_states)
        .map_err(ExactCognitiveError::Planning)?;

    let model = if plan.ops.is_empty() {
        Powl2Model::Silent
    } else {
        Powl2Model::Sequence(
            plan.ops
                .iter()
                .map(|operation| Powl2Model::Activity(operation.label.clone()))
                .collect(),
        )
    };
    let powl = compile_powl2(&model, &mut LowestIndexPolicy).map_err(ExactCognitiveError::Powl)?;
    let max_ticks = u32::from(powl.tape.len).saturating_add(1);
    let execution_receipt =
        execute_and_seal_v2(&powl.tape, &ConcurrencyGuardTable::empty(), max_ticks)
            .map_err(ExactCognitiveError::Receipt)?;

    Ok(ExactCognitiveWorkflow {
        admitted,
        plan,
        powl,
        execution_receipt,
        projection_standing: CognitiveProjectionStanding::ExactSequential,
    })
}

#[cfg(test)]
mod tests {
    use bcinr_powl_receipt::execution_v2::verify_execution_v2;

    use super::*;

    #[test]
    fn quantified_conditional_plan_composes_into_receipted_powl2() {
        let domain = r#"
        (define (domain batch)
          (:requirements :adl :typing)
          (:types item)
          (:predicates (ready ?x - item) (done ?x - item))
          (:action finish-all
            :parameters ()
            :precondition (forall (?x - item) (ready ?x))
            :effect (forall (?x - item) (when (ready ?x) (done ?x)))))
        "#;
        let problem = r#"
        (define (problem batch-p)
          (:domain batch)
          (:objects a b - item)
          (:init (ready a) (ready b))
          (:goal (and (done a) (done b))))
        "#;

        let workflow = plan_exact_cognitive_workflow(domain, problem).unwrap();
        assert_eq!(workflow.plan.ops.len(), 1);
        assert_eq!(workflow.plan.ops[0].label, "finish-all");
        assert_eq!(
            workflow.projection_standing,
            CognitiveProjectionStanding::ExactSequential
        );
        assert_eq!(workflow.powl.activity_slots.len(), 1);
        verify_execution_v2(
            &workflow.execution_receipt,
            &workflow.powl.tape,
            &ConcurrencyGuardTable::empty(),
            4,
        )
        .unwrap();
    }

    #[test]
    fn temporal_input_is_refused_before_projection() {
        let domain = r#"
        (define (domain temporal)
          (:requirements :durative-actions)
          (:predicates (done))
          (:durative-action finish
            :parameters ()
            :duration (= ?duration 1)
            :condition ()
            :effect (at end (done))))
        "#;
        let problem = "(define (problem p) (:domain temporal) (:init) (:goal (done)))";
        assert!(matches!(
            plan_exact_cognitive_workflow(domain, problem),
            Err(ExactCognitiveError::Admission(_))
                | Err(ExactCognitiveError::Planning(
                    ExactClassicalError::DurativeActionsUnsupported
                ))
        ));
    }
}
