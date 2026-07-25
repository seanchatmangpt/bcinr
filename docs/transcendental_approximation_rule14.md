# Branchless Transcendental Approximation in `bcinr`

In accordance with Rule 14 (Numeric-law requirements) and the strict branchless nature of the BCINR substrate (Rule 8: CC=1), transcendental functions like reciprocal, logarithm, and exponential are implemented entirely in fixed-point/integer arithmetic using bitwise operations, masking, and polynomial/linear approximation, strictly avoiding floating-point instructions (`f32`/`f64`).

Here is a detailed breakdown of the approximation strategies discovered in the codebase:

### 1. Reciprocal (`q16_recip`)
**Location:** `crates/bcinr-logic/src/fix.rs`
**Strategy:** Newton-Raphson Method
- **Guard & Saturation:** To prevent division-by-zero panics without branching, a zero-mask is created: `let zero_mask = 0i32.wrapping_sub((x == 0) as i32);`. This masks `x = 0` to a safe denominator `1`.
- **Initial Estimate:** Computed directly via 64-bit integer division: `(1 << 32) / safe_x`.
- **Refinement:** Applies two steps of the Newton-Raphson iteration $r_{n+1} = r_n \times (2 - x \times r_n)$ in Q16.16 arithmetic. The constant `2` is represented as `2 << 16` and the operations utilize branchless `q16_mul`.
- **Branchless Return:** At the end, the valid result is combined with the zero-mask: `(r2 & !zero_mask) | (i32::MAX & zero_mask)`, enforcing constant-time saturation to `i32::MAX` when the input is `0`.

### 2. Logarithm (`fixed_point_log2` and `q16_log2`)
**Locations:** `crates/bcinr-logic/src/algorithms/fixed_point_log2.rs` and `crates/bcinr-logic/src/fix.rs`
**Strategy:** Hardware Leading-Zero & Mantissa Linear Interpolation
- **Integer Part:** The base-2 integer exponent is derived purely using the hardware `leading_zeros()` instruction (`63 - clz(val)` for u64). A bitmask `nz` guarantees that an input of `0` cleanly maps to `0` without conditional control flow.
- **Fractional Part:** The implicit leading `1` bit is discarded using a wrapping left shift (`lz + 1`). The remaining lower bits (the mantissa) are isolated and linearly interpolated as the fractional part. 
- **Combination:** The result pieces are combined via `ip.wrapping_shl(fb).wrapping_add(frac)`. (In the Q16 variant, it linearly scales the mantissa to represent the fractional bits).

### 3. Exponential (`exp2_u64_fixed`)
**Location:** `crates/bcinr-logic/src/algorithms/exp2_u64_fixed.rs`
**Strategy:** Bit-Shift with Safe Masking & Branchless Saturation
- **Exponent Extraction:** The input is treated as Qx.16 format. The integer exponent is extracted purely via bit-shifting `val >> 16`.
- **Branchless Shift Guarding:** To prevent undefined behavior/panics from shifting a 64-bit integer by $\ge 64$, the shift amount is clamped using bitwise AND: `int_exp & 63`. 
- **Branchless Saturation:** Instead of conditional assignment `if int_exp >= 48 { u64::MAX }`, the code calculates a full-width saturation mask `let sat_mask = ((int_exp >= 48) as u64).wrapping_neg()`. 
- **Result Selection:** The final shifted value `65536 << safe_exp` is returned via bitwise selection `(result & !sat_mask) | sat_mask`, enforcing saturation boundary limits in deterministic constant time.
