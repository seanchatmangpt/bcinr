# SWAR: SIMD Within A Register

## What is SWAR?

SWAR (SIMD Within A Register) is a technique for processing multiple data items
simultaneously using standard integer registers — without hardware SIMD instructions.

A 64-bit integer can hold 8 bytes, 4 shorts, or 2 ints. By carefully crafting
bitwise arithmetic, we can process all 8 bytes in a single operation.

This matters for `bcinr` because:
1. It keeps algorithms `no_std` and portable — no `std::arch` imports.
2. The compiler can auto-vectorize SWAR patterns to real SIMD (SSE/AVX/NEON).
3. Even without vectorization, 8x data parallelism is free on 64-bit hardware.

## The Core Pattern: Zero Detection

The fundamental SWAR primitive detects zero bytes within a 64-bit word:

```rust
fn has_zero_byte(v: u64) -> bool {
    v.wrapping_sub(0x0101_0101_0101_0101) & !v & 0x8080_8080_8080_8080 != 0
}
```

This works because:
1. `v.wrapping_sub(0x0101...)` underflows on zero bytes (the subtraction borrows
   from the next byte lane, setting the high bit of the result byte).
2. `!v` has the high bit set only for bytes whose value is less than 128.
3. The AND of both conditions fires only where `v` held an exact zero byte.

The magic constants:
- `0x0101_0101_0101_0101` — value 1 broadcast to every byte lane
- `0x8080_8080_8080_8080` — high-bit mask for every byte lane

## Finding a Specific Byte

To find byte value `c` in a word `v`, XOR first to transform into zero-detection:

```rust
fn find_byte(v: u64, c: u8) -> u64 {
    // XOR turns every occurrence of `c` into a zero byte
    let x = v ^ (c as u64 * 0x0101_0101_0101_0101);
    // Now apply zero-byte detection on x
    x.wrapping_sub(0x0101_0101_0101_0101) & !x & 0x8080_8080_8080_8080
}
```

The return value has bit 7 of each matching byte lane set to 1 and all other
lanes at zero. To extract a lane index, use `trailing_zeros() / 8`.

## Performance Characteristics

| Operation              | Naive (per 8 bytes)    | SWAR (per 8 bytes)   | Speedup |
|------------------------|------------------------|----------------------|---------|
| Find byte              | 8 branch comparisons   | 5 arithmetic ops     | ~4x     |
| Count occurrences      | 8 comparisons + sum    | 7 arithmetic ops     | ~2-3x   |
| ASCII check (64 bytes) | 64 branches            | 8 arithmetic ops     | ~8x     |
| Lowercase conversion   | 64 branches + OR       | 16 arithmetic ops    | ~4x     |

Numbers are approximate; actual gains depend on branch prediction accuracy and
CPU micro-architecture. On real workloads where the data pattern is unpredictable
(e.g., parsing user-supplied text), the branch misprediction penalty makes the
advantage even larger.

## ASCII Validation

To check whether all bytes in a `u64` word are 7-bit ASCII (high bit clear):

```rust
fn is_ascii_word(v: u64) -> bool {
    v & 0x8080_8080_8080_8080 == 0
}
```

Process a byte slice 8 bytes at a time by ORing all words together and checking
once at the end:

```rust
fn is_ascii_slice(bytes: &[u8]) -> bool {
    let mut acc = 0u64;
    for chunk in bytes.chunks_exact(8) {
        let word = u64::from_le_bytes(chunk.try_into().unwrap());
        acc |= word;
    }
    // Handle remainder bytes individually
    for &b in bytes.chunks_exact(8).remainder() {
        acc |= b as u64;
    }
    acc & 0x8080_8080_8080_8080 == 0
}
```

This is exactly how `crate::scan::is_ascii_u64_slice` is implemented.

## ASCII Case Conversion

To convert 8 uppercase ASCII bytes to lowercase simultaneously:

```rust
fn to_lower_8(v: u64) -> u64 {
    // Step 1: identify uppercase bytes (0x41..=0x5A)
    // A byte is uppercase if (byte - 'A') <= 25, i.e., byte in ['A', 'Z']
    //
    // SWAR range check: subtract 'A' from all bytes, then check if result < 26
    let shifted = v.wrapping_sub(0x4141_4141_4141_4141); // subtract 'A' from each lane
    let in_range = shifted.wrapping_sub(0x1A1A_1A1A_1A1A_1A1A) // if was < 26, underflows
                   & !shifted                                    // high bit must not have been set before
                   & 0x8080_8080_8080_8080;                     // extract the underflow signals
    // Step 2: for uppercase bytes, set bit 5 (0x20) to make them lowercase
    let lowercase_bit = in_range >> 2; // 0x80 >> 2 = 0x20, aligned to bit 5
    v | lowercase_bit
}
```

## Relationship to Auto-vectorization

When the compiler sees tight SWAR loops over arrays, it often auto-vectorizes
them to actual SIMD instructions (SSE/AVX/NEON/SVE). The SWAR code thus serves
a dual purpose:

1. **Portable fallback** — already 2-8x faster than naive scalar code on any
   64-bit target, even without hardware SIMD.
2. **Vectorization hint** — the data-parallel structure signals to the compiler
   what parallelism is available, enabling auto-vectorization when the target
   supports it.

You can inspect generated assembly with `cargo asm` or on [Compiler Explorer](https://godbolt.org/).

## When to Use SWAR vs. Actual SIMD

| Scenario | Recommendation |
|----------|---------------|
| `no_std` target, portability required | **SWAR** |
| Processing fewer than 64 bytes | **SWAR** (setup cost of SIMD not worth it) |
| Maximum throughput, known x86-64 hardware | **SSE4.2 intrinsics** via `std::arch` |
| Large arrays (> 256 bytes), portable | **Auto-vectorization** with SWAR-shaped code |
| Embedded (ARM Cortex-M without NEON) | **SWAR** |
| WASM without simd128 feature | **SWAR** |

## Further Reading

- Hacker's Delight (Warren, 2nd ed.) — Chapter 6: Searching Words
- `crates/bcinr-logic/src/scan.rs` — SWAR in production (ASCII scan, byte search)
- `crates/bcinr-logic/src/swar.rs` — Additional SWAR building blocks
- `crates/bcinr-logic/src/utf8.rs` — UTF-8 validation using SWAR techniques
