# BCINR Fixed-Point Architecture: Branchless Q16.16 

Based on my research into the `bcinr-logic/src/fix.rs` implementation, the `bcinr` substrate strictly forbids floating-point representations (`f32`/`f64`) to comply with the constitution's deterministic, branchless, and CC=1 directives. Here is the mathematical architecture documenting how numeric bounds and mathematical projection (like `q_lens` and exponentiations) are enforced without architecture-dependent rounding, `NaN`s, or infinities.

## 1. The Q16.16 Representation Model
The substrate avoids floating-point anomalies (like `NaN` or un-ordered values) by representing real numbers as exact integers. 
Values are mapped onto a **Q16.16 fixed-point format**, backed by standard 32-bit registers (`i32` for signed, `u32` for unsigned):
* **Integer domain:** $\approx [-32768, 32767]$
* **Fractional resolution:** $1/65536 \approx 0.00001525$

Conversion from float is $round(V \times 65536)$. Because the values are exact integrals, comparisons are completely deterministic. There is no concept of subnormals or negative zero.

## 2. Mathematical Representation of Bounds (Branchless Saturating Arithmetic)
Bounds and overflow protection are executed logically without `if`, `match`, or panics.

### Clamp enforcement (`clamp_u32`)
Bounding a value into a strict `[min, max]` interval generates no conditional branches. It relies on deterministic state selection through logical bitmasks (`0` or `0xFFFFFFFF`).
```rust
let lt_min = (res < min) as u32;
// Replaces 'res' with 'min' if 'lt_min' is 1
res = (min & 0u32.wrapping_sub(lt_min)) | (res & !0u32.wrapping_sub(lt_min));
```

### Saturating limits (`add_sat`)
Instead of floating-point $\infty$, the architecture uses saturation (e.g., stopping exactly at `u32::MAX`). Overflows are caught using branchless wrapping rules and bit-shifts, applying a correction mask if the wrapping sum implies an overflow (`res < a`).

### Defeating Division-by-Zero (`q16_div`, `bucketize_u32`)
To avoid hardware exceptions (division by zero panics or $\infty$), the substrate intercepts the divisor.
```rust
// In q16_div
let safe_b = b | ((b == 0) as i32); 
```
If `b` is `0`, `safe_b` becomes `1`. The subsequent saturated bitwise logic prevents panics, maintaining execution determinism.

## 3. Emulating Non-Linear Algebra (Path to $q\_lens$)
The $q\_lens$ function mathematically represents a normalized exponentiation projection: $L_q(i) = \frac{p_i^q}{\sum_j p_j^q}$.
Implementing this normally requires `f64::powf`, risking non-determinism. Under `fix.rs`, this must be solved purely structurally using fixed-point primitives:

1. **Intermediate Up-Casting:** All Q16.16 multiplications and divisions escalate to `i64` intermediately to preserve precision and prevent overflow before bit-shifting back `>> 16`.
2. **Fixed-Unrolled Newton-Raphson:** Roots and reciprocals (like $\frac{1}{\sum p_j^q}$) are resolved using Newton-Raphson approximation (`q16_recip` and `isqrt_u32`). The iterations are bounded to exactly four unrolled steps, guaranteeing $O(1)$ constant time with no back-edges.
3. **Logarithms for Powers:** Exponentiation ($p^q$) is mathematically unrolled via $2^{q \times \log_2(p)}$. 
   - `fix.rs` provides `q16_log2`, which computes the exact integer log2 via hardware instructions (`leading_zeros`), and approximates the fractional mantissa with deterministic linear interpolation.
4. **Trigonometry (Bhaskara I):** Sine and Cosine are mapped to exact integer polynomials (e.g., $16x(\pi - x) / (5\pi^2 - 4x(\pi - x))$) using Q32.32 intermediary spaces, eliminating all transcendental FPU calls.

## Summary
By using explicit **bitwise polynomial selections** and **unrolled iterative approximations**, the Q16.16 architecture entirely removes the $f32/f64$ FPU reliance. $q\_lens$ probabilities, normalization weights, and bounds evaluate as strict combinations of boolean masks and `i64` shifts, sealing off any possibility of non-deterministic drift.
