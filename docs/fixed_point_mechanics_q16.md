# Q16.16 Fixed-Point Mechanics in `bcinr`

## Why Q16.16 Fixed-Point Over Floating-Point?
The `bcinr` (BranchlessCInRust) project enforces strict architectural laws (e.g., $CC=1$, zero allocation) to provide a deterministic, bounded, and branchless computation substrate. Q16.16 fixed-point representation is chosen over standard IEEE 754 floating-point for several critical reasons:

- **Determinism and Architecture Independence:** Floating-point math introduces architecture-dependent rounding, subtle non-determinism across platforms, and unhandled NaN/infinity states. `bcinr` mandates absolute mathematical determinism.
- **The Radon Law ($CC=1$):** `bcinr` strictly prohibits data-dependent branches (`if`, `match`, short-circuiting) in its authoritative runtime. Floating-point edge cases are notoriously difficult to handle branchlessly, and hardware floating-point instructions can introduce non-deterministic execution latencies.
- **Pure Arithmetic Logic:** Q16.16 allocates 16 bits for the integer part and 16 bits for the fractional part (representing values as $v \times 65536$) within standard integer types (`i32` for signed, `u32` for unsigned). This allows pure integer bit-level operations to approximate real numbers, keeping all logic strictly as "bitwise polynomials."

## Branchless Saturation Arithmetic
Instead of relying on conditional branches to enforce limits or allowing standard panic-inducing hardware faults (e.g., division by zero or integer overflow), `bcinr` uses bitwise logic and canonical masks (`0x00000000` for FALSE, `0xFFFFFFFF` for TRUE) to handle saturation explicitly.

### 1. Saturating Addition (`add_sat`)
When an addition overflows, it falls back to a saturated `MAX` value using only wrapping arithmetic and a carry-out mask:
```rust
pub const fn add_sat(a: u32, b: u32) -> u32 {
    let res = a.wrapping_add(b);
    // If res < a, overflow occurred. 
    // (res < a) as u32 evaluates to 1.
    // 0u32.wrapping_sub(1) yields 0xFFFFFFFF (u32::MAX).
    // res | 0xFFFFFFFF becomes u32::MAX.
    // If no overflow, it yields res | 0 = res.
    res | 0u32.wrapping_sub((res < a) as u32)
}
```

### 2. Division by Zero Protection (`q16_div`)
Division operations forcibly avoid panics by safely substituting a zero divisor with `1` branchlessly. The hardware division executes without trapping, allowing subsequent logic to handle the saturated state:
```rust
pub fn q16_div(a: i32, b: i32) -> i32 {
    // When b == 0, safe_b becomes 1 to prevent a division-by-zero hardware fault.
    let safe_b = b | ((b == 0) as i32);
    ((a as i64 * (1 << 16)) / safe_b as i64) as i32
}
```

### 3. Saturation via Mask Selection (`q16_recip`)
For operations like the reciprocal, an exact zero input must explicitly yield the maximum possible value (`i32::MAX`). This is resolved by computing a bit-mask from the zero check and using bitwise logic to select the final result without any `if/else` control flow:
```rust
let is_zero = (x == 0) as i32; // 0 or 1
let zero_mask = 0i32.wrapping_sub(is_zero); // 0x00000000 or 0xFFFFFFFF

// Safe input for internal Newton-Raphson math:
let safe_x = (x & !zero_mask) | (1i32 & zero_mask); 

// ... internal algorithm calculates r2 based on safe_x ...

// Return calculated r2 normally, or explicitly return i32::MAX if x was zero
(r2 & !zero_mask) | (i32::MAX & zero_mask)
```

## Summary
The Q16.16 representation enables `bcinr` to execute complex mathematical algorithms (such as Newton-Raphson square roots, Bhaskara I trigonometric approximations, and logarithm interpolations) safely over continuous domains. It achieves this without breaking its fixed instruction shape, violating zero-allocation constraints, or relying on any underlying float-processor states.
