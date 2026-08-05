//! Execution, projection, planning, and replay evidence.
//!
//! The legacy receipt family remains available for legacy POWL tapes. The
//! production POWL v2 rail uses [`execution_v2`] to commit to the compiled
//! tape, concurrency guards, every fired set, final state, and replay chain.
//!
//! This was the standalone `bcinr-powl-receipt` crate through 26.7.28. It was
//! folded in here because it is a subsystem of POWL execution rather than an
//! independent surface — it only ever depended on `bcinr-powl`, never the
//! reverse — and because it was the workspace's last Criterion holdout (see
//! [`crate::receipt_worker`], which implements the same
//! admissibility-before-sealing rule on the hot path).
//!
//! Note: [`ocel_emit`] uses `unsafe` for its bump arena, so this module does
//! not carry the `#![forbid(unsafe_code)]` that most of `bcinr-powl` does.

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
