//! `bcinr-mfw-ir` — shared IR types and trait contracts for the
//! multifractal-workflow (MFW) planner.
//!
//! This crate contains shared IR types and trait contracts. PDDL/POWL-specific
//! execution remains in downstream crates. Unsafe code is forbidden.

#![forbid(unsafe_code)]

pub mod causal;
pub mod concurrency;
pub mod contracts;
pub mod digest;
pub mod dme_epoch;
pub mod epoch;
pub mod event_set;
pub mod ids;
pub mod outcome;
pub mod projection;

pub use causal::{
    ActionOccurrence, ActionPair, AtomId, CausalAnalyzer, CausalPlan, CausalSupportEdge,
    ConstraintId, DependenceReason, DependenceWitness, EffectsCommuteWitness, FluentId,
    IndependenceRelation, IndependenceVerdict, IndependenceWitness, InvariantId,
    InvariantsStableWitness, NumericFlowWitness, PrecedenceEdge, PreconditionsStableWitness,
    StrictPartialOrder, SupportObject, TrajectoryWitness,
};
pub use concurrency::{
    ConcurrencyAnalyzer, ConcurrencyConflictWitness, ExecutableConcurrencyComplex, MinimalNonFace,
    ResourceConflictWitness,
};
pub use contracts::{
    ContractError, FormalLawRef, FormalStanding, SemanticOptimizationContract,
    LAW_CONCURRENCY_COMPLEX_DOWNWARD_CLOSED, LAW_CROWN_KERNEL_CHARACTERIZATION,
    LAW_EXECUTABLE_CONCURRENCY_INTERSECTION, LAW_MINIMAL_NONFACE_REPRESENTATION,
    LAW_OBSERVABLE_IFF_FIBER_CONSTANT, LAW_QLENS_RATIO, LAW_SPECTRUM_ESTIMATOR,
};
pub use digest::Digest;
pub use dme_epoch::{
    advance_epoch, DmeEpoch, EpochAdvance, EpochAuthority, EpochRefusal, ReceiptFeedback,
};
pub use epoch::{DescentMeter, EpochBounds};
pub use event_set::{EventSet, EventSetIter, EVENT_WORDS, MAX_EPOCH_EVENTS};
pub use ids::{
    ActionOccurrenceId, ConsequenceHorizonId, MeasureProfileId, PlanningEpochId, PowlNodeId,
    SearchProfileId, SelectorProfileId, TransformationProfileId,
};
pub use outcome::{
    BoundHit, BoundKind, ExhaustionWitness, InconsistencyWitness, PlannerFailure, PlannerOutcome,
    UnsupportedFeature,
};
pub use projection::{
    ActionNodeBijection, ConcurrencyPreservationWitness, OrderPreservationWitness,
    PowlProjectionWitness, PowlProjector,
};
