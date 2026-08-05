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
//! author. The result is gated by bounded language agreement
//! (`convert_and_verify`): its language must equal the WF-net's own replay up
//! to the checked bound, or it refuses rather than returning a model that
//! merely looks authoritative. That bound is not the paper's Theorem 5.5 --
//! see `convert_and_verify` for what it does and does not establish.
//!
//! Downstream, `bcinr_powl::process_toolkit::{dispatch_waves, ready_set}`
//! turn that partial order into dispatchable antichains -- sets of actions
//! safely runnable on separate agents simultaneously, with no coordination
//! protocol, because the precedence structure already encodes what must wait
//! on what. A flat `Sequence` structurally cannot express "these two are
//! unordered", which is why the hierarchical entry point is the one to call
//! when distribution is the goal.
//!
//! On refusal the hierarchical path falls back to the flat `Sequence` -- never
//! fabricate structure the analysis cannot prove -- and returns it as
//! [`HierarchicalProjection::NotDerived`] rather than as an ordinary success.
//! A flat model and a derived one have the same shape, so a caller that could
//! reach the tape without naming which it held would be unable to tell "this
//! plan has no exploitable concurrency" from "the concurrency was never
//! derived". The standing and any `Refusal` are still carried on the workflow;
//! the enum is what makes consulting them unavoidable.
//!
//! The lossy-effect refusal is decided per *path*, not per domain: an action
//! whose effects cannot survive STRIPS lowering only withholds concurrency
//! when it actually appears in the witnessed plan. `ground_v2::legacy_action`
//! records the lossy kind per action and `path_to_tape` refuses only on the
//! path, which surfaces here as `plan_is_label_only`.

#![cfg(feature = "mfw-planner")]

use bcinr_mfw_ir::{
    ActionOccurrence, ActionOccurrenceId, CausalAnalyzer, EpochBounds, PlannerFailure,
    PlanningEpochId,
};
use bcinr_powl::powl2::{compile_powl2, CompiledPowl2, LowestIndexPolicy, Powl2Error, Powl2Model};
use bcinr_powl::receipt::execution_v2::{
    execute_and_seal_v2, PowlV2ExecutionReceipt, PowlV2ReceiptError,
};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;

use crate::capability::{admit_planning_task, AdmittedPlanningTask, GroundedPlanningEpoch};
use crate::causal::{CausalAnalysisError, PddlCausalAnalyzer};
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
    ///
    /// On the hierarchical entry points this means the causal analysis ran
    /// and `causal_plan_to_powl2` declined to build structure from it; the
    /// accompanying `hierarchical_refusal` is `Some(_)` and says why. The
    /// sequential entry points emit this without ever attempting the
    /// analysis.
    ExactSequential,
    /// The plan was empty, so there was nothing to project. The model is
    /// `Silent`, not a flat `Sequence`.
    ///
    /// Distinct from every other variant: no analysis was needed, none was
    /// attempted, and no claim about concurrency is being made or withheld.
    EmptyPlan,
    /// `PddlCausalAnalyzer::analyze` returned an error, so no causal plan
    /// exists to decompose and the projection is the witnessed linear order.
    ///
    /// This is the analysis *failing*, not the analysis finding nothing to
    /// exploit — the plan replay was rejected as not a valid linear plan over
    /// the epoch, or an occurrence referenced an out-of-range action. The
    /// carried [`CausalAnalysisError`] is the reason, kept rather than
    /// discarded so a broken grounding is not indistinguishable from a
    /// legitimately flat plan.
    CausalAnalysisFailed(CausalAnalysisError),
    /// The plan was routed through `PddlCausalAnalyzer` -> `WfNet` ->
    /// Algorithm 3, and the projection may contain genuine
    /// `PartialOrder`/`ChoiceGraph` structure discovered from the causal
    /// analysis's independence witnesses, not just the witnessed order.
    CausalHierarchical,
    /// The domain uses effect forms that `Pddl8GroundAction` cannot carry, so
    /// the causal analysis was not attempted at all and the projection is the
    /// witnessed linear order.
    ///
    /// Distinct from [`Self::ExactSequential`] on both of that variant's
    /// paths: hierarchically it means the analysis ran and found nothing to
    /// exploit, and on the sequential entry points it means the analysis was
    /// never part of the rail to begin with. Neither is an objection to
    /// running the analysis. Here running it was never *sound*: the
    /// analyser replays over `preconditions`/`add_effects`/`del_effects` only
    /// (`causal::simulate_two`), so two actions conflicting solely through a
    /// `when`, `forall` or numeric effect have *identical* shadow effects,
    /// commute in the replay, and are reported independent. The conservative
    /// fallback in `analyze_pair` is guarded on `!commute` and therefore never
    /// fires for exactly this class. Independence claimed on that basis would
    /// reach `dispatch_waves` and co-schedule genuinely conflicting actions.
    RefusedLossyEffectModel,
}

