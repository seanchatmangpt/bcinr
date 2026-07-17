# Implementation Report: Branchless Q16.16 Fixed-Point Math in BCINR

**Source File:** [`crates/bcinr-cmca/src/fixed.rs`](file:///Users/sac/bcinr/crates/bcinr-cmca/src/fixed.rs)

## 1. Introduction
The core of BCINR's deterministic mandate is strict compliance with the "$CC=1$" law (Radon Law), where logic is expressed via bitwise polynomials and zero data-dependent branching. To achieve bounded execution without runtime panic paths, bounds-check traps, or unwinding, `fixed.rs` implements custom Q16.16 fixed-point arithmetic structures: `NonNegativeFixed` and `SignedFixed`.

## 2. SWAR Canonical Masking
Instead of using `if`/`else` control flow, `fixed.rs` implements SIMD Within A Register (SWAR) masking via the `CanonicalMask` struct. A canonical mask is defined to be strictly either `0xFFFFFFFF` (true) or `0x00000000` (false).

### Mask Definition and Selection
```rust
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMask { pub val: u32 }

impl CanonicalMask {
    pub const TRUE: Self = Self { val: u32::MAX };
    pub const FALSE: Self = Self { val: 0 };

    #[inline(always)]
    pub const fn select_u32(self, a: u32, b: u32) -> u32 {
        (a & self.val) | (b & !self.val)
    }
}
```
This `select` method mimics a ternary operator `mask ? a : b` but purely through logical `AND` and `OR` operations, assuring branchless machine code generation.

### Branchless Comparison
To evaluate conditionals without generating branch instructions, comparisons like "less-than" or "equality" avoid returning traditional boolean types. Instead, they emit canonical masks derived mathematically from bitwise behaviors:
```rust
#[inline(always)]
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    CanonicalMask { val: 0u32.wrapping_sub(diff) }
}
```

## 3. Branchless Q16.16 Fixed-Point Arithmetic
`fixed.rs` provides two primary data structures for Q16.16 fixed point: `NonNegativeFixed` and `SignedFixed`. Both structures pair the raw computational bits (`val`) with a separate `err` field for branchless error state accumulation.

```rust
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SignedFixed {
    pub val: i32,
    pub err: u32,
}
```

Operations such as addition dynamically calculate saturation bounds and mathematically select them based on calculated overflow masks.
```rust
    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let (sum, overflow) = self.val.overflowing_add(other.val);
        let is_neg = const_lt_i32(self.val, 0);
        let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
        let overflow_mask = CanonicalMask { val: 0u32.wrapping_sub(overflow as u32) };
        let e = overflow_mask.select_u32(StabilityRefusal::NumericRangeExceeded as u32, u32::MAX);
        Self {
            val: overflow_mask.select_i32(sat_val, sum),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, e)),
        }
    }
```
Notice the absence of `expect()`, `unwrap()`, or `if overflow` blocks. The entire structure evaluates branchlessly and seamlessly bounds out-of-range results.

## 4. Saturating Division
Hardware division heavily risks pipeline stalls and architecture-dependent traps (like Division By Zero). `NonNegativeFixed::saturating_div` guarantees safety via a branchless Newton-Raphson approximation technique.

First, it checks for a zero denominator and proactively substitutes a valid denominator `1` to avert hardware panics, all while calculating a deterministic failure path.
```rust
        let den_is_zero = const_eq_u32(other.val, 0);
        let d = den_is_zero.select_u32(1, other.val);
```
After extracting an initial reciprocal approximation based on leading zeros, three static Newton-Raphson iteration steps correct the reciprocal to high precision:
```rust
        let e0 = (1i128 << 94) - (d_norm as i128) * (x0 as i128);
        let x1 = ((x0 as i128) + (((x0 as i128) * (e0 >> 32)) >> 62)) as u64;
```
Finally, quotient precision bounds are handled via polynomial bit selection rather than condition checks.

## 5. Transcendental Approximations (exp2 / log2)
Floating-point units are strictly forbidden due to deterministic unpredictability across different architectures. Transcendental functions thus utilize rigorously tuned, branchless polynomial approximations.

### `exp2` Implementation
`SignedFixed::exp2` separates the argument into its integer (`ip`) and fractional (`fp`) portions. The fractional part relies on an accumulation of coefficients constituting a 4th-degree polynomial mapping $[0, 1) \rightarrow [1, 2)$.
```rust
        let res1 = (y.wrapping_mul(630)) >> 16;
        let res2 = (y.wrapping_mul(3637u32.wrapping_add(res1))) >> 16;
        let res3 = (y.wrapping_mul(15763u32.wrapping_add(res2))) >> 16;
        let res4 = (y.wrapping_mul(45506u32.wrapping_add(res3))) >> 16;
        let frac_part = 65536u32.wrapping_add(res4);
```
Shifting then applies the integer radix. Negative integer bases conditionally shift bits in the opposite direction relying totally on shift variable calculation rather than `if x < 0 { ... }`.

### `log2` Implementation
`log2` performs bit-counting by tracking the index of the highest set bit (`leading_zeros`) and resolving an interpolated fractional curve via fixed correction multiples.

## 6. Typed Stability Refusals
A core hallmark of `fixed.rs` is its continuous accumulation of error metrics without triggering `panic!()` macros, exceptions, or `Result`-based early-returns (`?`), effectively nullifying path-divergent control flow (`CC=1`).

This is fulfilled via a "Sticky Error Accumulator" implementation:
```rust
#[inline(always)]
pub const fn branchless_err_acc(e1: u32, e2: u32) -> u32 {
    let e1_is_ok = const_eq_u32(e1, u32::MAX);
    e1_is_ok.select_u32(e2, e1)
}
```
If an exceptional error mask like `den_is_zero` triggers, it branchlessly selects a `StabilityRefusal` enum representation:
```rust
let e = den_is_zero.select_u32(StabilityRefusal::UnsupportedDomain as u32, overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, u32::MAX));
```
Regardless of the failure, the function will return a saturated "safe" bounded mathematical equivalent, but inherently pairs it with the typed refusal via the object's `err` attribute. Operations chained sequentially preserve the earliest failure mode, which validation components handle strictly later.

## Conclusion
`fixed.rs` provides an elegant array of constant-time mathematical primitives strictly tuned for zero-allocation, bounded, and immutable execution states. By mapping complex branches into logical polynomial masks and algebraic SWAR substitutions, the BCINR CMCA module achieves provably secure execution flow immunity.
