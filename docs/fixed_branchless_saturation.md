I have inspected the `fixed.rs` file in `crates/bcinr-cmca/src/`. Here is the documentation detailing exactly how saturation is calculated branchlessly across the fixed-point operations.

# Branchless Saturation in `bcinr-cmca`

The library relies on a core primitive called `CanonicalMask` to perform branchless selections. A `CanonicalMask` is guaranteed to be either all-ones (`0xFFFFFFFF`) or all-zeros (`0x00000000`). It exposes a `select_u32(a, b)` (and `select_i32`) method that selects `a` if the mask is true (all-ones), or `b` if false (all-zeros), using bitwise operations: `(a & mask) | (b & !mask)`.

Using this primitive, the library elegantly routes around control-flow branches for saturation (clamping to max/min on overflow).

## `NonNegativeFixed` (Unsigned Q16.16)

For non-negative fixed-point numbers, saturation simply means clamping to `u32::MAX`.

### `saturating_add`
1. **Calculation**: It computes the sum using standard wrapping addition: `self.val.wrapping_add(other.val)`.
2. **Overflow Detection**: It compares the wrapped sum against the original value using a branchless comparator: `const_lt_u32(sum, self.val)`. If the sum is less than `self.val`, it means a wrap-around overflowed occurred. This returns a `CanonicalMask`.
3. **Clamping**: It picks the final value using the mask: `overflow.select_u32(u32::MAX, sum)`.

### `saturating_mul`
1. **Calculation**: Performs a 64-bit multiplication: `(self.val as u64).wrapping_mul(other.val as u64)` and shifts right by 16 bits to maintain Q16.16 scaling.
2. **Overflow Detection**: It extracts the high 32 bits (`res_u64 >> 32`). To branchlessly check if any of these bits are non-zero, it exploits two's complement sign-bit propagation:
   ```rust
   let overflow = (high | high.wrapping_neg()) >> 31;
   let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow));
   ```
   If `high != 0`, `overflow` becomes `1`, and wrapping subtraction from `0` turns it into `0xFFFFFFFF` (all-ones mask).
3. **Clamping**: Uses the mask to apply the upper bound: `overflow_mask.select_u32(u32::MAX, res_u64 as u32)`.

### `saturating_div`
1. **Calculation**: Uses Newton-Raphson approximation for division.
2. **Overflow Detection**: Accumulates conditions that would cause an invalid or oversized result:
   - Whether the denominator is zero (`den_is_zero`).
   - Whether the corrected quotient exceeds `u32::MAX` via two separate branchless boolean-conversion steps grouped into an `overflow` mask.
3. **Clamping**: Bitwise-ORs the raw masks together (`saturate = overflow.raw() | den_is_zero.raw()`) and applies the clamp: `saturate.select_u32(u32::MAX, q_corrected as u32)`.

---

## `SignedFixed` (Signed Q16.16)

For signed fixed-point numbers, saturation must determine whether to clamp to the positive maximum (`i32::MAX`) or negative minimum (`i32::MIN`).

### `saturating_add` and `saturating_sub`
1. **Calculation**: Utilizes the built-in `overflowing_add`/`overflowing_sub` which returns a tuple of the wrapped `sum`/`diff` and a boolean `overflow`.
2. **Direction Selection**: It checks if `self.val < 0` using `const_lt_i32(self.val, 0)`. It then branchlessly selects the target saturation bound:
   ```rust
   let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
   ```
3. **Mask Creation**: The boolean `overflow` is cast to `u32` and converted into a `CanonicalMask`: `0u32.wrapping_sub(overflow as u32)`.
4. **Clamping**: Picks the final value using the mask: `overflow_mask.select_i32(sat_val, sum)`.

### `saturating_mul`
1. **Calculation**: Performs 64-bit signed multiplication and normalizes the scaling (`prod >> 16`).
2. **Overflow Detection**: Creates two independent masks for bounds checking by converting boolean comparison results into masks:
   ```rust
   let overflow_max = CanonicalMask(0u32.wrapping_sub((res_i64 > i32::MAX as i64) as u32));
   let overflow_min = CanonicalMask(0u32.wrapping_sub((res_i64 < i32::MIN as i64) as u32));
   ```
3. **Clamping**: Sequentially applies the bounds via branchless selection:
   ```rust
   let mut res = overflow_min.select_i32(i32::MIN, res_i64 as i32);
   res = overflow_max.select_i32(i32::MAX, res);
   ```
   This ensures that if `overflow_min` is active, it clamps to `i32::MIN`; if `overflow_max` is active, it clamps to `i32::MAX`.
