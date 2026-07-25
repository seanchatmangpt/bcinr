# Bitset Calculus and Branchless Masking in BCINR

The BCINR deterministic substrate implements a strictly branchless, allocation-free execution model (the Radon Law, $CC=1$). At the core of its logic and autonomic control plane lie custom bitset primitives designed for constant-time, data-independent evaluation: `KBitSet` and `SwarMarking`.

These structures implement "bitset calculus" utilizing SWAR (SIMD Within A Register) techniques and masked selection to avoid control-flow branches (`if`/`else`) entirely.

## 1. Core Primitives

### `KBitSet<const WORDS: usize>`
Located in `crates/bcinr-logic/src/models/petri.rs`, `KBitSet` is a fixed-size, word-aligned bitset stored as an array of `u64` words. It represents tokens in places or required transition masks without any heap allocation (`#![no_std]` compliant).

**Key Branchless Operations:**
- **`set(bit)`**: Avoids panicking on out-of-bounds indices by using branchless bounds checking. It derives an `in_bounds` boolean, converts it to a full-width mask, and applies it to the mutation.
- **`contains(bit)`**: Extracts the target bit and masks it with the `in_bounds` check, safely evaluating to `false` for invalid indices without branching.
- **`satisfies(required)`**: A core Petri-net enabledness check. It validates that every bit set in the `required` mask is also set in the current bitset. It achieves this by iterating over all words and bitwise accumulating mismatches (`mismatch |= required & !current`). It returns true if `mismatch == 0`.

### `SwarMarking<const WORDS: usize>`
Also located in `crates/bcinr-logic/src/models/petri.rs`, `SwarMarking` wraps a `KBitSet` to track the current token distribution of a Petri net and orchestrate atomic, deterministic transition firing.

**Branchless Firing (`try_fire`):**
Instead of the typical pattern `if enabled { fire() } else { keep_current() }`, it computes the candidate next state and uses a mask-based selection to commit or discard the change.

```rust
pub fn try_fire(&self, input: KBitSet<WORDS>, output: KBitSet<WORDS>) -> (Self, bool) {
    let is_enabled = self.current.satisfies(input);
    
    // Predicate to full-width mask (Bitset Calculus)
    let mask = 0u64.wrapping_sub(is_enabled as u64); 
    
    let mut next = KBitSet::<WORDS>::zero();
    (0..WORDS).for_each(|i| {
        // Calculate candidate fired state
        let fired_word = (self.current.words[i] & !input.words[i]) | output.words[i];
        
        // SWAR selection: (candidate & mask) | (current & !mask)
        next.words[i] = (fired_word & mask) | (self.current.words[i] & !mask);
    });
    
    (Self { current: next }, is_enabled)
}
```

## 2. Bitset Calculus and Branchless Masking

The codebase heavily utilizes mathematical reductions of boolean predicates into full-width bit masks to satisfy the constitutional law `CC=1`.

### Predicate to Mask Expansion
To convert a boolean predicate (e.g., `is_enabled`) into a selection mask:
```rust
let mask = 0u64.wrapping_sub(predicate as u64);
```
- If `predicate` is `true` (1), `0 - 1` underflows to `0xFFFFFFFFFFFFFFFF` (all 1s).
- If `predicate` is `false` (0), `0 - 0` evaluates to `0x0000000000000000` (all 0s).

### SWAR Masked Selection
Once a full-width mask is derived, execution paths are selected bitwise rather than branching:
```rust
selected = (candidate & mask) | (current & !mask);
```
This satisfies the law $\operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b)$. The authoritative transaction shape computes the candidate state structurally, verifies all predicates to generate an admission mask, and performs a fieldwise masked commit.

## 3. High-Level Orchestration: `PriorityPetriEngine`

The `PriorityPetriEngine` (in `crates/bcinr-logic/src/patterns/swar_petri.rs`) aggregates the above primitives to evaluate up to 64 transitions sequentially and deterministically within a fixed execution time (WCET).

It iterates over all transitions using a statically unrolled `for_each` loop. In each step, it unconditionally invokes `try_fire`, unconditionally overwrites its state with the branchlessly selected outcome, and accumulates a `firing_mask` (`firing_mask |= (was_fired as u64) << bit_idx`) to track which transitions executed.

This guarantees that the instruction cache footprint, execution cycles, and memory access patterns are perfectly independent of the actual system state, fulfilling the project's civilizational-scale deterministic mandate.
