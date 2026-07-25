# Branchless Fixed-Point Arithmetic in BCINR

BCINR enforces strict adherence to **Rule 14** (Numeric-law requirements) and **Rule 8** (Absolute `CC=1` law) by implementing fixed-point arithmetic without any control-flow branches, conditionals (`if`/`match`), or data-dependent loops. 

Across the codebase (in `bcinr-cmca/src/fixed.rs`, `bcinr-logic/src/fix.rs`, and algorithmic implementations), fixed-point multiplication and division use widening, bitwise logic, masks, and Newton-Raphson approximation to ensure deterministic, bounded, and constant-time execution.

## 1. Advanced Fault-Tracking Implementations (`bcinr-cmca/src/fixed.rs`)

The `bcinr-cmca` crate introduces a strictly encapsulated `NonNegativeFixed` (and `SignedFixed`) struct, which tracks both Q16.16 values and accumulated branchless `NumericFaultSet` values (e.g., `OVERFLOW`, `DIVIDE_BY_ZERO`).

### Multiplication (`saturating_mul`)
- **Widening and Shifting:** Multiplies via `u64` to avoid intermediate overflow (`(self.val as u64).wrapping_mul(other.val as u64)`).
- **Branchless Masking:** Checks if the high 32 bits indicate an overflow using bitwise polynomial logic:
  ```rust
  let high = (res_u64 >> 32) as u32;
  let overflow = (high | high.wrapping_neg()) >> 31;
  let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow));
  ```
- **Saturating Selection:** The result uses `CanonicalMask::select_u32` to branchlessly pick `u32::MAX` on overflow or the computed product otherwise. It also joins `OVERFLOW` and `SATURATION` faults branchlessly via bitwise union.

### Division Replacement (`saturating_div`)
- **Hardware Division Evasion:** Hardware division instructions (`idiv`/`div`) represent non-constant time operations and throw traps (hidden branches) on zero division. BCINR completely replaces this with a **Newton-Raphson approximation**.
- **Zero Divisor Handling:** Branchlessly replaces a zero divisor with `1` to avoid undefined logic during intermediate steps:
  ```rust
  let den_is_zero = const_eq_u32(other.val, 0);
  let d = den_is_zero.select_u32(1, other.val);
  ```
- **Newton-Raphson Steps:** Approximates the reciprocal using constants and sequential `i128` operations (`x0`, `x1`, `x2`, `x3`). 
- **Residual Correction:** Multiplies the reciprocal by the numerator and performs branchless correction based on the remainder:
  ```rust
  let is_lt = ((rem >> 63) & 1) as u64;
  let is_ge = (((!diff) >> 63) & 1) as u64;
  let q_corrected = q.wrapping_add(is_ge).wrapping_sub(is_lt);
  ```
- **Saturation and Faults:** Uses `CanonicalMask` to clamp the value to `MAX` if the divisor was zero or overflow occurred, while accumulating `DIVIDE_BY_ZERO` and `INVALID_DOMAIN` faults without a single `if` statement.

## 2. Basic Q16.16 Operations (`bcinr-logic/src/fix.rs`)

For simpler Q16.16 logic without fault tracking, `bcinr-logic` uses `i32` fixed-point representations.

### Multiplication (`q16_mul`)
- Simply widens to `i64`, multiplies, and applies a logical right shift to maintain Q16 precision:
  ```rust
  pub fn q16_mul(a: i32, b: i32) -> i32 {
      ((a as i64 * b as i64) >> 16) as i32
  }
  ```

### Division (`q16_div`)
- Avoids the hardware division-by-zero trap by bitwise ORing the divisor with `1` if it is `0`.
  ```rust
  pub fn q16_div(a: i32, b: i32) -> i32 {
      // Branchless: replace zero divisor with 1 to avoid division by zero.
      let safe_b = b | ((b == 0) as i32);
      ((a as i64 * (1 << 16)) / safe_b as i64) as i32
  }
  ```

## 3. High-Precision u64/u128 Implementations (`bcinr-logic/src/algorithms/`)

The documentation inside BCINR (`docs/algorithm_fixed_point_mul_div.md`) also highlights robust `u64` fixed-point algorithms (`fp_mul_u32_q16` and `fp_div_u32_q16`).

### Multiplication (`fp_mul_u32_q16`)
- Safely casts `u64` to `u128` for multiplication to guarantee an overflow trap is impossible, then shifts by `16`.

### Division (`fp_div_u32_q16`)
- **Divisor Substitution:** `(aux as u128 | 1)` to eliminate the hardware trap safely.
- **Bitwise Polynomial Logic for Clamping:** To return exactly `0` when dividing by `0` (enforcing the domain contract), a full-width mask is constructed:
  ```rust
  let mask = (aux == 0) as u64;
  let res = (((val as u128) << 16) / (aux as u128 | 1)) as u64;
  res & (!mask.wrapping_neg()) // Projects valid output or forces 0
  ```
  If `aux == 0`, `mask` is 1. Its wrapping negation becomes `0xFFFFFFFFFFFFFFFF`, the bitwise NOT makes it `0x0`, clamping the outcome to `0` purely through arithmetic logic.
