//! Universe64 — UniverseOS Petri64³ Institutional State Geometry
//!
//! Representing institutional workflow reality as a fixed 32 KiB
//! executable Boolean universe, executed under the Dual-Plane model:
//! - **Data Plane (D)**: 32 KiB resident `UniverseBlock`
//! - **Scratch Plane (S)**: 32 KiB bounded `UniverseScratch`
//! - **L1D-class envelope**: 64 KiB total
//!
//! Tiers: T0 (≤2 ns), T1 (≤200 ns), T2 (≤5 µs), T3 (≤10 µs), T4 (external).

// Core spec
pub mod constants;
pub mod block;
pub mod coord;
pub mod instruction;
pub mod masks;
pub mod receipt;
pub mod scratch;
pub mod transition;

// Wave A — Geometry index + admission lifecycle
pub mod index_plane;
pub mod admission_lifecycle;

// Wave B — Law, admission evaluation, boundary, conformance, drift
pub mod law;
pub mod admission;
pub mod boundary;
pub mod conformance;
pub mod drift;

// Wave C — Delta tape + bus
pub mod delta_tape;
pub mod delta_bus;

// Wave D — Projection, ready mask, scope planner
pub mod projection;
pub mod ready_mask;
pub mod scope_planner;

// Wave E — RL, probability, scenario, executor
pub mod rl;
pub mod probability;
pub mod scenario;
pub mod executor;

// Wave F — Capability tokens
pub mod ubit_capability;

// Wave G — Snapshot / fork / restore
pub mod ubit_image;

// Wave H — Self-healing supervisor
pub mod ubit_supervisor;

// Wave I — 2 MiB operating field (alloc-gated)
pub mod ubit_field;

// ---------------------------------------------------------------------------
// Re-exports — core spec
// ---------------------------------------------------------------------------
pub use block::{UniverseBlock, UniverseDelta};
pub use constants::{
    ACTIVE_WORD_CAPACITY, CELL_COUNT, COORD_BITS, COORD_MASK, DOMAIN_COUNT,
    FNV1A_64_OFFSET_BASIS, FNV1A_64_PRIME, L1_ENVELOPE_BYTES, MAX_WORD_INDEX, PLACE_COUNT,
    SCRATCH_BYTES, SIZE_BYTES, T0_BUDGET_NS, T1_BUDGET_NS, T2_BUDGET_NS, T3_BUDGET_NS,
    T4_EXTERNAL, TOTAL_PLACES, UNIVERSE_WORDS,
};
pub use coord::UCoord;
pub use instruction::{UInstrKind, UInstruction, UTier};
pub use masks::{BoundaryMask, CellMask, DomainMask, UniverseMask};
pub use receipt::{new_receipt, receipt_mix_transition, TransitionReceipt};
pub use scratch::{ActiveWordSet, UDelta, UScope, UniverseScratch};
pub use transition::{
    boundary_handoff, cell_missing_prerequisites, cell_try_fire, compute_universe_delta,
    domain_hamming_distance, institutional_conformance_distance,
};

// ---------------------------------------------------------------------------
// Re-exports — Wave A
// ---------------------------------------------------------------------------
pub use index_plane::{
    BitTransitionIndex, BoundaryWordIndex, IndexPlane, TransitionWordIndex, WordProjectionIndex,
    WordBit, coord_to_word_bit,
    MAX_TRANSITIONS, MAX_PROJECTIONS, MAX_BOUNDARIES,
};
pub use admission_lifecycle::{
    GeometryCompiler, MaskBank, MaskEntry, MaskKind, TransitionEntry, TransitionRegistry,
    VerifyResult, verify_cell_mask, verify_transition,
    MASK_BANK_CAPACITY, TRANSITION_REGISTRY_CAPACITY,
};

