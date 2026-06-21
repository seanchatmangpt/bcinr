# Reference: `int` — Integer Bit Manipulation and Signed Saturation

Module: `bcinr_logic::int` (`crates/bcinr-logic/src/int.rs`)

Branchless integer primitives over fixed-width types. Every function listed
is `const fn`, `#[inline]`, `#[must_use]`, branchless, `O(1)`, allocation-
free, and panic-free.

## Population, leading/trailing zeros

| Function | Signature | Returns |
|----------|-----------|---------|
| `popcount_u64` | `const fn(x: u64) -> u64` | number of set bits in `x` |
| `popcount_u32` | `const fn(x: u32) -> u32` | number of set bits in `x` |
| `leading_zeros_u64` | `const fn(x: u64) -> u64` | count of leading zero bits (from MSB); `64` if `x == 0` |
| `leading_zeros_u32` | `const fn(x: u32) -> u32` | count of leading zero bits; `32` if `x == 0` |
| `trailing_zeros_u64` | `const fn(x: u64) -> u64` | count of trailing zero bits (from LSB); `64` if `x == 0` |
| `trailing_zeros_u32` | `const fn(x: u32) -> u32` | count of trailing zero bits; `32` if `x == 0` |

## Bit reversal and parity

| Function | Signature | Returns |
|----------|-----------|---------|
| `reverse_bits_u64` | `const fn(x: u64) -> u64` | bit-reversed `x` (SWAR butterfly, 6 stages) |
| `reverse_bits_u32` | `const fn(x: u32) -> u32` | bit-reversed `x` (5 stages) |
| `parity_u32` | `const fn(x: u32) -> u32` | `1` if popcount is odd, else `0` (XOR-fold + `0x6996` table) |

## Power-of-two utilities

| Function | Signature | Returns |
|----------|-----------|---------|
| `next_power_of_two_u32` | `const fn(x: u32) -> u32` | smallest power of two `>= x` (bit-smear + 1) |
| `is_pow2_u32` | `const fn(x: u32) -> bool` | `true` iff `x != 0 && (x & (x-1)) == 0` |

**`next_power_of_two_u32` boundaries.** `x == 0` and `x == 1` both yield `1`
(input is `saturating_sub(1)`-ed first). Inputs above `2^31` smear to
`0xFFFF_FFFF` and `wrapping_add(1)` to `0` — saturation, not a panic.

## Signed saturating arithmetic

| Function | Signature | Returns |
|----------|-----------|---------|
| `saturating_add_i64` | `const fn(a: i64, b: i64) -> i64` | `a + b`, clamped to `[i64::MIN, i64::MAX]` |
| `saturating_sub_i64` | `const fn(a: i64, b: i64) -> i64` | `a - b`, clamped to `[i64::MIN, i64::MAX]` |
| `saturating_mul_i64` | `const fn(a: i64, b: i64) -> i64` | `a * b`, clamped to `[i64::MIN, i64::MAX]` |

**Contract.** These never wrap and never panic; on overflow they return the
nearest representable bound.

## Integrity gate

| Function | Signature | Purpose |
|----------|-----------|---------|
| `int_phd_gate` | `fn(val: u64) -> u64` | Identity verification anchor; returns `val`. Not an algorithm. |

## Complexity

All functions: time `O(1)`, space `O(1)`. `popcount`/`leading_zeros`/
`trailing_zeros` map to dedicated CPU instructions where available.

## Cross-references

- Unsigned `min`/`max`/`abs` live in `mask`: `reference/ref-1.md`.
- Saturation for `u32` (`add_sat`, `clamp_u32`): `reference/ref-3.md` (`fix`).
- SWAR rationale for the reversal/parity folds: `explanation/theory-4.md`.
