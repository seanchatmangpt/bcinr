# PDEP and PEXT Instructions in BCINR: Branchless Execution and Architecture Rules

## What are PDEP and PEXT?
`PDEP` (Parallel Bits Deposit) and `PEXT` (Parallel Bits Extract) are advanced bit manipulation instructions typically provided by the x86 BMI2 instruction set. 
- **`PEXT` (Gather)**: Extracts scattered bits from a source value at the positions specified by a mask, packing them contiguously into the least significant bits (LSB) of the result.
- **`PDEP` (Scatter)**: Takes contiguous bits from the LSB of a source value and deposits them into the result at positions specified by a mask, leaving non-masked bit positions as zero.

## Governance under `@von_neumann_bypass` (Rule 4)
Under Rule 4 of the BCINR Constitution (`AGENTS.md`), the `@von_neumann_bypass` role (Architect of Arithmetic Logic) has exclusive authority over branchless arithmetic design, SWAR (SIMD Within A Register) construction, and the admitted use of PDEP/PEXT.

In the `bcinr` deterministic substrate, sequential semantic decisions must be transformed into bit-parallel masks and arithmetic selection. `PDEP` and `PEXT` are essential tools for achieving this without branching:
- **SWAR Operations & Evaluation**: `PEXT` allows `bcinr` to take dispersed boolean flags, numerical bounds, or configuration bits across a large state vector and gather them into a single, contiguous register. This allows for bulk capacity evaluation and multi-lane logic (via shifting and masking) without iterative variable loops.
- **Branchless State Selection**: `PDEP` enables transactional, masked commits without speculative mutation or branch prediction penalties. Instead of writing `if condition { state.flag = true; }`, logic computes updated packed bits and uses `PDEP` to map them directly back to their sparse locations in a single constant-time bit-scatter operation.

## Rule 22 and Architecture-Specific Requirements
Rule 22 explicitly dictates that architecture-specific instructions like PDEP/PEXT require:
1. An admitted target capability.
2. A lawful fallback target or a typed refusal.
3. Separate disassembly evidence.

### Why a "Lawful Fallback Target or Typed Refusal" is Strictly Required
BCINR is a deterministic computational substrate. The most absolute runtime law is the **Radon Law ($CC=1$)**, which demands that the full authoritative call graph contain zero data-dependent conditional branches, zero panic paths, and zero loop backedges.

If a runtime is executing on an architecture that lacks native `PDEP`/`PEXT` support (e.g., older x86 chips or different ISAs), a naive software fallback would iterate over the 64 bits with a `while` loop and an `if` statement to check the mask. **This is strictly prohibited.** A branching fallback would introduce Jump Conditional Code (JCC) violations and timing side-channels, completely undermining the branchless guarantee.

To protect the substrate, Rule 22 strictly requires:
1. **Lawful Fallback Target**: The software fallback must satisfy the identical structural laws as the primary hardware intrinsic (bounded, zero allocation, $CC=1$). In `bcinr`, this is accomplished through fully unrolled, 6-stage constant-time parallel-prefix algorithms adapted from *Hacker's Delight* (as seen in `expand_bits_u64.rs`). These fallbacks use pure bitwise logic (`^`, `&`, `|`, `<<`, `>>`) to guarantee execution time remains data-independent.
2. **Typed Refusal**: If an architecture lacks native support and a branchless $O(1)$ software fallback cannot be built, the system is forbidden from degrading into branching, panicking, or returning default data. Instead, it must immediately return a deterministic **Typed Refusal** (e.g., an `Err(SupportMismatch)`).

This uncompromising rule guarantees that `bcinr` logic remains physically incapable of timing side-channels and structurally branchless regardless of the deployment target.
