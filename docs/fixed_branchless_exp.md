# Branchless Fixed-Point Exponential Function Analysis

I inspected `crates/bcinr-cmca/src/fixed.rs` and found the implementation for calculating $e^x$ without branching. It is part of the `SignedFixed` struct (Q16.16 format) and heavily utilizes the `CanonicalMask` construct for branchless condition selection.

Here is a step-by-step breakdown of how it works:

## 1. Base Conversion (`exp`)
The function first converts $e^x$ to $2^{x \cdot \log_2(e)}$ so that it can rely on an optimized base-2 exponential function.
```rust
pub fn exp(self) -> NonNegativeFixed {
    let x = self.val;
    // 94548 is ~1.442695 * 65536, which is log2(e) in Q16.16
    let z = (((x as i64).wrapping_mul(94548)) >> 16) as i32;
    SignedFixed { val: z, faults: self.faults }.exp2()
}
```

## 2. Integer and Fractional Split (`exp2`)
It separates the Q16.16 input into its integer (`ip`) and fractional (`fp`) parts to compute $2^{\text{integer}} \cdot 2^{\text{fraction}}$:
```rust
let ip = x >> 16;
let fp = x.wrapping_sub(ip.wrapping_shl(16));
```

## 3. Polynomial Approximation
The fractional part $y \in [0, 1)$ is processed using Horner's method for a 4th-degree minimax polynomial, avoiding floating-point math entirely:
```rust
let y = fp as u32;
let res1 = (y.wrapping_mul(630)) >> 16;
let res2 = (y.wrapping_mul(3637u32.wrapping_add(res1))) >> 16;
let res3 = (y.wrapping_mul(15763u32.wrapping_add(res2))) >> 16;
let res4 = (y.wrapping_mul(45506u32.wrapping_add(res3))) >> 16;
let frac_part = 65536u32.wrapping_add(res4); // Adds 1.0 (Q16.16)
```

## 4. Branchless Shifts
To multiply by $2^{\text{integer}}$, it computes both a left shift and a right shift. A `CanonicalMask` then selects the correct result based on the sign of `ip`, avoiding any `if` statements:
```rust
let shl = (ip & 31) as u32;
let shr = ((ip.wrapping_neg()) & 31) as u32;
let val_shl = frac_part.wrapping_shl(shl);
let val_shr = frac_part.wrapping_shr(shr);

// ip_neg is all 1s if ip < 0, else 0
let ip_neg = CanonicalMask(0u32.wrapping_sub(((ip >> 31) & 1) as u32));
let val_shifted = ip_neg.select_u32(val_shr, val_shl);
```

## 5. Branchless Bounds and Saturation
Finally, it performs bitwise arithmetic to detect overflows (`ip >= 16`) and underflows (`ip <= -17`). The boolean results are expanded into bitmasks (`is_overflow`, `is_underflow`) which select the final saturated value and record faults:
```rust
let is_overflow = CanonicalMask(0u32.wrapping_sub(((((ip.wrapping_sub(16)) >> 31) ^ 1) & 1) as u32));
let is_underflow = CanonicalMask(0u32.wrapping_sub((((((-17i32).wrapping_sub(ip)) >> 31) ^ 1) & 1) as u32));

let res = is_overflow.select_u32(
    u32::MAX, 
    is_underflow.select_u32(0, val_shifted)
);
```

This implementation perfectly adheres to the rigid project laws (e.g., Radon Law $CC=1$ and mask-based execution) by substituting all control-flow with masks, bitwise algebra, and fixed-size math operations.
