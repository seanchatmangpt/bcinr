//! Slow-rail DTOs and routing utilities for downstream PDDL → POWL consumers.
//!
//! These types are serialization and integration utilities. They are not part
//! of the authoritative branchless scheduler call graph and perform no external
//! actuation. Two execution standings are kept distinct:
//!
//! - witnessed-concurrent STRIPS/typing, including PDDL state replay; and
//! - exact richer classical PDDL, conservatively projected as a sequence.

#![cfg(feature = "mfw-planner")]

use bcinr_mfw_ir::PlannerFailure;
use bcinr_powl::receipt::execution_v2::{digest_tape, verify_execution_v2};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use serde::{Deserialize, Serialize};

use crate::cognitive::{
    plan_exact_cognitive_workflow_bounded, ExactCognitiveError, ExactCognitiveWorkflow,
};
use crate::ground_v2::{EXACT_MAX_GROUND_ACTIONS, EXACT_MAX_PLAN_DEPTH, EXACT_MAX_SEARCH_STATES};
use crate::mfw::planner::MfwPlanError;
use crate::{PddlPowlConfig, PddlPowlError, PddlPowlExecution, PddlPowlRuntime};

/// One admitted scheduler tick. `actions` excludes silent POWL transitions;
/// `fired_mask` still commits to the complete tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PddlPowlBatch {
    pub tick: u32,
    pub fired_mask: u64,
    pub actions: Vec<String>,
}

/// Portable receipt-and-result view for the witnessed-concurrent rail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PddlPowlExecutionSummary {
    pub version: u16,
    pub planning_receipt_root: String,
    pub projection_receipt_root: String,
    pub powl_execution_root: String,
    pub state_execution_root: String,
    pub batches: Vec<PddlPowlBatch>,
    pub final_state: Vec<String>,
    pub goal_reached: bool,
    pub cache_hit: bool,
}

impl PddlPowlExecutionSummary {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl PddlPowlExecution {
    /// Return typed scheduler batches for dispatch adapters and UIs.
    pub fn batches(&self) -> Result<Vec<PddlPowlBatch>, PddlPowlError> {
        self.execution_batches()?
            .into_iter()
            .zip(self.powl_receipt.fired_masks.iter().copied())
            .enumerate()
            .map(|(tick, (actions, fired_mask))| {
                Ok(PddlPowlBatch {
                    tick: tick as u32,
                    fired_mask,
                    actions,
                })
            })
            .collect()
    }

    /// Manufacture a stable downstream DTO after verifying both receipt rails.
    pub fn summary(&self) -> Result<PddlPowlExecutionSummary, PddlPowlError> {
        self.verify()?;
        Ok(PddlPowlExecutionSummary {
            version: 1,
            planning_receipt_root: self.workflow.planning_receipt.hash.to_string(),
            projection_receipt_root: self.workflow.projection_receipt.hash.to_string(),
            powl_execution_root: self.powl_receipt.chain_root.clone(),
            state_execution_root: self.state_receipt.chain_root.clone(),
            batches: self.batches()?,
            final_state: self.final_state_labels(),
            goal_reached: self.state_receipt.goal_reached,
            cache_hit: self.workflow.cache_hit,
        })
    }
}

/// The semantic standing used for one downstream execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CognitiveExecutionStanding {
    /// STRIPS/typing plan with pairwise independence witnesses and parallel
    /// PDDL state replay for each POWL tick.
    WitnessedConcurrentStrips,
    /// Rich classical PDDL evaluated exactly, then projected sequentially so
    /// no concurrency is invented without a rich-semantics independence proof.
    ExactSequentialClassical,
}

/// Portable result common to both production rails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitivePddlExecutionSummary {
    pub version: u16,
    pub standing: CognitiveExecutionStanding,
    /// Root that binds the semantic input, selected plan, and POWL execution.
    pub execution_root: String,
    pub powl_execution_root: String,
    pub batches: Vec<PddlPowlBatch>,
    /// Available for the concurrent STRIPS rail, whose PDDL state is replayed
    /// tick-by-tick. The exact sequential rail currently exposes a deterministic
    /// semantic replay receipt instead of its internal rich numeric state.
    pub final_state: Option<Vec<String>>,
    pub goal_reached: bool,
    pub cache_hit: bool,
}

