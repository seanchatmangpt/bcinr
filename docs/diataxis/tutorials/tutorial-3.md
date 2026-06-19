# Tutorial 3: Saturating Arithmetic for Safe Counters

Counters overflow. A `u32` metric that wraps from `u32::MAX` back to `0` can turn
a dashboard green when the system is on fire. In this tutorial you build counters
that *clamp* at their limits instead of wrapping — using the branchless
saturating kernels in `int` and `fix`.

## What you'll build

A bounded event counter and a signed accumulator that both refuse to overflow.
You will use `fix::add_sat` (unsigned `u32`) and `int::saturating_add_i64` /
`int::saturating_sub_i64` (signed `i64`), and you will see exactly where each
one pins.

**Prerequisites:** [Tutorial 1](./tutorial-1.md) and
[Tutorial 2](./tutorial-2.md). You should recognize the
"turn a bool into a mask" pattern, because saturation is built on it.

## Step 1: Understand unsigned saturation

`fix::add_sat` (`crates/bcinr-logic/src/fix.rs`) adds two `u32`s and clamps to
`u32::MAX` on overflow:

```rust
pub fn add_sat(a: u32, b: u32) -> u32 {
    let res = a.wrapping_add(b);
    res | 0u32.wrapping_sub((res < a) as u32)
}
```

The trick: if the wrapping sum is *smaller* than `a`, the add overflowed.
`(res < a) as u32` is `1` in that case; `wrapping_sub` turns it into all-ones,
and ORing all-ones forces the result to `u32::MAX`. No overflow ever escapes, and
there is no branch.

## Step 2: Build a bounded counter

```rust
use bcinr_logic::fix::add_sat;

struct EventCounter {
    count: u32,
}

impl EventCounter {
    fn new() -> Self {
        Self { count: 0 }
    }

    /// Record `n` events; pins at u32::MAX instead of wrapping.
    fn record(&mut self, n: u32) {
        self.count = add_sat(self.count, n);
    }
}

fn main() {
    let mut c = EventCounter::new();
    c.record(3);
    c.record(4);
    println!("after normal adds: {}", c.count);

    c.record(u32::MAX);              // would overflow
    println!("after overflow:   {}", c.count);
}
```

## Step 3: Run it

```bash
cargo run
```

Expected output:

```
after normal adds: 7
after overflow:   4294967295
```

The counter pins at `u32::MAX` (`4294967295`) and stays there. A wrapping `+`
would have reported a small, misleading number.

## Step 4: Add a signed accumulator

For deltas that can go up or down, use the signed `i64` kernels from
`crates/bcinr-logic/src/int.rs`:

```rust
pub const fn saturating_add_i64(a: i64, b: i64) -> i64 { a.saturating_add(b) }
pub const fn saturating_sub_i64(a: i64, b: i64) -> i64 { a.saturating_sub(b) }
```

These pin at `i64::MAX` on positive overflow and `i64::MIN` on negative overflow:

```rust
use bcinr_logic::int::{saturating_add_i64, saturating_sub_i64};

struct Balance {
    value: i64,
}

impl Balance {
    fn credit(&mut self, amount: i64) {
        self.value = saturating_add_i64(self.value, amount);
    }
    fn debit(&mut self, amount: i64) {
        self.value = saturating_sub_i64(self.value, amount);
    }
}

fn main() {
    let mut b = Balance { value: i64::MAX - 1 };
    b.credit(100);                  // would overflow positive
    println!("pinned high: {}", b.value);

    b.value = i64::MIN + 1;
    b.debit(100);                   // would overflow negative
    println!("pinned low:  {}", b.value);
}
```

## Step 5: Run the signed example

```bash
cargo run
```

Expected output:

```
pinned high: 9223372036854775807
pinned low:  -9223372036854775808
```

Those are `i64::MAX` and `i64::MIN`. The accumulator clamps at both ends.

## Step 6: Lock the behavior in with a test

```rust
use bcinr_logic::fix::add_sat;
use bcinr_logic::int::{saturating_add_i64, saturating_sub_i64};

#[test]
fn saturation_pins_at_the_limits() {
    // unsigned: clamps at u32::MAX
    assert_eq!(add_sat(u32::MAX, 1), u32::MAX);
    assert_eq!(add_sat(200, 100), 300); // no overflow -> exact

    // signed: clamps at both extremes
    assert_eq!(saturating_add_i64(i64::MAX, 1), i64::MAX);
    assert_eq!(saturating_sub_i64(i64::MIN, 1), i64::MIN);
}
```

```bash
cargo test saturation_pins_at_the_limits
```

```
test saturation_pins_at_the_limits ... ok
```

## What you learned

- `fix::add_sat` clamps unsigned `u32` addition at `u32::MAX` using an
  overflow-detect mask (`res < a`) — no branch.
- `int::saturating_add_i64` / `saturating_sub_i64` clamp signed `i64` at both
  `i64::MAX` and `i64::MIN`.
- Saturating counters keep metrics honest: a maxed-out counter looks maxed out,
  not freshly reset.

## Next steps

- [Tutorial 4: Packing state into bitsets and counting bits](./tutorial-4.md) —
  store many flags in one word and query them branchlessly.
- [Tutorial 6: Fixed-point math with the `fix` module](./tutorial-6.md) — clamp
  and bucketize values into fixed ranges.
