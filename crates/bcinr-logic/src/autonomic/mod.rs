//! Autonomic Computing: Generic MAPE-K building blocks for self-managing systems.
//! 
//! Provides primitives for building autonomic loops without imposing a specific execution model.

pub mod packed_key_table;
pub mod rl_state;
pub mod autonomic_substrate;
pub mod policy_guard;
pub mod metric_accumulator;
#[cfg(feature = "alloc")]
pub mod kernel;

pub use packed_key_table::PackedKeyTable;
pub use rl_state::RlState;
pub use autonomic_substrate::AutonomicSubstrate;
pub use policy_guard::PolicyGuard;
pub use metric_accumulator::MetricAccumulator;
#[cfg(feature = "alloc")]
pub use kernel::{AutonomicKernel, AutonomicState, ActionKind, ActionRisk, AutonomicAction, AutonomicResult, AutonomicFeedback};

/// Generic Autonomic Health/Integrity state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutonomicHealth {
    /// Normalized health score [0.0, 1.0].
    pub score: f32,
    /// System-wide integrity score [0.0, 1.0].
    pub integrity: f32,
}

impl Default for AutonomicHealth {
    fn default() -> Self {
        Self {
            score: 1.0,
            integrity: 1.0,
        }
    }
}
