//! Production PDDL → POWL v2 planning, execution, and replay facade.
//!
//! This module is the downstream boundary for cognitive-breed composition. It
//! hides the internal admission, search, causal-analysis, concurrency, POWL
//! compilation, scheduling, and receipt plumbing behind one bounded API.
//!
//! The state executor uses parallel STRIPS semantics for every POWL scheduler
//! tick: all action preconditions are checked against the same pre-state, batch
//! interference is refused, then aggregate deletes and adds are committed. The
//! resulting state must equal the validated sequential plan's final state and
//! satisfy the PDDL goal before a receipt is returned.

#![cfg(feature = "mfw-planner")]

use std::collections::BTreeSet;

use bcinr_mfw_ir::{EpochBounds, PlannerFailure, PowlNodeId, UnsupportedFeature};
use bcinr_powl::compiler::v2::{compile_powl_v2, CompileErrorV2, CompiledPowlV2};
use bcinr_powl_receipt::execution_v2::{
    execute_and_seal_v2, verify_execution_v2, PowlV2ExecutionReceipt, PowlV2ReceiptError,
};
use wasm4pm_compat::pddl::{Pddl8GroundAction, Pddl8GroundAtom};

use crate::capability::{CapabilityProfile, SemanticSupport};
use crate::causal_v2::PddlCausalAnalyzerV2;
use crate::concurrency::PddlConcurrencyAnalyzer;
use crate::consequence::GoalReachabilityHorizon;
use crate::mfw::planner::{MfwPlanError, MfwPlanner, PlannedWorkflow};
use crate::mfw::{QLensError, QValue};
use crate::parse::{domain31_from_pddl, problem31_from_pddl};
use crate::production_capability::ProductionCapabilityProfile;
use crate::semantic_features::content_features;

/// Production PDDL → causal/concurrency → POWL composition rail.
pub type ProductionMfwPlanner = MfwPlanner<
    GoalReachabilityHorizon,
    PddlCausalAnalyzerV2,
    PddlConcurrencyAnalyzer,
    bcinr_powl::projection::PowlProjector,
>;

/// Bounded runtime configuration for downstream consumers.
#[derive(Debug, Clone)]
pub struct PddlPowlConfig {
    pub bounds: EpochBounds,
    pub exploit_q: QValue,
    pub max_gap: usize,
    pub max_search_ticks: usize,
    pub max_execution_ticks: u32,
}

impl PddlPowlConfig {
    /// Replace the q-lens exponent while preserving all other bounds.
    pub fn with_exploit_q(mut self, q: f64) -> Result<Self, QLensError> {
        self.exploit_q = QValue::new(q)?;
        Ok(self)
    }
}

impl Default for PddlPowlConfig {
    fn default() -> Self {
        Self {
            bounds: EpochBounds {
                max_ground_actions: 4_096,
                max_plan_depth: 64,
                max_search_steps: 100_000,
                max_partition_boxes: 64,
            },
            exploit_q: QValue::new(1.0).expect("1.0 is a finite q-lens exponent"),
            max_gap: 4,
            max_search_ticks: 100_000,
            max_execution_ticks: 64,
        }
    }
}

/// Errors from the complete PDDL → POWL execution boundary.
#[derive(Debug)]
pub enum PddlPowlError {
    Plan(MfwPlanError),
    Compile(CompileErrorV2),
    PowlExecution(PowlV2ReceiptError),
    MissingProvenance {
        node: u64,
    },
    MissingOccurrence {
        occurrence: u32,
    },
    ActionIndexOutOfRange {
        action_index: u64,
    },
    PreconditionFailed {
        tick: u32,
        action: String,
        atom: String,
    },
    BatchInterference {
        tick: u32,
        left: String,
        right: String,
        atom: String,
    },
    GoalNotReached,
    ParallelReplayMismatch,
    StateReceiptMismatch,
}

