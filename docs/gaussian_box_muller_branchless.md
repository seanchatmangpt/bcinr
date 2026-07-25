# Branchless Box-Muller Transform in BCINR

## The Core Misconception
While a traditional Box-Muller transform relies on computing continuous mathematics—specifically $Z_0 = \sqrt{-2 \ln U_1} \cos(2 \pi U_2)$—the implementation in BCINR's hot path **does not use fixed-point approximations of logarithms or square roots**. 

The strict adherence to the **Radon Law ($CC=1$)**, the zero-allocation boundary, and the absolute prohibition of floating-point arithmetic (`f64`) necessitates a complete departure from traditional mathematical models. Simulating complex functions like `ln` or `sqrt` via fixed-point arithmetic without branching is extremely difficult to do efficiently, and would risk violating the substrate's constant-time latency budgets.

## How "Gaussian Noise" is Actually Generated
Instead of computing the actual trigonometric and logarithmic operations, BCINR bypasses the continuous math completely by using a **constant-time integer surrogate**. 

The implementation found in `crates/bcinr-logic/src/algorithms/gaussian_noise_box_muller.rs` achieves this by combining the two input uniforms and passing them directly through a **SplitMix64 avalanche finalizer**.

Here is the exact zero-branch implementation:

```rust
pub fn gaussian_noise_box_muller(val: u64, aux: u64) -> u64 {
    // Combine the two uniform words, then run the SplitMix64 finalizer.
    let mut z = val.wrapping_add(aux).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
```

### Execution Breakdown:
1. **Seed Initialization**: The two externally injected uniform words (`val` and `aux`) are combined via a `wrapping_add` alongside the fractional golden-ratio constant (`0x9E37_79B9_7F4A_7C15`).
2. **First Avalanche (Xor-Shift-Multiply)**: The state is xor-shifted by 30 bits, then mixed using a wrapping multiplication with the prime constant `0xBF58_476D_1CE4_E5B9`.
3. **Second Avalanche**: The state is xor-shifted by 27 bits, followed by another multiplication with `0x94D0_49BB_1331_11EB`.
4. **Final Output**: A final 31-bit xor-shift yields a thoroughly mixed 64-bit value.

## Why this Architecture?
According to `docs/deterministic_entropy_generation.md`, the hot path in BCINR is completely barred from non-deterministic syscalls and control-flow-heavy entropy generation. 

By utilizing the SplitMix64 finalizer as a surrogate for true Box-Muller sampling, the runtime guarantees:
- **Absolute Branchlessness**: Execution time is entirely independent of the input values, satisfying $CC=1$.
- **0 Allocations**: Uses purely fixed-width, stack-allocated variables.
- **Deterministic Replayability**: Since the "randomness" is just a pure mathematical function of the explicitly provided `val` and `aux` inputs, executions can be perfectly replayed and mathematically certified.
