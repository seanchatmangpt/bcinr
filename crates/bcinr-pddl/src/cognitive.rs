//! Exact PDDL → executable POWL 2.0 cognitive-composition rail.
//!
//! This is the production entry point for the full-semantics classical PDDL
//! fragment. It parses and admits PDDL, runs exact bounded search, projects the
//! witnessed plan as a POWL 2.0 model, executes the compiled v2 tape, and seals
//! a replayable whole-run receipt.
//!
//! # Two projections, and which one to call
//!
//! [`plan_exact_cognitive_workflow`] projects the witnessed plan as a flat
//! `Sequence`. That is deliberate and it is *not* a claim that no concurrency
//! exists -- it is a refusal to guess concurrency from a linear plan.
//!
//! [`plan_exact_cognitive_workflow_hierarchical`] is the one that *derives*
//! concurrency: it routes the plan through `PddlCausalAnalyzer` ->
//! [`crate::wf_net_bridge`] -> Algorithm 3, producing a `PartialOrder` in
//! which independent actions are genuinely unordered. Two actions end up
//! concurrent because neither's effects touch the other's preconditions --
//! derived from the domain's own causal structure, never declared by an
//! author. The result is gated by Theorem 1 (`convert_and_verify`): its
//! language must equal the WF-net's own replay, or it refuses rather than
//! returning a model that merely looks authoritative.
//!
//! Downstream, `bcinr_powl::process_toolkit::{dispatch_waves, ready_set}`
//! turn that partial order into dispatchable antichains -- sets of actions
//! safely runnable on separate agents simultaneously, with no coordination
//! protocol, because the precedence structure already encodes what must wait
//! on what. A flat `Sequence` structurally cannot express "these two are
//! unordered", which is why the hierarchical entry point is the one to call
//! when distribution is the goal.
//!
//! On refusal the hierarchical path falls back to the flat `Sequence` (never
//! fabricate structure the analysis cannot prove) but reports the `Refusal` in
//! [`ExactCognitiveWorkflow::hierarchical_refusal`], so "no concurrency here"
//! stays distinguishable from "the bridge refused".

#![cfg(feature = "mfw-planner")]

use bcinr_mfw_ir::{
    ActionOccurrence, ActionOccurrenceId, CausalAnalyzer, EpochBounds, PlannerFailure,
    PlanningEpochId,
};
use bcinr_powl::powl2::{compile_powl2, CompiledPowl2, LowestIndexPolicy, Powl2Error, Powl2Model};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use bcinr_powl_receipt::execution_v2::{
    execute_and_seal_v2, PowlV2ExecutionReceipt, PowlV2ReceiptError,
};

use crate::capability::{admit_planning_task, AdmittedPlanningTask, GroundedPlanningEpoch};
use crate::causal::PddlCausalAnalyzer;
use crate::error::Pddl8Error;
use crate::ground_v2::{
    ExactClassicalCapabilityProfile, ExactClassicalError, ExactClassicalProblem,
    EXACT_MAX_GROUND_ACTIONS, EXACT_MAX_PLAN_DEPTH, EXACT_MAX_SEARCH_STATES,
};
use crate::parse::{domain31_from_pddl, problem31_from_pddl};
use crate::wf_net_bridge::causal_plan_to_powl2;
use crate::Pddl8Tape;

/// Standing of the PDDL-to-POWL projection emitted by this rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveProjectionStanding {
    /// Every plan action is preserved in its witnessed linear order. No
    /// concurrency claim is made without an independence witness.
    ExactSequential,
    /// The plan was routed through `PddlCausalAnalyzer` -> `WfNet` ->
    /// Algorithm 3, and the projection may contain genuine
    /// `PartialOrder`/`ChoiceGraph` structure discovered from the causal
    /// analysis's independence witnesses, not just the witnessed order.
    CausalHierarchical,
}

/// Complete output of one exact cognitive-composition request.
#[derive(Debug)]
pub struct ExactCognitiveWorkflow {
    pub admitted: AdmittedPlanningTask,
    pub plan: Pddl8Tape,
    pub powl: CompiledPowl2,
    pub execution_receipt: PowlV2ExecutionReceipt,
    pub projection_standing: CognitiveProjectionStanding,
    /// Why the hierarchical projection was not used, when
    /// `projection_standing` is `ExactSequential` on the *hierarchical* entry
    /// points ([`plan_exact_cognitive_workflow_hierarchical`] and its bounded
    /// form). `None` on the sequential entry points, which never attempt it.
    ///
    /// Falling back to a flat `Sequence` is the correct response to a genuine
    /// refusal -- never fabricate structure the analysis cannot prove -- but
    /// "this plan has no exploitable concurrency" and "the bridge refused"
    /// must not look identical from outside, or a broken bridge is
    /// indistinguishable from a legitimately flat plan. This field is what
    /// distinguishes them: `Some(refusal)` means real structure was
    /// attempted and declined (e.g. `LanguageMismatch` -- Theorem 1 did not
    /// hold -- or `IrreducibleFragment`), `None` with `ExactSequential`
    /// means the causal analysis itself found nothing to exploit.
    pub hierarchical_refusal: Option<bcinr_powl::wf_to_powl::Refusal>,
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
        // This entry point projects sequentially by design and never attempts
        // the hierarchical bridge, so there is no refusal to report.
        hierarchical_refusal: None,
    })
}

