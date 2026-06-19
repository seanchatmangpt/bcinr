# How to Sort a Small Fixed-Size Array Branchlessly

**Goal:** Sort a tiny array (8 or 16 elements) in constant time with a fixed sequence of compare-exchanges, so latency does not depend on the input order — ideal for inner loops, medians, and top-k.

**Prerequisites:** The element count is known at compile time and small. For data-dependent sizes a sorting network is the wrong tool; this recipe targets fixed widths. Primitives are in [`network.rs`](../../../crates/bcinr-logic/src/network.rs). A sorting network does the same comparisons every time regardless of values, which is what makes it branchless.

## Steps

1. Use the ready-made bitonic networks for the supported widths. They sort in place and emit no data-dependent branches:

   ```rust
   use bcinr_logic::network::{bitonic_sort_8u32, bitonic_sort_16u32};

   let mut a: [u32; 8] = [5, 1, 4, 2, 8, 0, 7, 3];
   bitonic_sort_8u32(&mut a);
   assert_eq!(a, [0, 1, 2, 3, 4, 5, 7, 8]);
   ```

2. The building block is `compare_exchange`, which swaps two slots into ascending order using a mask — no `if`. Use it directly to build a network for a width the library does not ship (e.g. a 3-element sort):

   ```rust
   use bcinr_logic::network::compare_exchange;

   fn sort3(a: &mut [u32; 3]) {
       compare_exchange(a, 0, 1);
       compare_exchange(a, 1, 2);
       compare_exchange(a, 0, 1);
   }
   ```

   Each `compare_exchange(a, i, j)` guarantees `a[i] <= a[j]` afterward, branchlessly.

3. Derive a branchless **median** from a sort without extracting the middle conditionally — after sorting, the median is simply the center index:

   ```rust
   let mut window: [u32; 8] = read_window();
   bitonic_sort_8u32(&mut window);
   let lower_median = window[3]; // constant-time, position is fixed
   ```

   For a true 9-element median there is a purpose-built kernel, `algorithms::median9_u32`, which avoids a full sort.

4. Keep widths at the supported sizes. The shipped networks cover 8 and 16 `u32` lanes; for other sizes compose `compare_exchange` into a known-correct network rather than looping with a data-dependent bound.

## Verify it worked

- The output is sorted and is a permutation of the input. Validate against a reference over random inputs (see [guide-9](./guide-9.md)):

  ```rust
  proptest::proptest! {
      #[test]
      fn bitonic8_sorts(mut a in any::<[u32; 8]>()) {
          let mut want = a;
          want.sort_unstable();
          bitonic_sort_8u32(&mut a);
          prop_assert_eq!(a, want);
      }
  }
  ```

- The path is constant-time: disassemble per [guide-1](./guide-1.md) and confirm the comparisons compile to `cmov`/mask arithmetic with no conditional jumps.

See also: [Verify a function compiles to branchless code](./guide-1.md), [Replace an if/else hot path with mask::select](./guide-2.md), [Add a Criterion benchmark and read the report](./guide-6.md).
