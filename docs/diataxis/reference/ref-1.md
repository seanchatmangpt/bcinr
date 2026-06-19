# Reference: `mask` — Branchless Selection and Mask Algebra

Module: `bcinr_logic::mask` (`crates/bcinr-logic/src/mask.rs`)

Mask generation and branchless selection. A *mask* is a full-width value
that is either all-ones (`0xFFFF_FFFF`) or all-zeros (`0x0`); it is the
arithmetic encoding of a boolean. All functions are `#[inline(always)]`,
`#[must_use]`, branchless (single execution path), and `O(1)`.

## Selection

| Function | Signature | Returns |
|----------|-----------|---------|
| `select_u32` | `fn(mask: u32, a: u32, b: u32) -> u32` | `a` if `mask` all-ones, `b` if all-zeros |
| `select_u64` | `fn(mask: u64, a: u64, b: u64) -> u64` | `a` if `mask` all-ones, `b` if all-zeros |

Implementation: `(mask & a) | (!mask & b)`.

**Contract.** `mask` MUST be all-ones or all-zeros. Any other value yields a
per-bit blend of `a` and `b` (defined, but rarely intended). Produce masks
only via the generators below.

## Mask generators

| Function | Signature | All-ones result when |
|----------|-----------|----------------------|
| `eq_mask_u32` | `fn(a: u32, b: u32) -> u32` | `a == b` |
| `is_zero_mask_u32` | `fn(x: u32) -> u32` | `x == 0` |
| `nonzero_mask_u32` | `fn(x: u32) -> u32` | `x != 0` |
| `lt_mask_u32` | `fn(a: u32, b: u32) -> u32` | `a < b` (unsigned) |

Each returns `0xFFFF_FFFF` (predicate true) or `0x0000_0000` (predicate
false). `lt_mask_u32` lowers to `SETB` + `NEG` on x86-64 — no branch
instruction.

## Derived extrema and magnitude

| Function | Signature | Result | Notes |
|----------|-----------|--------|-------|
| `min_u32` | `fn(a: u32, b: u32) -> u32` | unsigned minimum | `select_u32(lt_mask_u32(a,b), a, b)` |
| `max_u32` | `fn(a: u32, b: u32) -> u32` | unsigned maximum | `select_u32(lt_mask_u32(a,b), b, a)` |
| `abs_i32` | `fn(x: i32) -> i32` | `|x|` | `(x ^ (x>>31)).wrapping_sub(x>>31)` |

**`abs_i32` boundary.** `abs_i32(i32::MIN)` overflows (no `i32` represents
`2^31`); the wrapping result is `i32::MIN`. Callers requiring a total
absolute value must widen to `i64` first.

## Integrity gate

| Function | Signature | Purpose |
|----------|-----------|---------|
| `mask_phd_gate` | `fn(val: u64) -> u64` | Identity verification anchor; returns `val`. Not an algorithm. See `phd_gates.md`. |

## Complexity

All functions: time `O(1)`, space `O(1)`, no allocation, no panic, single
branchless path.

## Cross-references

- Theory of the mask encoding: `explanation/theory-3.md`.
- Why selection is branchless: `explanation/theory-1.md`.
- Signed bit-twiddling counterparts: `reference/ref-2.md` (`int`).