/// Complete output of one exact cognitive-composition request.
#[derive(Debug)]
pub struct ExactCognitiveWorkflow {
    pub admitted: AdmittedPlanningTask,
    pub plan: Pddl8Tape,
    /// `true` when `plan` came from `ExactClassicalProblem::find_label_plan`
    /// rather than `find_plan`, i.e. its ops carry labels and order only and
    /// every `ops[..].action` has empty `preconditions`/`add_effects`/
    /// `del_effects`.
    ///
    /// `Pddl8Tape` lives in `wasm4pm-compat` and has no slot to record this,
    /// so the distinction lives here instead of on the tape. Read it before
    /// touching `ops[..].action`: an empty effect list under `true` means
    /// "not carried", not "this action has no effects". This rail sets it
    /// only when the exact lowering refused with
    /// [`ExactClassicalError::EffectNotRepresentable`] or
    /// [`ExactClassicalError::PreconditionNotRepresentable`] -- the projection
    /// below consumes labels alone, so a plan that exists is preferable to a
    /// refusal over fields it never reads.
    pub plan_is_label_only: bool,
    /// The projected POWL 2.0 model, before compilation to a tape.
    ///
    /// `powl` below is this model linearized into an executable tape, and a
    /// tape answers "what runs next" rather than "what may run together". The
    /// partial order is the derived answer this rail exists to produce -- it is
    /// what [`bcinr_powl::process_toolkit::dispatch_waves`] reads to compute
    /// antichains, and what a consumer needs in order to distribute work
    /// without being told the concurrency. Compiling it away and keeping only
    /// the tape discards the deliverable and keeps the byproduct.
    pub model: Powl2Model,
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
    /// attempted and declined (e.g. `BoundedLanguageAgreementFailed` -- the
    /// two enumerations disagreed at the checked bound -- or
    /// `IrreducibleFragment`).
    ///
    /// On the hierarchical entry points `ExactSequential` is emitted *only*
    /// alongside `Some(refusal)`: a successful causal analysis that
    /// Algorithm 3 accepts always yields `CausalHierarchical`, so there is no
    /// "analysis ran, found nothing" outcome to report. The two ways the
    /// hierarchical path can fall back without an Algorithm 3 refusal each
    /// have their own standing --
    /// [`CognitiveProjectionStanding::CausalAnalysisFailed`] (the analysis
    /// errored, reason carried) and
    /// [`CognitiveProjectionStanding::RefusedLossyEffectModel`] (it was never
    /// sound to run) -- and both leave this field `None`. `EmptyPlan` also
    /// leaves it `None`.
    pub hierarchical_refusal: Option<bcinr_powl::wf_to_powl::Refusal>,
}

