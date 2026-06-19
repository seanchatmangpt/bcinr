# Reference: `fix` — Saturating and Clamped Fixed-Point Arithmetic

Module: `bcinr_logic::fix` (`crates/bcinr-logic/src/fix.rs`)

Branchless `u32` arithmetic that stays within bounds instead of wrapping.
All functions are `#[inline(always)]`, branchless, `O(1)`, allocation-free,
and panic-free.

## Functions

| Function | Signature | Returns |
|----------|-----------|---------|
| `add_sat` | `fn(a: u32, b: u32) -> u32` | `a + b`, saturated to `u32::MAX` on overflow |
| `clamp_u32` | `fn(val: u32, min: u32, max: u32) -> u32` | `val` clamped into `[min, max]` |
| `bucketize_u32` | `fn(val: u32, step: u32) -> u32` | `val` floored to the nearest lower multiple of `step` |

## `add_sat`

Computes `res = a.wrapping_add(b)`, detects overflow with `res < a`, and ORs
in an all-ones mask on overflow so the result becomes `u32::MAX`. Never
wraps below the true sum; never panics.

```
  add_sat(u32::MAX, 1)  ==  u32::MAX
  add_sat(10, 20)       ==  30
```

## `clamp_u32`

Two masked replacements (no branches): first replace with `min` where
`val < min`, then replace with `max` where the intermediate `> max`.

**Contract.** Caller SHOULD pass `min <= max`. If `min > max`, the lower
clamp applies first and the upper clamp second, so the result equals `max`
(the upper bound wins); this is defined but usually not intended.

## `bucketize_u32`

Returns `(val / step') * step` where `step' = step + (step == 0)`. The
`+ (step == 0)` term replaces a zero divisor with `1` branchlessly, so the
function never divides by zero.

**Contract.** `step == 0` is handled (treated as `1`, yielding `val`), not a
panic. For `step >= 1`, result is the largest multiple of `step` that is
`<= val`.

```
  bucketize_u32(17, 5)  ==  15
  bucketize_u32(17, 0)  ==  17     // zero step folded to 1
```

## Complexity

All functions: time `O(1)`, space `O(1)`. `bucketize_u32` includes an
integer division.

## Cross-references

- Signed saturation (`saturating_add_i64`, …): `reference/ref-2.md` (`int`).
- The masked-replacement idiom used by `clamp_u32`: `explanation/theory-3.md`.
- Why saturation matters for actuation/limits: `explanation/anti-patterns.md`
  (item 2).
