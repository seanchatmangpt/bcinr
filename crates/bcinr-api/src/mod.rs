//! Public API for the BranchlessCInRust library.
//!
//! This module provides a high-level, stable interface for the branchless algorithms
//! implemented in the `logic` module.

/// Bitset Algebra — public API
pub mod bitset;
/// DFA-based parsing — public API
pub mod dfa;
/// Fixed-point arithmetic — public API
pub mod fix;
/// Low-level integer bit manipulation — public API
pub mod int;
/// Bitwise masks and selection — public API
pub mod mask;
/// Sorting and permutation networks — public API
pub mod network;
/// Fast decimal and number parsing — public API
pub mod parse;
/// Vectorized horizontal reductions — public API
pub mod reduce;
/// Linear and selective scans — public API
pub mod scan;
/// SIMD-accelerated low-level primitives — public API
pub mod simd;
/// Fast hashing and sketching — public API
pub mod sketch;
/// UTF-8 validation and processing — public API
pub mod utf8;

pub use bitset::*;
pub use dfa::*;
pub use fix::*;
pub use int::*;
pub use mask::*;
pub use network::*;
pub use parse::*;
pub use reduce::*;
pub use scan::*;
pub use simd::*;
pub use sketch::*;
pub use utf8::*;
