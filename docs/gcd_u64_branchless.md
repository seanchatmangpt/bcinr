# Branchless GCD (`u64`) Implementation in `bcinr`

According to the `bcinr` constitution (`AGENTS.md`) and project mandate (`GEMINI.md`), mathematical algorithms must be strictly deterministic, branchless, allocation-free, and bounded. 

Traditional Greatest Common Divisor (GCD) uses the Euclidean or Stein's algorithm with a data-dependent `while a != 0` loop. In `bcinr`, this is forbidden by the "Radon Law ($CC=1$)" and Rule 13 ("No unbounded execution"). 

Here is how GCD for `u64` must be implemented to adhere to these constitutional requirements, based on the reference implementation found in `crates/bcinr-logic/src/algorithms/gcd_u64_branchless.rs`.

## 1. Fixed Iteration Bound
Instead of a `while` loop that terminates early when a variable hits `0`, the algorithm uses a fixed iteration loop: `for _ in 0..64`. 
- **Rule Satisfied**: Rule 13 ("No unbounded execution") dictates that all authoritative iteration must be compile-time fixed, macro-unrolled, or explicitly fixed in Rust and demonstrated as fully unrolled. 
- **Why 64?**: For a 64-bit binary GCD (Stein's algorithm) combined with a trailing-zeros shift (`tz = v_val.trailing_zeros()`), the maximum number of shift-and-subtract steps is bounded such that 64 iterations guarantee convergence for all `u64` inputs.

## 2. Mask-Based Execution (Rule 9)
All conditional state transitions (such as `if v == 0 { break }` or swapping `u` and `v` if `u > v`) must be rewritten as bitwise mask selections `select(m, a, b) = (m & a) | (!m & b)`.
- **Predicate to Mask**: A boolean condition is cast to an integer (`as u64`) and converted to a full-width mask (all 1s or all 0s) using `.wrapping_neg()`.
- **State Preservation**: When the virtual "loop" has mathematically finished (i.e., `v_val == 0`), the update mask `m_update = (v_val != 0) as u64; m_update = m_update.wrapping_neg();` becomes `0`. The bitwise selection ensures that state stops updating without needing an invalid `break` statement.

## 3. Branchless Swapping & Arithmetic
The inner loop avoids `if u > v { swap(u, v) }` by utilizing absolute differences and masked selection:
```rust
// Mask for if u > v
let cond = (u_val > v_val) as u64;
let cond_mask = cond.wrapping_neg();

// Branchless min(u_val, v_val)
let next_u = (v_val & cond_mask) | (u_val & !cond_mask);

// Absolute difference
let diff = (u_val as i128 - v_val as i128).unsigned_abs() as u64;
```
This guarantees constant-time operations for the core arithmetic logic, preserving $CC=1$ and bounded execution work (Rule 14 & Rule 4).

## 4. Zero Edge Cases
Handling `0` as an input cannot use an early `return` branch. Instead, a `zero_mask` is derived up front. The variables are modified with `| zero_mask` so that mathematical operations (like `.trailing_zeros()`) do not trigger panics or unpredictable behavior on `0`. At the very end of the algorithm, the answer is selected between a predefined fallback and the computed answer using the `zero_mask`.

```rust
let is_u_zero = (u == 0) as u64;
let is_v_zero = (v == 0) as u64;
let zero_mask = is_u_zero | is_v_zero;
let zero_mask_full = zero_mask.wrapping_neg();
let fallback = u | v; // If one is 0, result is the other. If both are 0, result is 0.

// Avoid zero values entering the core loop
let u_safe = u | zero_mask;
let v_safe = v | zero_mask;

// Execute exactly the same arithmetic for all inputs ...
// ...
let ans = u_val << shift;

// Branchless return
(fallback & zero_mask_full) | (ans & !zero_mask_full)
```

## Summary
The constant-time GCD leverages a binary GCD approach executing for a fixed `N` iterations. It replaces early-exit and conditional swapping with bitwise operations, full-width masking (`.wrapping_neg()`), and multiplexing to satisfy the mandate for branchless, bounded, arithmetic-only execution.