impl std::fmt::Display for PddlPowlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plan(e) => write!(f, "PDDL planning failed: {e}"),
            Self::Compile(e) => write!(f, "POWL v2 compilation failed: {e:?}"),
            Self::PowlExecution(e) => write!(f, "POWL v2 execution failed: {e}"),
            Self::MissingProvenance { node } => {
                write!(f, "POWL node {node} has no source-action provenance")
            }
            Self::MissingOccurrence { occurrence } => {
                write!(
                    f,
                    "POWL provenance references missing occurrence {occurrence}"
                )
            }
            Self::ActionIndexOutOfRange { action_index } => {
                write!(
                    f,
                    "occurrence references action index {action_index} outside the epoch"
                )
            }
            Self::PreconditionFailed { tick, action, atom } => write!(
                f,
                "POWL tick {tick} tried to fire {action:?} without precondition {atom:?}"
            ),
            Self::BatchInterference {
                tick,
                left,
                right,
                atom,
            } => write!(
                f,
                "POWL tick {tick} batched interfering actions {left:?} and {right:?} on {atom:?}"
            ),
            Self::GoalNotReached => write!(
                f,
                "POWL execution completed without satisfying the PDDL goal"
            ),
            Self::ParallelReplayMismatch => write!(
                f,
                "POWL parallel replay final state differs from the validated PDDL plan"
            ),
            Self::StateReceiptMismatch => write!(f, "PDDL state-execution receipt mismatch"),
        }
    }
}

impl std::error::Error for PddlPowlError {}

impl From<MfwPlanError> for PddlPowlError {
    fn from(value: MfwPlanError) -> Self {
        Self::Plan(value)
    }
}

impl From<CompileErrorV2> for PddlPowlError {
    fn from(value: CompileErrorV2) -> Self {
        Self::Compile(value)
    }
}

impl From<PowlV2ReceiptError> for PddlPowlError {
    fn from(value: PowlV2ReceiptError) -> Self {
        Self::PowlExecution(value)
    }
}

/// State-transition receipt chained to a POWL v2 scheduler receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PddlPowlStateReceipt {
    pub version: u16,
    pub initial_state_root: String,
    pub batch_roots: Vec<String>,
    pub final_state_root: String,
    pub goal_root: String,
    pub goal_reached: bool,
    pub powl_chain_root: String,
    pub chain_root: String,
}

/// Planned and compiled workflow, ready for bounded execution.
#[derive(Debug)]
pub struct PddlPowlPlan {
    pub workflow: PlannedWorkflow,
    pub compiled: CompiledPowlV2,
    max_execution_ticks: u32,
}

impl PddlPowlPlan {
    /// Execute the POWL geometry and the corresponding PDDL state transitions.
    pub fn execute(self) -> Result<PddlPowlExecution, PddlPowlError> {
        let powl_receipt = execute_and_seal_v2(
            &self.compiled.tape,
            &self.compiled.guards,
            self.max_execution_ticks,
        )?;
        verify_execution_v2(
            &powl_receipt,
            &self.compiled.tape,
            &self.compiled.guards,
            self.max_execution_ticks,
        )?;

        let (final_state, state_receipt) = replay_pddl_trace(&self.workflow, &powl_receipt)?;

        Ok(PddlPowlExecution {
            workflow: self.workflow,
            compiled: self.compiled,
            powl_receipt,
            state_receipt,
            final_state,
            max_execution_ticks: self.max_execution_ticks,
        })
    }
}

/// Complete execution artifact for downstream actuation, audit, and replay.
#[derive(Debug)]
pub struct PddlPowlExecution {
    pub workflow: PlannedWorkflow,
    pub compiled: CompiledPowlV2,
    pub powl_receipt: PowlV2ExecutionReceipt,
    pub state_receipt: PddlPowlStateReceipt,
    pub final_state: BTreeSet<Pddl8GroundAtom>,
    max_execution_ticks: u32,
}

impl PddlPowlExecution {
    /// Replay both the POWL scheduler and PDDL state-transition receipts.
    pub fn verify(&self) -> Result<(), PddlPowlError> {
        verify_execution_v2(
            &self.powl_receipt,
            &self.compiled.tape,
            &self.compiled.guards,
            self.max_execution_ticks,
        )?;
        let (final_state, state_receipt) = replay_pddl_trace(&self.workflow, &self.powl_receipt)?;
        if final_state != self.final_state || state_receipt != self.state_receipt {
            return Err(PddlPowlError::StateReceiptMismatch);
        }
        Ok(())
    }

    /// Human-readable action labels grouped by simultaneous POWL tick.
    pub fn execution_batches(&self) -> Result<Vec<Vec<String>>, PddlPowlError> {
        self.powl_receipt
            .fired_masks
            .iter()
            .map(|&mask| action_labels_for_mask(&self.workflow, mask))
            .collect()
    }

