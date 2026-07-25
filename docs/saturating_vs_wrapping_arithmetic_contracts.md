# Saturating vs. Wrapping Arithmetic: Mathematical Contracts in BCINR

According to **Rule 14 (Numeric-law requirements)** in the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the authoritative runtime must be deterministic and explicitly use either saturating or wrapping arithmetic according to a declared mathematical contract. Standard `+` or `-` operators (which can panic on overflow in debug or wrap silently in release) are prohibited in the hot path. Every arithmetic operation must declare its overflow boundary behavior explicitly.

The choice between saturating and wrapping arithmetic is not an implementation detail; it defines the underlying algebraic structure of the domain.

## The Mathematical Contract Differences

### Wrapping Arithmetic (Modular Arithmetic)
- **Algebraic Structure:** Models the mathematical ring of integers modulo $2^N$ (i.e., $\mathbb{Z}/2^N\mathbb{Z}$).
- **Contract:** Operations logically wrap around at the type boundaries ($0$ and $2^N - 1$). The absolute magnitude of the value is meaningless on its own; what matters are the *relative distances* (deltas) and the *algebraic properties* (such as invertibility) preserved by the modulo operations.
- **Invariant:** A domain violation never occurs strictly due to overflow, as the state seamlessly loops.

### Saturating Arithmetic (Bounded Arithmetic)
- **Algebraic Structure:** Models a clamped sub-interval of integers, typically $[MIN, MAX]$. 
- **Contract:** If an operation exceeds the maximum limit or falls below the minimum limit, the result is clamped to the boundary value. The system permanently loses the "spillover" magnitude. 
- **Invariant:** The semantic value of the number matters physically. An overflow or underflow would represent a catastrophic domain violation (e.g., negative mass, impossible probabilities, or infinite resources).

---

## When to use Wrapping Arithmetic (Ring Modulo $2^N$)

Wrapping arithmetic is mathematically required when you are dealing with structural indices, cryptography, or continuous monotonic sequences where only the delta matters.

**Examples from BCINR:**
1. **Epoch Rotation & Sequence Counters:** 
   In `patterns/deterministic_mpmc.rs`, the lock-free ring buffer sequence uses wrapping arithmetic:
   ```rust
   slot.sequence.store(h.wrapping_add(1), Ordering::Release);
   ```
   When the sequence counter reaches `u32::MAX`, it must wrap to `0` to continue tracking ring buffer positions correctly. The relative distance between `head` and `tail` calculated via two's complement subtraction remains correct regardless of the wrap.
   
2. **Mask Generation & Bitwise Layouts:**
   Calculations for memory alignment, index masking, or flag clearing use wrapping math because they manipulate bits within fixed-width boundaries, treating integers as bit vectors rather than physical quantities.

3. **Hashing & PRNG (e.g., `wyhash_64`):**
   Avalanche effects in hash algorithms rely on wrapping multiplication and addition to mix bits uniformly across the 64-bit width. Saturating here would destroy the entropy distribution.

---

## When to use Saturating Arithmetic (Clamped Bounds)

Saturating arithmetic is mathematically required when you are tracking physical quantities, statistical weights, or bounds where "wrapping" to zero would break causality or logic.

**Examples from BCINR:**
1. **Sketch Algorithms (e.g., Count-Min Sketch / Heavy Hitters):**
   In algorithms like `count_min_sketch_add.rs` and `heavy_hitter_update.rs`, counters track frequency. If a highly frequent item reaches the integer limit (e.g., `u32::MAX`), wrapping to `0` would reset the count, completely destroying the statistical model. It must use saturating arithmetic to indicate "at least this many occurrences".
   ```rust
   // From add_sat_i32 implementations
   (val as i32).saturating_add(aux as i32)
   ```

2. **Bounding Probabilities & Weights:**
   When calculating metrics like Kullback-Leibler (KL) accumulation, probabilities, or autonomic policy weights, values must remain within the domain bounds (e.g., $[0.0, 1.0]$, represented in fixed-point). Wrapping would cause an extremely high probability to suddenly become zero.

3. **Resource Pricing & Economy:**
   When adding prices or allocating resources, a price that is too high must clamp to the maximum possible value. Wrapping to a small value would allow adversaries to overflow costs and purchase resources practically for free.
