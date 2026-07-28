//! bcinr-pddl — PDDL8 → POWL tape → Prolog8 admission → OCEL → BLAKE3
//!
//! # BRCE stack position
//! ```text
//! PDDL8   = candidate-future grammar  (parse + ground modules)
//! POWL    = process geometry          (wasm4pm_compat::pddl::Pddl8Tape)
//! Prolog8 = R ⊢ A gate               (execute module)
//! OCEL    = execution trace           (wasm4pm_compat::ocel::OCEL)
//! BLAKE3  = receipt / replay          (wasm4pm_compat::pddl::Pddl8ExecutionReceipt)
//! ```
//!
//! # Isolation guarantee
//! The PDDL reader is implemented in this crate as a bounded S-expression
//! parser. No third-party PDDL parser or parser license crosses the boundary.
//!
//! `bcinr-powl` and `bcinr-powl-receipt` are optional path dependencies,
//! enabled only by the `mfw-planner` feature. Default consumers do not pull
//! either crate into their dependency graph.
//!
//! Canonical cross-crate types live in `wasm4pm_compat::pddl`.

#![feature(once_cell_try)]

pub mod alloc_counter;
pub mod capability;
pub mod capability_router;
pub mod causal;
pub mod causal_numeric;
pub mod causal_v2;
#[cfg(feature = "mfw-planner")]
pub mod cognitive;
pub mod concurrency;
pub mod consequence;
pub mod dfcm_crown;
#[cfg(feature = "mfw-planner")]
pub mod downstream;
#[cfg(feature = "mfw-planner")]
pub mod embedded;
pub mod error;
/// Real verifier output becomes admitted planning facts. Unconditional: it
/// depends only on `blake3` and `serde`, both non-optional here.
pub mod evidence;
pub mod execute;
pub mod ground;
pub mod ground_v2;
pub mod llm_bridge;
pub mod logical_time;
pub mod mfw;
// The parser deliberately mirrors the canonical external predicate declaration
// tuple so admission does not introduce a second shadow AST.
#[allow(clippy::type_complexity)]
pub mod parse;
pub mod powl_bridge;
#[cfg(feature = "mfw-planner")]
pub mod prelude;
#[cfg(feature = "mfw-planner")]
pub mod problem_builder;
// These explicit lifetimes document that returned ground-action references are
// borrowed from one admitted workflow epoch and cannot outlive its receipt.
#[cfg(feature = "mfw-planner")]
#[allow(clippy::needless_lifetimes)]
pub mod production;
pub mod production_capability;
pub mod rdf_pddl;
pub mod resource_ledger;
pub mod schedule_analysis;
pub mod search;
mod semantic_features;
mod sexpr;
#[cfg(feature = "mfw-planner")]
pub mod task;
pub mod validate;
#[cfg(feature = "mfw-planner")]
pub mod wf_net_bridge;
#[cfg(feature = "mfw-planner")]
pub mod wf_net_bridge_temporal;
#[cfg(feature = "mfw-planner")]
pub mod workflow_cmd;
pub use capability::{
    admit_planning_task, feature_bit, unsupported_mask, AdmittedPlanningTask, CapabilityProfile,
    DefaultCapabilityProfile, GroundedPlanningEpoch, PddlFeature, SemanticSupport,
    ALL_PDDL_FEATURES,
};
pub use capability_router::{
    route_capability_plan, CapabilityRouteReceipt, CapabilityTask, CostVector, DesiredEffect,
};
pub use causal::{CausalAnalysisError, PddlCausalAnalyzer};
pub use causal_v2::PddlCausalAnalyzerV2;
#[cfg(feature = "mfw-planner")]
pub use cognitive::{
    plan_exact_cognitive_workflow, plan_exact_cognitive_workflow_bounded,
    plan_exact_cognitive_workflow_hierarchical, plan_exact_cognitive_workflow_hierarchical_bounded,
    CognitiveProjectionStanding, ExactCognitiveError, ExactCognitiveWorkflow,
};
pub use concurrency::{ConcurrencyAnalysisError, PddlConcurrencyAnalyzer};
pub use consequence::{
    plan_with_standing_cache, ConsequenceHorizon, ExactStateKey, GoalReachabilityHorizon,
    MakespanObservation, MinimumMakespanHorizon, PlanningResult, ResidualDecision,
    ResidualObligation, Residualizer, StandingConsequenceCache,
};
pub use dfcm_crown::{run_dfcm_crown_suite, DfcmBenchReceipt};
#[cfg(feature = "mfw-planner")]
pub use downstream::{
    execute_cognitive_pddl, CognitiveExecutionStanding, CognitivePddlConfig, CognitivePddlError,
    CognitivePddlExecution, CognitivePddlExecutionSummary, CognitivePddlRuntime,
    ExactCognitiveBounds, PddlPowlBatch, PddlPowlExecutionSummary,
};
#[cfg(feature = "mfw-planner")]
pub use embedded::{
    ActionInvocation, ActionLabelError, EmbeddedWorkflow, EmbeddedWorkflowError,
    TypedWorkflowBatch, TypedWorkflowPlan, VerifiedWorkflowPlan, WorkflowBatch, WorkflowProblem,
};
pub use ground_v2::{
    ExactClassicalCapabilityProfile, ExactClassicalError, ExactClassicalProblem, ExactGroundAction,
    LossyLowering, EXACT_MAX_GROUND_ACTIONS, EXACT_MAX_PLAN_DEPTH, EXACT_MAX_SEARCH_STATES,
};
pub use llm_bridge::{
    admit_candidate_domain, admit_candidate_problem, manufacture_world, AdmittedDomain,
    AdmittedProblem, WorldManufactureReceipt,
};
pub use mfw::{
    q_lens, FrontierBoxes, FrontierMeasure, MassVector, PositiveDistribution, PositiveMass,
    QLensError, QValue, WeightedDistribution,
};
#[cfg(feature = "mfw-planner")]
pub use problem_builder::{
    PddlAtomBuilder, PddlBuildError, PddlObjectBuilder, PddlProblemDocument, StripsProblemBuilder,
};
#[cfg(feature = "mfw-planner")]
pub use production::{
    execute_pddl_to_powl, PddlPowlConfig, PddlPowlError, PddlPowlExecution, PddlPowlPlan,
    PddlPowlRuntime, PddlPowlStateReceipt, ProductionMfwPlanner,
};
pub use production_capability::ProductionCapabilityProfile;
pub use resource_ledger::{Resource, ResourceLease, ResourceLedger, ResourceMode, ResourceRefusal};
pub use schedule_analysis::{
    analyze_schedule, analyze_schedule_instrumented, AnalysisSubstageNs, CapacityDelta,
    ScheduleAnalysis64,
};
pub use search::{
    ExactBfsRail, ExactSearchRail, ExactStepOutcome, ExploitSearchRail, ExploitStepOutcome,
    FairRailScheduler, MfwPortfolio, PortfolioOutcome, QLensRail, RailSelection,
};
#[cfg(feature = "mfw-planner")]
pub use task::{execute_cognitive_task, OwnedPddlTask, PddlTask};
#[cfg(feature = "mfw-planner")]
pub use workflow_cmd::*;

