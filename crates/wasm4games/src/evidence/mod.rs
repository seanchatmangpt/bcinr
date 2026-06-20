//! Candidate process evidence emitted by pattern kernels.
//!
//! wasm4games *emits* candidate evidence; admission/refusal is the job of the external
//! `wasm4pm` authority (see [`crate::compat`]). The types here are intentionally shaped
//! to map 1:1 onto `wasm4pm-compat` canonical types.
//!
//! # Module overview
//!
//! | Sub-module | Purpose |
//! |---|---|
//! | [`ocel`] | Object-centric events (OCEL-2.0 style). Each event links to typed objects via [`ocel::ObjectRefs`]. |
//! | [`otel`] | 16-bit runtime span codes (OpenTelemetry-inspired). Hot paths never carry strings. |
//! | [`receipt`] | Tamper-evident rolling receipt chain (FNV-1a via `bcinr_logic`). Witnesses execution order. |
//! | [`replay`] | Deterministic replay frames. Re-folding identical frames reproduces an identical digest. |
//!
//! # `no_std` compatibility
//!
//! All types in this module use only `core::*`. The `alloc` feature gates the heap-backed
//! log types ([`ocel::OcelLog`]). All other types are always `const`-constructible with zero
//! heap allocation.
//!
//! # Invariants
//!
//! * All multi-byte integers are encoded as big-endian so the byte stream is identical across
//!   architectures — a prerequisite for cross-host receipt comparison.
//! * OCEL events are emitted with `status` codes that mirror the [`crate::class`] admission
//!   outcome at emission time; `wasm4pm` may re-evaluate admission later.
//! * The receipt chain is append-only and non-cryptographic: it witnesses execution order for
//!   replay comparison, not for adversarial tamper-proof guarantees.

pub mod ocel;
pub mod otel;
pub mod receipt;
pub mod replay;
