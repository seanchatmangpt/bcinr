//! bcinr-powl — Partially Ordered Workflow Language runtime for bcinr.
//!
//! # Nightly features
//!
//! This crate uses `#![feature(adt_const_params)]` to allow [`TopologyKind`]
//! as a const generic parameter in the [`typestate`] module.
//!
//! `generic_const_exprs` is reserved for future tape-size proofs but is not
//! yet enabled here because it conflicts with existing `const { assert! }` blocks
//! in `dispatcher.rs` under that feature's stricter evaluation rules.
#![feature(adt_const_params)]

pub mod admit;
pub mod compiler;
pub mod enterprise;
pub mod dispatcher;
pub mod scheduler;
pub mod scheduler_wired;
pub mod tape;
pub mod typestate;
