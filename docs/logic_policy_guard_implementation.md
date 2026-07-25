I have located `PolicyGuard` in `/Users/sac/bcinr/crates/bcinr-logic/src/autonomic/policy_guard.rs` and analyzed its implementation. 

Here is the requested markdown documenting the mathematical logic and bitwise masks it uses to filter actions branchlessly:

# PolicyGuard Bitwise Logic Analysis

`PolicyGuard` is a struct located at `crates/bcinr-logic/src/autonomic/policy_guard.rs` that provides branchless safety checks for autonomic systems. It filters actions by generating full-width bitwise masks (`0xFFFFFFFFFFFFFFFF` for accept or `0x0000000000000000` for reject) instead of using typical `if/else` control flow branches. This strictly adheres to the project's $CC=1$ rule.

## Core Mathematical Logic

The guard functions use a two's-complement arithmetic trick on `u64` primitives to securely map boolean conditions into an all-1s or all-0s bitmask. 

For any condition check:
1. It evaluates a comparison (`>`, `<`, or `==`), yielding a boolean.
2. It casts the boolean to a `u64`, resulting in exactly `1` (true) or `0` (false).
3. It computes the result by subtracting this value from `0` using `wrapping_sub`.

**Mathematical behavior of wrapping subtraction:**
- If condition is `false` (evaluates to 0): `0u64 - 0u64 = 0u64` (Bitmask: `0x0000000000000000`)
- If condition is `true` (evaluates to 1): `0u64 - 1u64 = -1`. Since this operates on unsigned integers, it underflows to `u64::MAX` (Bitmask: `0xFFFFFFFFFFFFFFFF`, logically `!0`).

## Implemented Primitives

### 1. `mask_gt` (Greater Than)
```rust
pub fn mask_gt(val: u64, threshold: u64) -> u64 {
    let check = (val > threshold) as u64;
    0u64.wrapping_sub(check)
}
```

### 2. `mask_lt` (Less Than)
```rust
pub fn mask_lt(val: u64, threshold: u64) -> u64 {
    let check = (val < threshold) as u64;
    0u64.wrapping_sub(check)
}
```

### 3. `mask_eq` (Equal)
```rust
pub fn mask_eq(val: u64, threshold: u64) -> u64 {
    let check = (val == threshold) as u64;
    0u64.wrapping_sub(check)
}
```

## How It Filters Actions

Downstream components use `PolicyGuard` to enforce safety boundaries without branching. By applying a bitwise `AND` (`&`) between an action state and this mask, the system can accept or drop operations. Valid actions are retained (`action & 0xFFFFFFFFFFFFFFFF = action`), while invalid actions are completely zeroed out (`action & 0x0000000000000000 = 0`), ensuring deterministic and branchless filtering behavior.
