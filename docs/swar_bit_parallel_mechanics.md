# Bit-Parallel Mechanics and SWAR in `bcinr`

## Overview
In `bcinr`, `@von_neumann_bypass` employs SWAR (SIMD Within A Register) to execute state matrix transitions and enforce policy constraints without conditional branches. By evaluating logic strictly via arithmetic across standard 64-bit integer registers, the system avoids unpredictable pipeline stalls (adhering to the `CC=1` Radon Law), remains deterministic, and is independent of architecture-specific SIMD extensions like AVX/NEON.

## How `@von_neumann_bypass` Avoids Byte-Sequential Control Flow
Instead of byte-sequential control flow (using `if`/`else` branches, which violates the `bcinr` mandate), `@von_neumann_bypass` relies on **Mask Calculus**.

Boolean conditionals are eliminated in favor of bitwise polynomial arithmetic using standard ALUs. The core B-Calculus selection identity is:
```rust
M(mask, a, b) = (mask & a) | (!mask & b)
```
Where `mask` is strictly all-ones (`0xFFFFFFFFFFFFFFFF` representing `true`) or all-zeros (`0x0000000000000000` representing `false`). 

Functions calculate these branchless masks via bitwise identities, producing an evaluation guaranteed to execute in deterministic `O(1)` time. 

For byte-level operations (like CSV row scanning, text classification, or finding delimiters), SWAR operates over 8 bytes at a time per `u64` register. Instead of classic but flawed SWAR zero-tests that suffer from cross-talk, `@von_neumann_bypass` applies a cascade-safe SWAR polynomial:
```rust
!(((x & 0x7F..7F) + 0x7F..7F | x) & 0x80..80)
```
This guarantees an exact, cross-talk-free zero-byte detection over all 8 lanes in a 64-bit register, completely avoiding dependencies on `_mm_testz_si128` or other vector intrinsics.

## Concurrent Semantic Evaluations in a Single Register
`bcinr` uses SWAR to evaluate state matrices concurrently across 64-bit bounds, exemplifying "Bitset Calculus". This is prominent in the Petri-net marking wrapper (`SwarMarking`). 

1. **Evaluation**:
   The current token state evaluates the condition iteratively over packed words using standard bitwise operators without short-circuiting:
   ```rust
   let mut mismatch = 0u64;
   (0..WORDS).for_each(|i| {
       mismatch |= required.words[i] & !self.words[i];
   });
   let is_enabled = mismatch == 0;
   ```

2. **Admission Mask Generation**:
   The boolean predicate (`is_enabled`) is deterministically converted into an all-ones or all-zeros admission mask without a branch:
   ```rust
   let mask = 0u64.wrapping_sub(is_enabled as u64);
   ```

3. **Concurrent Mask Application**:
   Finally, the transition executes over all lanes simultaneously. The new candidate state is combined with the original state using the B-Calculus selection equation:
   ```rust
   let mut next = KBitSet::<WORDS>::zero();
   (0..WORDS).for_each(|i| {
       // Calculate candidate fired state
       let fired_word = (self.current.words[i] & !input.words[i]) | output.words[i];
       
       // SWAR selection: (candidate & mask) | (current & !mask)
       next.words[i] = (fired_word & mask) | (self.current.words[i] & !mask);
   });
   ```

This transaction structure perfectly follows `AGENTS.md` §10 (No mutation before complete admission): *Current state → Candidate state → Branchless Admission mask → Fieldwise masked commit*. The state matrix evaluates and applies simultaneously within 64-bit ALUs rather than explicitly unrolling SIMD vectors, achieving deterministic execution globally.
