//! C ABI surface over the `wasm4games` kernels — the cross-language portability proof.
//!
//! This crate compiles to a `staticlib` so a C program can link it and call the same
//! branchless kernels the Rust tests exercise. If the C-linked
//! [`w4g_corpus_digest`] equals the native [`wasm4games::corpus::GOLDEN_CORPUS_DIGEST`],
//! the pattern law produced identical results across two languages from one source — the
//! executable core of the portability falsifier.
//!
//! Offline-pure: depends only on the `no_std` `wasm4games` core (no git/network deps), so
//! it is a safe workspace member and builds in CI (unlike the excluded `wasm4games-wasm4pm`
//! bridge, which pulls external git repos).
//!
//! # Unsafe-code policy
//!
//! This crate intentionally omits `#![forbid(unsafe_code)]` because `#[no_mangle]` — which
//! is required to export C symbols from a `staticlib` — is classified by the Rust
//! `unsafe_code` lint as an unsafe attribute. There are no `unsafe` blocks, `unsafe fn`
//! definitions, or raw-pointer operations anywhere in this file. Every exported function
//! delegates unconditionally to the safe, `#![forbid(unsafe_code)]`-guarded `wasm4games`
//! core. The exemption is structural (C ABI export), not algorithmic.

use wasm4games::corpus;
use wasm4games::patterns::PATTERN_REGISTRY;

/// Number of patterns in the registry.
#[no_mangle]
pub extern "C" fn w4g_pattern_count() -> u32 {
    PATTERN_REGISTRY.len() as u32
}

/// Run a single pattern kernel by id over a packed-u64 `(state, input)` ABI.
#[no_mangle]
pub extern "C" fn w4g_kernel(pattern_id: u16, state: u64, input: u64) -> u64 {
    corpus::dispatch(pattern_id, state, input)
}

/// Recompute the full corpus digest (the portability oracle) from this C-linked build.
#[no_mangle]
pub extern "C" fn w4g_corpus_digest() -> u64 {
    corpus::corpus_digest()
}

/// The pinned native golden digest, exported so a C harness can compare without hardcoding.
#[no_mangle]
pub extern "C" fn w4g_golden_corpus_digest() -> u64 {
    corpus::GOLDEN_CORPUS_DIGEST
}