impl CognitivePddlExecutionSummary {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Bounds for the exact richer-classical rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactCognitiveBounds {
    pub max_ground_actions: usize,
    pub max_plan_depth: usize,
    pub max_search_states: usize,
}

impl Default for ExactCognitiveBounds {
    fn default() -> Self {
        Self {
            max_ground_actions: EXACT_MAX_GROUND_ACTIONS,
            max_plan_depth: EXACT_MAX_PLAN_DEPTH,
            max_search_states: EXACT_MAX_SEARCH_STATES,
        }
    }
}

/// Unified downstream routing configuration.
#[derive(Debug, Clone, Default)]
pub struct CognitivePddlConfig {
    pub concurrent: PddlPowlConfig,
    pub exact: ExactCognitiveBounds,
}

/// Errors from the unified downstream boundary.
#[derive(Debug)]
pub enum CognitivePddlError {
    Concurrent(PddlPowlError),
    Exact(ExactCognitiveError),
    ExactReplayMismatch,
}

impl std::fmt::Display for CognitivePddlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Concurrent(error) => write!(f, "concurrent PDDL → POWL rail failed: {error}"),
            Self::Exact(error) => write!(f, "exact PDDL → POWL rail failed: {error}"),
            Self::ExactReplayMismatch => {
                write!(f, "exact PDDL → POWL deterministic replay did not match")
            }
        }
    }
}

impl std::error::Error for CognitivePddlError {}

/// One completed execution, preserving which semantic rail earned standing.
///
/// Both heavyweight rail artifacts are boxed so this application-facing handle
/// remains stack-small even as receipt and replay evidence grows.
#[derive(Debug)]
pub enum CognitivePddlExecution {
    Concurrent(Box<PddlPowlExecution>),
    ExactSequential {
        workflow: Box<ExactCognitiveWorkflow>,
        domain_pddl: String,
        problem_pddl: String,
        bounds: ExactCognitiveBounds,
        semantic_root: String,
    },
}

impl CognitivePddlExecution {
    pub fn standing(&self) -> CognitiveExecutionStanding {
        match self {
            Self::Concurrent(_) => CognitiveExecutionStanding::WitnessedConcurrentStrips,
            Self::ExactSequential { .. } => CognitiveExecutionStanding::ExactSequentialClassical,
        }
    }

    /// Replay the same semantic rail and verify its POWL execution receipt.
    pub fn verify(&self) -> Result<(), CognitivePddlError> {
        match self {
            Self::Concurrent(execution) => {
                execution.verify().map_err(CognitivePddlError::Concurrent)
            }
            Self::ExactSequential {
                workflow,
                domain_pddl,
                problem_pddl,
                bounds,
                semantic_root,
            } => {
                let max_ticks = u32::from(workflow.powl.tape.len).saturating_add(1);
                verify_execution_v2(
                    &workflow.execution_receipt,
                    &workflow.powl.tape,
                    &ConcurrencyGuardTable::empty(),
                    max_ticks,
                )
                .map_err(|error| CognitivePddlError::Exact(ExactCognitiveError::Receipt(error)))?;

                let replay = plan_exact_cognitive_workflow_bounded(
                    domain_pddl,
                    problem_pddl,
                    bounds.max_ground_actions,
                    bounds.max_plan_depth,
                    bounds.max_search_states,
                )
                .map_err(CognitivePddlError::Exact)?;
                let original_labels = workflow
                    .plan
                    .ops
                    .iter()
                    .map(|operation| operation.label.as_str())
                    .collect::<Vec<_>>();
                let replay_labels = replay
                    .plan
                    .ops
                    .iter()
                    .map(|operation| operation.label.as_str())
                    .collect::<Vec<_>>();
                if original_labels != replay_labels
                    || workflow.admitted.theory_digest != replay.admitted.theory_digest
                    || workflow.execution_receipt != replay.execution_receipt
                    || digest_tape(&workflow.powl.tape) != digest_tape(&replay.powl.tape)
                    || *semantic_root != exact_semantic_root(workflow)
                    || *semantic_root != exact_semantic_root(&replay)
                {
                    return Err(CognitivePddlError::ExactReplayMismatch);
                }
                Ok(())
            }
        }
    }

