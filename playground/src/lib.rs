//! Playground crate for the branchless Process Intelligence engine implementations.
//! Adheres strictly to `#![no_std]` and zero heap allocations.

#![no_std]

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
