# NonNegativeFixed Division Replacement in BCINR

In the BCINR architecture, the **Radon Law ($CC=1$)** mandates that all authoritative runtime execution must run in constant time with zero data-dependent branches and deterministic instruction cycles. Hardware integer division instructions (e.g., `udiv` in AArch64 or `div` in x86-64) inherently violate this requirement. They are multi-cycle, non-pipelined, iterative subtract-and-shift operations with variable latencies, which introduce unacceptable timing side-channels and break the deterministic execution contract.

To maintain fixed-width deterministic clock cycles for `NonNegativeFixed` (Q16.16) arithmetic, BCINR completely bans the use of the standard division operator on the hot path. Instead, it replaces division with a **Branchless Reciprocal Approximation** driven by Slow-Rail-precomputed minimax constants and bounded multiplication via Newton-Raphson refinement.

## 1. Slow-Rail-Precomputed Minimax Approximation

Rather than attempting to store a precomputed reciprocal for every possible value (which would massively exceed memory limits) or using branching integer division, the substrate dynamically calculates the reciprocal on the fly. It begins with a linear minimax initial approximation using optimized coefficients precomputed on the Slow-Rail:

$$ X_0 = A_{\text{scale}} - B_{\text{coeff}} \cdot d_{\text{norm}} $$

Where:
* $A_{\text{scale}} = 13021703673752174592 \approx 2.8235 \times 2^{62}$
* $B_{\text{coeff}} = 2021160080 \approx 1.8824 \times 2^{30}$

This gives a high-precision signed Q2.62 reciprocal estimation in constant time.

## 2. The Branchless Execution Pipeline

The implementation of `NonNegativeFixed::saturating_div` in `crates/bcinr-cmca/src/fixed.rs` executes the following purely sequential, branchless steps:

### A. Divisor Normalization
The divisor is normalized to ensure its most significant bit is set, utilizing the hardware `leading_zeros` instruction (e.g., `lzcnt`), which executes in a single cycle. Zero divisors are mapped branchlessly to 1 to prevent undefined behavior before normalization.
```rust
let den_is_zero = const_eq_u32(other.val, 0);
let d = den_is_zero.select_u32(1, other.val);
let lz = d.leading_zeros();
let d_norm = d << lz;
```

### B. Newton-Raphson Iteration in Signed 128-Bit Space
To refine the initial Slow-Rail estimate $X_0$ without loss of precision, BCINR performs exactly three unrolled iterations of Newton-Raphson refinement in signed 128-bit space (`i128`).
For each iteration $k \in \{0, 1, 2\}$, the residual error is calculated and the reciprocal is updated:
1. **Error Calculation:** $e_k = 2^{94} - d_{\text{norm}} \cdot X_k$
2. **Reciprocal Update:** $X_{k+1} = X_k + \frac{X_k \cdot (e_k \gg 32)}{2^{62}}$

Shifting the error before multiplication branchlessly prevents 128-bit signed overflow. After exactly three iterations, the reciprocal $X_3$ achieves an absolute error bounded by 1 LSB.

### C. Quotient Computation
With the highly precise reciprocal ($X_3 \approx 2^{94} / d_{\text{norm}}$), the preliminary uncorrected quotient $q$ is computed via a single 128-bit multiplication and a two-stage shift to account for the earlier normalization (total shift of $78 - lz$):
```rust
let n = self.val as u128;
let q_u128 = n.wrapping_mul(x3 as u128);
let q_shifted_46 = (q_u128 >> 46) as u64;
let q = q_shifted_46 >> (32 - lz);
```

### D. Branchless Remainder Correction
Because integer division truncates, the reciprocal approximation can result in a quotient that differs by exactly $\pm 1$ LSB. A final branchless remainder correction is applied:
1. The exact remainder is calculated: $\text{rem} = (N \cdot 2^{16}) - q \cdot D$
2. Using bitwise sign extraction (shifting right by 63 bits), the architecture branchlessly checks if the remainder is negative ($q$ overshot) or greater than or equal to $D$ ($q$ undershot).
3. The quotient is corrected by wrapping additions/subtractions of those boolean masks.

### E. Mask-Based Fault Accumulation
In compliance with the `@von_neumann_bypass` protocol, the operation does not panic or short-circuit on invalid domains (like division by zero) or saturating overflows. Instead, it utilizes `CanonicalMask` and `NumericFaultSet` to branchlessly select and union faults:
```rust
let e = CanonicalMask::select_faults(
    den_is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    CanonicalMask::select_faults(
        overflow,
        NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
        NumericFaultSet::EMPTY,
    ),
);
```

## Summary
By mathematically decomposing Q16.16 division into Slow-Rail precomputed minimax constants, 3 stages of unrolled Newton-Raphson refinement, and bitwise fault masking, BCINR completely replaces hardware division. The resulting pipeline achieves 100% bit-identical accuracy to hardware division while guaranteeing constant-time deterministic cycles ($CC=1$) under the most hostile adversarial execution environments.
