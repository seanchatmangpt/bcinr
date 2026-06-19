# Tutorial 6: Fixed-Point Math with the `fix` Module

Floating point is non-deterministic across platforms and forbidden in many
`no_std` and safety-critical contexts. The `fix` module gives you integer-based
fixed-point primitives with constant-time behavior. In this tutorial you build a
brightness control that clamps and quantizes a raw sensor value into a fixed
range — branchlessly.

## What you'll build

A "knob normalizer" that takes a raw `u32` reading, clamps it into a valid range
with `fix::clamp_u32`, snaps it to a discrete step with `fix::bucketize_u32`, and
keeps a running total with the overflow-safe `fix::add_sat` from
[Tutorial 3](./tutorial-3.md).

**Prerequisites:** [Tutorial 1](./tutorial-1.md) and
[Tutorial 3](./tutorial-3.md). You should recognize the saturating-add pattern.

## Step 1: Understand branchless clamp

From `crates/bcinr-logic/src/fix.rs`:

```rust
pub fn clamp_u32(val: u32, min: u32, max: u32) -> u32 {
    let mut res = val;
    let lt_min = (res < min) as u32;
    res = (min & 0u32.wrapping_sub(lt_min)) | (res & !0u32.wrapping_sub(lt_min));
    let gt_max = (res > max) as u32;
    res = (max & 0u32.wrapping_sub(gt_max)) | (res & !0u32.wrapping_sub(gt_max));
    res
}
```

It is two `select`s back to back: first replace `res` with `min` when it is too
small, then replace it with `max` when it is too large. Each replacement uses the
"bool -> all-ones mask" trick (`0u32.wrapping_sub(cond)`). No branch, and the
clamp window is honored for every input.

> **Contract:** `clamp_u32` assumes `min <= max`. Order your endpoints before
> calling, or use `algorithms::clamp_i64` (see
> [Tutorial 8](./tutorial-8.md)'s neighbors), which sorts them for you.

## Step 2: Understand branchless bucketize

`bucketize_u32` snaps a value down to the nearest multiple of `step`:

```rust
pub fn bucketize_u32(val: u32, step: u32) -> u32 {
    val.wrapping_div(step.wrapping_add((step == 0) as u32))
        .wrapping_mul(step)
}
```

The `(step == 0) as u32` term quietly bumps a zero `step` to `1`, so a
zero-divisor can never panic — branchless defense against bad input. For a normal
`step`, this is integer floor-division times `step`, i.e. quantization.

## Step 3: Build the knob normalizer

```rust
use bcinr_logic::fix::{add_sat, bucketize_u32, clamp_u32};

const MIN_LEVEL: u32 = 10;
const MAX_LEVEL: u32 = 250;
const STEP: u32 = 16; // quantize brightness into 16-unit notches

/// Clamp a raw reading into range, then snap it to a discrete notch.
fn normalize(raw: u32) -> u32 {
    let clamped = clamp_u32(raw, MIN_LEVEL, MAX_LEVEL);
    bucketize_u32(clamped, STEP)
}

fn main() {
    for raw in [3u32, 100, 255, 200] {
        println!("{:>3} -> {}", raw, normalize(raw));
    }
}
```

## Step 4: Run it

```bash
cargo run
```

Expected output:

```
  3 -> 0
100 -> 96
255 -> 240
200 -> 192
```

Walk through `raw = 3`: it clamps up to `10`, then `bucketize_u32(10, 16)` floors
to `0`. And `raw = 255` clamps down to `250`, which snaps to `240` (the largest
multiple of `16` not exceeding `250`).

## Step 5: Accumulate without overflow

Sum the normalized notches with the saturating adder so a long session never
wraps:

```rust
use bcinr_logic::fix::add_sat;

fn main() {
    let readings = [3u32, 100, 255, 200, 200];
    let mut total = 0u32;
    for r in readings {
        total = add_sat(total, normalize(r));
    }
    println!("total exposure: {}", total);
}
```

(`normalize` is from Step 3.)

```bash
cargo run
```

Expected output:

```
total exposure: 528
```

That is `0 + 96 + 240 + 192 + 192`. If the sum had ever exceeded `u32::MAX`,
`add_sat` would have pinned it instead of wrapping.

## Step 6: Lock it in

```rust
use bcinr_logic::fix::{bucketize_u32, clamp_u32};

#[test]
fn clamp_then_bucketize() {
    // clamp honors both ends
    assert_eq!(clamp_u32(3, 10, 250), 10);
    assert_eq!(clamp_u32(999, 10, 250), 250);
    assert_eq!(clamp_u32(100, 10, 250), 100);

    // bucketize floors to a multiple of step, and tolerates step == 0
    assert_eq!(bucketize_u32(100, 16), 96);
    assert_eq!(bucketize_u32(100, 0), 100); // no panic: step folded to 1
}
```

```bash
cargo test clamp_then_bucketize
```

```
test clamp_then_bucketize ... ok
```

## What you learned

- `fix::clamp_u32` is two branchless `select`s (clamp low, then clamp high) and
  assumes `min <= max`.
- `fix::bucketize_u32` quantizes by floor-division, and folds a zero `step` to
  `1` so it can never divide by zero.
- Combining clamp, bucketize, and `add_sat` gives a fully deterministic
  fixed-point pipeline with no floats and no panics.

## Next steps

- [Tutorial 7: Branchless scans over byte slices](./tutorial-7.md) — apply the
  same constant-time mindset to whole buffers.
- [Tutorial 8: A branchless AABB overlap test](./tutorial-8.md) — pack several
  fixed-point fields into one word and compare them at once.