// ---------------------------------------------------------------------------
// Re-exports — Wave B
// ---------------------------------------------------------------------------
pub use law::{CellLaw, DomainLaw, LawKernel, UniverseLaw, word_violation};
pub use admission::{AdmissionDecision, AdmissionEvaluator, cell_admit, eval_cell, eval_sparse, eval_full, missing_prerequisites};
pub use boundary::{BoundaryKernel, BoundaryResult, BoundaryStatus, BoundarySide};
pub use conformance::{ConformanceKernel, ConformanceState, conformance_distance};
pub use drift::{DriftKernel, DriftReport, ExpectedModel, word_drift, word_drift_bits};

// ---------------------------------------------------------------------------
// Re-exports — Wave C
// ---------------------------------------------------------------------------
pub use delta_tape::{DeltaTape, TapePosition, DELTA_TAPE_CAPACITY};
pub use delta_bus::{DeltaBus, Subscriber, SubscriberChannel, MAX_SUBSCRIBERS};

// ---------------------------------------------------------------------------
// Re-exports — Wave D
// ---------------------------------------------------------------------------
pub use projection::{
    ProjectionCache, ProjectionDescriptor, ProjectionOp, ProjectionRegistry, ProjectionUpdater,
};
pub use ready_mask::{ReadyMask, READY_MASK_WORDS};
pub use scope_planner::{ScopePlanner, ScopePlanResult};

// ---------------------------------------------------------------------------
// Re-exports — Wave E
// ---------------------------------------------------------------------------
pub use rl::{RLKernel, RLState, RewardSpec, RewardTable};
pub use probability::{FixedHistogram, PopcountHistogram, HISTOGRAM_BUCKETS};
pub use scenario::{ScenarioConfig, ScenarioRunner, ScenarioSummary, splitmix64, MAX_SCENARIO_STEPS};
pub use executor::{EpochStats, ExecutorState, UniverseExecutor, MAX_EPOCH_INSTRUCTIONS};

// ---------------------------------------------------------------------------
// Re-exports — Wave F: UBitCapability
// ---------------------------------------------------------------------------
pub use ubit_capability::{
    UBitCapability, UBitCapabilityId, UBitCapabilityTable, NullCapability,
    UBIT_CAPABILITY_TABLE_CAPACITY, cap_admit,
};
pub use admission::{eval_cell_with_cap, eval_sparse_with_cap, eval_domain_with_cap, eval_full_with_cap};

// ---------------------------------------------------------------------------
// Re-exports — Wave G: UBitImage
// ---------------------------------------------------------------------------
pub use ubit_image::{UBitImage, UBitScratchSnapshot};

// ---------------------------------------------------------------------------
// Re-exports — Wave H: UBitScopePlanner (in scope_planner module)
// ---------------------------------------------------------------------------
pub use scope_planner::{
    KernelClass, PlanDecision, CostModel, UBitScopePlanner,
};

// ---------------------------------------------------------------------------
// Re-exports — Wave I: UBitSupervisor
// ---------------------------------------------------------------------------
pub use ubit_supervisor::{
    UBitSupervisor, SupervisorCounters, SupervisorThresholds, RecoveryKind,
    UBIT_SUPERVISOR_RING_CAPACITY, UBIT_FAMILY_COUNT,
};

// ---------------------------------------------------------------------------
// Re-exports — Wave J: UBitField (alloc-gated)
// ---------------------------------------------------------------------------
#[cfg(feature = "alloc")]
pub use ubit_field::{UBitField, UBitPlaneRole, UBIT_FIELD_PLANE_COUNT, UBIT_FIELD_BYTES};
pub use ubit_field::{
    ROLE_LAW, ROLE_CAPABILITY, ROLE_EXPECTED, ROLE_REWARD_GOOD, ROLE_REWARD_BAD,
    ROLE_POLICY, ROLE_VALUE, ROLE_DRIFT, ROLE_PROJECTION_0, ROLE_SCENARIO_0,
    ROLE_CHECKPOINT_0, ROLE_CUSTOM_0,
};
