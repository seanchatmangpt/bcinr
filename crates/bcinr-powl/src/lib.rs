//! bcinr-powl — Partially Ordered Workflow Language runtime for bcinr.
//!
//! # Nightly features
//!
//! - `adt_const_params`: allows [`crate::typestate::TopologyKind`] as a const
//!   generic parameter.
//! - `generic_const_exprs`: enables `[u64; N]` in const generic position,
//!   unlocking compile-time topology encoding in [`const_scheduler`] and
//!   512-op wide tapes in [`scheduler_wide`].
//!
//! The former `const { assert! }` blocker in `dispatcher.rs` has been resolved
//! by moving the bound to an `impl`-level `const _OPS_BOUND` item.
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![allow(
    incomplete_features,
    clippy::unnecessary_cast,
    clippy::needless_range_loop
)] // generic_const_exprs is still being stabilised

pub mod ocel;
pub use ocel::ConformanceResult;
pub mod admit;
pub mod auto_select_bridge;
pub mod auto_select_execution_dispatch;
pub mod auto_select_final_integration;
pub mod auto_select_pipeline;
pub mod auto_select_refusal_aggregation;
pub mod compiler;
pub mod const_scheduler;
pub mod dispatcher;
pub mod enterprise;
pub mod full_mapek_loop;
pub mod mapek_loop;
pub mod model;
pub mod projection;
pub mod receipt_worker;
pub mod scheduler;
pub mod scheduler_wide;
pub mod scheduler_wired;
pub mod tape;
pub mod typestate;
