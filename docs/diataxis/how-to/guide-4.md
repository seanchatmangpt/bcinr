# How to Choose Between Saturating and Wrapping Arithmetic

**Goal:** Pick the right overflow behaviour for a hot path so it stays branchless and panic-free, instead of falling back to a checked `if` that re-introduces a branch.

**Prerequisites:** You are working in release or `no_std` where a debug overflow panic is unacceptable. You know the value range your accumulator can reach. Primitives live in [`fix.rs`](../../../crates/bcinr-logic/src/fix.rs) and [`int.rs`](../../../crates/bcinr-logic/src/int.rs).

## Steps

1. Decide what "overflow" *means* for your domain:
   - **Saturating** — clamp to the type's min/max. Correct for signal levels, counters that must not roll over, pixel/audio mixing.
   - **Wrapping** — modular `2^n` arithmetic. Correct for hashes, ring buffers, checksums, PRNG state.
   - Never reach for a guarding `if a > MAX - b { ... }`: that is a data-dependent branch (see [guide-1](./guide-1.md)).

2. For saturating addition of unsigned bytes/words, use the branchless saturating add — it carries the overflow into an all-ones OR-mask rather than branching:

   ```rust
   use bcinr_logic::fix::add_sat;
   assert_eq!(add_sat(200, 100), u32::MAX); // clamps, does not wrap
   ```

3. For signed saturating arithmetic, use the `i64` helpers in `int`:

   ```rust
   use bcinr_logic::int::{saturating_add_i64, saturating_sub_i64, saturating_mul_i64};
   assert_eq!(saturating_add_i64(i64::MAX, 1), i64::MAX);
   assert_eq!(saturating_sub_i64(i64::MIN, 1), i64::MIN);
   ```

4. For wrapping arithmetic, use the standard-library `wrapping_*` methods directly — they are already branchless and `const`:

   ```rust
   let h = 0x9E37_79B9u32.wrapping_mul(x).wrapping_add(seed); // hash mixing
   ```

5. To force a value into a range (rather than just handling overflow at the edges), clamp branchlessly instead of writing `min`/`max` chains by hand:

   ```rust
   use bcinr_logic::fix::clamp_u32;
   assert_eq!(clamp_u32(500, 0, 255), 255); // [min, max], no branch
   ```

## Verify it worked

- Boundary behaviour is what you expect at the extremes: test `T::MAX`, `T::MIN`, and `0`.
- No accidental panics: build and test in release (debug builds panic on plain `+` overflow but not on `wrapping_*`/`saturating_*`):

  ```bash
  cargo make test
  ```

- The path stayed branchless: confirm with the complexity gate (`cargo make contract-gate`) or by disassembling per [guide-1](./guide-1.md).

See also: [Verify a function compiles to branchless code](./guide-1.md), [Replace an if/else hot path with mask::select](./guide-2.md).
