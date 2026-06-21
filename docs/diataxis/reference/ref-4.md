# Reference: `bitset` — Bit Manipulation, Rank/Select, Set Algebra

Module: `bcinr_logic::bitset` (`crates/bcinr-logic/src/bitset.rs`)

Branchless bit and bitset-slice operations. Single-word functions are
`O(1)`; slice functions are `O(n)` over the words processed, with a fully
branchless body per word. No allocation, no panic (subject to the contracts
noted).

## Single-word bit operations

| Function | Signature | Returns |
|----------|-----------|---------|
| `set_bit_u64` | `const fn(x: u64, pos: usize) -> u64` | `x` with bit `pos & 63` set |
| `clear_bit_u64` | `const fn(x: u64, pos: usize) -> u64` | `x` with bit `pos & 63` cleared |
| `rank_u64` | `fn(x: u64, pos: usize) -> usize` | count of set bits in `x` at indices `0..=pos` |
| `select_bit_u64` | `fn(x: u64, n: usize) -> Option<usize>` | index of the `n`-th set bit (0-based), or `None` |

Notes. `set_bit_u64`/`clear_bit_u64` mask the position with `& 63`, so any
`pos` is in range (no out-of-bounds shift). `rank_u64` saturates the mask for
`pos >= 63` to cover the full word. `select_bit_u64` uses a 6-step
bit-parallel binary search and returns `None` when fewer than `n+1` bits are
set. `set_bit`/`clear_bit` are `const fn`; `rank`/`select` are not.

## Slice similarity and distance

| Function | Signature | Returns |
|----------|-----------|---------|
| `parity_u64_slice` | `fn(a: &[u64]) -> u64` | `1` if total popcount over `a` is odd, else `0` |
| `jaccard_u64_slices` | `fn(a: &[u64], b: &[u64]) -> f32` | `|A∩B| / |A∪B|` over the common prefix |
| `hamming_u64_slices` | `fn(a: &[u64], b: &[u64]) -> usize` | count of differing bits over the common prefix |

Length handling. `jaccard`/`hamming` operate over `min(a.len(), b.len())`
words (computed branchlessly); trailing words of the longer slice are
ignored. `jaccard` returns `0.0` when the union is empty (divisor is
`+ (union == 0)`-guarded, so no division by zero).

## In-place set algebra

| Function | Signature | Effect |
|----------|-----------|--------|
| `intersect_u64_slices` | `fn(a: &mut [u64], b: &[u64])` | `a[i] &= b[i]` over the common prefix |
| `union_u64_slices` | `fn(a: &mut [u64], b: &[u64])` | `a[i] |= b[i]` over the common prefix |
| `any_bit_set_u64_slice` | `fn(a: &[u64]) -> bool` | `true` if any bit in `a` is set |

`intersect`/`union` write only the first `min(a.len(), b.len())` words of
`a`; remaining words of `a` are left unchanged.

## Integrity gate

| Function | Signature | Purpose |
|----------|-----------|---------|
| `bitset_phd_gate` | `fn(val: u64) -> u64` | Verification anchor; returns `val.wrapping_add(1)`. Not an algorithm. |

## Complexity

| Function class | Time | Space |
|----------------|------|-------|
| `set`/`clear`/`rank`/`select` | `O(1)` | `O(1)` |
| slice ops (`parity`, `jaccard`, `hamming`, `intersect`, `union`, `any`) | `O(n)` words | `O(1)` |

## Cross-references

- Why `rank`/`select` replace linear scans: `explanation/anti-patterns.md`
  (item 4).
- Constant-time comparison via accumulate-don't-short-circuit: see
  `hamming_u64_slices` and `explanation/theory-6.md`.
