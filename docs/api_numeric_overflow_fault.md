Here is the markdown documentation explaining how `NumericFaultSet::OVERFLOW` is set branchlessly and what mathematical conditions it represents in the BCINR framework:

```markdown
# Documentation: `NumericFaultSet::OVERFLOW` in `bcinr-cmca`

In the BCINR framework, `NumericFaultSet::OVERFLOW` is a bitflag representing a numerical overflow fault. To adhere to the deterministic, branchless, and `CC=1` laws, the framework completely avoids control flow (`if`, `match`, etc.) when detecting and applying overflow faults.

## Definition

Located in `crates/bcinr-cmca/src/fixed.rs`, the bitmask is defined inside the `NumericFaultSet` struct, which accumulates errors via bitwise union:

```rust
pub struct NumericFaultSet(u32);

impl NumericFaultSet {
    pub const EMPTY: Self = Self(0);
    pub const OVERFLOW: Self = Self(1 << 0);
    // ...
    pub const SATURATION: Self = Self(1 << 6);
}
```

It is almost always paired with saturation when applied:
`NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION)`

## Branchless Application Mechanism

Overflow is applied using `CanonicalMask::select_faults`. A `CanonicalMask` is a struct containing a `u32` that is strictly either all zeros (`0x00000000` for `FALSE`) or all ones (`0xFFFFFFFF` for `TRUE`).

`select_faults` operates entirely through bitwise arithmetic:
```rust
#[inline(always)]
pub const fn select_faults(self, a: NumericFaultSet, b: NumericFaultSet) -> NumericFaultSet {
    NumericFaultSet::from_bits_raw((a.0 & self.0) | (b.0 & !self.0))
}
```
If the underlying operation overflows, the generated mask is all `1`s, selecting the `OVERFLOW | SATURATION` bits. Otherwise, it selects `EMPTY`.

## Mathematical Conditions and Mask Derivation

The mathematical condition for an "overflow" varies by operation. The framework derives a `CanonicalMask` from these conditions strictly using bitwise arithmetic.

### 1. Unsigned Addition (`NonNegativeFixed::saturating_add`)
* **Mathematical Condition**: The sum of two values wraps around (e.g., $a + b \pmod{2^{32}} < a$).
* **Branchless Derivation**:
  The condition is checked via `const_lt_u32(sum, self.val)`. This function evaluates the "less than" condition entirely with bitwise operators:
  ```rust
  let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
  CanonicalMask(0u32.wrapping_sub(diff))
  ```

### 2. Unsigned Multiplication (`NonNegativeFixed::saturating_mul`)
* **Mathematical Condition**: The 64-bit product, right-shifted by 16 for fixed-point normalization, exceeds `u32::MAX` ($> 2^{32} - 1$). Thus, the upper 32 bits of the 64-bit result must be non-zero.
* **Branchless Derivation**:
  It extracts the top 32 bits into `high` and checks if it's non-zero using a standard bit-parallel trick (`high | high.wrapping_neg()`). The sign bit will be `1` if `high != 0`.
  ```rust
  let high = (res_u64 >> 32) as u32;
  let overflow = (high | high.wrapping_neg()) >> 31;
  let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow));
  ```

### 3. Signed Addition/Subtraction (`SignedFixed::saturating_add` & `saturating_sub`)
* **Mathematical Condition**: Standard two's complement overflow (e.g., adding two positive numbers yields a negative number).
* **Branchless Derivation**:
  It delegates to Rust's intrinsic `overflowing_add/sub`, which maps to hardware flags and returns a `bool` without branching. This boolean is then stretched to a canonical mask via two's complement negation:
  ```rust
  let (sum, overflow) = self.val.overflowing_add(other.val);
  let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
  ```

### 4. Signed Multiplication (`SignedFixed::saturating_mul`)
* **Mathematical Condition**: The normalized 64-bit product exceeds `i32::MAX` or falls below `i32::MIN`.
* **Branchless Derivation**:
  The condition is computed by casting boolean comparisons to `u32` (which evaluates to 0 or 1), then stretching them into canonical masks:
  ```rust
  let overflow_max = CanonicalMask(0u32.wrapping_sub((res_i64 > i32::MAX as i64) as u32));
  let overflow_min = CanonicalMask(0u32.wrapping_sub((res_i64 < i32::MIN as i64) as u32));
  let is_overflow = overflow_max.raw() | overflow_min.raw();
  // Followed by fault selection using `const_eq_u32(is_overflow, 0)`
  ```
```
