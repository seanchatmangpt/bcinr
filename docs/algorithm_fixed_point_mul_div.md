# Branchless Fixed-Point Arithmetic in BCINR

I have located the implementations for fixed-point multiplication and division within `crates/bcinr-logic/src/algorithms/`. Specifically, they are implemented as Q16 fixed-point operations in `fp_mul_u32_q16.rs` and `fp_div_u32_q16.rs`.

Here is how their mathematical laws are enforced branchlessly, strictly adhering to **Rule 14** (Numeric-law requirements) and the core architectural laws ($CC=1$).

## 1. Fixed-Point Multiplication (`fp_mul_u32_q16`)

**Implementation:**
```rust
pub fn fp_mul_u32_q16(val: u64, aux: u64) -> u64 {
    ((val as u128 * aux as u128) >> 16) as u64
}
```

**Branchless Enforcement (Rule 14):**
- **No Overflow Panic or Conditionals:** The multiplication temporarily casts the 64-bit operands to a wider `u128` type. The maximum possible product of two `u64` values easily fits within a `u128`, mathematically proving that an overflow trap is impossible.
- **Fixed-width Scaling:** Instead of conditionally checking bounds or branching for normalization, it universally applies a logical right shift (`>> 16`) to maintain the Q16 fixed-point scaling factor.
- **Determinism:** The operations strictly consist of widening, multiplication, shifting, and truncation. This compiles into straight-line machine code with no loop backedges or conditional jumps, guaranteeing constant-time execution.

## 2. Fixed-Point Division (`fp_div_u32_q16`)

**Implementation:**
```rust
pub fn fp_div_u32_q16(val: u64, aux: u64) -> u64 {
    let mask = (aux == 0) as u64;
    let res = (((val as u128) << 16) / (aux as u128 | 1)) as u64;
    res & (!mask.wrapping_neg())
}
```

**Branchless Enforcement (Rule 14 & Rule 9):**
- **Division-by-Zero Evasion:** Hardware division instructions trap (panic) on division by zero, which acts as a hidden branch. To eliminate this, the divisor is unconditionally ORed with 1 (`aux as u128 | 1`). If `aux` is 0, the denominator safely becomes 1, bypassing the hardware exception without any conditional `if` blocks.
- **Mask-based State Selection:** To fulfill the contract (which dictates returning `0` when dividing by `0`), the algorithm constructs a full-width bitmask (`!mask.wrapping_neg()`). 
  - When `aux == 0`, `mask` is `1`. `1.wrapping_neg()` yields `0xFFFFFFFFFFFFFFFF`, and the bitwise NOT `!` turns it into `0x0`.
  - When `aux != 0`, `mask` is `0`. `0.wrapping_neg()` is `0x0`, and the bitwise NOT `!` yields `0xFFFFFFFFFFFFFFFF`.
- **Bitwise Polynomial Logic:** The tentative result is bitwise ANDed with this full-width mask. This acts as a mathematical projection: isolating the valid division output when `aux != 0` and forcefully clamping the result to `0` when `aux == 0`. It replaces the control-flow sequential decision (`if aux == 0 { 0 } else { val / aux }`) with bit-parallel polynomial arithmetic.
- **Precision:** The numerator is shifted left by 16 (`<< 16`) *before* division within the `u128` width, preserving Q16 precision dynamically without requiring any floating-point approximations.

Both primitives fulfill the mandate of being deterministic, branchless, and bounded by fixed-width types.