    /// Scheduler ticks with silent transitions filtered from the action list.
    pub fn batches(&self) -> Result<Vec<PddlPowlBatch>, CognitivePddlError> {
        match self {
            Self::Concurrent(execution) => {
                execution.batches().map_err(CognitivePddlError::Concurrent)
            }
            Self::ExactSequential { workflow, .. } => {
                let activity_slots = workflow.powl.activity_slots.to_vec();
                Ok(workflow
                    .execution_receipt
                    .fired_masks
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(tick, fired_mask)| {
                        let actions = activity_slots
                            .iter()
                            .filter(|(slot, _)| fired_mask & (1u64 << *slot) != 0)
                            .map(|(_, offset)| {
                                workflow.powl.tape.label_slab.get(*offset).to_string()
                            })
                            .collect();
                        PddlPowlBatch {
                            tick: tick as u32,
                            fired_mask,
                            actions,
                        }
                    })
                    .collect())
            }
        }
    }

    /// The receipt root downstream brokers should bind into their own actuation receipt.
    pub fn execution_root(&self) -> &str {
        match self {
            Self::Concurrent(execution) => &execution.state_receipt.chain_root,
            Self::ExactSequential { semantic_root, .. } => semantic_root,
        }
    }

    /// Manufacture a portable summary after receipt/replay verification.
    pub fn summary(&self) -> Result<CognitivePddlExecutionSummary, CognitivePddlError> {
        self.verify()?;
        let (powl_execution_root, final_state, goal_reached, cache_hit) = match self {
            Self::Concurrent(execution) => (
                execution.powl_receipt.chain_root.clone(),
                Some(execution.final_state_labels()),
                execution.state_receipt.goal_reached,
                execution.workflow.cache_hit,
            ),
            Self::ExactSequential { workflow, .. } => (
                workflow.execution_receipt.chain_root.clone(),
                None,
                true,
                false,
            ),
        };
        Ok(CognitivePddlExecutionSummary {
            version: 1,
            standing: self.standing(),
            execution_root: self.execution_root().to_string(),
            powl_execution_root,
            batches: self.batches()?,
            final_state,
            goal_reached,
            cache_hit,
        })
    }
}

/// Stateful router. It attempts witnessed concurrency first and falls back to
/// the exact sequential rail only when the first rail issues a typed
/// `Unsupported` capability refusal. Other failures are never hidden.
pub struct CognitivePddlRuntime {
    concurrent: PddlPowlRuntime,
    config: CognitivePddlConfig,
}

impl CognitivePddlRuntime {
    pub fn new(config: CognitivePddlConfig) -> Self {
        Self {
            concurrent: PddlPowlRuntime::new(config.concurrent.clone()),
            config,
        }
    }

    pub fn execute(
        &mut self,
        domain_pddl: &str,
        problem_pddl: &str,
    ) -> Result<CognitivePddlExecution, CognitivePddlError> {
        match self.concurrent.execute(domain_pddl, problem_pddl) {
            Ok(execution) => Ok(CognitivePddlExecution::Concurrent(Box::new(execution))),
            Err(PddlPowlError::Plan(MfwPlanError::Admission(PlannerFailure::Unsupported(_)))) => {
                let bounds = self.config.exact;
                let workflow = plan_exact_cognitive_workflow_bounded(
                    domain_pddl,
                    problem_pddl,
                    bounds.max_ground_actions,
                    bounds.max_plan_depth,
                    bounds.max_search_states,
                )
                .map_err(CognitivePddlError::Exact)?;
                let semantic_root = exact_semantic_root(&workflow);
                Ok(CognitivePddlExecution::ExactSequential {
                    workflow: Box::new(workflow),
                    domain_pddl: domain_pddl.to_string(),
                    problem_pddl: problem_pddl.to_string(),
                    bounds,
                    semantic_root,
                })
            }
            Err(error) => Err(CognitivePddlError::Concurrent(error)),
        }
    }
}