/// The result of a *hierarchical* projection, with "a partial order was
/// derived" separated from "a flat sequence is all there is" at the type level.
///
/// [`ExactCognitiveWorkflow`] already carries both facts, in
/// `projection_standing` and `hierarchical_refusal`. It carried them well
/// enough to be correct and badly enough to be useless: the fields are written
/// on every hierarchical call and read by nothing outside this module's own
/// tests, because `powl` is reachable without consulting them. A caller that
/// takes the tape and dispatches it cannot tell a plan with no exploitable
/// concurrency from one whose concurrency was never derived -- and those two
/// have the same shape, so the mistake is silent.
///
/// PDDL states preconditions and effects and never states what runs in
/// parallel; the partial order is the derived answer, and this rail exists to
/// derive it. Handing back a flat sequence is the correct response to a
/// refusal, but it is the *absence* of that answer, and the absence must be as
/// visible as the answer. Matching an arm here is the acknowledgement.
#[derive(Debug)]
pub enum HierarchicalProjection {
    /// Independence was derived from the domain's own causal structure and
    /// survived Algorithm 3, so `powl` may carry genuine `PartialOrder` /
    /// `ChoiceGraph` structure. Exactly
    /// [`CognitiveProjectionStanding::CausalHierarchical`].
    Derived(ExactCognitiveWorkflow),
    /// No partial order was derived. The workflow is correct and complete --
    /// no step is ever dropped -- but its order is the single witnessed one,
    /// and any concurrency the domain implies is *not* represented in it.
    ///
    /// Read `projection_standing` for which of the four reasons applies, and
    /// `hierarchical_refusal` when the standing is `ExactSequential`.
    NotDerived(ExactCognitiveWorkflow),
}

impl HierarchicalProjection {
    fn classify(workflow: ExactCognitiveWorkflow) -> Self {
        match workflow.projection_standing {
            CognitiveProjectionStanding::CausalHierarchical => Self::Derived(workflow),
            CognitiveProjectionStanding::ExactSequential
            | CognitiveProjectionStanding::EmptyPlan
            | CognitiveProjectionStanding::CausalAnalysisFailed(_)
            | CognitiveProjectionStanding::RefusedLossyEffectModel => Self::NotDerived(workflow),
        }
    }

    /// The standing, without consuming the projection.
    pub fn standing(&self) -> &CognitiveProjectionStanding {
        match self {
            Self::Derived(w) | Self::NotDerived(w) => &w.projection_standing,
        }
    }

    /// Take the workflow whether or not concurrency was derived.
    ///
    /// Deliberately verbose: this is the opt-in that reintroduces the old
    /// behaviour, and it should be visible at the call site that a flat result
    /// is being accepted without checking why it is flat.
    pub fn into_workflow_ignoring_standing(self) -> ExactCognitiveWorkflow {
        match self {
            Self::Derived(w) | Self::NotDerived(w) => w,
        }
    }
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
    // This entry point projects `plan.ops[..].label` and nothing else (see
    // `model` below, and `downstream`'s `batches`/`verify`/`exact_semantic_root`
    // which are its only in-tree consumers). So when the exact lowering refuses
    // because an effect *or a precondition* cannot ride the flat tape, drop to
    // the label-only tape rather than fail the request -- and record which one
    // was used, because `plan` is public and an empty effect list must not
    // read as "no effects".
    let (plan, plan_is_label_only) = match grounded.find_plan(max_plan_depth, max_search_states) {
        Ok(plan) => (plan, false),
        Err(
            ExactClassicalError::EffectNotRepresentable { .. }
            | ExactClassicalError::PreconditionNotRepresentable { .. },
        ) => (
            grounded
                .find_label_plan(max_plan_depth, max_search_states)
                .map_err(ExactCognitiveError::Planning)?,
            true,
        ),
        Err(error) => return Err(ExactCognitiveError::Planning(error)),
    };

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
        plan_is_label_only,
        model,
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
) -> Result<HierarchicalProjection, ExactCognitiveError> {
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
) -> Result<HierarchicalProjection, ExactCognitiveError> {
    let domain = domain31_from_pddl(domain_text).map_err(ExactCognitiveError::Parse)?;
    let problem = problem31_from_pddl(problem_text).map_err(ExactCognitiveError::Parse)?;
    let admitted = admit_planning_task(&domain, &problem, &ExactClassicalCapabilityProfile)
        .into_result()
        .map_err(ExactCognitiveError::Admission)?;
    let grounded = ExactClassicalProblem::build(&domain, &problem, max_ground_actions)
        .map_err(ExactCognitiveError::Planning)?;
    // Unlike the sequential entry point, this one *does* read
    // `plan.ops[..].action` -- `build_hierarchical_model` feeds it to
    // `PddlCausalAnalyzer` to derive independence. So it takes the exact tape
    // whenever one exists, and only drops to the label-only tape when the
    // exact lowering refuses -- at which point `build_hierarchical_model` is
    // told so explicitly and declines to run the analysis at all.
    let (plan, plan_is_label_only) = match grounded.find_plan(max_plan_depth, max_search_states) {
        Ok(plan) => (plan, false),
        Err(
            ExactClassicalError::EffectNotRepresentable { .. }
            | ExactClassicalError::PreconditionNotRepresentable { .. },
        ) => (
            grounded
                .find_label_plan(max_plan_depth, max_search_states)
                .map_err(ExactCognitiveError::Planning)?,
            true,
        ),
        Err(error) => return Err(ExactCognitiveError::Planning(error)),
    };

    let (model, projection_standing, hierarchical_refusal) = build_hierarchical_model(
        &admitted,
        &grounded,
        &plan,
        plan_is_label_only,
        max_ground_actions,
        max_plan_depth,
        max_search_states,
    );

    let powl = compile_powl2(&model, &mut LowestIndexPolicy).map_err(ExactCognitiveError::Powl)?;
    let max_ticks = u32::from(powl.tape.len).saturating_add(1);
    let execution_receipt =
        execute_and_seal_v2(&powl.tape, &ConcurrencyGuardTable::empty(), max_ticks)
            .map_err(ExactCognitiveError::Receipt)?;

    Ok(HierarchicalProjection::classify(ExactCognitiveWorkflow {
        admitted,
        plan,
        plan_is_label_only,
        model,
        powl,
        execution_receipt,
        projection_standing,
        hierarchical_refusal,
    }))
}

