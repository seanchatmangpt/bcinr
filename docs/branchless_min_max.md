# Branchless Minimum and Maximum in BCINR

In `bcinr`'s deterministic substrate, algorithms are forbidden from using data-dependent branches (`CC=1` rule). Therefore, conditional logic like `if a < b { a } else { b }` must be transformed into continuous bitwise operations. This is implemented in `bcinr-logic`'s `ct.rs` using two's complement arithmetic and the Hacker's Delight borrow bit trick.

## The Foundation: Constant-Time Less-Than (`ct_lt_u32`)

Before calculating `min` or `max`, the system needs a branchless way to evaluate `a < b`. This is implemented via bitwise logic that isolates the borrow bit during subtraction:

```rust
pub fn ct_lt_u32(a: u32, b: u32) -> u32 {
    // Isolates the borrow bit for unsigned subtraction
    ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1
}
```
This formula evaluates to `1` if `a < b`, and `0` otherwise.

## Generating the Selection Mask

Once the condition evaluates to `0` or `1`, it is expanded into a full-width bitwise mask using two's complement underflow:

```rust
let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
```
- **If `a < b`**: `ct_lt_u32(a, b)` returns `1`. `0 - 1` results in `0xFFFFFFFF` (all ones) due to two's complement wrapping.
- **If `a >= b`**: `ct_lt_u32(a, b)` returns `0`. `0 - 0` results in `0x00000000` (all zeros).

## Branchless `ct_min`

To calculate the minimum without branches, the substrate uses the following logic:

```rust
pub fn ct_min_u32(a: u32, b: u32) -> u32 {
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    b.wrapping_add(a.wrapping_sub(b) & mask)
}
```
- **When `a < b`**: `mask` is `0xFFFFFFFF`. The bitwise AND `(a - b) & mask` yields `a - b`. The expression evaluates to `b + (a - b) = a`.
- **When `a >= b`**: `mask` is `0x00000000`. The bitwise AND `(a - b) & mask` yields `0`. The expression evaluates to `b + 0 = b`.

## Branchless `ct_max`

Similarly, the maximum is computed by flipping the base and delta operands:

```rust
pub fn ct_max_u32(a: u32, b: u32) -> u32 {
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    a.wrapping_add(b.wrapping_sub(a) & mask)
}
```
- **When `a < b`**: `mask` is `0xFFFFFFFF`. The bitwise AND `(b - a) & mask` yields `b - a`. The expression evaluates to `a + (b - a) = b`.
- **When `a >= b`**: `mask` is `0x00000000`. The bitwise AND `(b - a) & mask` yields `0`. The expression evaluates to `a + 0 = a`.

## Compliance with Substrate Numeric Laws

By constructing `min` and `max` purely out of fixed-width bitwise operations (`^`, `|`, `&`, `>>`) and wrapping arithmetic (`wrapping_add`, `wrapping_sub`), the implementation successfully avoids hardware branch predictors. This satisfies the structural audit requirement of zero conditional jumps in object code (`@turing_machine` authority) and perfectly maps to the fixed-point algebraic selection laws required by Rule 14.
