# Tutorial 7: Branchless Scans Over Byte Slices

Scanning a buffer — "is this all ASCII?", "where are the commas?", "skip the
leading spaces" — is normally a loop full of branches, one per byte. The `scan`
and `reduce` modules do it with SWAR (SIMD-Within-A-Register) and bitmask tricks,
processing many bytes per step with no data-dependent control flow. This tutorial
walks you through the core scan kernels.

## What you'll build

A miniature CSV field locator: you will validate that a buffer is ASCII with
`scan::is_ascii_u64_slice`, build a bitmask of every comma with
`scan::find_byte_mask`, count the fields with `int::popcount_u64`, and reduce a
packed lane vector with `reduce::horizontal_sum_u8x8`.

**Prerequisites:** [Tutorial 1](./tutorial-1.md) and
[Tutorial 4](./tutorial-4.md) (bitmasks and `popcount`).

## Step 1: Validate ASCII in bulk (SWAR)

From `crates/bcinr-logic/src/scan.rs`, `is_ascii_u64_slice` ORs eight bytes at a
time and checks the high bit of each lane:

```rust
pub fn is_ascii_u64_slice(bytes: &[u8]) -> bool {
    let mut accumulator = 0u64;
    let chunks = bytes.chunks_exact(8);
    chunks.for_each(|chunk| {
        let val = u64::from_le_bytes([/* 8 bytes */ chunk[0], /* ... */ chunk[7]]);
        accumulator |= val & 0x8080_8080_8080_8080; // high bit of every byte
    });
    // ...handle the tail byte-by-byte...
    accumulator == 0
}
```

`0x80` is the high bit of one byte; the repeated `0x8080_8080_8080_8080` checks
all eight lanes at once. If no high bit is ever set, every byte was `< 128`, i.e.
ASCII. Eight bytes per iteration, zero branches inside the chunk.

```rust
use bcinr_logic::scan::is_ascii_u64_slice;

fn main() {
    println!("{}", is_ascii_u64_slice(b"name,age,city")); // pure ASCII
    println!("{}", is_ascii_u64_slice("café".as_bytes())); // 'é' has high bits
}
```

```bash
cargo run
```

Expected output:

```
true
false
```

## Step 2: Build a match bitmask

`find_byte_mask` returns a `u64` where bit `i` is set when `bytes[i]` equals the
target (scanning up to the first 64 bytes):

```rust
pub fn find_byte_mask(bytes: &[u8], target: u8) -> u64 {
    let mut mask = 0u64;
    // for each of up to 64 positions:
    //   mask |= ((bytes[i] == target) as u64) << i;
    mask
}
```

```rust
use bcinr_logic::scan::find_byte_mask;

fn main() {
    let line = b"name,age,city";
    let commas = find_byte_mask(line, b',');
    println!("comma mask = {:#016b}", commas);
}
```

```bash
cargo run
```

Expected output (bits 4 and 8 — the two comma positions — set):

```
comma mask = 0b0000000100010000
```

## Step 3: Count fields with popcount

The number of fields is the number of separators plus one. Reuse `popcount_u64`
from [Tutorial 4](./tutorial-4.md):

```rust
use bcinr_logic::int::popcount_u64;
use bcinr_logic::scan::find_byte_mask;

fn count_fields(line: &[u8]) -> u64 {
    let separators = popcount_u64(find_byte_mask(line, b','));
    separators + 1
}

fn main() {
    println!("{}", count_fields(b"name,age,city")); // 2 commas -> 3 fields
    println!("{}", count_fields(b"single"));        // 0 commas -> 1 field
}
```

```bash
cargo run
```

Expected output:

```
3
1
```

You scanned the whole line and counted its fields with two branchless kernels and
one add — no per-character `if`.

## Step 4: Skip leading whitespace

`scan::skip_whitespace` (in the `parse` module) and `scan::skip_spaces` both
return the count of leading spaces branchlessly. Here is `skip_spaces`:

```rust
pub fn skip_spaces(bytes: &[u8]) -> usize {
    let mut offset = 0;
    // offset only advances while every prior byte was also a space
    bytes.iter().enumerate().for_each(|(i, &b)| {
        let is_space = (b == b' ') as usize;
        let contiguous = (offset == i) as usize;
        offset += is_space & contiguous;
    });
    offset
}
```

```rust
use bcinr_logic::scan::skip_spaces;

fn main() {
    println!("{}", skip_spaces(b"   value")); // three leading spaces
    println!("{}", skip_spaces(b"value"));    // none
}
```

```bash
cargo run
```

Expected output:

```
3
0
```

## Step 5: Reduce packed lanes

When you pack eight small counters into one `u64`, `reduce::horizontal_sum_u8x8`
adds all eight lanes in a logarithmic number of steps:

```rust
use bcinr_logic::reduce::horizontal_sum_u8x8;

fn main() {
    // eight lanes: 1,2,3,4,5,6,7,8 packed little-endian
    let packed: u64 = 0x0807_0605_0403_0201;
    println!("lane sum = {}", horizontal_sum_u8x8(packed)); // 1+2+...+8
}
```

```bash
cargo run
```

Expected output:

```
lane sum = 36
```

## Step 6: Lock it in

```rust
use bcinr_logic::int::popcount_u64;
use bcinr_logic::reduce::horizontal_sum_u8x8;
use bcinr_logic::scan::{find_byte_mask, is_ascii_u64_slice};

#[test]
fn scan_pipeline() {
    let line = b"a,b,c,d";
    assert!(is_ascii_u64_slice(line));
    assert_eq!(popcount_u64(find_byte_mask(line, b',')), 3); // three commas
    assert_eq!(horizontal_sum_u8x8(0x0807_0605_0403_0201), 36);
}
```

```bash
cargo test scan_pipeline
```

```
test scan_pipeline ... ok
```

## What you learned

- `is_ascii_u64_slice` uses SWAR to test the high bit of eight bytes per step.
- `find_byte_mask` turns a buffer into a position bitmask you can feed straight
  into `popcount_u64` or the bitset kernels.
- `skip_spaces` advances an offset branchlessly, and `horizontal_sum_u8x8`
  reduces eight packed lanes without a per-lane branch.

## Next steps

- [Tutorial 5: A tiny branchless state machine](./tutorial-5.md) — feed your
  classified bytes into a DFA.
- [Tutorial 8: A branchless AABB overlap test](./tutorial-8.md) — pack multiple
  values into one word and compare them in parallel.
