# Branchless Clamp Mechanics in BCINR

In `bcinr`, the deterministic substrate constitution mandates that all operations must be structurally deterministic with $CC=1$ (cyclomatic complexity of 1), and absolutely no data-dependent branching (`if/else` logic) is permitted. 

To bound values in constant time without conditional statements like `if x < min` or `if x > max`, the `clamp` operation relies entirely on bitwise math and integer arithmetic (bitwise polynomials) to derive the masks and outcomes. 

Based on the `ct.rs` module in `bcinr-logic`, here is how the clamp operations are derived branchlessly:

## 1. Constant-Time Less-Than (`ct_lt`)
Before we can calculate `min` or `max`, we need a branchless way to evaluate the `<` predicate. Instead of a CPU comparison that sets status flags and jumps, `bcinr` uses the Hacker's Delight borrow propagation trick to isolate the borrow bit during subtraction.

For 32-bit unsigned integers, this looks like:
```rust
pub fn ct_lt_u32(a: u32, b: u32) -> u32 {
    // Isolates the borrow bit of `a - b` without a comparison opcode
    ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1
}
```

For signed integers (like `i64`), it examines the sign bits:
```rust
pub fn ct_lt_i64(a: i64, b: i64) -> u64 {
    let ua = a as u64;
    let ub = b as u64;
    let sign_a = ua >> 63;
    let sign_b = ub >> 63;
    let sign_diff = ua.wrapping_sub(ub) >> 63;
    let signs_differ = sign_a ^ sign_b;
    // If signs differ, negative is smaller. If same, check sign of difference.
    ((signs_differ & sign_a) | ((!signs_differ) & sign_diff)) & 1
}
```

## 2. Deriving the Mask
Using the constant-time `lt` function, the result (which is strictly `0` or `1`) is extended into a full-width bitmask using wrapping subtraction:
```rust
// 0 - 1 = 0xFFFFFFFF (all ones)
// 0 - 0 = 0x00000000 (all zeros)
let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
```

## 3. Constant-Time `Max` and `Min`
With the mask generated, `min` and `max` are derived purely through masking and wrapping arithmetic:

**Maximum Selection:**
```rust
pub fn ct_max_u32(a: u32, b: u32) -> u32 {
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    // If a < b (mask = 0xFFFFFFFF): a + (b - a) = b
    // If a >= b (mask = 0x00000000): a + 0 = a
    a.wrapping_add(b.wrapping_sub(a) & mask)
}
```

**Minimum Selection:**
```rust
pub fn ct_min_u32(a: u32, b: u32) -> u32 {
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    // If a < b (mask = 0xFFFFFFFF): b + (a - b) = a
    // If a >= b (mask = 0x00000000): b + 0 = b
    b.wrapping_add(a.wrapping_sub(b) & mask)
}
```

## 4. Constant-Time Clamp Composition
Finally, clamping a value `val` to `[lo, hi]` is merely the composition of the constant-time `min` and `max` operations. It guarantees that execution strictly flows linearly, producing identical performance characteristics regardless of whether the value is below, within, or above the bounds.

```rust
pub fn ct_clamp_u32(val: u32, lo: u32, hi: u32) -> u32 {
    ct_min_u32(ct_max_u32(val, lo), hi)
}
```

*Note on compiler intrinsics*: Higher-level operations in the crate (e.g., `clamp_i64` and `clamp_slice_branchless`) utilize Rust's native `.min()` and `.max()` methods. Rust safely compiles these into architecture-specific unconditional selection instructions like `cmov` (Conditional Move), which satisfies the zero branch condition at the object-code level without requiring explicit bit-fiddling. However, the underlying mathematical proofs verifying the strict bitwise polynomial logic depend on the explicit arithmetic definitions shown above.
