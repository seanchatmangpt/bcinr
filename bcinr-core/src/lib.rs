//! BranchlessCInRust: Academic-grade branchless algorithm library
//!
//! This crate is a facade that re-exports the `bcinr-api` and `bcinr-logic` crates.

#![no_std]
#![deny(missing_docs)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Public facade for the library (re-exported from bcinr-api)
pub use bcinr_api as api;
/// Core algorithmic implementations (re-exported from bcinr-logic)
pub use bcinr_logic as logic;

