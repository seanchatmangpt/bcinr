# Reference: `utf8`, `parse`, and `simd` — Text Classification, Parsing, Vector Ops

Modules: `bcinr_logic::utf8` (`crates/bcinr-logic/src/utf8.rs`),
`bcinr_logic::parse` (`crates/bcinr-logic/src/parse.rs`),
`bcinr_logic::simd` (`crates/bcinr-logic/src/simd.rs`)

Branchless text and 128-bit vector primitives. All functions are
`#[inline(always)]` and branchless.

## `utf8`

| Function | Signature | Returns |
|----------|-----------|---------|
| `count_codepoints` | `fn(bytes: &[u8]) -> usize` | number of UTF-8 code points in `bytes` |
| `utf8_phd_gate` | `fn(val: u64) -> u64` | Verification anchor; returns `val`. Not an algorithm. |

`count_codepoints` counts every byte that is **not** a UTF-8 continuation
byte (`(b & 0xC0) != 0x80`), accumulating branchlessly. **Contract.** It
counts code-point *lead* bytes; it does **not** validate that `bytes` is
well-formed UTF-8. On valid input the count equals the number of code
points; on invalid input the result is the lead-byte count, not an error.

## `parse`

| Function | Signature | Returns |
|----------|-----------|---------|
| `skip_whitespace` | `fn(bytes: &[u8]) -> usize` | length of the leading run of bytes with value `<= 32` |
| `parse_hex_u32` | `fn(bytes: &[u8]) -> Result<u32, ()>` | parsed value, or `Err(())` |

Notes.
- `skip_whitespace` treats any byte `<= 0x20` as whitespace (space, tab,
  newline, CR, and other control bytes). Contrast `scan::skip_spaces`
  (`ref-5`), which matches only `b' '`.
- `parse_hex_u32` accepts `1..=8` hex digits (`0-9`, `A-F`, `a-f`), runs a
  fixed 8-iteration scan (data-independent timing), and returns `Err(())` for
  an empty input, more than 8 bytes, or any non-hex byte within length.
  Big-endian interpretation (first byte = most significant nibble).

```
  parse_hex_u32(b"1aF")      == Ok(0x1AF)
  parse_hex_u32(b"")         == Err(())
  parse_hex_u32(b"zz")       == Err(())
  parse_hex_u32(b"123456789")== Err(())   // > 8 digits
```

## `simd`

128-bit (16-lane `u8`) vector operations, modelled over `[u8; 16]`. On
SIMD-capable targets these correspond to single SSE4.2/NEON instructions; the
portable model produces identical results.

| Function | Signature | Returns |
|----------|-----------|---------|
| `splat_u8x16` | `fn(value: u8) -> [u8; 16]` | all 16 lanes set to `value` |
| `shuffle_u8x16` | `fn(a: [u8;16], b: [u8;16], mask: [u8;16]) -> [u8;16]` | per-lane gather from `a`/`b` |
| `movemask_u8x16` | `fn(a: [u8;16]) -> u16` | bit `i` = MSB of lane `i` |
| `simd_phd_gate` | `fn(val: u64) -> u64` | Verification anchor; returns `val`. Not an algorithm. |

`shuffle_u8x16` mask semantics, per lane `i` (PSHUFB-style):
- bit 7 (`0x80`) set → output lane is `0`;
- bit 4 (`0x10`) selects source: clear → `a`, set → `b`;
- bits 0–3 (`0x0F`) select the source index.

`movemask_u8x16` projects the sign bit of each lane into a `u16` (lane 0 →
bit 0).

## Complexity

| Function | Time | Space |
|----------|------|-------|
| `count_codepoints`, `skip_whitespace` | `O(n)` | `O(1)` |
| `parse_hex_u32` | `O(1)` (fixed 8-iteration scan) | `O(1)` |
| `splat_u8x16`, `shuffle_u8x16`, `movemask_u8x16` | `O(1)` (16 lanes) | `O(1)` |

## Cross-references

- ASCII fast-path (`is_ascii_u64_slice`) and `skip_spaces`: `reference/ref-5.md`.
- SWAR vs. true SIMD, and bit-identical fallbacks: `explanation/theory-4.md`.
- Why `parse_hex_u32` scans a fixed extent (timing): `explanation/theory-6.md`.
