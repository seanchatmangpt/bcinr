# How to Harden a Secret-Dependent Comparison Against Timing Side-Channels

**Goal:** Make an operation that touches secret data (a key, a token, a tag) run in time and with a memory-access pattern that is independent of the secret, defeating timing side-channels.

**Prerequisites:** You know which values are secret. You understand that early-exit comparisons (`==` on slices, the `?` operator on the first mismatch, table lookups indexed by a secret) leak timing. Primitives are in [`mask.rs`](../../../crates/bcinr-logic/src/mask.rs).

## Steps

1. Identify the leak. The classic one is short-circuit equality, which returns as soon as the first differing byte is found:

   ```rust
   // LEAKS: returns early on first mismatch -> time depends on how many
   // leading bytes matched.
   fn insecure_eq(a: &[u8], b: &[u8]) -> bool {
       a == b
   }
   ```

2. Replace the comparison with a constant-time accumulator. Fold every element into a single difference register so the loop always runs to completion, then collapse the register to a mask:

   ```rust
   use bcinr_logic::mask::is_zero_mask_u32;

   /// Constant-time equality. Returns `0xFFFF_FFFF` iff equal, else `0x0`.
   pub fn ct_eq(a: &[u8; 32], b: &[u8; 32]) -> u32 {
       let mut diff: u32 = 0;
       for i in 0..32 {
           diff |= u32::from(a[i] ^ b[i]); // never short-circuits
       }
       is_zero_mask_u32(diff) // all-ones iff diff == 0
   }
   ```

3. Consume the result as a mask, not a `bool`. Branching on the outcome would re-introduce a timing signal, so select with it instead (see [guide-2](./guide-2.md)):

   ```rust
   use bcinr_logic::mask::select_u32;
   let authed = ct_eq(&tag, &expected);     // 0xFFFF_FFFF or 0x0
   let status = select_u32(authed, OK, DENY); // no branch on the secret
   ```

4. Avoid secret-indexed memory. Never use a secret as a table index or slice offset — that leaks through the cache. Compute over the whole input with masks instead, exactly as in step 2.

## Verify it worked

- Equality semantics are correct: `assert_eq!(ct_eq(&x, &x), 0xFFFF_FFFF)` and `assert_eq!(ct_eq(&x, &y), 0)` for `x != y`.
- The path is data-independent: disassemble `ct_eq` per [guide-1](./guide-1.md) and confirm the loop has a fixed trip count and no conditional jumps inside it.
- Empirically, timing variance shrinks toward zero. Benchmark equal vs. near-equal inputs and compare the Criterion distributions (see [guide-6](./guide-6.md)); the two should overlap.

See also: [Replace an if/else hot path with mask::select](./guide-2.md), [Guarantee WCET](./guarantee-wcet.md), [Anti-Patterns](../explanation/anti-patterns.md).
