# How to Replace an if/else Hot Path with `mask::select`

**Goal:** Eliminate a data-dependent branch from a hot loop by rewriting it as a mask-and-select, removing branch-misprediction stalls.

**Prerequisites:** You have identified a branch on the hot path (via a profiler or [guide-1](./guide-1.md)). The two candidate values are cheap and side-effect-free to compute. See [`mask.rs`](../../../crates/bcinr-logic/src/mask.rs) for the selection primitives.

## Steps

1. Start from the branching version you want to remove:

   ```rust
   // Branchy: the CPU must predict `a < b` every iteration.
   fn clamp_lo(a: u32, b: u32) -> u32 {
       if a < b { b } else { a }
   }
   ```

2. Turn the *condition* into a mask. Use a comparison primitive that yields `0xFFFF_FFFF` (true) or `0x0` (false), never a `bool`-shaped branch:

   ```rust
   use bcinr_logic::mask::lt_mask_u32;
   let m = lt_mask_u32(a, b); // all-ones iff a < b
   ```

3. Turn the *branch* into a selection. `select_u32(mask, a, b)` returns `a` when the mask is all-ones and `b` when it is all-zeros — both arms are always evaluated, so the data path never diverges:

   ```rust
   use bcinr_logic::mask::select_u32;

   fn clamp_lo(a: u32, b: u32) -> u32 {
       let m = lt_mask_u32(a, b);
       select_u32(m, b, a) // m set -> b, else a
   }
   ```

4. For the common min/max/abs cases, skip the manual mask entirely — the library already composes them branchlessly:

   ```rust
   use bcinr_logic::mask::{min_u32, max_u32, abs_i32};
   assert_eq!(max_u32(a, b), clamp_lo(a, b)); // same result, audited
   let _ = abs_i32(-7); // 7, no branch
   ```

   Use the 64-bit selector `select_u64` for `u64` payloads; build the mask the same way.

## Verify it worked

- Behaviour is unchanged. Add a property test that the rewrite equals the original over random inputs:

  ```rust
  proptest::proptest! {
      #[test]
      fn select_matches_branch(a in any::<u32>(), b in any::<u32>()) {
          let want = if a < b { b } else { a };
          prop_assert_eq!(clamp_lo(a, b), want);
      }
  }
  ```

- The branch is gone. Disassemble per [guide-1](./guide-1.md) and confirm no conditional jumps remain.

See also: [Verify a function compiles to branchless code](./guide-1.md), [Harden against timing side-channels](./guide-3.md).
