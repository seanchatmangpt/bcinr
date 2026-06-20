//! Candidate process evidence emitted by pattern kernels.
//!
//! wasm4games *emits* candidate evidence; admission/refusal is the job of the external
//! `wasm4pm` authority (see [`crate::compat`]). The types here are intentionally shaped
//! to map 1:1 onto `wasm4pm-compat` canonical types.
//!
//! - [`ocel`]: object-centric events (OCEL-style).
//! - [`otel`]: 16-bit runtime span codes.
//! - [`receipt`]: tamper-evident rolling receipt chain (FNV-1a, via `bcinr_logic`).
//! - [`replay`]: deterministic replay frames.

pub mod ocel;
pub mod otel;
pub mod receipt;
pub mod replay;
