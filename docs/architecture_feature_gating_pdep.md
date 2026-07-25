# Research Report: Rule 22 and Architecture-Specific Instructions in BCINR

Based on a review of `AGENTS.md` and the internal documentation (e.g., `docs/pdep_pext_hardware_instructions.md`, `docs/feature_and_target_matrix_verification.md`, and `docs/pdep_pext_fallback.md`), here is how Rule 22 and architecture feature gating are handled:

## Rule 22 (Feature and Target Matrix) Overview
Rule 22 dictates that passing structural gates in only a single configuration is strictly insufficient to establish repository standing. All verification gates must run across a combinatorial matrix including:
- `default features`, `no default features`, `all features`
- `release profile`, `test profile where relevant`
- `supported architectures`
- `generated clean tree`

For architecture-specific instructions like `PDEP` and `PEXT`, the matrix strictly requires:
1. An admitted target capability.
2. A lawful fallback target or a typed refusal.
3. Separate disassembly evidence.

## Architecture Feature Gating
Use of architecture-specific hardware intrinsics is tightly controlled and gated. Code utilizing these instructions must isolate them in specific, audited modules, and they are enabled via target-specific configuration attributes. For example, in `simd_dispatch.rs`, the implementation uses attributes like `#[target_feature(enable = "sse4.2,ssse3")]` or `#[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]` to ensure intrinsics are only compiled and called when the underlying target formally supports them at compile time. 

## PDEP / PEXT Handling and Fallbacks

`PDEP` (Parallel Bit Deposit / Scatter) and `PEXT` (Parallel Bit Extract / Gather) are advanced bit manipulation tools (BMI2 on x86) used under the `@von_neumann_bypass` protocol to enforce branchless execution (the Radon Law, $CC=1$). They allow sequential semantic decisions to be evaluated through constant-time bit-parallel logic without requiring control flow branches. 

When native hardware support for `PDEP`/`PEXT` is absent, the fallback mechanisms must obey strict rules:

### 1. Lawful Fallback Targets
A naïve branching fallback (e.g., a `while` loop with an `if` condition) is **strictly prohibited** in the authoritative path because it introduces Jump Conditional Code (JCC) violations and timing side-channels. (For instance, the naive `pdep_u64` loop in `simd_dispatch.rs` is intentionally flagged by the maturity matrix to demonstrate a JCC violation).

Instead, the true, compliant fallbacks (e.g., `expand_bits_u64.rs` for `PDEP` and `compress_bits_u64.rs`/`bext_u64.rs` for `PEXT`) utilize **fully unrolled, 6-stage constant-time parallel-prefix algorithms** (adapted from *Hacker's Delight*). These implementations consist purely of primitive bitwise arithmetic (`^`, `|`, `&`, `<<`, `>>`) without loops or branches, ensuring they satisfy the exact same structural laws ($CC=1$) as the primary hardware intrinsic.

### 2. Typed Refusals
If a target lacks native support and an $O(1)$ constant-time software fallback cannot be constructed, the runtime is forbidden from degrading into a branching loop, panicking, or silently truncating data. Instead, it must immediately yield a deterministic **Typed Refusal** (e.g., `Err(SupportMismatch)`).

### 3. Separate Disassembly Evidence
Using hardware-specific intrinsics and their unrolled fallbacks must both be independently proven at the object-code level. Disassembly evidence must be generated separately for the hardware-accelerated target and the fallback target to definitively prove that neither compilation path introduces a branch, panic, or loop backedge into the final machine code.