impl Default for CognitivePddlRuntime {
    fn default() -> Self {
        Self::new(CognitivePddlConfig::default())
    }
}

/// Stateless unified convenience function.
pub fn execute_cognitive_pddl(
    domain_pddl: &str,
    problem_pddl: &str,
) -> Result<CognitivePddlExecution, CognitivePddlError> {
    CognitivePddlRuntime::default().execute(domain_pddl, problem_pddl)
}

fn exact_semantic_root(workflow: &ExactCognitiveWorkflow) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:exact-cognitive-pddl-powl:v1");
    hasher.update(workflow.admitted.theory_digest.as_bytes());
    hasher.update(workflow.execution_receipt.chain_root.as_bytes());
    hasher.update(digest_tape(&workflow.powl.tape).as_bytes());
    for operation in &workflow.plan.ops {
        hasher.update(&(operation.label.len() as u64).to_le_bytes());
        hasher.update(operation.label.as_bytes());
    }
    for choice in &workflow.powl.selected_choices {
        hasher.update(&(choice.graph_depth as u64).to_le_bytes());
        hasher.update(&(choice.from as u64).to_le_bytes());
        hasher.update(&(choice.to as u64).to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_summary_is_json_serializable_and_receipt_complete() {
        let execution = crate::execute_pddl_to_powl(
            "(define (domain d) (:requirements :strips) (:predicates (p)) \
             (:action make-p :parameters () :precondition () :effect (p)))",
            "(define (problem p) (:domain d) (:init) (:goal (p)))",
        )
        .unwrap();
        let summary = execution.summary().unwrap();
        let json = summary.to_json().unwrap();
        assert!(json.contains("state_execution_root"));
        assert_eq!(summary.batches[0].actions, vec!["make-p".to_string()]);
        assert!(summary.goal_reached);
    }

    #[test]
    fn router_uses_concurrent_rail_for_strips() {
        let execution = execute_cognitive_pddl(
            "(define (domain d) (:requirements :strips) (:predicates (a) (b)) \
             (:action a :parameters () :precondition () :effect (a)) \
             (:action b :parameters () :precondition () :effect (b)))",
            "(define (problem p) (:domain d) (:init) (:goal (and (a) (b))))",
        )
        .unwrap();
        assert_eq!(
            execution.standing(),
            CognitiveExecutionStanding::WitnessedConcurrentStrips
        );
        execution.verify().unwrap();
        assert!(execution.summary().unwrap().final_state.is_some());
    }

    #[test]
    fn router_falls_back_to_exact_rail_for_adl() {
        let execution = execute_cognitive_pddl(
            "(define (domain d) (:requirements :adl :typing) (:types item) \
             (:predicates (ready ?x - item) (done ?x - item)) \
             (:action finish-all :parameters () \
               :precondition (forall (?x - item) (ready ?x)) \
               :effect (forall (?x - item) (when (ready ?x) (done ?x)))))",
            "(define (problem p) (:domain d) (:objects a b - item) \
             (:init (ready a) (ready b)) (:goal (and (done a) (done b))))",
        )
        .unwrap();
        assert_eq!(
            execution.standing(),
            CognitiveExecutionStanding::ExactSequentialClassical
        );
        assert!(execution
            .batches()
            .unwrap()
            .iter()
            .any(|batch| batch.actions == vec!["finish-all".to_string()]));
        execution.verify().unwrap();
        let summary = execution.summary().unwrap();
        assert!(summary.final_state.is_none());
        assert_ne!(summary.execution_root, summary.powl_execution_root);
    }

    #[test]
    fn tampered_exact_semantic_root_is_refused() {
        let mut execution = execute_cognitive_pddl(
            "(define (domain d) (:requirements :adl) (:predicates (done)) \
             (:action finish :parameters () :precondition () :effect (done)))",
            "(define (problem p) (:domain d) (:init) (:goal (done)))",
        )
        .unwrap();
        let CognitivePddlExecution::ExactSequential { semantic_root, .. } = &mut execution else {
            panic!("ADL should route to exact sequential standing");
        };
        semantic_root.push('0');
        assert!(matches!(
            execution.verify(),
            Err(CognitivePddlError::ExactReplayMismatch)
        ));
    }
}
