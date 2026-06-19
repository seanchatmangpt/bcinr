# Tutorial 2: Branchless min, max, and abs

In [Tutorial 1](./tutorial-1.md) you used `select_u32` to choose between two
values. Now you will see how that single primitive composes into the three
comparison kernels you reach for constantly: `min_u32`, `max_u32`, and `abs_i32`.

## What you'll build

A small "magnitude normalizer" that, given a signed reading and a ceiling,
reports the magnitude of the reading capped at the ceiling — all branchlessly.
Along the way you will read the real implementations and confirm they are just
`select` plus a mask.

**Prerequisites:** [Tutorial 1](./tutorial-1.md). You should be comfortable with
the all-ones / all-zeros mask convention.

## Step 1: See min and max as one selector

From `crates/bcinr-logic/src/mask.rs`:

```rust
pub fn min_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, a, b)  // a < b ? a : b
}

pub fn max_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, b, a)  // a < b ? b : a
}
```

They share the *same* mask. `min` keeps `a` when `a < b`; `max` keeps `b` when
`a < b`. Swapping the two `select` arguments is the only difference. No branch in
either.

## Step 2: See abs as a sign-mask trick

`abs_i32` does not even call `select` — it builds a mask from the sign bit:

```rust
pub fn abs_i32(x: i32) -> i32 {
    let mask = x >> 31;            // arithmetic shift: 0 if x>=0, all-ones if x<0
    (x ^ mask).wrapping_sub(mask)  // negate-if-negative, identity otherwise
}
```

For non-negative `x`, `mask == 0`: `x ^ 0` is `x`, and subtracting `0` leaves it
unchanged. For negative `x`, `mask == -1` (all ones): `x ^ -1` is `!x`, and
`wrapping_sub(-1)` adds 1 — which is exactly two's-complement negation. One
branchless expression covers both signs.

## Step 3: Write the normalizer

```rust
use bcinr_logic::mask::{abs_i32, min_u32};

/// Magnitude of `reading`, capped at `ceiling`.
fn capped_magnitude(reading: i32, ceiling: u32) -> u32 {
    let magnitude = abs_i32(reading) as u32;
    min_u32(magnitude, ceiling)
}

fn main() {
    println!("{}", capped_magnitude(-37, 100)); // |-37| = 37, under cap
    println!("{}", capped_magnitude(250, 100)); //  250  -> capped to 100
    println!("{}", capped_magnitude(0, 100));   //   0
}
```

## Step 4: Run it

```bash
cargo run
```

Expected output:

```
37
100
0
```

## Step 5: Watch the boundary

`abs_i32` has one famous trap: `i32::MIN` has no positive counterpart in two's
complement, so the branchless trick wraps (exactly like `i32::wrapping_abs`).
Test the *safe* boundary so you know precisely where the edge is:

```rust
use bcinr_logic::mask::abs_i32;

#[test]
fn abs_boundaries() {
    assert_eq!(abs_i32(5), 5);
    assert_eq!(abs_i32(-5), 5);
    assert_eq!(abs_i32(0), 0);
    assert_eq!(abs_i32(i32::MIN + 1), i32::MAX); // largest representable magnitude
}
```

```bash
cargo test abs_boundaries
```

```
test abs_boundaries ... ok
```

If your domain can produce `i32::MIN`, widen to `i64` *before* taking the
magnitude. Branchless kernels never paper over overflow; they expose it so you
choose the policy.

## Step 6: Confirm min/max agree with the standard library

A free correctness check: the branchless results must match `core::cmp`.

```rust
use bcinr_logic::mask::{max_u32, min_u32};

#[test]
fn agrees_with_core_cmp() {
    for (a, b) in [(5u32, 3), (3, 5), (7, 7), (0, u32::MAX)] {
        assert_eq!(min_u32(a, b), a.min(b));
        assert_eq!(max_u32(a, b), a.max(b));
    }
}
```

```
test agrees_with_core_cmp ... ok
```

## What you learned

- `min_u32` and `max_u32` are the *same* `select_u32` with its arguments
  swapped, driven by one `lt_mask_u32`.
- `abs_i32` builds a mask from the arithmetic-shifted sign bit — negate-if-set
  with no branch.
- Branchless arithmetic surfaces overflow (`i32::MIN`) instead of hiding it; you
  pick the policy.

## Next steps

- [Tutorial 3: Saturating arithmetic for safe counters](./tutorial-3.md) — make
  arithmetic that *cannot* overflow, ideal for metrics and rate limiters.
- [Tutorial 8: A branchless AABB overlap test](./tutorial-8.md) — see min/max
  thinking applied to 2-D geometry.
