# bcinr-logic API reference

`bcinr-logic` is the branchless/SIMD kernel crate: bitset algebra, fixed-point
arithmetic, constant-time select, DFA stepping, SWAR string primitives, and a
generated algorithm catalog. Every function is total (no panic on any input)
and branch-free on its hot path.

## Bitset algebra (`bitset`)

Single-word bitset operations over `u64`.

- `set_bit_u64(x, pos)` sets bit `pos`; `clear_bit_u64(x, pos)` clears it.
- `rank_u64(x, pos)` counts set bits below `pos`; `select_bit_u64(x, n)`
  returns the position of the `n`-th set bit.
- `parity_u64_slice(a)` folds XOR over a slice of words.
- `jaccard_u64_slices(a, b)`, `hamming_u64_slices(a, b)`,
  `intersect_u64_slices(a, b)`, `union_u64_slices(a, b)`,
  `any_bit_set_u64_slice(a)`: slice-level set algebra.

Rank/select round-trip holds: `rank_u64(x, select_bit_u64(x, n)) == n + 1`.

## Constant-time primitives (`ct`)

Mask-based selection without branches:

- `ct_select_u8/u32/u64(condition, a, b)` — branchless multiplexers.
- `ct_select_i64(condition, a, b)` — signed variant.
- `ct_eq_u8(a, b)` produces an all-ones byte mask on equality.

## DFA stepping (`dfa`)

Table-driven automata:

- `dfa_advance(state, input, table, alphabet_size)` — one transition.
- `dfa_run(table, alphabet_size, initial_state, input)` — fold a byte slice.
- `dfa_is_accepting(state, accept_states)` — membership test.

Tables are flat `&[usize]` with row-major layout
(`state * alphabet_size + input`).

## Fixed-point Q16.16 (`fix`)

Signed fixed-point arithmetic with saturation:

- Conversions: `f32_to_q16(x)`, `q16_to_f32(x)`.
- Arithmetic: `q16_mul(a, b)`, `q16_div(a, b)`, `q16_recip(x)`.
- Roots and logs: `isqrt_u32(n)`, `q16_sqrt(x)`, `ilog2_u32(x)`, `q16_log2(x)`.
- Trigonometry: `q16_sin_bhaskara(theta)`, `q16_sin_approx(theta)`,
  `q16_cos_approx(theta)` (Bhaskara I approximation, constants `PI`,
  `PI_OVER_2`, `TWO_PI`).
- Saturation helpers: `add_sat(a, b)`, `clamp_u32(val, min, max)`.

## Integer primitives (`int`)

- Bit counting: `popcount_u64`, `popcount_u32`, `leading_zeros_u64/u32`,
  `trailing_zeros_u64/u32`, `reverse_bits_u64/u32`, `parity_u32`.
- Saturation: `saturating_add_i64`, `saturating_sub_i64`,
  `saturating_mul_i64`.
- Rounding division: `div_floor_i64(a, b)`, `div_ceil_i64(a, b)`.
- Number theory: `gcd_u64`, `lcm_u64`, `is_pow2_u32`, `next_power_of_two_u32`,
  `next_pow2_u64`.
- Misc: `abs_diff_u32`, `clamp_u64`, `decimal_digits_u64`.

## Mask generation and select (`mask`)

- `select_u32(mask, a, b)`, `select_u64(mask, a, b)` — per-bit multiplexer.
- Masks from comparisons: `eq_mask_u32(a, b)`, `is_zero_mask_u32(x)`,
  `nonzero_mask_u32(x)`.
- Min/max and arithmetic masks: `min_u32`, `max_u32`, `abs_i32`,
  `saturating_add`, `saturating_sub`, `saturating_mul`.

## Sorting networks (`network`)

Fixed-size branchless sorts:

- `compare_exchange(a, i, j)` — conditional swap primitive.
- `bitonic_sort_8u32(a)` / `bitonic_sort_16u32(a)` — in-place bitonic networks
  for `[u32; 8]` and `[u32; 16]`.

## Parsing primitives (`parse`)

Byte-slice parsing without allocations:

- `skip_whitespace(bytes)` returns the count of leading whitespace bytes.
- `parse_hex_u32(bytes)` / `parse_decimal_u64(bytes)` return `Result` with
  `Err` on empty, non-digit, or overflow input.

## Horizontal reduction (`reduce`)

- `horizontal_or_u32(slice)`, `horizontal_and_u32(slice)`,
  `horizontal_xor_u32(slice)` — fold a slice to one word.
- `horizontal_sum_u8x8(v)`, `horizontal_max_u8x8(v)`,
  `horizontal_min_u8x8(v)` — lane-wise folds within a `u64` of 8 lanes.
  Note: the max/min variants are currently OPEN-defective (see
  DOC_COVERAGE_LOG.md, triple 6).

## Scans and search (`scan`)

- `find_byte_mask(bytes, target)` — per-64-bit-block find mask.
- `prefix_sum_u32x16(arr)`, `exclusive_scan_u32x16(arr)` — 16-lane scans.
- `is_ascii_u64_slice(bytes)` — bulk ASCII validation.
- `skip_spaces(bytes)` — whitespace skip via word-at-a-time scanning.

## SWAR string primitives (`swar`, `swar_str`)

Word-at-a-time string ops on `u64` words:

- `find_byte_in_word(word, byte)`, `count_byte_in_word(word, byte)`,
  `has_byte_in_word(word, byte)`, `first_byte_position(word, byte)`.
- Case folding: `to_lower_ascii_word(word)`, `to_upper_ascii_word(word)`.

## SIMD abstraction (`simd`, `simd_dispatch`)

Portable 16-lane u8 kernels (`splat_u8x16`, `movemask_u8x16`,
`compare_eq_u8x16`, `add_saturating_u8x16`, `and_u8x16`, `or_u8x16`),
dispatched at runtime between a scalar fallback and a SIMD path
(`simd_dispatch`).

## Sketches (`sketch`)

Count-min sketch over a flat table:

- `count_min_sketch_update(table, hash, depth, width)`.
- `count_min_sketch_query(table, hash, depth, width) -> u32`.

## UTF-8 classification (`utf8`)

`is_continuation_byte`, `is_ascii_byte`, `is_2byte_lead`, `is_3byte_lead`,
`is_4byte_lead`, plus multi-byte validation entry points — all byte-range
predicates suitable for SWAR composition.

## Memory arenas (`mem`, `abstractions`)

- Bump/slab allocation: `LockFreeSlab::new(capacity)`, `alloc(size)`,
  `reset()`.
- Epoch reclamation: `advance_epoch`, `EpochState`
  (`abstractions/epoch_reclamation.rs`).
- `bump_arena_gate`, `epoch_reclamation_gate`: PHD-style integrity gates that
  fold a crate-local constant over their argument; each module also carries
  its own `*_gate` function (`mask_phd_gate`, `scan_gate`, `ct_phd_gate`,
  ...) used by the stability certificate harness to detect stale builds.

## Generated algorithm catalog (`algorithms`)

~308 algorithm functions generated from the ontology tables
(`ALGORITHM_CATALOG.md`). Each carries a `/// # Branchless Contract` doc
comment; the catalog cross-section is exercised by examples (see
DOC_COVERAGE_LOG.md triple 9 and `examples/sorting_networks.rs` et al.).
