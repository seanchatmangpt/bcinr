# Research Report: Branchless Approximations in `bcinr-logic/src/algorithms/`

### Fast Inverse Square Root (`fast_inverse_sqrt_u32.rs`)

An implementation of the famous Quake III fast inverse square root approximation is located at `crates/bcinr-logic/src/algorithms/fast_inverse_sqrt_u32.rs`. It strictly adheres to the project's **Radon Law (CC=1)** (as defined in the constitution), meaning it operates in constant time without any conditional jumps (`if`, `match`, etc.).

#### Implementation
```rust
pub fn fast_inverse_sqrt_u32(val: u64, aux: u64) -> u64 {
    let x = (val & 0xFFFFFFFF) as f32;
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    f32::from_bits(i) as u64
}
```

#### How Mathematical Bounds are Enforced Branchlessly

1. **Bitwise Input Clamping:** 
   To restrict the `u64` input to a valid 32-bit domain before operating on it as a 32-bit float, the implementation uses a bitwise mask (`val & 0xFFFFFFFF`). This entirely avoids traditional bounds-check branching (e.g., `if val > u32::MAX`).
   
2. **Fixed-Width Polynomials and Bit Tricks:** 
   The core approximation is executed using raw memory transmutations and linear arithmetic (`x.to_bits()`, `>> 1`, and subtraction from the magic constant `0x5f3759df`). Because these operations map sequentially onto the CPU and do not observe the value to dictate control flow, computational latency and execution geometry remain mathematically fixed.
   
3. **Rigorous Equivalence Oracles:** 
   Rather than relying on runtime assertions that would introduce branching panics, correctness is verified at test time. The file contains a reference implementation (`fast_inverse_sqrt_u32_reference`) and asserts exhaustive boundary matches for inputs like `0` and `u64::MAX`.
   
4. **Hostile Mutants Matrix:**
   Per the BCINR adversarial testing requirements, the module uses "negative mutants" (deliberately flawed versions like `Identity bluff`, `Bit-skip bluff`, and `Operator-swap bluff`) to ensure the test suite is capable of catching single-bit deviations or invalid ranges. This creates a provable standard of safety without compromising hot-path performance with bounds checks.

*(Note: There were no implementations of eigenvalue lower bounds found within `crates/bcinr-logic/src/algorithms/`. References to eigenvalue lower bounds are present instead within the structural telemetry logic in `crates/bcinr-cmca/`.)*