    /// Deterministic labels for every fact true after execution.
    pub fn final_state_labels(&self) -> Vec<String> {
        self.final_state
            .iter()
            .map(Pddl8GroundAtom::label)
            .collect()
    }

    /// Query one concrete final-state atom without exposing collection internals.
    pub fn contains_fact(&self, predicate: &str, args: &[&str]) -> bool {
        self.final_state.contains(&Pddl8GroundAtom {
            pred: predicate.to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        })
    }
}

/// Stateful downstream runtime. The internal consequence cache is preserved
/// across calls, while every request still emits independent receipts.
pub struct PddlPowlRuntime {
    planner: ProductionMfwPlanner,
    config: PddlPowlConfig,
}

impl PddlPowlRuntime {
    pub fn new(config: PddlPowlConfig) -> Self {
        let planner = ProductionMfwPlanner::new(
            GoalReachabilityHorizon,
            bcinr_powl::projection::PowlProjector,
            config.bounds,
            config.exploit_q,
            config.max_gap,
            config.max_search_ticks,
        );
        Self { planner, config }
    }

    /// Admit, plan, project, and compile without actuating the workflow.
    pub fn plan(
        &mut self,
        domain_pddl: &str,
        problem_pddl: &str,
    ) -> Result<PddlPowlPlan, PddlPowlError> {
        admit_concurrent_content(domain_pddl, problem_pddl)?;
        let workflow =
            self.planner
                .plan(domain_pddl, problem_pddl, &ProductionCapabilityProfile)?;
        let compiled = compile_powl_v2(&workflow.powl_model)?;
        Ok(PddlPowlPlan {
            workflow,
            compiled,
            max_execution_ticks: self.config.max_execution_ticks,
        })
    }

    /// One-call PDDL text → POWL v2 execution → receipt/replay.
    pub fn execute(
        &mut self,
        domain_pddl: &str,
        problem_pddl: &str,
    ) -> Result<PddlPowlExecution, PddlPowlError> {
        self.plan(domain_pddl, problem_pddl)?.execute()
    }
}

impl Default for PddlPowlRuntime {
    fn default() -> Self {
        Self::new(PddlPowlConfig::default())
    }
}

/// Stateless convenience function for downstream callers that do not need to
/// preserve the planning cache across requests.
pub fn execute_pddl_to_powl(
    domain_pddl: &str,
    problem_pddl: &str,
) -> Result<PddlPowlExecution, PddlPowlError> {
    PddlPowlRuntime::default().execute(domain_pddl, problem_pddl)
}

fn admit_concurrent_content(domain_pddl: &str, problem_pddl: &str) -> Result<(), PddlPowlError> {
    let domain = domain31_from_pddl(domain_pddl)
        .map_err(|error| PddlPowlError::Plan(MfwPlanError::Parse(error)))?;
    let problem = problem31_from_pddl(problem_pddl)
        .map_err(|error| PddlPowlError::Plan(MfwPlanError::Parse(error)))?;

    if !domain.processes.is_empty() || !domain.events.is_empty() {
        return Err(unsupported_content(
            "PDDL+",
            "PDDL+ process/event blocks have no admitted concurrent execution rail",
        ));
    }

    for feature in content_features(&domain, &problem) {
        if ProductionCapabilityProfile.support(feature) == SemanticSupport::Unsupported {
            return Err(unsupported_content(
                &format!("{feature:?}"),
                &format!(
                    "parsed task content uses PddlFeature::{feature:?}, which the concurrent \
                     ProductionCapabilityProfile marks Unsupported regardless of omitted \
                     :requirements declarations"
                ),
            ));
        }
    }
    Ok(())
}

fn unsupported_content(feature_name: &str, context: &str) -> PddlPowlError {
    PddlPowlError::Plan(MfwPlanError::Admission(PlannerFailure::Unsupported(
        UnsupportedFeature {
            feature_name: feature_name.to_string(),
            context: context.to_string(),
        },
    )))
}

