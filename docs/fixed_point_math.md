# Q16.16 Fixed-Point Arithmetic in `bcinr`

The `bcinr` (BranchlessCInRust) project enforces strict architectural laws (e.g., $CC=1$, zero allocation) to provide deterministic, bounded, branchless, allocation-free execution. To satisfy these requirements while supporting real-number approximations, Q16.16 fixed-point arithmetic is implemented entirely without floating-point types and avoiding all data-dependent branches.

The implementation is located across two primary files: `crates/bcinr-logic/src/fix.rs` and `crates/bcinr-cmca/src/fixed.rs`.

## Core Representation

Q16.16 fixed-point numbers allocate 16 bits for the integer part and 16 bits for the fractional part, representing a value `v` as `v * 65536`.
- **`i32` / `SignedFixed`**: Signed representation, approximate range `[-32768, 32767]`.
- **`u32` / `NonNegativeFixed`**: Unsigned representation.

These types allow arithmetic on real numbers without linking float-based math libraries or violating the $CC=1$ (Cyclomatic Complexity = 1) rule.

## Branchless Mechanics

Conditional branches (e.g., `if`, `match`, short-circuit operators) are strictly prohibited. The runtime predicates are instead transformed into branchless arithmetic using bit-parallel masks.

### 1. Canonical Masks
Logical states are represented as full-width masks: `0x00000000` (FALSE) and `0xFFFFFFFF` (TRUE).
For example, to check equality branchlessly:
```rust
pub const fn const_eq_u32(a: u32, b: u32) -> CanonicalMask {
    let x = a ^ b;
    // Extract sign bit of x | -x to determine if there's any non-zero bit
    let nonzero = (x | x.wrapping_neg()) >> 31; 
    CanonicalMask(0u32.wrapping_sub(1u32.wrapping_sub(nonzero)))
}
```

### 2. Mask-Based Selection
Instead of branching (`if condition { a } else { b }`), the system uses bitwise logic (e.g., `(m & a) | (~m & b)`):
```rust
pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```
This guarantees execution in constant time, with a fixed instruction shape, regardless of the input value.

### 3. Saturation and Fault Handling
Errors like overflow and division-by-zero do not panic or short-circuit.
- **Saturating Arithmetic**: Handled using wrapping arithmetic coupled with mask-based fallback. E.g., for addition, if overflow occurs, it selects `u32::MAX` using a mask.
- **Division by Zero**: Guarded against by coercing a zero divisor into 1 using bitwise operations before division:
  ```rust
  let safe_b = b | ((b == 0) as i32);
  ```
- **`NumericFaultSet`**: A bitwise union fault set that propagates faults (OVERFLOW, DIVIDE_BY_ZERO, etc.) deterministically alongside the values via `union()` without early returns or branching.

## Advanced Mathematical Approximations

Advanced functions approximate operations natively using pure integers and bit-shifting:

- **Square Root (`isqrt_u32` / `q16_sqrt`) & Reciprocal (`q16_recip`)**: Implemented using fixed iterations of the Newton-Raphson method starting from bit-shift seeds. Loops are manually unrolled to remove bounded-loop control flow.
- **Trigonometry (`q16_sin_approx` / `q16_cos_approx`)**: Uses pure-integer Bhaskara I approximation: 
  $\sin(x) \approx \frac{16x(\pi - x)}{5\pi^2 - 4x(\pi - x)}$
  Inputs outside $[0, \pi]$ are projected using symmetry properties and integer modulo operations, resolving sign flips branchlessly.
- **Logarithm (`q16_log2`)**: Extracts the integer component directly from the position of the leading bit (`leading_zeros`), and computes the fractional part using a linear interpolation on the normalized mantissa.
- **Exponentiation (`exp2` / `exp`)**: Extracts the integer part for final shifting and feeds the fractional part into an unrolled 4-stage polynomial approximation. Left/right shifting based on sign is resolved using `select_u32`.

By translating all semantic decisions into boolean masks, arithmetic selections, and straight-line bitwise operations, the system achieves deterministic fixed-point math suitable for highly rigorous execution contexts.