fn build_hierarchical_model(
    admitted: &AdmittedPlanningTask,
    grounded: &ExactClassicalProblem,
    plan: &Pddl8Tape,
    plan_is_label_only: bool,
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
            CognitiveProjectionStanding::EmptyPlan,
            None,
        );
    }

    // Refuse to derive concurrency from a shadow. `plan.ops[..].action` is
    // `ground_v2::legacy_action`'s output -- add/delete atoms only -- so an
    // action carrying any other effect form creates conflicts the independence
    // relation below cannot see.
    //
    // `plan_is_label_only` is the whole test, because the lossy signal is
    // already per-action and already checked per-path:
    //
    //   - `legacy_action` marks an action `LossyLowering::Effect` for every
    //     non-Add/Del form -- `When`, `Forall`, `Numeric`, `Timed` -- and
    //     `LossyLowering::Precondition` for a dropped condition kind.
    //   - `path_to_tape` walks the witnessed path and returns
    //     `EffectNotRepresentable` / `PreconditionNotRepresentable` if any
    //     action on it is lossy.
    //   - the hierarchical entry point catches exactly those two errors and
    //     re-plans with `find_label_plan`, setting `plan_is_label_only`.
    //
    // So `!plan_is_label_only` means every action on the witnessed path lowered
    // exactly -- and the analyser only ever compares occurrences drawn from
    // that path. A domain-level scan was the coarser form of this fact: it
    // withheld concurrency from every plan over a domain containing one
    // conditional effect, including plans in which no lossy action appears at
    // all. That is not conservatism, it is erasure: a flat result and a
    // genuinely sequential plan are indistinguishable downstream, so the cost
    // was invisible.
    if plan_is_label_only {
        return (
            fallback_sequential(plan),
            CognitiveProjectionStanding::RefusedLossyEffectModel,
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
    // Algorithm 3 `Refusal` to report, because Algorithm 3 was never reached
    // -- so `hierarchical_refusal` stays `None`. The reason is carried in the
    // standing instead, so this stays distinguishable from both a successful
    // analysis and a real Algorithm 3 refusal below.
    let causal_plan = match PddlCausalAnalyzer.analyze(&epoch, &occurrences) {
        Ok(p) => p,
        Err(e) => {
            return (
                fallback_sequential(plan),
                CognitiveProjectionStanding::CausalAnalysisFailed(e),
                None,
            );
        }
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
    use bcinr_powl::receipt::execution_v2::verify_execution_v2;

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
