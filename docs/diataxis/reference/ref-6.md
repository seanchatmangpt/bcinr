# Reference: `network` — Sorting Networks

Module: `bcinr_logic::network` (`crates/bcinr-logic/src/network.rs`)

Branchless, fixed-schedule sorting via compare-exchange networks. A sorting
network performs the *same* sequence of compare-exchange operations
regardless of input order, giving data-independent timing. All functions are
`#[inline(always)]` and branchless.

## Functions

| Function | Signature | Effect |
|----------|-----------|--------|
| `compare_exchange` | `fn(a: &mut [u32], i: usize, j: usize)` | orders `a[i]`, `a[j]` ascending (swaps iff `a[i] > a[j]`) |
| `bitonic_sort_8u32` | `fn(a: &mut [u32; 8])` | sorts 8 elements ascending in place |
| `bitonic_sort_16u32` | `fn(a: &mut [u32; 16])` | sorts 16 elements ascending in place |

## `compare_exchange`

The network primitive. Computes `mask = (a[i] > a[j]) as u32`, derives a
full-width swap mask, and conditionally exchanges via XOR:

```rust
let diff = (a[i] ^ a[j]) & 0u32.wrapping_sub(mask);
a[i] ^= diff;  a[j] ^= diff;
```

No branch; the swap is a masked XOR. **Contract.** `i` and `j` must be valid
indices into `a` (slice indexing will panic on out-of-bounds). Callers
normally use the fixed `bitonic_sort_*` schedules, which only generate valid
indices.

## `bitonic_sort_8u32` / `bitonic_sort_16u32`

Fixed-depth bitonic networks. The compare-exchange schedule is determined
solely by the array length, not the data, so the operation count is constant
for a given size and timing is input-independent. Both sort ascending, in
place, with no allocation.

## Integrity / oracle

This module's verification scaffold (reference + 3 counterfactual mutants)
lives in its `#[cfg(test)]` module; there is no exported `*_phd_gate`
function. See `phd_gates.md`.

## Complexity

| Function | Time | Space | Notes |
|----------|------|-------|-------|
| `compare_exchange` | `O(1)` | `O(1)` | single masked swap |
| `bitonic_sort_8u32` | `O(1)` (fixed network) | `O(1)` in place | constant compare-exchange count |
| `bitonic_sort_16u32` | `O(1)` (fixed network) | `O(1)` in place | constant compare-exchange count |

A bitonic network of size *n* has `O(n log² n)` comparators; for the fixed
sizes 8 and 16 this is a compile-time constant, hence `O(1)` per call.

## Availability note

This module exports the 8- and 16-element sorters and the `compare_exchange`
primitive. Other element counts referenced elsewhere in the docs are not
present in this module's source.

## Cross-references

- Why fixed schedules give tight WCET: `explanation/theory-7.md`.
- The masked-swap encoding: `explanation/theory-3.md`.
