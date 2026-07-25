# bcinr's Sealing Mechanism in `#![no_std]`

## `StabilityCandidate::seal`

In `crates/bcinr-cmca/src/stability.rs`, the binding of stability parameters is achieved via the `StabilityCandidate::seal` function. This method enforces Invariant 3 (from `authority-and-c3.md`), which requires that every domain-specific identity—including the static domination comparison matrix `G` and the positive witness `d`—be bound explicitly and independently.

```rust
fn seal(
    g: [[i64; DIM]; DIM],
    d: [i64; DIM],
    margin_delta: i64,
    noise_radius: i64,
    switch_radius: i64,
    q_ceiling: i64,
    gram_distinguishability_floor: i64,
    dwell_law_id: u64,
    pricing_loop_bound: i64,
    comparison_derivation_identity: u64,
) -> u64 {
    let mut acc = comparison_derivation_identity;
    for row in g.iter() {
        for &v in row.iter() {
            acc = mix64(acc, v as u64);
        }
    }
    for &v in d.iter() {
        acc = mix64(acc, v as u64);
    }
    // ... progressively folds in the remaining margin, radii, ceiling, etc.
    acc
}
```

By preventing these arguments from being collapsed into a single struct, the contract ensures no parameter is implicitly omitted from the seal. The `seal` function initializes an accumulator `acc` with the `comparison_derivation_identity` and then iteratively folds each component (such as elements of `G` and `d`) into it using `mix64`.

## The `mix64` Hashing Mechanism

The `mix64` function is defined in `crates/bcinr-cmca/src/proposal.rs`. 

```rust
pub(crate) fn mix64(a: u64, b: u64) -> u64 {
    let mut x = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x
}
```

### How `bcinr` Meets the $CC=1$ and Allocation-Free Laws

1. **Branchless Avalanche Mixing**: The `mix64` function employs a fixed SplitMix64-style avalanche mix. It relies entirely on fast, constant-time arithmetic operations (`wrapping_mul`, bit-shifts, and XORs). This avoids `if` branches and prevents panic edges completely, making it fully compliant with the `CC=1` (cyclomatic complexity of 1) and zero-allocation requirements.
2. **Fixed-Size Loops**: Because inputs like `G` and `d` use fixed, compile-time bounds (`DIM = 2`), iterating over them resolves to fixed-size unrolled logic rather than variable bounds, keeping execution work strictly bounded and memory access patterns fixed.
3. **Hot-Path vs. Slow-Rail Cryptography Distinction**: The code explicitly comments that `mix64` **is not a cryptographic hash**. It acts as a fast, structural identity binder for equality checks necessary within the highly restrictive authoritative hot path. Whenever actual cryptographically collision-resistant proofs (like BLAKE3 receipts) are required, they are computed on the allocating, branching "slow rail" (in modules like `bcinr-powl-receipt`), avoiding the hot path entirely.

Thus, `bcinr` achieves "sealing" in the hot path by combining strict parameter enumeration with deterministic integer avalanche mixing, and delegates true cryptographic verification properties to the slow rail context.
