I have located the fixed-point branchless sigmoid implementation in the codebase. Here are the details you requested:

### Location
[`crates/bcinr-logic/src/algorithms/sigmoid_sat_u32.rs`](file:///Users/sac/bcinr/crates/bcinr-logic/src/algorithms/sigmoid_sat_u32.rs)

### Branchless SWAR Implementation
The `sigmoid_sat_u32` function interprets the input as a signed Q16 fixed-point number and computes a piecewise linear approximation:
`sigmoid(x) ≈ 0.5 + x/8`

To ensure branchless execution (`CC=1`, meeting the Radon Law) and zero-allocation:
1. **Input Clamping**: The input `x` is first clamped to `[-8.0, 8.0]` in Q16 (`±524288`) to prevent overflow during slope computation.
2. **Piecewise Slope**: It adds the clamped value divided by 8 to the Q16 representation of 0.5 (`32768i64 + x_clamped / 8`).
3. **Saturation**: The result is constrained to `[0, 65536]` (where `65536` represents `1.0`) using Rust's `clamp()`, which lowers to a constant-time `CMOV` instruction on modern architectures, avoiding any control flow branches.

```rust
pub fn sigmoid_sat_u32(val: u64, _aux: u64) -> u64 {
    let x = val as i64; 
    const Q16: i64 = 65536;
    const CLAMP: i64 = 8 * Q16; 
    let x_clamped = x.clamp(-CLAMP, CLAMP);

    let half = 32768i64; 
    let approx = half + x_clamped / 8;

    let approx = approx.clamp(0, Q16);
    approx as u64
}
```

### Error Envelope & Axiomatic Properties
The file defines the function's Hoare-logic contract and error envelope explicitly:
- **Precondition:** `val ∈ U64` interpreted as signed Q16.
- **Postcondition:** `result ∈ [0, 65536]` and `|result - sigmoid_f64(val)*65536| <= 3277`.
- **Error Bound:** The maximum approximation error relative to the `f64` logistic reference across the entire domain is bounded by **±3277 (~5%)**.
- **Accuracy Profile:** 
  - Exact at `x = 0` (returns exactly 32768).
  - The maximum error occurs at `x = ±4`, where the true sigmoid is `~0.982` but the approximation saturates early to `1.0` (error `≈ 0.018 * 65536 ≈ 1180`).
- **Mathematical Safety**: Maintains monotonic behavior and has no discontinuities across the clamped boundaries.
