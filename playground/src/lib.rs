//! Playground crate for the branchless Process Intelligence engine implementations.
//! Adheres strictly to `#![no_std]` and zero heap allocations.

#![no_std]
#![allow(
    clippy::unwrap_used,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::unused_self,
    clippy::inline_always,
    clippy::needless_range_loop,
    clippy::large_stack_arrays,
    clippy::pub_underscore_fields,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::unreadable_literal,
    dead_code,
    clippy::enum_variant_names,
    clippy::mutable_key_type,
    clippy::string_extend_chars,
    clippy::vec_init_then_push,
    clippy::large_types_passed_by_value,
    clippy::wrong_self_convention,
)]

/// BranchTorch: Branchless Neural Training
pub mod branchtorch;
/// Branchless Chess Bitboards
pub mod chess;
/// Branchless Chess Validation Matrix (POWL v2)
pub mod chess_validator;
/// Branchless Binarized Graph Neural Network
pub mod gnn;
pub mod hoeg;
/// Kogge-Stone Branchless FIDE Legal Move Generators
pub mod legal_moves;
/// Branchless NNUE Distillation Logic
pub mod nnue;
/// Petri net token replay engine.
pub mod petri;
/// POWL compiler/executor.
pub mod powl;
/// Temporal Event Knowledge Graph compiler.
pub mod tekg;
/// WASM boundary interface.
pub mod wasm;
/// YAWL routing engine.
pub mod yawl;
