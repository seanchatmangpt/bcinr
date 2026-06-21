# Tutorial 4: Packing State into Bitsets and Counting Bits

A `u64` is 64 boolean flags hiding in a single register. When you store state as
bits instead of an array of `bool`, set/clear/test become one instruction each,
and "how many are set?" becomes a single `popcount`. This tutorial teaches the
`bitset` and `int` kernels that make that practical and branchless.

## What you'll build

A 64-slot "feature flag" set: you will set and clear flags, count how many are
active with `popcount`, ask "how many flags below index *k* are set?" with
`rank`, and find the position of the *n*-th set flag with `select_bit`.

**Prerequisites:** [Tutorial 1](./tutorial-1.md). Comfort with hex literals and
bit shifts helps.

## Step 1: Meet the bitset primitives

From `crates/bcinr-logic/src/bitset.rs` and `int.rs`:

```rust
pub const fn set_bit_u64(x: u64, pos: usize) -> u64   { x |  (1u64 << (pos & 63)) }
pub const fn clear_bit_u64(x: u64, pos: usize) -> u64 { x & !(1u64 << (pos & 63)) }
pub fn rank_u64(x: u64, pos: usize) -> usize;             // set bits in [0, pos]
pub fn select_bit_u64(x: u64, n: usize) -> Option<usize>; // index of the n-th set bit
pub const fn popcount_u64(x: u64) -> u64;                 // total set bits (from int)
```

Note `pos & 63` in set/clear: the index is masked into range, so an out-of-bounds
position can never trigger a panic — branchless safety by construction.

## Step 2: Set some flags

```rust
use bcinr_logic::bitset::{clear_bit_u64, set_bit_u64};

const DARK_MODE: usize = 0;
const BETA_UI: usize = 3;
const TELEMETRY: usize = 7;

fn main() {
    let mut flags: u64 = 0;
    flags = set_bit_u64(flags, DARK_MODE);
    flags = set_bit_u64(flags, BETA_UI);
    flags = set_bit_u64(flags, TELEMETRY);

    // turn telemetry back off
    flags = clear_bit_u64(flags, TELEMETRY);

    println!("flags = {:#018b}", flags);
}
```

## Step 3: Run it

```bash
cargo run
```

Expected output (bit 0 and bit 3 set, bit 7 cleared):

```
flags = 0b0000000000001001
```

## Step 4: Count active flags with popcount

```rust
use bcinr_logic::bitset::set_bit_u64;
use bcinr_logic::int::popcount_u64;

fn main() {
    let mut flags: u64 = 0;
    for pos in [0usize, 3, 7, 40] {
        flags = set_bit_u64(flags, pos);
    }
    println!("active flags: {}", popcount_u64(flags));
}
```

```bash
cargo run
```

Expected output:

```
active flags: 4
```

`popcount_u64` lowers to a single `POPCNT` instruction on SSE4.2 targets — O(1),
no loop over bits.

## Step 5: Use rank and select

`rank_u64(x, pos)` counts set bits in the inclusive range `[0, pos]` — useful for
"index within the active set". `select_bit_u64(x, n)` is the inverse: it returns
the position of the *n*-th set bit (0-based), or `None` if there are fewer than
`n + 1` set bits.

```rust
use bcinr_logic::bitset::{rank_u64, select_bit_u64, set_bit_u64};

fn main() {
    // set bits at positions 0, 3, 7
    let flags = set_bit_u64(set_bit_u64(set_bit_u64(0, 0), 3), 7);

    // how many set bits at or below position 3? (positions 0 and 3)
    println!("rank up to 3: {}", rank_u64(flags, 3));

    // where is the 0th, 1st, 2nd set bit?
    println!("0th set bit: {:?}", select_bit_u64(flags, 0));
    println!("1st set bit: {:?}", select_bit_u64(flags, 1));
    println!("2nd set bit: {:?}", select_bit_u64(flags, 2));
    println!("3rd set bit: {:?}", select_bit_u64(flags, 3)); // none exists
}
```

```bash
cargo run
```

Expected output:

```
rank up to 3: 2
0th set bit: Some(0)
1st set bit: Some(3)
2nd set bit: Some(7)
3rd set bit: None
```

`rank` and `select` are inverses on set bits: `select_bit_u64(x, n)` returns
position `p` exactly when `rank_u64(x, p)` equals `n + 1` and bit `p` is set.

## Step 6: Lock it in

```rust
use bcinr_logic::bitset::{rank_u64, select_bit_u64, set_bit_u64};
use bcinr_logic::int::popcount_u64;

#[test]
fn bitset_roundtrip() {
    let flags = set_bit_u64(set_bit_u64(set_bit_u64(0, 0), 3), 7);
    assert_eq!(popcount_u64(flags), 3);
    assert_eq!(rank_u64(flags, 7), 3);
    assert_eq!(select_bit_u64(flags, 1), Some(3));
    assert_eq!(select_bit_u64(flags, 9), None);
}
```

```bash
cargo test bitset_roundtrip
```

```
test bitset_roundtrip ... ok
```

## What you learned

- `set_bit_u64` / `clear_bit_u64` mask the index (`pos & 63`), so they are
  panic-free and branchless.
- `popcount_u64` answers "how many?" in one instruction.
- `rank_u64` and `select_bit_u64` are inverses, giving you O(1) positional
  queries over a packed flag set.

## Next steps

- [Tutorial 5: A tiny branchless state machine](./tutorial-5.md) — feed bytes
  through a DFA whose transition path never branches.
- [Tutorial 7: Branchless scans over byte slices](./tutorial-7.md) — turn whole
  buffers into bitmasks and reduce them.
