//! C ABI staticlib surface over the `wasm4games` pattern kernels.
//!
//! # What this crate is
//!
//! `wasm4games-capi` compiles to a `staticlib` that exposes four C-callable symbols over
//! the same branchless kernels the Rust test-suite exercises. A C program can link the
//! archive, include the generated header, and call the kernels without any Rust toolchain
//! at runtime.
//!
//! # How to use it
//!
//! 1. Build the staticlib: `cargo build --release -p wasm4games-capi`
//! 2. The archive lands at `target/release/libwasm4games_capi.a`.
//! 3. Generate (or hand-write) the C header by mirroring the four declarations below.
//! 4. Compile your C program and link it against the archive:
//!    ```text
//!    cc harness.c -L target/release -lwasm4games_capi -o harness
//!    ```
//!
//! # Exported symbols
//!
//! | Symbol | Signature | Purpose |
//! |--------|-----------|---------|
//! | [`w4g_pattern_count`] | `uint32_t (void)` | Total registered kernels |
//! | [`w4g_kernel`] | `uint64_t (uint16_t, uint64_t, uint64_t)` | Dispatch one kernel by id |
//! | [`w4g_corpus_digest`] | `uint64_t (void)` | Full corpus digest from C-linked build |
//! | [`w4g_golden_corpus_digest`] | `uint64_t (void)` | Pinned native Rust golden digest |
//!
//! # Portability proof
//!
//! If `w4g_corpus_digest() == w4g_golden_corpus_digest()` the C-linked build produced
//! byte-identical results to the native Rust build — the same pattern law across two
//! languages from one source. This is the executable core of the portability falsifier.
//!
//! Offline-pure: depends only on the `no_std` `wasm4games` core (no git/network deps), so
//! it is a safe workspace member and builds in CI (unlike the excluded `wasm4games-wasm4pm`
//! bridge, which pulls external git repos).

use wasm4games::corpus;
use wasm4games::patterns::PATTERN_REGISTRY;

/// Returns the number of registered pattern kernels.
///
/// Use this to validate `pattern_id` bounds before calling [`w4g_kernel`].
///
/// # Safety
///
/// No pointer arguments. Safe to call from any C translation unit that links the
/// `wasm4games-capi` staticlib. This function follows the platform C calling convention
/// (`extern "C"`).
///
/// # Returns
///
/// The count of all kernels in the wasm4games pattern registry. A return value of `0`
/// indicates an empty registry and should be treated as a configuration error.
#[no_mangle]
#[must_use]
pub extern "C" fn w4g_pattern_count() -> u32 {
    PATTERN_REGISTRY.len() as u32
}

/// Dispatches a single pattern kernel by id over a packed-u64 `(state, input)` ABI.
///
/// # Safety
///
/// No pointer arguments. All three parameters are passed by value; there are no
/// lifetime or aliasing constraints. Safe to call from C as long as `pattern_id` is
/// within `[0, w4g_pattern_count())`. Out-of-range ids are handled defensively by
/// [`wasm4games::corpus::dispatch`].
///
/// # Returns
///
/// The packed `u64` output of the requested kernel. Semantics are kernel-specific; see
/// the wasm4games pattern registry for field layout.
#[no_mangle]
#[must_use]
pub extern "C" fn w4g_kernel(pattern_id: u16, state: u64, input: u64) -> u64 {
    corpus::dispatch(pattern_id, state, input)
}

/// Recomputes the full corpus digest from this C-linked build.
///
/// Folds every pattern kernel over its test corpus and produces a single `u64` rolling
/// hash. Compare against [`w4g_golden_corpus_digest`] to verify cross-language
/// equivalence.
///
/// # Safety
///
/// No pointer arguments. Pure computation; safe to call from any C context.
///
/// # Returns
///
/// A `u64` digest reflecting every registered kernel's output over the shared corpus.
/// This value should equal [`w4g_golden_corpus_digest`] on a correct build.
#[no_mangle]
#[must_use]
pub extern "C" fn w4g_corpus_digest() -> u64 {
    corpus::corpus_digest()
}

/// Returns the pinned native golden digest.
///
/// This constant is compiled into the staticlib from the Rust side so a C harness can
/// compare it against the dynamically computed [`w4g_corpus_digest`] without
/// hardcoding a magic number in C source.
///
/// # Safety
///
/// No pointer arguments. Returns a compile-time constant; safe to call from any C
/// context.
///
/// # Returns
///
/// The canonical `GOLDEN_CORPUS_DIGEST` from [`wasm4games::corpus`]. A mismatch
/// between this value and [`w4g_corpus_digest`] indicates an ABI regression.
#[no_mangle]
#[must_use]
pub extern "C" fn w4g_golden_corpus_digest() -> u64 {
    corpus::GOLDEN_CORPUS_DIGEST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_count_is_nonzero() {
        assert!(
            w4g_pattern_count() > 0,
            "w4g_pattern_count() returned 0 — pattern registry is empty"
        );
    }

    #[test]
    fn corpus_digest_matches_golden() {
        let got = w4g_corpus_digest();
        let want = w4g_golden_corpus_digest();
        assert_eq!(
            got, want,
            "C-ABI corpus digest (0x{got:016X}) does not match golden (0x{want:016X}) \
             — portability proof failed"
        );
    }
}
