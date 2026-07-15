//! bcinr-powl-receipt — OCEL event emission and causal receipt generation for bcinr-powl.
//!
//! # Two receipt families
//!
//! The original family (`causal_receipt`, `replay`, `conformance`,
//! `denial`, `ocel_emit`, `pm_bridge`, `intern`) attests to
//! execution/replay conformance of an **already-compiled** POWL tape —
//! untouched by this phase.
//!
//! The new family ([`chain`], [`projection`], [`execution`], [`planning`])
//! attests to something this crate had zero concept of before: that a
//! PDDL-to-POWL **projection** (the compilation step itself) preserved
//! source semantics, that a single scheduler tick's firing decision is
//! attested ([`execution::ExecutionReceipt`]), and that a whole planning
//! epoch's evidence bundles into one chained receipt
//! ([`planning::PlanningReceipt`]). See each module's doc comment for
//! exactly what is real versus what is a stated scope boundary.

pub mod causal_receipt;
pub mod chain;
pub mod conformance;
pub mod denial;
pub mod execution;
pub mod intern;
pub mod ocel_emit;
pub mod planning;
pub mod pm_bridge;
pub mod projection;
pub mod replay;
