# Branchless Fixed-Point Sign Inversion in `bcinr`

In accordance with the `bcinr` deterministic substrate's absolute runtime laws (specifically the Radon Law requiring $CC=1$ and zero panic paths), sign inversion (negation) of fixed-point integers is computed using bitwise polynomials and explicitly wrapping arithmetic.

## Branchless Two's Complement Negation

The standard `if x < 0 { -x } else { x }` control flow violates the deterministic substrate constitution by introducing a data-dependent branch. To bypass this, `bcinr` dynamically computes the two's complement negation ($!x + 1$) using branchless bitwise masks. 

This pattern is prominently visible in primitives like `ct_abs_i64` and `copy_sign_i64`:

```rust
// Arithmetic right shift propagates the sign bit into all positions.
// mask = 0 for non-negative, -1 (all ones) for negative.
let mask = x >> 63; 

// (x ^ mask) - mask: for negative, flips all bits then adds 1 = two's complement negation.
let result = (x ^ mask).wrapping_sub(mask);
```

**How it works:**
1. **Mask Generation:** An arithmetic right shift on a signed integer duplicates the sign bit across all bits. A negative value generates a mask of `111...111` (which is `-1` in two's complement), while a non-negative value generates `000...000` (`0`).
2. **Bitwise NOT:** The XOR operation `x ^ mask` acts as a no-op if the mask is `0`. If the mask is `-1`, it flips every bit of `x` (equivalent to `!x`).
3. **Plus One:** The function then subtracts the mask using `.wrapping_sub(mask)`. Subtracting `-1` is mathematically identical to adding `1`, completing the two's complement algorithm ($!x + 1$). Subtracting `0` leaves the value unchanged.

## Handling the `-i32::MIN` Edge Case

The fixed-point representation (e.g., Q16.16 mapped over `i32`) has an asymmetric bounds problem: `i32::MIN` ($-2147483648$) has no valid positive counterpart since `i32::MAX` is $2147483647$. A standard `-x` operation on `i32::MIN` would ordinarily trigger a runtime panic in Rust when overflow checks are active, directly violating the `no panic paths` constitutional mandate.

The substrate evades this runtime panic path through explicit wrapping semantics:

1. **Panic-Free Overflow:** By using `.wrapping_sub()` or `.wrapping_neg()` directly (such as in the `epoch_based_reclamation_step.rs` and `fixed.rs` non-zero checks), the integer overflow implicitly wraps without invoking Rust's panic machinery.
2. **Bit-Level Wrap:** For `i32::MIN` (`1000...000` in binary), flipping the bits results in `0111...111` (`i32::MAX`). Adding `1` causes an arithmetic overflow that drops the carry and yields `1000...000`, silently reverting to `i32::MIN`.

This is explicitly acknowledged in the `bcinr` documentation (e.g., `ct_abs_i64` notes that `-i64::MIN` returns `i64::MIN` due to two's complement overflow). When true saturation is required rather than wrapping, `bcinr` utilizes separate fixed-point primitives (like `SignedFixed::saturating_sub`) which use canonical branchless selections (`is_neg.select_i32(i32::MIN, i32::MAX)`) to securely clamp the domain.
