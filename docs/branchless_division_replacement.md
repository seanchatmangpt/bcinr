# Branchless Fixed-Point Division Replacement in BCINR

In the BCINR architecture, hardware integer division instructions (e.g., `udiv` in AArch64 or `div` in x86-64) are strictly banned. These instructions violate the **Radon Law ($CC=1$)** because they are multi-cycle, non-pipelined, iterative operations with variable latencies. This introduces timing side-channels and breaks the deterministic, constant-time execution contract required by the substrate. 

To achieve division branchlessly in `Q16.16` fixed-point arithmetic (`Fixed` / `NonNegativeFixed`), BCINR mathematically decomposes the operation. The strategy differs depending on whether the divisor is a **compile-time constant** or a **dynamic variable**.

## 1. Division by Compile-Time Constants (Magic Reciprocal Multiplication)

When the divisor is known at compile time, the substrate avoids hardware division entirely by replacing it with **magic reciprocal multiplication** and bit-shifting. This technique relies on precomputed magic numbers to emulate floating-point division using purely fixed-point integer arithmetic.

For example, in `crates/bcinr-logic/src/algorithms/base85_encode_ascii85.rs`, division by `85` is required to encode Ascii85 digits. Rather than using the hardware modulo or division operator, it uses a constant-time closure:

```rust
let div85 = |x: u64| (x.wrapping_mul(0xC0C0C0C1)) >> 38;
```

### How the Magic Number Works:
- We want to compute $x / 85$. Mathematically, this is $x \times (1/85)$.
- We scale the fractional reciprocal $1/85$ by a large power of two, such as $2^{38}$, to map it into the integer domain.
- $2^{38} / 85 \approx 3233857729$, which in hexadecimal is the "magic constant" `0xC0C0C0C1`.
- Multiplying the input $x$ by this magic number and then shifting right by 38 bits (`>> 38`) perfectly simulates division by $85$ for 32-bit inputs.

This pattern operates in exactly $O(1)$ hardware cycles, ensuring zero branches, zero hardware division latency, and mathematically sound deterministic outputs. Additionally, LLVM compiler optimizations are natively leveraged to perform this transformation for modulo and division operations against known constants.

## 2. Division by Dynamic Variables (Newton-Raphson Replacement)

When evaluating division with an unknown, dynamic divisor at runtime, BCINR utilizes a **Branchless Reciprocal Approximation** driven by precomputed minimax constants, bounded multiplication, and Newton-Raphson refinement.

### Divisor Normalization
The first step avoids division-by-zero undefined behavior and normalizes the divisor to ensure its most significant bit is set:
- A mask checks if the divisor is zero. If so, it is mapped branchlessly to `1` using a `select` operation.
- The hardware `leading_zeros()` instruction (single cycle) is used to find the shift amount (`lz`).
- The divisor is shifted left by `lz` to produce $d_{\text{norm}}$.

### Slow-Rail-Precomputed Minimax Initial Approximation
Rather than storing a lookup table, BCINR dynamically calculates the initial reciprocal in a signed Q2.62 format using a linear minimax approximation:

$$ X_0 = A_{\text{scale}} - B_{\text{coeff}} \cdot d_{\text{norm}} $$

Where:
* $A_{\text{scale}} = 13021703673752174592 \approx 2.8235 \times 2^{62}$
* $B_{\text{coeff}} = 2021160080 \approx 1.8824 \times 2^{30}$

### Newton-Raphson Iteration
To refine $X_0$ without loss of precision, three unrolled iterations of Newton-Raphson refinement are performed in signed 128-bit space (`i128`). 
1. **Error Calculation:** $e_k = 2^{94} - d_{\text{norm}} \cdot X_k$
2. **Reciprocal Update:** $X_{k+1} = X_k + \frac{X_k \cdot (e_k \gg 32)}{2^{62}}$

Shifting the error right by 32 bits before multiplication mathematically guarantees that intermediate products will not overflow the signed 128-bit integer bounds.

### Quotient Computation and Shift Alignment
With the highly precise reciprocal ($X_3 \approx 2^{94} / d_{\text{norm}}$), the preliminary quotient $q$ is computed via a single unsigned multiplication:

$$ q = \frac{N \cdot X_3}{2^{78 - lz}} $$

### Branchless Remainder Correction
Because integer math truncates and the initial approximation can drift slightly, the uncorrected quotient $q$ might differ from the exact mathematical quotient by $\pm 1$ LSB. BCINR applies a final branchless remainder correction to achieve **100% bit-identical accuracy**:
1. The exact remainder is calculated: $\text{rem} = (N \cdot 2^{16}) - q \cdot D$
2. Using bitwise sign extraction, the architecture branchlessly checks if the remainder is negative (overshot) or $\ge D$ (undershot).
3. The quotient is corrected via wrapping additions and subtractions of these integer-cast boolean masks.

By replacing `udiv`/`div` with this multi-stage pipeline and magic reciprocal multiplication, BCINR achieves identical fixed-point division outputs while securely eliminating timing side-channels and upholding the perfect $CC=1$ determinism requirement.
