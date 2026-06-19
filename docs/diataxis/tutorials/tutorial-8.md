# Tutorial 8: A Branchless AABB Overlap Test

Collision detection runs in the hottest loops of games and physics engines, and
the classic axis-aligned bounding box (AABB) overlap test is full of `&&`
short-circuits — each one a branch. The `algorithms::aabb_intersect_branchless`
kernel packs two boxes into two `u64`s and answers "do they overlap?" with pure
arithmetic. In this tutorial you drive it end to end.

## What you'll build

A 2-D overlap checker: you will pack a box's four 16-bit coordinates into a single
`u64`, call `aabb_intersect_branchless`, and interpret the result. You will also
compare against the slow-but-obvious branchful version to build trust.

**Prerequisites:** [Tutorial 1](./tutorial-1.md) and
[Tutorial 2](./tutorial-2.md) (min/max thinking). Familiarity with bit shifting
from [Tutorial 4](./tutorial-4.md) helps with the packing.

## Step 1: Understand the packed layout

The kernel (`crates/bcinr-logic/src/algorithms/aabb_intersect_branchless.rs`)
treats each `u64` as four 16-bit fields:

```text
bits  0..16  -> x_min
bits 16..32  -> x_max
bits 32..48  -> y_min
bits 48..64  -> y_max
```

The implementation unpacks both boxes and ANDs four comparisons:

```rust
pub fn aabb_intersect_branchless(val: u64, aux: u64) -> u64 {
    let x1_min = val & 0xFFFF;
    let x1_max = (val >> 16) & 0xFFFF;
    let y1_min = (val >> 32) & 0xFFFF;
    let y1_max = (val >> 48) & 0xFFFF;
    let x2_min = aux & 0xFFFF;
    let x2_max = (aux >> 16) & 0xFFFF;
    let y2_min = (aux >> 32) & 0xFFFF;
    let y2_max = (aux >> 48) & 0xFFFF;
    ((x1_min <= x2_max) & (x2_min <= x1_max)
        & (y1_min <= y2_max) & (y2_min <= y1_max)) as u64
}
```

The crucial detail: those are **bitwise `&`**, not logical `&&`. Bitwise AND does
not short-circuit, so all four comparisons are always evaluated — the runtime is
identical whether the boxes overlap or not. The result is `1` for overlap, `0`
otherwise.

## Step 2: Write a packing helper

```rust
/// Pack an AABB given as (x_min, x_max, y_min, y_max) into one u64.
fn pack_aabb(x_min: u16, x_max: u16, y_min: u16, y_max: u16) -> u64 {
    (x_min as u64)
        | ((x_max as u64) << 16)
        | ((y_min as u64) << 32)
        | ((y_max as u64) << 48)
}
```

## Step 3: Test two overlapping boxes

```rust
use bcinr_logic::algorithms::aabb_intersect_branchless::aabb_intersect_branchless;

fn main() {
    // box A: x in [0,10], y in [0,10]
    let a = pack_aabb(0, 10, 0, 10);
    // box B: x in [5,15], y in [5,15]  (overlaps A in the corner)
    let b = pack_aabb(5, 15, 5, 15);

    let overlap = aabb_intersect_branchless(a, b);
    println!("A vs B overlap: {}", overlap);
}
```

(`pack_aabb` is from Step 2.)

## Step 4: Run it

```bash
cargo run
```

Expected output:

```
A vs B overlap: 1
```

## Step 5: Test disjoint and touching boxes

```rust
use bcinr_logic::algorithms::aabb_intersect_branchless::aabb_intersect_branchless;

fn main() {
    let a = pack_aabb(0, 10, 0, 10);

    let far = pack_aabb(20, 30, 20, 30);    // clearly disjoint
    let touch = pack_aabb(10, 20, 0, 10);   // shares the edge x = 10

    println!("disjoint: {}", aabb_intersect_branchless(a, far));
    println!("touching: {}", aabb_intersect_branchless(a, touch));
}
```

```bash
cargo run
```

Expected output:

```
disjoint: 0
touching: 1
```

The comparison uses `<=`, so boxes that share only an edge count as overlapping.
That is a *policy* baked into the kernel — if you need strict overlap, subtract
one from the touching boundary before packing.

## Step 6: Trust it against a branchful reference

The repository ships its own reference oracle for exactly this purpose. Mirror it
in a quick test so you can see the equivalence yourself:

```rust
use bcinr_logic::algorithms::aabb_intersect_branchless::aabb_intersect_branchless;

/// The obvious, branchful version (short-circuiting &&).
fn aabb_reference(val: u64, aux: u64) -> u64 {
    let (ax0, ax1) = (val & 0xFFFF, (val >> 16) & 0xFFFF);
    let (ay0, ay1) = ((val >> 32) & 0xFFFF, (val >> 48) & 0xFFFF);
    let (bx0, bx1) = (aux & 0xFFFF, (aux >> 16) & 0xFFFF);
    let (by0, by1) = ((aux >> 32) & 0xFFFF, (aux >> 48) & 0xFFFF);
    if ax0 <= bx1 && bx0 <= ax1 && ay0 <= by1 && by0 <= ay1 {
        1
    } else {
        0
    }
}

#[test]
fn branchless_matches_branchful() {
    let cases = [
        (pack_aabb(0, 10, 0, 10), pack_aabb(5, 15, 5, 15)),  // overlap
        (pack_aabb(0, 10, 0, 10), pack_aabb(20, 30, 20, 30)), // disjoint
        (pack_aabb(0, 10, 0, 10), pack_aabb(10, 20, 0, 10)),  // touching
    ];
    for (a, b) in cases {
        assert_eq!(aabb_intersect_branchless(a, b), aabb_reference(a, b));
    }
}
```

```bash
cargo test branchless_matches_branchful
```

```
test branchless_matches_branchful ... ok
```

## What you learned

- An AABB packs into one `u64` as four 16-bit fields (`x_min`, `x_max`, `y_min`,
  `y_max`).
- `aabb_intersect_branchless` ANDs four `<=` comparisons with **bitwise** `&`, so
  no `&&` short-circuits and the latency is data-independent.
- Edge-touching boxes count as overlapping (`<=`); change the policy by adjusting
  the packed coordinates.

## Next steps

- [Tutorial 9: Property-testing a branchless kernel](./tutorial-9.md) — replace
  the three hand-picked cases above with an exhaustive `proptest`.
- [Tutorial 10: Benchmarking a kernel with Criterion](./tutorial-10.md) — measure
  just how flat this kernel's latency really is.
