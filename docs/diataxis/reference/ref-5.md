# Reference: `scan` and `reduce` — Scanning and Horizontal Reduction

Modules: `bcinr_logic::scan` (`crates/bcinr-logic/src/scan.rs`) and
`bcinr_logic::reduce` (`crates/bcinr-logic/src/reduce.rs`)

Branchless byte scanning and horizontal (lane-collapsing) reductions. Scan
functions are `#[inline(always)]`. All bodies are branchless and
allocation-free.

## `scan` — byte scanning

| Function | Signature | Returns |
|----------|-----------|---------|
| `find_byte_mask` | `fn(bytes: &[u8], target: u8) -> u64` | bit `i` set iff `bytes[i] == target`, for `i < min(len, 64)` |
| `skip_spaces` | `fn(bytes: &[u8]) -> usize` | length of the leading run of ASCII space (`b' '`) bytes |
| `is_ascii_u64_slice` | `fn(bytes: &[u8]) -> bool` | `true` iff every byte is `< 0x80` (ASCII) |
| `scan_gate` | `fn(val: u64) -> u64` | Verification anchor; returns `val`. Not an algorithm. |

Notes.
- `find_byte_mask` processes at most 64 bytes; bytes beyond index 63 are not
  represented in the returned mask.
- `skip_spaces` counts the prefix of `b' '` only (not tabs/newlines); it
  stops at the first non-space. See `skip_whitespace` in `parse` (`ref-9`)
  for the `<= 0x20` variant.
- `is_ascii_u64_slice` uses 64-bit SWAR over 8-byte chunks (mask
  `0x8080_8080_8080_8080`) plus a scalar remainder loop.

## `reduce` — horizontal reductions

| Function | Signature | Returns |
|----------|-----------|---------|
| `horizontal_or_u32` | `fn(slice: &[u32]) -> u32` | OR of all elements; `0` for an empty slice |
| `horizontal_and_u32` | `fn(slice: &[u32]) -> u32` | AND of all elements; `0` for an empty slice |
| `horizontal_xor_u32` | `fn(slice: &[u32]) -> u32` | XOR of all elements; `0` for an empty slice |
| `horizontal_sum_u8x8` | `fn(v: u64) -> u64` | sum of the 8 bytes packed in `v` |
| `horizontal_max_u8x8` | `fn(v: u64) -> u8` | maximum of the 8 bytes packed in `v` |
| `horizontal_min_u8x8` | `fn(v: u64) -> u8` | minimum of the 8 bytes packed in `v` |

Notes.
- `horizontal_and_u32` returns `0` (not all-ones) on an empty slice — the
  identity is masked off branchlessly via the `is_empty` flag. Verify this
  matches your intended identity element before relying on it.
- `horizontal_sum_u8x8` is a 3-stage widening tree (masks
  `0x00FF00FF...`, `0x0000FFFF...`, `0x00000000FFFFFFFF`); intermediate sums
  widen so lanes never carry into neighbours.
- `horizontal_max_u8x8`/`horizontal_min_u8x8` are 3-stage SWAR lane compares
  using broadcast `0x0101...` masks; no branches.

## Complexity

| Function class | Time | Space |
|----------------|------|-------|
| `find_byte_mask` | `O(min(n,64))` | `O(1)` |
| `skip_spaces`, `is_ascii_u64_slice` | `O(n)` | `O(1)` |
| `horizontal_*_u32` | `O(n)` | `O(1)` |
| `horizontal_*_u8x8` | `O(1)` (fixed 8-lane) | `O(1)` |

## Cross-references

- SWAR reduction theory: `explanation/theory-4.md`.
- `parse` whitespace/number scanning: `reference/ref-9.md`.
