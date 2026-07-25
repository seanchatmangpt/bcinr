# Branchless Fixed-Point and Integer API Documentation

The `fix.rs` and `int.rs` modules in `crates/bcinr-api/src/` act as re-exports for the core deterministic implementations housed in the `bcinr_logic` crate. True to the project's "Radon Law" (Cyclomatic Complexity = 1), these APIs are engineered to be entirely branchless.

By aggressively eliminating `if`, `match`, and data-dependent `loop`s, the fixed-point APIs and integer boundaries execute in constant time, utilizing bitwise masks and SWAR (SIMD Within A Register) operations.

Below are the primary mechanisms used to achieve this branchless execution:

## 1. Mask-based Clamping and Saturation
Instead of relying on conditional jumps for boundaries (e.g., checking if a value exceeds a maximum), conditionals are transformed into full-width bit masks (either `0x00000000` or `0xFFFFFFFF`).

*   **Saturating Addition (`add_sat`)**:
    Instead of `if res < a { u32::MAX } else { res }`, the logic evaluates the carry-out to a boolean, casts it to `u32` (0 or 1), and subtracts it from 0 to form a mask:
    `let mask = 0u32.wrapping_sub((res < a) as u32);`
    The result is clamped using a simple bitwise OR: `res | mask`.

*   **Clamping Boundaries (`clamp_u32` / `clamp_u64`)**:
    Clamping is done using branchless multiplexers: `(limit & mask) | (value & !mask)`. The masks for `lt_min` and `gt_max` are generated using the same wrapping subtraction technique as above, guaranteeing that boundary selection requires no CPU branch prediction.

## 2. Division-by-Zero Guarding
To prevent hardware traps on division by zero without using `if divisor == 0 { ... }`, the API dynamically forces the divisor to a safe non-zero value (usually `1`) when zero is detected.

*   **`bucketize_u32(val, step)`**: 
    The step divisor is protected using `step.wrapping_add((step == 0) as u32)`. If `step` is `0`, the division safely executes as `val / 1`. The result is subsequently multiplied by the original `step` (`0`), yielding `0` as intended, entirely branchlessly.
*   **`q16_div(a, b)`**: 
    A similar technique is used for fixed-point division: `let safe_b = b | ((b == 0) as i32);`. 

## 3. Fixed-Iteration Root and Reciprocal Approximations
Data-dependent loop termination (`while` loops) is forbidden. Algorithms that conventionally loop until a precision threshold is met are statically unrolled.

*   **Square Root (`isqrt_u32`)**: Calculates a branchless initial shift estimate and performs exactly four unrolled Newton-Raphson iterations, which mathematically guarantees full `u32` convergence.
*   **Overshoot Correction**: The typical `if x * x > n { x -= 1 }` correction step is reduced to: `x - ((x as u64 * x as u64 > n as u64) as u32)`, safely applying the correction via an arithmetic cast.

## 4. O(1) Threshold Summation
Operations that often use loops for digit scanning (e.g. `decimal_digits_u64`) instead use threshold comparison trees. 
The algorithm branchlessly evaluates powers of 10 and sums the boolean results: `1 + (n >= 10) as u32 + (n >= 100) as u32 + ...`. This guarantees O(1) bounds-checked execution.

## 5. Parallel Bit-Swap Networks (SWAR)
Bitwise manipulations operate via parallel logarithmic passes instead of bit-by-bit iteration.
*   **Bit Reversal (`reverse_bits_u64`)**: Operates in exactly 6 interleaved passes (`O(log₂ 64)`). It repeatedly swaps adjacent groups of 1, 2, 4, 8, 16, and 32 bits utilizing alternating static masks (like `0x5555555555555555` and `0x3333333333333333`) and bitwise shifts.

## 6. Integer Trigonometry
Approximating sine (`q16_sin_approx`) uses integer modulo arithmetic to fold arbitrary Q16.16 angles into the `[0, pi]` domain. It calculates the Bhaskara I approximation using purely integer operations over scaled Q32.32 temporaries, fully bypassing FPUs (Floating-Point Units) and ensuring strict, deterministic bounds without any conditional flow.
