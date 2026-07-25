# Branchless Minimum/Maximum Evaluation in `bcinr`

In the `bcinr` codebase, evaluating minimums and maximums must strictly adhere to the project's **Radon Law ($CC=1$)** and **Mask-based execution law**, which prohibit data-dependent conditional branches (`if`, `match`, or implicit jumps like `cmp`/`jle`). 

To achieve this, the hot path evaluates both operands unconditionally and uses bitwise polynomials to select the correct result. This guarantees constant-time execution and prevents timing side-channels.

The evaluation process is broken into two distinct steps:
1. **Generating a full-width canonical mask** (`0xFFFF_FFFF` for true, `0x0000_0000` for false) representing the `<` relationship.
2. **Applying a bitwise selection multiplexer** to route the correct value.

## 1. Generating the Less-Than Mask

Instead of 1-bit booleans, the runtime operates on 32-bit (or 64-bit) masks. `bcinr` implements this in two different crates using two strategies:

### Method A: Pure Bit-Parallel Comparison (`bcinr-cmca/src/fixed.rs`)
In the highly strict `cmca` authoritative runtime crate, `<` is evaluated using pure bit-parallel logic without ever invoking the native `<` operator. This ensures the AST itself is structurally branchless before any LLVM optimization:

```rust
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    // Calculates the sign bit of the difference securely
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    
    // Broadcasts the 1-bit difference into a 32-bit mask via wrapping subtraction
    CanonicalMask(0u32.wrapping_sub(diff))
}
```

### Method B: Compiler Intrinsic Mapping (`bcinr-logic/src/mask.rs`)
In the logic crate, the mask relies on compiler lowering optimizations. On x86-64, this sequence safely compiles down to branchless `SETB` + `NEG` instructions rather than conditional jumps:

```rust
pub const fn lt_mask_u32(a: u32, b: u32) -> u32 {
    // (a < b) as u32 produces 0 or 1.
    // 0u32.wrapping_sub(1) -> 0xFFFF_FFFF
    // 0u32.wrapping_sub(0) -> 0x0000_0000
    0u32.wrapping_sub((a < b) as u32)
}
```

## 2. The Bitwise Select Operation (`select_u32`)

Once a full-width mask is generated, `bcinr` uses a `select_u32` function (or `CanonicalMask::select_u32`) to pick between `a` and `b`. The bitwise arithmetic `(mask & a) | (!mask & b)` acts as a branchless multiplexer:

```rust
pub const fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}
```

- If `mask == 0xFFFF_FFFF` (True), `!mask == 0x0000_0000`. The result evaluates to `(0xFFFF_FFFF & a) | (0x0000_0000 & b)` = `a | 0` = `a`.
- If `mask == 0x0000_0000` (False), `!mask == 0xFFFF_FFFF`. The result evaluates to `(0x0000_0000 & a) | (0xFFFF_FFFF & b)` = `0 | b` = `b`.

## 3. Composing Branchless Min and Max

By combining the boolean mask generation and the selection operation, `min` and `max` are flawlessly evaluated without control flow, fulfilling the Hoare contract:

```rust
/// Branchless minimum: returns the lesser of `a` and `b`
pub const fn min_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, a, b)
}

/// Branchless maximum: returns the greater of `a` and `b`
pub const fn max_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    // Notice the reversed order of a and b in the select function
    select_u32(mask, b, a)
}
```

### Mathematical Summary
The entire hot path operation maps cleanly to the algebraic specification demanded by `@hoare_oracle` and `@von_neumann_bypass`:
$$ \operatorname{mask} = \operatorname{lt\_mask}(a, b) $$
$$ \min(a, b) = (\operatorname{mask} \land a) \lor (\neg \operatorname{mask} \land b) $$
$$ \max(a, b) = (\operatorname{mask} \land b) \lor (\neg \operatorname{mask} \land a) $$

This architectural implementation ensures that sequential semantic decisions are successfully transformed into arithmetic selection, completely devoid of dynamic jumps.