fn replay_pddl_trace(
    workflow: &PlannedWorkflow,
    powl_receipt: &PowlV2ExecutionReceipt,
) -> Result<(BTreeSet<Pddl8GroundAtom>, PddlPowlStateReceipt), PddlPowlError> {
    let mut state = workflow.epoch.initial_state.clone();
    let initial_state_root = digest_state(&state);
    let goal_root = digest_goal(&workflow.epoch.goal);
    let mut batch_roots = Vec::with_capacity(powl_receipt.fired_masks.len());
    let mut chain = blake3::hash(b"bcinr:pddl-powl-state:v1");

    for (tick, &mask) in powl_receipt.fired_masks.iter().enumerate() {
        let actions = actions_for_mask(workflow, mask)?;
        let before_root = digest_state(&state);

        for action in &actions {
            if let Some(missing) = action
                .preconditions
                .iter()
                .find(|precondition| !state.contains(*precondition))
            {
                return Err(PddlPowlError::PreconditionFailed {
                    tick: tick as u32,
                    action: action.label.clone(),
                    atom: missing.label(),
                });
            }
        }

        for left_index in 0..actions.len() {
            for right_index in (left_index + 1)..actions.len() {
                if let Some(atom) = interference_atom(actions[left_index], actions[right_index]) {
                    return Err(PddlPowlError::BatchInterference {
                        tick: tick as u32,
                        left: actions[left_index].label.clone(),
                        right: actions[right_index].label.clone(),
                        atom: atom.label(),
                    });
                }
            }
        }

        let mut deletes = BTreeSet::new();
        let mut adds = BTreeSet::new();
        for action in &actions {
            deletes.extend(action.del_effects.iter().cloned());
            adds.extend(action.add_effects.iter().cloned());
        }
        for atom in deletes {
            state.remove(&atom);
        }
        state.extend(adds);

        let after_root = digest_state(&state);
        let batch_root = digest_batch(tick as u32, mask, &actions, &before_root, &after_root);
        let mut hasher = blake3::Hasher::new();
        hasher.update(chain.as_bytes());
        hasher.update(powl_receipt.chain_root.as_bytes());
        hasher.update(batch_root.as_bytes());
        chain = hasher.finalize();
        batch_roots.push(batch_root);
    }

    let goal_reached = workflow.epoch.goal.iter().all(|goal| state.contains(goal));
    if !goal_reached {
        return Err(PddlPowlError::GoalNotReached);
    }
    if state != workflow.validated_plan.result.final_state {
        return Err(PddlPowlError::ParallelReplayMismatch);
    }

    let final_state_root = digest_state(&state);
    let mut final_hasher = blake3::Hasher::new();
    final_hasher.update(chain.as_bytes());
    final_hasher.update(final_state_root.as_bytes());
    final_hasher.update(goal_root.as_bytes());
    final_hasher.update(&[goal_reached as u8]);
    let chain_root = final_hasher.finalize().to_hex().to_string();

    Ok((
        state,
        PddlPowlStateReceipt {
            version: 1,
            initial_state_root,
            batch_roots,
            final_state_root,
            goal_root,
            goal_reached,
            powl_chain_root: powl_receipt.chain_root.clone(),
            chain_root,
        },
    ))
}

fn actions_for_mask<'a>(
    workflow: &'a PlannedWorkflow,
    mask: u64,
) -> Result<Vec<&'a Pddl8GroundAction>, PddlPowlError> {
    let mut actions = Vec::new();
    let mut remaining = mask;
    while remaining != 0 {
        let slot = remaining.trailing_zeros() as usize;
        remaining &= remaining - 1;
        actions.push(action_for_slot(workflow, slot)?);
    }
    Ok(actions)
}

fn action_labels_for_mask(
    workflow: &PlannedWorkflow,
    mask: u64,
) -> Result<Vec<String>, PddlPowlError> {
    actions_for_mask(workflow, mask).map(|actions| {
        actions
            .into_iter()
            .map(|action| action.label.clone())
            .collect()
    })
}

fn action_for_slot<'a>(
    workflow: &'a PlannedWorkflow,
    slot: usize,
) -> Result<&'a Pddl8GroundAction, PddlPowlError> {
    let node = PowlNodeId(slot as u64);
    let occurrence_id = workflow
        .powl_model
        .provenance
        .get(&node)
        .copied()
        .ok_or(PddlPowlError::MissingProvenance { node: slot as u64 })?;
    let occurrence = workflow
        .causal_plan
        .occurrences
        .iter()
        .find(|occurrence| occurrence.id == occurrence_id)
        .ok_or(PddlPowlError::MissingOccurrence {
            occurrence: occurrence_id.0,
        })?;
    workflow
        .epoch
        .actions
        .get(occurrence.action as usize)
        .ok_or(PddlPowlError::ActionIndexOutOfRange {
            action_index: occurrence.action,
        })
}

