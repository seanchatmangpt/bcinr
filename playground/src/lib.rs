//! Playground crate for the branchless Process Intelligence engine implementations.
//! Adheres strictly to `#![no_std]` and zero heap allocations.

#![no_std]

/// Petri net token replay engine.
pub mod petri;
/// YAWL routing engine.
pub mod yawl;
/// POWL compiler/executor.
pub mod powl;
/// WASM boundary interface.
pub mod wasm;
/// Temporal Event Knowledge Graph compiler.
pub mod tekg;
pub mod hoeg;
/// Branchless Binarized Graph Neural Network
pub mod gnn;
/// Branchless Chess Bitboards
pub mod chess;
/// BranchTorch: Branchless Neural Training
pub mod branchtorch;
/// Branchless Chess Validation Matrix (POWL v2)
pub mod chess_validator;
/// Kogge-Stone Branchless FIDE Legal Move Generators
pub mod legal_moves;
/// Branchless NNUE Distillation Logic
pub mod nnue;

