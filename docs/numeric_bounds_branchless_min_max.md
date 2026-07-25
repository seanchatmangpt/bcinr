# Branchless Implementation of `min`, `max`, and `clamp`

Under Rule 14 (Numeric-law requirements) of the BCINR Deterministic Substrate Constitution, authoritative arithmetic must be fixed-width, deterministic, and free of architecture-dependent behavior, ensuring absolute branchlessness (Radon Law $CC=1$). Control-flow branches are strictly prohibited, and sequential semantic decisions must be transformed into full-width masks (B-Calculus) or arithmetic selections.

Here is how `min`, `max`, and `clamp` are implemented branchlessly in the codebase using bitwise masks and mathematical logic:

## 1. Constant-Time Less-Than (`ct_lt_u32`)

To build `min` and `max` without comparisons that compile to branches, the substrate uses a branchless bit manipulation trick (derived from Hacker's Delight) to isolate the borrow bit during unsigned subtraction.

```rust
// From crates/bcinr-logic/src/ct.rs
pub fn ct_lt_u32(a: u32, b: u32) -> u32 {
    // Technique: for unsigned a < b, the borrow propagation trick.
    // (a ^ ((a ^ b) | ((a.wrapping_sub(b)) ^ b))) >> 31 isolates the borrow bit.
    ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1
}
```

## 2. Arithmetic Masking (`ct_min_u32` and `ct_max_u32`)

Using `ct_lt_u32`, the operations derive a full-width mask where all bits are `1` (i.e., `0xFFFFFFFF`) if the condition is met, and `0` otherwise. They then use bitwise arithmetic to select the target value without using `if`.

### `ct_min_u32`
```rust
// From crates/bcinr-logic/src/ct.rs
pub fn ct_min_u32(a: u32, b: u32) -> u32 {
    // If a < b: mask = all-ones, selects a; else mask = 0, b + 0 = b.
    // b + ((a - b) & mask): when a < b, a - b is the negative delta; b + (a-b) = a.
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    b.wrapping_add(a.wrapping_sub(b) & mask)
}
```

### `ct_max_u32`
```rust
// From crates/bcinr-logic/src/ct.rs
pub fn ct_max_u32(a: u32, b: u32) -> u32 {
    // max(a, b) = a + ((b - a) & mask) where mask is all-ones when a < b.
    // When a < b: ct_lt_u32(a,b) = 1 => mask = 0xFFFFFFFF => result = a + (b-a) = b.
    // When a >= b: ct_lt_u32(a,b) = 0 => mask = 0 => result = a + 0 = a.
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    a.wrapping_add(b.wrapping_sub(a) & mask)
}
```

## 3. Bitwise Selection (`min_element_branchless_u32`)

Alternatively, the substrate implements B-Calculus by casting booleans directly to `u32`, then using `0u32.wrapping_sub(...)` to expand `0` or `1` into a full mask. It then performs a pure bitwise `select(mask, a, b)` using `(a & mask) | (b & !mask)`:

```rust
// From crates/bcinr-logic/src/algorithms/min_element_branchless_u32.rs
pub fn min_element_branchless_u32(val: u64, aux: u64) -> u64 {
    let a = (val & 0xFFFFFFFF) as u32;
    let b = (val >> 32) as u32;
    // ...
    // Cast boolean to 1 or 0, subtract from 0 to get 0xFFFFFFFF or 0x00000000
    let m1 = 0u32.wrapping_sub((a < b) as u32);
    // Bitwise select using the mask and inverted mask
    let min1 = (a & m1) | (b & !m1);
    // ...
}
```

## 4. Branchless Clamping

With `min` and `max` guaranteed branchless, `clamp` simply composes them in constant time.

```rust
// From crates/bcinr-logic/src/ct.rs
pub fn ct_clamp_u32(val: u32, lo: u32, hi: u32) -> u32 {
    ct_min_u32(ct_max_u32(val, lo), hi)
}
```

By ensuring conditional state transitions happen completely through continuous mathematical operations and bitwise masking, these implementations remain rigidly deterministic, execute in bounded constant time, and adhere to the strict Rule 14 directives of the BCINR substrate.
