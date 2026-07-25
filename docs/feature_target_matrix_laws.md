# Rule 22: Feature and Target Matrix Law

In the `bcinr` deterministic substrate, Rule 22 mandates that all constitutional verification gates—including source-level cheat scanning, structural enforcement of `CC=1`, and release-profile object-code disassembly—must be executed across **every** supported combination of feature flags (`default`, `no default`, `all`) and target architectures.

## Why Every Combination Must Be Verified
The core premise of `bcinr` is whole-call-graph branchlessness and zero-allocation determinism (the "Radon Law"). Executing gates on a single configuration is mathematically insufficient because:
1. **Compiler Backend Variance:** Rust source code that compiles to branchless, straight-line assembly on `x86_64` might trigger implicit conditional jumps, loop backedges, or runtime library calls on `aarch64` or `wasm32` depending on how different LLVM backends optimize numeric or logical operations.
2. **Feature-Bound Instability:** A project might perfectly conform to `#![no_std]` and branchless execution under `default features`, but conditionally activating or deactivating features could silently link branching primitives, panic handlers, or heap allocations.

## Preventing Conditional Compile-Time Cheats and Untested Fallbacks
In traditional systems programming, it is standard practice to conditionally compile a highly optimized architectural intrinsic (such as BMI2's `PDEP` or `PEXT` on `x86_64`) while providing a simpler, branching fallback (like a variable `while` loop) for unsupported environments via `#[cfg(...)]` attributes.

Rule 22 actively prevents developers from using conditional compilation to bypass constitutional laws:
- **Eradicating Dead-Path Compliance (CHEAT-007):** A developer cannot pass structural audits by writing a lawful hot path for CI, while hiding an unlawful, branching `while` loop behind `#[cfg(not(target_arch = "x86_64"))]`. By demanding audits on all supported target configurations, every fallback is exposed to the `@turing_machine` enforcer.
- **Lawful Fallbacks:** The rule explicitly dictates that any fallback implementation must satisfy the *exact same structural laws* as the primary implementation. A generic SWAR (SIMD Within A Register) fallback must pass the same independent oracle tests, hostile mutant injections, and exact object-code disassembly for its target as the hardware-accelerated version.
- **Typed Refusals:** If a specific target architecture cannot mathematically or structurally support a branchless fallback, `bcinr` forbids the developer from compromising the runtime with branches or panics. Instead, the implementation must structurally return a branchless `TypedRefusal` (e.g., `SupportMismatch`).

By rigidly testing every coordinate in the feature-target matrix, Rule 22 guarantees that the substrate's execution shape remains absolutely deterministic and mathematically bounded, regardless of environment or build configuration.
