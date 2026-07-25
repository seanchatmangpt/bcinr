# UTF-8 Validation Analysis in `bcinr`

## API Boundary (`crates/bcinr-api/src/utf8.rs`)
The `utf8.rs` file within the `bcinr-api` crate acts strictly as a zero-cost facade. It contains no inline logic and simply re-exports primitives from the underlying core logic crate:

```rust
pub use bcinr_logic::utf8::{validate_utf8, count_codepoints, ascii_prefix_len, first_invalid_byte};
```

This enforces the project's strict separation of concerns where the API layer does not introduce any abstraction penalties, mapping directly to the branchless logic implementation.

## Branchless Implementation Mechanics (`crates/bcinr-logic/src/utf8.rs`)
While the API exports `validate_utf8`, the current source tree implementation in `bcinr-logic/src/utf8.rs` focuses on the foundational branchless primitives necessary for full validation and analysis. 

The implementation adheres strictly to the **Radon Law (CC=1)** (Cyclomatic Complexity of 1), avoiding any data-dependent branches, `if` statements, or short-circuiting loops. 

### 1. Constant-Time Byte Classification
Instead of branching to determine byte types, the logic uses bitwise masking and direct evaluation:
- **Continuation Bytes (`10xxxxxx`)**: `(byte & 0xC0) == 0x80`
- **ASCII Bytes (`0xxxxxxx`)**: `byte < 0x80`
- **2-Byte Leads (`110xxxxx`)**: `(byte & 0xE0) == 0xC0`
- **3-Byte Leads (`1110xxxx`)**: `(byte & 0xF0) == 0xE0`
- **4-Byte Leads (`11110xxx`)**: `(byte & 0xF8) == 0xF0`

These predicates compile down to unconditional bitwise ANDs and comparisons, generating deterministic masks rather than jumps.

### 2. Branchless State Accumulation (`count_codepoints`)
The `count_codepoints` function provides a canonical example of how operations across the slice are performed branchlessly:

```rust
pub fn count_codepoints(bytes: &[u8]) -> usize {
    let mut count = 0;
    (0..bytes.len()).for_each(|i| {
        count += ((bytes[i] & 0xC0) != 0x80) as usize;
    });
    count
}
```

**Mechanics:**
- It iterates over a bounded `0..bytes.len()` range.
- For each byte, it computes a boolean expression identifying if the byte is **not** a continuation byte.
- It casts the boolean directly to a `usize` (0 or 1) and adds it to the accumulator.
- **No conditional execution occurs:** The processor never guesses a branch path, completely eliminating pipeline-stalling mispredictions.

### 3. Missing Implementations and SWAR Context
Currently, the exact implementations for `validate_utf8`, `ascii_prefix_len`, and `first_invalid_byte` are **missing** from `bcinr-logic/src/utf8.rs`, despite being exported by the API. 

Based on the repository's architectural documentation (`swar_techniques.md`), full UTF-8 validation handles external inputs by elevating these byte-level predicates into **SWAR (SIMD Within A Register)** operations. By processing 8 bytes concurrently inside a `u64` register and utilizing zero-byte detection polynomials (e.g., `(v - 0x0101...) & ~v & 0x8080...`), the runtime can validate entire chunks in parallel while mathematically preventing execution timing variance on adversarial inputs.
