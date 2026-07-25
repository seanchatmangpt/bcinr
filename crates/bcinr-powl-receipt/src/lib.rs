//! bcinr-powl-receipt — execution, projection, planning, and replay evidence.
//!
//! The legacy receipt family remains available for legacy POWL tapes. The
//! production POWL v2 rail uses [`execution_v2`] to commit to the compiled
//! tape, concurrency guards, every fired set, final state, and replay chain.

pub mod causal_receipt;
pub mod chain;
pub mod conformance;
pub mod denial;
pub mod execution;
pub mod execution_v2;
pub mod intern;
pub mod ocel_emit;
pub mod planning;
pub mod pm_bridge;
pub mod projection;
pub mod replay;
