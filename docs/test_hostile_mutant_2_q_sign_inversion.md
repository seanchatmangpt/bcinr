Here is the implementation and analysis of `mutant_2` from `hostile_mutants.rs`:

### Implementation
```rust
#[cfg(all(feature = "mutant_2", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_2_q_sign_inversion() {
    let result_mutant = run_alloc_baseline().map(|x| x.value_bits());
    // Named law: mutant_2 negates each lens's signed `q` value before it enters the
    // exponential weighting update, inverting the sign convention the lens-weighting law
    // requires (higher payoff must increase, not decrease, relative weight).
    assert_eq!(
        result_mutant, WRONG_M2_Q_SIGN_INVERSION,
        "Mutant 2 (lens q value sign-inverted) must produce this exact corrupted allocation array"
    );
    assert_ne!(
        result_mutant, CORRECT_BASELINE,
        "Mutant 2 should deviate from correct baseline"
    );
}
```

### Broken Mathematical Law
**Lens-Weighting Law (Sign Convention):** The law dictates that a higher payoff must *increase* relative weight, not decrease it. `mutant_2` breaks this by negating each lens's signed `q` value before it enters the exponential weighting update, thereby inverting the required sign convention.

### Expected Outcome / Refusal
The mutant deterministically corrupts the allocation array, yielding an exact wrong state rather than simply failing or triggering a typed runtime refusal. The expected outcome is that the resulting weights exactly match the `WRONG_M2_Q_SIGN_INVERSION` signature array (`[8342, 10040, 7893, 7893, 7892, 7892, 6684, 8900]`), which is distinct from the `CORRECT_BASELINE` array. The test `kill_mutant_2_q_sign_inversion` asserts exactly on this specific corrupted state to verify that the mutant is properly killed by this specific detection.
