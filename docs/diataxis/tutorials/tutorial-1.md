# Tutorial 1: Your First Branchless Select

Welcome to bcinr. This is the very first rung on the ladder: by the end you will
have written and run a program that chooses between two values **without a single
`if`**. Everything else in the library — min/max, saturating counters, DFAs,
fixed-point clamps — is built on the primitive you learn here.

## What you'll build

A tiny program that picks the larger of two prices using `mask::select_u32`, the
branchless multiplexer at the heart of the `mask` calculus. You will see *why*
the result has no data-dependent branch, and you will prove to yourself that the
two halves of the selection are always evaluated.

**Prerequisites:** a working Rust toolchain (`rustc --version` ≥ 1.70) and a
checkout of the bcinr workspace. No prior branchless experience required.

## Step 1: Add the dependency

From a binary crate that lives next to the workspace, depend on the logic crate:

```toml
# Cargo.toml
[dependencies]
bcinr-logic = { path = "../bcinr/crates/bcinr-logic" }
```

`bcinr-logic` is `#![no_std]` and has **zero** runtime dependencies, so this adds
nothing to your binary except the kernels you actually call.

## Step 2: Understand the selector

The whole primitive is three lines (`crates/bcinr-logic/src/mask.rs`):

```rust
pub fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}
```

The contract is simple:

- If `mask` is **all ones** (`0xFFFF_FFFF`), every bit of `a` survives and every
  bit of `b` is zeroed, so the function returns `a`.
- If `mask` is **all zeros** (`0x0`), the opposite happens and it returns `b`.

There is no comparison and no jump. The CPU computes *both* `mask & a` and
`!mask & b` every time, then ORs them. That is the essence of branchless code:
the work is constant, only the data changes.

## Step 3: Make a mask from a condition

A "true/false" boolean is not yet a mask. To turn `a < b` into the all-ones /
all-zeros form `select_u32` expects, use `lt_mask_u32`:

```rust
pub fn lt_mask_u32(a: u32, b: u32) -> u32 {
    // (a < b) is 0 or 1; wrapping_sub turns it into 0x00000000 or 0xFFFFFFFF.
    0u32.wrapping_sub(u32::from(a < b))
}
```

`u32::from(a < b)` is `0` or `1`. Subtracting it from `0` wraps `1` around to
`0xFFFF_FFFF` and leaves `0` as `0`. The compiler emits a `SETB`/`NEG` pair on
x86-64 — still no branch.

## Step 4: Write the program

```rust
use bcinr_logic::mask::{lt_mask_u32, select_u32};

fn larger_price(a: u32, b: u32) -> u32 {
    // mask is all-ones exactly when a < b
    let a_is_smaller = lt_mask_u32(a, b);
    // ...so when a < b we want b, otherwise a
    select_u32(a_is_smaller, b, a)
}

fn main() {
    println!("{}", larger_price(199, 249)); // a < b  -> b
    println!("{}", larger_price(500, 120)); // a >= b -> a
    println!("{}", larger_price(42, 42));   // equal  -> a
}
```

## Step 5: Run it

```bash
cargo run
```

Expected output:

```
249
500
42
```

Notice the equal case (`42`, `42`): `lt_mask_u32(42, 42)` is `0` (not strictly
less), so `select_u32` returns the `a` branch. Branchless code forces you to be
explicit about ties — there is no "fall-through" to hide behind.

## Step 6: Prove there is no hidden branch

Add a quick assertion that mirrors the truth table directly:

```rust
use bcinr_logic::mask::select_u32;

#[test]
fn select_is_a_pure_multiplexer() {
    assert_eq!(select_u32(0xFFFF_FFFF, 10, 20), 10); // all ones -> a
    assert_eq!(select_u32(0x0000_0000, 10, 20), 20); // all zeros -> b
}
```

```bash
cargo test
```

```
test select_is_a_pure_multiplexer ... ok
```

Both `10` and `20` are present in the call regardless of the mask. The selection
is data *movement*, not control *flow*.

## What you learned

- `select_u32(mask, a, b)` is a branchless multiplexer: all-ones picks `a`,
  all-zeros picks `b`.
- A boolean becomes a usable mask via `lt_mask_u32` (or `eq_mask_u32`,
  `is_zero_mask_u32`, `nonzero_mask_u32`).
- Both inputs are always evaluated, which is exactly what gives the kernel its
  constant, predictable latency.

## Next steps

- [Tutorial 2: Branchless min, max, and abs](./tutorial-2.md) — compose the
  selector into the workhorse comparison kernels.
- [Tutorial 9: Property-testing a branchless kernel](./tutorial-9.md) — prove a
  selector matches a branchful reference for *all* inputs.
