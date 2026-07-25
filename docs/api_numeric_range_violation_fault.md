Here is the requested documentation on how `NumericFaultSet::RANGE_VIOLATION` is set and what mathematical condition it represents:

# Documentation for `NumericFaultSet::RANGE_VIOLATION`

In `crates/bcinr-cmca/src/fixed.rs`, the `NumericFaultSet::RANGE_VIOLATION` bitmask is used to signal that an operation's result exceeds the representable precision or bounds of the Q16.16 fixed-point type, specifically during the base-2 exponentiation `SignedFixed::exp2()`.

## Mathematical Condition

For a given signed Q16.16 fixed-point number, the integer part `ip` (derived from shifting the internal 32-bit value right by 16) must stay within valid bounds for `2^x` to produce a representable result in `NonNegativeFixed`:
- **Overflow:** Occurs when $x \ge 16$ (i.e., `ip >= 16`). Since $2^{16} = 65536$, this exceeds the maximum representable `NonNegativeFixed` value (`65535.9999...`, backed by `u32::MAX`).
- **Underflow:** Occurs when $x \le -17$ (i.e., `ip <= -17`). The value $2^{-17}$ is smaller than the smallest non-zero positive value representable in Q16.16 ($2^{-16}$), meaning the result decays completely to `0`.

If either of these limits is crossed, it mathematically constitutes a `RANGE_VIOLATION`.

## Branchless Implementation Details

The bitmask is set entirely without control-flow branches, adhering to the mathematical constraints and `CC=1` laws of the deterministic substrate.

1. **Mask Derivation:**
   The `is_overflow` and `is_underflow` conditions are evaluated using bitwise manipulation of the sign bit (`>> 31`). The `^ 1` logical negation creates a 1 or 0 flag, which is then broadcast into an all-ones (`u32::MAX`) or all-zeros (`0`) `CanonicalMask` using `0u32.wrapping_sub(...)`.
   ```rust
   let is_overflow = CanonicalMask(0u32.wrapping_sub(((((ip.wrapping_sub(16)) >> 31) ^ 1) & 1) as u32));
   let is_underflow = CanonicalMask(0u32.wrapping_sub((((((-17i32).wrapping_sub(ip)) >> 31) ^ 1) & 1) as u32));
   ```

2. **Branchless Fault Selection:**
   The two masks are combined bitwise (`is_overflow.raw() | is_underflow.raw()`). A branchless equality check (`const_eq_u32(..., 0)`) outputs a `CanonicalMask` that is `TRUE` (all-ones) if both conditions are `0`, and `FALSE` otherwise.
   ```rust
   let e = CanonicalMask::select_faults(
       const_eq_u32(is_overflow.raw() | is_underflow.raw(), 0),
       NumericFaultSet::EMPTY,
       NumericFaultSet::RANGE_VIOLATION,
   );
   ```
   `CanonicalMask::select_faults` is a bitwise multiplexer (`(a & m) | (b & !m)`) that selects `EMPTY` when within bounds, and `RANGE_VIOLATION` if out of bounds.

3. **Fault Accumulation:**
   Finally, the selected fault bitmask `e` is unified with any pre-existing faults from the operand via the semilattice operator `union` (a direct bitwise `OR`):
   ```rust
   faults: self.faults.union(e),
   ```
