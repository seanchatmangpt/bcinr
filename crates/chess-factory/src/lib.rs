//! GGEN Chess Factory.
//!
//! A branchless chess decision system manufactured from semantic law
//! (TTL -> SPARQL -> GGEN -> Tera -> Rust). The runtime stations are
//! `no_std`, `#![forbid(unsafe_code)]`, and each public station kernel
//! is held to cyclomatic complexity 1 by the contract gate.
#![no_std]
#![forbid(unsafe_code)]

pub mod station;
pub mod position;
pub mod rays;
pub mod aggregator;
pub mod select;
pub mod stations;
pub mod motifs;
pub mod weights;
pub mod defects;
pub mod evidence;

/// Evidence cells: per-move receipts, Petri conformance, and the replay verifier.
///
/// This module is `std`-backed: it serializes receipts (serde/serde_json),
/// hash-chains them (blake3), and re-derives moves through the `chess` crate
/// boundary during verification.
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod receipts;

/// Manufactured search wrapper (alpha-beta + quiescence + MVV-LVA) over the
/// generated aggregator. `std`-only; the hand-authored search boundary.
#[cfg(feature = "std")]
pub mod search;
