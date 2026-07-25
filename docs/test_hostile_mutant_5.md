Here is the requested documentation on `mutant_5`:

### Mathematical Law Broken
`mutant_5` breaks the **consequence-mass truncation law**. 

According to this law, the consequence array `mu` must be strictly bounded and clamped to its admitted range `[0, mu_max]` *before* it is used to price leaf costs. 

In `crates/bcinr-cmca/src/allocator.rs`, the authoritative implementation enforces this:
```rust
#[cfg(not(feature = "mutant_5"))]
let mu_actual = clip(mu[x & 7], NonNegativeFixed::ZERO, mu_max);
```
However, when `mutant_5` is enabled, the code explicitly skips this boundary enforcement and uses the raw, unclamped value:
```rust
#[cfg(feature = "mutant_5")]
let mu_actual = mu[x & 7];
```

### Expected Outcome / Refusal
The verification test `kill_mutant_5_consequence_truncation` in `crates/bcinr-cmca/tests/hostile_mutants.rs` provides a fixture with deliberately negative (out-of-bounds) `mu` values.

Because `mutant_5` skips the clipping procedure, the pricing logic evaluates these invalid parameters. Rather than producing the correct clamped-baseline array (`CORRECT_MU_COST`), the outcome evaluates to a specific **corrupted allocation array**. In the context of this test fixture's inputs, this corrupted output coincides numerically exactly with the `CORRECT_BASELINE` array, and the test ensures this specific corrupted outcome is caught:

```rust
assert_eq!(
    result_mutant, CORRECT_BASELINE,
    "Mutant 5 (mu clipping to [0, mu_max] skipped) must produce this exact corrupted \
     allocation array (coincides numerically with CORRECT_BASELINE's array at this fixture's \
     inputs; it is not equal to CORRECT_MU_COST, the law this mutant violates)"
);
```
