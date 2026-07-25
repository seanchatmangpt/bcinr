# Rule 12: No Runtime Theorem Discovery — Implementation Details

## Overview
According to **AGENTS.md §12 ("No runtime theorem discovery")**, the authoritative runtime is strictly forbidden from searching for or discovering a stability witness using dynamic runtime algorithms (like spectral-radius estimation, power iteration, or dynamic graph analysis). Instead, the hot path must only *verify* a witness and stability matrix provided by the slow rail. 

## Verification Mechanism (`derive_stability_candidate`)
The verification logic lives in `crates/bcinr-cmca/src/stability.rs` acting as Authority hop 4 of the C3 chain. The function `derive_stability_candidate` ensures stability branchlessly by checking the static domination law:

```text
G d <= (1 - delta) d      (elementwise)
```

Where:
- `G`: The comparison matrix (`DIM x DIM` array, where `DIM = 2` is compile-time fixed).
- `d`: The claimed positive witness vector.
- `delta`: The contraction margin.

All of these are provided as Q16.16 scaled fixed-point integers to avoid floating-point operations.

### Branchless Verification Flow
The verification runs across a completely bounded, branchless, non-allocating execution path:

1. **Precondition Validations**: 
   The function validates invariants through fixed loops:
   - Ensures the jump isn't a policy jump (`UpstreamJumpNotStabilityRelevant`).
   - Iterates through the elements of `d` to ensure they are strictly positive (`WitnessNotPositive`).
   - Verifies `delta` is properly inside the `(0, 1)` range (`MarginOutOfRange`).

2. **Bounded Matrix-Vector Multiplication ($G \cdot d$)**:
   The computation occurs over small, fixed dimensions without relying on variable loops. To prevent overflow during multiplication without runtime checks, the intermediate step leverages `i128`:
   ```rust
   for r in 0..DIM {
       let mut acc: i128 = 0;
       for c in 0..DIM {
           acc += (g[r][c] as i128) * (d[c] as i128);
       }
       gd[r] = (acc / SCALE as i128) as i64; // Rescaled to Q16.16
   }
   ```

3. **Elementwise Domination Check**:
   It calculates the right side of the static domination equation `(1 - delta) * d` securely using Q16.16 arithmetic. If any element `G * d` exceeds the bound, it halts processing and explicitly returns a typed refusal:
   ```rust
   return Err(StabilityDerivationRefusal::ContractionMarginInsufficient);
   ```

4. **Cryptographic Sealing**:
   If the inequality mathematically holds, the function does not just assert validity. It derives a `candidate_digest` using a `mix64` bitwise hash of every domain-specific identity (`G`, `d`, `margin_delta`, `noise_radius`, `q_ceiling`, etc.). 
   This hash is bundled into a `StabilityCandidate` struct, proving that the verification passed, guaranteeing no speculative theorem discovery occurred, and binding the matrix safely to the hot path's state machine.

## Typed Refusals
Failure states generate bounded, strict `enum` types representing the exact failure law, in alignment with rule requirements for explicit refusals:

```rust
pub enum StabilityDerivationRefusal {
    WitnessNotPositive,
    MarginOutOfRange,
    ContractionMarginInsufficient,
    UpstreamJumpNotStabilityRelevant,
}
```

Through this implementation structure (compile-time limits `DIM = 2`, `i128` overflow prevention, Q16.16 fixed-point types, and mathematical cryptographic seals), the codebase strictly adheres to Rule 12.
