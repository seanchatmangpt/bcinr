# Mutant 1 Analysis Report

## Locations in Codebase
- **Test File:** `crates/bcinr-cmca/tests/hostile_mutants.rs`
- **Implementation File:** `crates/bcinr-cmca/src/allocator.rs` (around line 1857-1860)

## Mathematical Law Broken
The mutant intentionally breaks the **per-measure canonical-mixing law** (or the lens-weighting law). The true algorithm specifies that each of the $K$ measures must independently weight the allocation. 

`mutant_1` corrupts this by statically pinning the measure index variable (`k_actual`) to `0` inside the main iteration over $k$:
```rust
    unroll_4_static!(k, {
        #[cfg(feature = "mutant_1")]
        const k_actual: usize = 0;
        #[cfg(not(feature = "mutant_1"))]
        const k_actual: usize = k;
```
This forces all measure calculations to repeatedly use measure index 0, collapsing the per-measure canonical-mixing law into a single-measure result and dropping the influence of all other measures.

## Expected Outcome / Refusal
Since this mutant alters the resulting distribution flow directly without triggering a hard runtime violation, it does not yield a specific `StabilityRefusal` code. Instead, it yields an incorrect allocation array.

Per `AGENTS.md` and the BCINR maturity matrices, the test framework defines a deterministic independent check for this corruption rather than a simple `assert_ne!`. The execution returns a corrupted allocation array that must exactly match a known mathematically-flawed baseline:

- **Expected exact corrupted output array:** `[8528, 7445, 7506, 7506, 7506, 7506, 12033, 7506]` (defined as `WRONG_M1_MEASURE_COLLAPSE`).

The test `kill_mutant_1_single_measure_collapse` verifies that this exact array is returned, proving the exact structural defect was caught by the hostile mutation checks.
