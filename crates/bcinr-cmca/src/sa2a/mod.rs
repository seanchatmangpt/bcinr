//! SA2A third-consumer boundary for CMCA.
//! This module consumes powerless PreparedEffect/resource/receipt identities only.
//! It never mints authority or performs effects.
pub mod admission;
pub mod contract;
pub mod interop;
pub mod ocel;
pub mod prepared_effect_ref;
pub mod receipt;
pub mod recovery;
pub mod replay;
pub mod resource_envelope;
pub mod symbolic_envelope;

pub use admission::{admit, Refusal};
pub use contract::{Envelope, RecoveryDecision, CONTRACT};
pub use resource_envelope::{Allocation, ResourceEnvelope};
pub use symbolic_envelope::{
    peak_reusable_usage, pipeline_latency, AffineResourceExpr, PipelineKnobs, SymbolDomain,
    SymbolicFamily, SymbolicRefusal, SymbolicWitness, TimedAllocation,
};
