# SIMD Dispatch and Feature Flags in BCINR

BCINR achieves extreme performance through native hardware acceleration (SIMD, BMI2) while strictly adhering to its core architectural mandate: the **Radon Law ($CC=1$)**, which prohibits data-dependent branches, loops, and runtime feature detection.

Here is how the system safely dispatches to native instructions versus SWAR (SIMD Within A Register) fallbacks while maintaining mathematical equivalence and branchless constraints.

## 1. Zero-Overhead Compile-Time Dispatch

BCINR completely avoids runtime CPU feature detection (such as `is_x86_feature_detected!`). Runtime checks inherently require control-flow branches (`if` statements), which violate the strict $CC=1$ rule of the authoritative hot path.

Instead, dispatch is handled entirely at compile-time using Rust's `cfg` feature flags in `crates/bcinr-logic/src/simd_dispatch.rs`:
- Public API functions conditionally compile their bodies using `#[cfg(all(target_arch = "...", target_feature = "..."))]`.
- Based on the target compiler flags, the function routes to either the x86 fast-path (e.g., `sse4.2,ssse3`), the AArch64 fast-path (e.g., `neon`), or a portable scalar SWAR fallback.
- The hardware-specific functions are annotated with `#[target_feature(enable = "...")]` to assure the compiler that the required CPU intrinsics are available.

## 2. Academic-Grade SWAR Fallbacks

When native instructions (like SIMD vectors or BMI2 `PDEP`/`PEXT`) are unavailable on the target architecture, BCINR must still guarantee $CC=1$ compliance. It does this via deeply optimized, purely algebraic SWAR algorithms.

**SIMD Fallbacks (`simd_dispatch.rs`)**:
Scalar fallbacks simulate SIMD operations without branching. For example, `max_u8x16_scalar` avoids conditional `if (a > b)` logic by computing a branchless XOR-mask pattern to implement a `select` operation: `b ^ ((a ^ b) & mask)`.

**BMI2 PDEP/PEXT Fallbacks (`docs/pdep_pext_fallback.md`)**:
Native `PDEP` (Parallel Bit Deposit) and `PEXT` (Parallel Bit Extract) instructions are highly efficient for branchless bit manipulation. When BMI2 is unavailable, standard `while` loop fallbacks trigger Jump Conditional Code (JCC) scanner violations. To maintain $CC=1$, BCINR relies on fully unrolled, 6-stage parallel-prefix algorithms adapted from *Hacker's Delight* (found in `expand_bits_u64.rs` and `compress_bits_u64.rs`). These operate entirely via constant-time arithmetic primitives (`^`, `|`, `&`, `<<`, `>>`).

## 3. Identical Oracles and Hoare-Logic Verification

To guarantee that the native hardware paths and the SWAR fallbacks behave exactly identically, the system utilizes **Independent Oracles**.

- **The Oracle (Slow Rail)**: For every primitive, an independent reference function is written (e.g., `bext_u64_reference`). As dictated by `AGENTS.md`, this oracle is allowed to use standard branching and loops (the "slow rail") because it is never invoked in the authoritative hot path.
- **Verification Invariant**: The Hoare-logic invariant explicitly requires: `{ hardware path ≡ scalar fallback ≡ oracle }`. Both the native implementation and the SWAR fallback are exhaustively tested against the oracle.
- **Hostile Mutants**: The test suite employs hostile counterfactual mutants (e.g., identity bluffs, bit-skips) to prove that any single-bit deviation from the oracle is caught, granting the module "PhD-Verified" standing.

## 4. Isolation of Unsafe Code

Calling CPU intrinsics natively requires `unsafe` code. The BCINR codebase strictly enforces `#![forbid(unsafe_code)]` with very few exceptions.
- `simd_dispatch.rs` is granted `#![allow(unsafe_code)]` exclusively to call hardware intrinsics.
- Every `unsafe` block must be heavily documented with a `// SAFETY:` proof demonstrating that pointer accesses (like stack-allocated array buffers) and alignments are mathematically sound before the intrinsic is invoked, maintaining Rust's memory safety guarantees without runtime bounds checking.
