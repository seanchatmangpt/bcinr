# `bcinr` SWAR (SIMD Within A Register) Techniques

## Overview
In `bcinr`, `@von_neumann_bypass` employs SWAR (SIMD Within A Register) to execute state matrix transitions and enforce policy constraints without conditional branches. By evaluating logic strictly via arithmetic across standard 64-bit integer registers, the system avoids unpredictable pipeline stalls (`CC=1` Radon Law), remains deterministic, and is independent of architecture-specific SIMD extensions like AVX/NEON.

## Mask Calculus
At the foundation of this branchless substrate is **Mask Calculus**, where conditionals (`if` / `else`) are completely eliminated in favor of bitwise polynomial arithmetic using standard ALUs.

The core B-Calculus identity is:
```rust
M(mask, a, b) = (mask & a) | (!mask & b)
```
Where `mask` is strictly all-ones (`0xFFFFFFFFFFFFFFFF` representing `true`) or all-zeros (`0x0000000000000000` representing `false`). This fundamentally removes the need to branch. Functions like `is_zero_mask_u32`, `lt_mask_u32`, and `eq_mask_u32` calculate these branchless masks via bitwise identities (e.g., `lt_mask` maps to a `SETB` + `NEG` sequence on x86-64), producing an evaluation guaranteed to execute in deterministic `O(1)` time.

## Evaluating State Matrices Concurrently
The `SwarMarking` Petri-net marking wrapper (`crates/bcinr-logic/src/models/petri.rs`) is a textbook example of how state matrix evaluation executes branchlessly across 64-bit bounds.

1. **Evaluation**:
   The current token state evaluates the condition iteratively over packed words using standard bitwise operators:
   ```rust
   let mut mismatch = 0u64;
   (0..WORDS).for_each(|i| {
       mismatch |= required.words[i] & !self.words[i];
   });
   let is_enabled = mismatch == 0;
   ```
2. **Admission Mask Generation**:
   The `is_enabled` boolean is deterministically converted into an all-ones or all-zeros admission mask without a branch:
   ```rust
   let mask = 0u64.wrapping_sub(is_enabled as u64);
   ```

3. **Concurrent Mask Application**:
   Finally, the transition executes over all lanes simultaneously. The new candidate state is combined with the original state using the B-Calculus selection equation:
   ```rust
   let mut next = KBitSet::<WORDS>::zero();
   (0..WORDS).for_each(|i| {
       let fired_word = (self.current.words[i] & !input.words[i]) | output.words[i];
       next.words[i] = (fired_word & mask) | (self.current.words[i] & !mask);
   });
   ```

This transaction structure perfectly follows `AGENTS.md` §10 (No mutation before complete admission): *Current state → Candidate state → Branchless Admission mask → Fieldwise masked commit*. The state matrix applies simultaneously within 64-bit ALUs rather than explicitly unrolling SIMD vectors, achieving deterministic execution times globally.

## Byte-Level SWAR Scans
`bcinr` extends this paradigm to byte-scanning loops (such as CSV row scanning, text classification, or finding delimiters), operating over 8 bytes at a time per `u64` register. 

Instead of the classic but flawed zero-byte SWAR test `(x - 0x01..01) & !x & 0x80..80` (which suffers from a cascade-bug with adjacent matched bytes causing cross-talk), `@von_neumann_bypass` applies the cascade-safe SWAR polynomial:
```rust
!(((x & 0x7F..7F) + 0x7F..7F | x) & 0x80..80)
```
This guarantees an exact, cross-talk-free zero-byte detection over all 8 lanes in a 64-bit register, completely avoiding dependencies on `_mm_testz_si128` or other architecture-specific vector intrinsics.
