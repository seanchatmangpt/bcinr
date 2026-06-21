# Reference: Contract Conventions and Glossary

This page is the lookup reference for the conventions, terms, and status
markers used throughout the bcinr API documentation. It defines *what the
words mean* so the per-module references (`ref-1` … `ref-9`) can stay terse.

## Naming conventions

| Pattern | Meaning | Example |
|---------|---------|---------|
| `*_u8` / `*_u16` / `*_u32` / `*_u64` | operand/result width | `select_u32`, `popcount_u64` |
| `*_i32` / `*_i64` | signed operand/result | `abs_i32`, `saturating_add_i64` |
| `*_mask_*` | returns a full-width mask (`0` or all-ones) | `eq_mask_u32`, `lt_mask_u32` |
| `*_sat` / `saturating_*` | clamps to type bounds instead of wrapping | `add_sat`, `saturating_mul_i64` |
| `*_u8x16` / `*_u8x8` | packed lanes (SIMD / SWAR) | `splat_u8x16`, `horizontal_sum_u8x8` |
| `*_slice` / `*_slices` | operates over `&[T]` / `&mut [T]` | `parity_u64_slice`, `union_u64_slices` |
| `*_phd_gate`, `*_gate` | verification anchor, not an algorithm | `mask_phd_gate`, `scan_gate` |

## Contract vocabulary

| Term | Definition |
|------|------------|
| **Branchless** | Single execution path; no data-dependent conditional branch in the source (see `explanation/theory-1.md`). |
| **Mask** | A value that is all-ones (`0xFF…`) or all-zeros (`0x00…`); the arithmetic encoding of a boolean. |
| **Full-width** | A mask spanning every bit of its type (e.g. `0xFFFF_FFFF` for `u32`). |
| **Saturating** | On overflow, returns the nearest representable bound rather than wrapping or panicking. |
| **Common prefix** | For two slices, the first `min(a.len(), b.len())` elements; trailing elements of the longer slice are ignored. |
| **Defined, not panic** | An edge case (e.g. zero divisor, out-of-range index) is handled by arithmetic/masking and returns a specified value instead of panicking. |
| **Verification anchor / PhD Gate** | A line documenting a completed Hoare-logic proof, not a stub. See `phd_gates.md`. |

## Complexity notation

| Symbol | Meaning |
|--------|---------|
| `O(1)` | constant time, independent of input *values* and *length* |
| `O(n)` | linear in the number of elements/words/bytes processed |
| `O(k)` | linear in a secondary length (e.g. `accept_states.len()`) |

For branchless primitives, the stated complexity is also the **worst case**:
best, average, and worst coincide because there is no data-dependent path
(see `explanation/theory-7.md`).

## Attribute conventions

| Attribute | Meaning in this codebase |
|-----------|--------------------------|
| `#[inline(always)]` / `#[inline]` | primitive is intended to inline into the caller's hot path |
| `#[must_use]` | discarding the result is almost certainly a bug |
| `const fn` | usable in `const` contexts and at compile time |

## Error and edge-case conventions

- **`Result<T, ()>`** — fallible parse-style operations return `Ok(value)` or
  the unit error `Err(())`. Example: `parse_hex_u32` (`ref-9`).
- **`Option<T>`** — partial lookups return `Some(value)` or `None`. Example:
  `select_bit_u64` (`ref-4`).
- **No panics in normal flow** — primitives are designed to avoid panics; the
  remaining panic sources are explicit *caller contracts* (out-of-bounds
  slice indices, a zero modulus/width). These are noted per function in the
  module references. There are no panics arising from arithmetic overflow —
  saturation or wrapping arithmetic is used instead.

## Safety and `unsafe`

Every algorithm module carries `#![forbid(unsafe_code)]`. The only files
permitted to contain `unsafe` are `mem.rs`,
`autonomic/packed_key_table.rs`, and `patterns/deterministic_mpmc.rs`, each
with a recorded proof in `SAFETY.md`. None of the primitive modules
documented in `ref-1` … `ref-9` contain `unsafe`.

## Determinism and portability

| Property | Statement |
|----------|-----------|
| **Determinism** | Same inputs → same outputs and same instruction sequence, every run. |
| **Portability** | Core is `no_std`, allocator-free; identical source compiles for x86-64, ARM, and WebAssembly (`explanation/theory-9.md`). |
| **Bit-identical SIMD** | The `simd` model and any accelerated path must produce results identical to the portable computation. |

## Module index

| Module | Reference | Purpose |
|--------|-----------|---------|
| `mask` | `ref-1` | branchless selection and mask algebra |
| `int` | `ref-2` | integer bit manipulation, signed saturation |
| `fix` | `ref-3` | saturating/clamped `u32` arithmetic |
| `bitset` | `ref-4` | rank/select, set algebra over `[u64]` |
| `scan`, `reduce` | `ref-5` | byte scanning, horizontal reduction |
| `network` | `ref-6` | sorting networks |
| `dfa` | `ref-7` | table-driven finite automata |
| `sketch` | `ref-8` | Count-Min Sketch update |
| `utf8`, `parse`, `simd` | `ref-9` | text classification, parsing, 128-bit vectors |
| conventions | `ref-10` | this page |
