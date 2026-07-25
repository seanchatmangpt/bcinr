# PDEP and PEXT Usage and Fallbacks in BCINR

## Overview
In the BCINR codebase, the `PDEP` (Parallel Bit Deposit) and `PEXT` (Parallel Bit Extract) instructions are critical for branchless bit manipulation. They are fundamental primitives used to execute sequential semantic decisions as constant-time, bit-parallel mechanics without requiring control flow branching.

### Usage
- **PDEP (Parallel Bit Deposit)** maps to `expand_bits_u64`. It deposits the low-order bits of a value into the positions selected by a mask.
- **PEXT (Parallel Bit Extract)** maps to `compress_bits_u64` and `bext_u64`. It gathers the bits of a value selected by a mask and packs them contiguously into the low-order bits.

According to the architecture specifications (`docs/reqs_von_neumann_bypass.md`), these instructions are utilized alongside SWAR (SIMD Within A Register) construction to:
- Gather dispersed numeric bounds or boolean flags from state vectors into contiguous registers for bulk capacity evaluation.
- Enable multi-lane evaluation logic by shifting and masking without data-dependent control flow hazards.

## Fallback Mechanisms
Because `PDEP` and `PEXT` are architecture-specific (BMI2), BCINR requires strictly conforming software fallbacks for architectures that do not support them. These fallbacks must adhere to the project's **Radon Law ($CC=1$)**, meaning they must operate with zero data-dependent branches, loops, or allocations.

### The "Hacker's Delight" Unrolled Branchless Contract
To satisfy the $CC=1$ mandate, the authoritative fallbacks (`bcinr-logic/src/algorithms/expand_bits_u64.rs`, `compress_bits_u64.rs`, and `bext_u64.rs`) are implemented using algorithms adapted from *Hacker's Delight*.

These are not implemented as standard bit-by-bit `while` loops. Instead, they are completely unrolled into **6 fixed, parallel-prefix stages**:
- The operations consist purely of arithmetic bitwise primitives (`^`, `|`, `&`, `<<`, `>>`).
- They execute in strictly constant time regardless of the mask or value data, avoiding timing side-channels and keeping control flow entirely data-independent.

### `simd_dispatch.rs` and the Loop Fallbacks
The repository also contains `pdep_u64` and `pext_u64` in `crates/bcinr-logic/src/simd_dispatch.rs`. These implementations utilize a `while i < 64` loop to simulate the behavior. However, project maturity reports (`maturity_results.txt`) correctly flag these looping variants for Jump Conditional Code (JCC) violations:
> `Issues: JCC detected in pdep_u64, JCC detected in pext_u64`

Consequently, the fully unrolled, 6-stage constant-time implementations (`expand_bits_u64` and `compress_bits_u64`) are the true compliant "Academic-grade" fallbacks that correctly pass the structural gates and independent oracle testing without triggering the bcinr cheat scanner.

## Summary
In line with BCINR's overarching principle of "Rich semantics upstream, fixed deterministic mechanics downstream," the usage of PDEP/PEXT enables high-performance branchless state transitions. When native instructions are missing, the framework falls back onto fully unrolled, 6-stage algorithmic SWAR equivalents to preserve strict execution determinism and guarantee $CC=1$ compliance across all target architectures.
