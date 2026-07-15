//! bcinr-powl-receipt — OCEL event emission and causal receipt generation for bcinr-powl.
//!
//! # Two receipt families
//!
//! The original family (`causal_receipt`, `replay`, `conformance`,
//! `denial`, `ocel_emit`, `pm_bridge`, `intern`) attests to
//! execution/replay conformance of an **already-compiled** POWL tape —
//! untouched by this phase.
//!
//! A new family, landing incrementally starting with [`chain`] (the shared
//! BLAKE3 hash-chain fold every new receipt kind uses), attests to
//! something this crate had zero concept of before: that a PDDL-to-POWL
//! projection preserved source semantics, that individual scheduler ticks
//! are attested, and that a whole planning epoch's evidence bundles into
//! one chained receipt.

pub mod causal_receipt;
pub mod chain;
pub mod conformance;
pub mod denial;
pub mod intern;
pub mod ocel_emit;
pub mod pm_bridge;
pub mod replay;
