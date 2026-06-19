# How Masks Encode Conditionals

A *conditional* is a choice between values that depends on a predicate. A
*mask* is the arithmetic representation of that predicate. This document
explains the encoding that the whole library is built on — the move from
"`if P then a else b`" to "compute a number that *is* `P`, then blend." If
you understand this one idea, the implementations in `mask.rs`, `fix.rs`,
`bitset.rs`, and `dfa.rs` all read the same way.

## A boolean is a one-bit integer

The pivot is mundane: in Rust, `(a < b)` is a `bool`, and `bool as u32` is
`0` or `1`. That `0`/`1` is a *one-bit* mask. To turn it into a *full-width*
mask — all-zeros or all-ones — negate it in two's complement:

```rust
let m = 0u32.wrapping_sub((a < b) as u32);   // 0x0000_0000 or 0xFFFF_FFFF
```

`wrapping_sub` of `1` from `0` wraps to `0xFFFF_FFFF`; of `0` it stays `0`.
This is exactly `lt_mask_u32` in `mask.rs`. The comment there is worth
internalizing: on x86-64 the compiler emits a `SETB` + `NEG`, *not* a
branch — the predicate becomes data, not control flow.

## Blending with a full-width mask

Once you hold an all-ones / all-zeros mask, selection is pure bitwise
algebra. This is `select_u32`:

```rust
pub fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}
```

Read it as a truth table. If `mask` is all-ones: `(a) | (0) = a`. If `mask`
is all-zeros: `(0) | (b) = b`. There is no third case, because the mask is
*structurally* all-ones or all-zeros — never anything in between. That
invariant is what makes the algebra total, and it is the kind of property
the B-Calculus (`theory-5.md`) insists every primitive preserve.

## Generating the masks you need

`mask.rs` provides a small algebra of mask *generators*, each of which turns
a comparison into a full-width mask without branching:

| Generator | Mask is all-ones when | Technique |
|-----------|-----------------------|-----------|
| `eq_mask_u32(a,b)` | `a == b` | `x = a^b`; collapse `x!=0` to MSB; `sub 1` |
| `is_zero_mask_u32(x)` | `x == 0` | same collapse on `x` |
| `nonzero_mask_u32(x)` | `x != 0` | negate the zero-mask |
| `lt_mask_u32(a,b)` | `a < b` | `0 - (a<b) as u32` |

The `eq`/`is_zero` trick deserves a sentence. `x | x.wrapping_neg()` has its
top bit set for every non-zero `x` (a value and its negation cannot both
have a clear sign bit unless `x == 0`). Shifting that bit down to position 0
gives `1` for non-zero and `0` for zero; `wrapping_sub(1)` then flips it to
the full-width mask that means "equal."

## Composition: masks plus arithmetic

Selection is the simplest use. The deeper pattern is that a mask can gate
*any* operation, because `x & 0 == 0` and `x & !0 == x`. `clamp_u32` in
`fix.rs` clamps in two masked steps — replace with `min` only where
`val < min`, then replace with `max` only where the result `> max` — and
never branches. `dfa_advance` in `dfa.rs` uses a mask to fold an
out-of-range table index to a safe `0`, turning a bounds *check* into a
bounds *arithmetic*. Same encoding, different payload.

```
  predicate  --(as uN)-->  {0,1}  --(0 - x)-->  {0x00.., 0xFF..}  --(& / |)-->  blended result
```

## Why this is the canonical form

Encoding conditionals as masks is not a trick for speed alone; it is what
makes the formal story tractable. A masked expression has *one* execution
path, so its latency is input-independent (`theory-1.md`), its timing leaks
nothing (`theory-6.md`), and its postcondition can be stated as a single
algebraic identity rather than a case split — which is precisely what the
Hoare-logic gates in the source files assert and the proptest oracles check.
