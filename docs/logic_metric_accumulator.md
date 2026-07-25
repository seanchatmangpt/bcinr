Here is the analysis of `crates/bcinr-logic/src/autonomic/metric_accumulator.rs`:

### Struct Fields
`MetricAccumulator` is a unit struct with no fields:
```rust
pub struct MetricAccumulator;
```

### Branchless Accumulation Mechanisms
It achieves branchless metric accumulation using two primary methods:

1. **Saturating Arithmetic**:
   It relies on Rust's intrinsic `saturating_add` method to aggregate values up to a maximum bound without relying on `if` conditionals. This gets compiled down to branchless saturation instructions on modern ISAs (Instruction Set Architectures).
   ```rust
   pub fn metric_accumulator_sat_add(current: u64, val: u64) -> u64 {
       current.saturating_add(val)
   }
   ```

2. **Exponential Moving Average (EMA)**:
   It uses straight-line floating-point arithmetic to compute the moving average, avoiding loop or branch-based conditional updates.
   *(Note: The `bcinr` constitution strictly prohibits floating-point operations unless explicitly admitted, so this `f32` method might be subject to scrutiny depending on admission status).*
   ```rust
   pub fn ema(current: f32, val: f32, alpha: f32) -> f32 {
       (alpha * val) + (1.0 - alpha) * current
   }
   ```