/// Parse, admit, plan, and project as [`plan_exact_cognitive_workflow`], but
/// additionally route the witnessed plan through `PddlCausalAnalyzer` ->
/// [`crate::wf_net_bridge`] -> Algorithm 3, so independent actions are
/// projected as genuine `PartialOrder`/`ChoiceGraph` structure instead of a
/// single witnessed order. Falls back to the always-correct flat sequence
/// (never silently drops steps) whenever the causal analysis or the WF-net
/// decomposition cannot proceed -- a fallback is a standing, not a failure.
pub fn plan_exact_cognitive_workflow_hierarchical(
    domain_text: &str,
    problem_text: &str,
) -> Result<ExactCognitiveWorkflow, ExactCognitiveError> {
    plan_exact_cognitive_workflow_hierarchical_bounded(
        domain_text,
        problem_text,
        EXACT_MAX_GROUND_ACTIONS,
        EXACT_MAX_PLAN_DEPTH,
        EXACT_MAX_SEARCH_STATES,
    )
}

/// Bounded form of [`plan_exact_cognitive_workflow_hierarchical`].
pub fn plan_exact_cognitive_workflow_hierarchical_bounded(
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

    let (model, projection_standing, hierarchical_refusal) = build_hierarchical_model(
        &admitted,
        &grounded,
        &plan,
        max_ground_actions,
        max_plan_depth,
        max_search_states,
    );

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
        projection_standing,
        hierarchical_refusal,
    })
}

fn build_hierarchical_model(
    admitted: &AdmittedPlanningTask,
    grounded: &ExactClassicalProblem,
    plan: &Pddl8Tape,
    max_ground_actions: usize,
    max_plan_depth: usize,
    max_search_states: usize,
) -> (
    Powl2Model,
    CognitiveProjectionStanding,
    Option<bcinr_powl::wf_to_powl::Refusal>,
) {
    if plan.ops.is_empty() {
        return (
            Powl2Model::Silent,
            CognitiveProjectionStanding::ExactSequential,
            None,
        );
    }

    let epoch = GroundedPlanningEpoch {
        id: PlanningEpochId(0),
        theory_digest: admitted.theory_digest,
        initial_state: grounded.initial_facts.clone(),
        goal: Vec::new(),
        actions: plan.ops.iter().map(|op| op.action.clone()).collect(),
        bounds: EpochBounds {
            max_ground_actions,
            max_plan_depth,
            max_search_steps: max_search_states as u64,
            max_partition_boxes: 8,
        },
    };
    let occurrences: Vec<ActionOccurrence> = (0..plan.ops.len())
        .map(|i| ActionOccurrence {
            id: ActionOccurrenceId(i as u32),
            action: i as u64,
        })
        .collect();

    // The causal analysis failing is not a bridge refusal -- there is no
    // `Refusal` to report, the analysis simply produced no partial order to
    // decompose -- so it falls back with `None`, distinct from a real
    // Algorithm 3 refusal below.
    let Ok(causal_plan) = PddlCausalAnalyzer.analyze(&epoch, &occurrences) else {
        return (
            fallback_sequential(plan),
            CognitiveProjectionStanding::ExactSequential,
            None,
        );
    };

    match causal_plan_to_powl2(&epoch, &causal_plan) {
        Ok(model) => (model, CognitiveProjectionStanding::CausalHierarchical, None),
        // Fall back to the flat sequence -- correct, and never fabricates
        // structure the bridge could not verify -- but carry the refusal out
        // so it stays distinguishable from a legitimately flat plan.
        Err(refusal) => (
            fallback_sequential(plan),
            CognitiveProjectionStanding::ExactSequential,
            Some(refusal),
        ),
    }
}

fn fallback_sequential(plan: &Pddl8Tape) -> Powl2Model {
    Powl2Model::Sequence(
        plan.ops
            .iter()
            .map(|op| Powl2Model::Activity(op.label.clone()))
            .collect(),
    )
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