// Re-export canonical types from wasm4pm-compat so callers only need one import.
pub use wasm4pm_compat::ocel::{OCELEvent, OCEL};
pub use wasm4pm_compat::pddl::{
    DerivedPredicate, DurationConstraint, DurativeAction, Metric, MetricDir, MetricExpr,
    NumericEffect, NumericExpr, NumericOp, Pddl31Action, Pddl31Domain, Pddl31Problem,
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8ExecutionLog, Pddl8ExecutionReceipt,
    Pddl8GroundAction, Pddl8GroundAtom, Pddl8Problem, Pddl8StepResult, Pddl8Tape, Pddl8TapeOp,
    PddlCondition, PddlConstraint, PddlEffect, PddlEvent, PddlFunction, PddlPreference,
    PddlProcess, PddlType, TemporalExecutionReceipt, TemporalPlan, TemporalPlanStep, TimeSpecifier,
    TimedLiteral, TrajectoryConstraint, PDDL8_MAX_ARITY, PDDL8_MAX_CONJUNCTS, PDDL8_MAX_GROUND,
    PDDL8_MAX_PARAMS, PDDL8_MAX_PLAN_DEPTH,
};

pub use error::{Pddl8Error, PlannerOutcome};
pub use execute::{
    compute_plan_chain, execute_tape, execute_temporal_plan_instrumented, SubstageNs,
};
pub use ground::{
    monitors::{ConstraintMonitor, MonitorFactory, MonitorState},
    trajectory_policy::{ConstraintViolation, TrajectoryPolicy},
    GroundDerivedPredicate, GroundDurativeAction, GroundProblem, GroundTemporalProblem,
    QuantifierDomain, TypeIndex,
};
pub use logical_time::LogicalTime;
pub use parse::{domain31_from_pddl, domain_from_pddl, problem31_from_pddl, problem_from_pddl};
pub use validate::{
    validate_plan, validate_temporal_plan_shape, PlanViolation, TemporalShapeViolation,
};