fn interference_atom(
    left: &Pddl8GroundAction,
    right: &Pddl8GroundAction,
) -> Option<Pddl8GroundAtom> {
    left.del_effects
        .iter()
        .find(|atom| right.preconditions.contains(*atom) || right.add_effects.contains(*atom))
        .cloned()
        .or_else(|| {
            right
                .del_effects
                .iter()
                .find(|atom| left.preconditions.contains(*atom) || left.add_effects.contains(*atom))
                .cloned()
        })
}

fn digest_state(state: &BTreeSet<Pddl8GroundAtom>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:pddl-state:v1");
    for atom in state {
        let label = atom.label();
        hasher.update(&(label.len() as u64).to_le_bytes());
        hasher.update(label.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn digest_goal(goal: &[Pddl8GroundAtom]) -> String {
    let sorted: BTreeSet<Pddl8GroundAtom> = goal.iter().cloned().collect();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:pddl-goal:v1");
    for atom in sorted {
        let label = atom.label();
        hasher.update(&(label.len() as u64).to_le_bytes());
        hasher.update(label.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn digest_batch(
    tick: u32,
    mask: u64,
    actions: &[&Pddl8GroundAction],
    before_root: &str,
    after_root: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:pddl-powl-batch:v1");
    hasher.update(&tick.to_le_bytes());
    hasher.update(&mask.to_le_bytes());
    hasher.update(before_root.as_bytes());
    hasher.update(after_root.as_bytes());
    for action in actions {
        hasher.update(&(action.label.len() as u64).to_le_bytes());
        hasher.update(action.label.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEPENDENT_DOMAIN: &str = "(define (domain d) \
        (:requirements :strips) \
        (:predicates (ready) (left-done) (right-done)) \
        (:action left :parameters () :precondition (ready) :effect (left-done)) \
        (:action right :parameters () :precondition (ready) :effect (right-done)))";
    const INDEPENDENT_PROBLEM: &str = "(define (problem p) (:domain d) \
        (:init (ready)) (:goal (and (left-done) (right-done))))";

    #[test]
    fn production_runtime_executes_independent_actions_in_one_powl_tick() {
        let execution = execute_pddl_to_powl(INDEPENDENT_DOMAIN, INDEPENDENT_PROBLEM).unwrap();
        execution.verify().unwrap();
        let batches = execution.execution_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], vec!["left", "right"]);
        assert!(execution.contains_fact("left-done", &[]));
        assert!(execution.contains_fact("right-done", &[]));
        assert!(execution.state_receipt.goal_reached);
    }

    #[test]
    fn dependent_actions_execute_in_distinct_ticks() {
        let execution = execute_pddl_to_powl(
            "(define (domain d) (:requirements :strips) \
             (:predicates (start) (middle) (done)) \
             (:action first :parameters () :precondition (start) \
              :effect (and (middle) (not (start)))) \
             (:action second :parameters () :precondition (middle) :effect (done)))",
            "(define (problem p) (:domain d) (:init (start)) (:goal (done)))",
        )
        .unwrap();
        assert_eq!(
            execution.execution_batches().unwrap(),
            vec![vec!["first".to_string()], vec!["second".to_string()]]
        );
        execution.verify().unwrap();
    }

    #[test]
    fn tampered_state_receipt_is_refused() {
        let mut execution = execute_pddl_to_powl(INDEPENDENT_DOMAIN, INDEPENDENT_PROBLEM).unwrap();
        execution.state_receipt.final_state_root.push('0');
        assert!(matches!(
            execution.verify(),
            Err(PddlPowlError::StateReceiptMismatch)
        ));
    }

    #[test]
    fn undeclared_rich_semantics_are_refused_before_concurrent_grounding() {
        let error = execute_pddl_to_powl(
            "(define (domain d) (:requirements :strips) (:predicates (locked) (done)) \
             (:action finish :parameters () :precondition (not (locked)) :effect (done)))",
            "(define (problem p) (:domain d) (:init) (:goal (done)))",
        )
        .unwrap_err();
        assert!(matches!(
            error,
            PddlPowlError::Plan(MfwPlanError::Admission(PlannerFailure::Unsupported(_)))
        ));
    }
}
