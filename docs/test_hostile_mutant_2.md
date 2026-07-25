# Analysis of `mutant_2`

## Locations Found
- `crates/bcinr-cmca/tests/hostile_mutants.rs`
- `crates/bcinr-cmca/src/allocator.rs`

## Mathematical Law Broken
**The lens-weighting sign convention law.**

`mutant_2` explicitly negates each lens's signed `q` value before it enters the exponential weighting update. This inverts the required sign convention of the lens-weighting law, which mandates that a **higher payoff must increase, not decrease, the relative weight**.

In `src/allocator.rs`, this is implemented as:
```rust
let q_val_mutated = SignedFixed::from_parts(
    0i32.wrapping_sub(q_val_mutated.value_bits()),
    q_val_mutated.faults(),
);
```

## Expected Outcome / Refusal
The mutated code deterministically corrupts the allocation combination process, producing a specific corrupted allocation array.

The test `kill_mutant_2_q_sign_inversion()` in `tests/hostile_mutants.rs` verifies that:
1. The execution produces the exact corrupted array `WRONG_M2_Q_SIGN_INVERSION` (`[8342, 10040, 7893, 7893, 7892, 7892, 6684, 8900]`), confirming the deterministic failure of the sign-inversion rather than a generic divergence.
2. The output strictly deviates from the `CORRECT_BASELINE` array.

Additionally, this mutant's specific verification is masked if `mutant_7` is active simultaneously. `mutant_7` breaks the `const_eq_u32` zero-denominator check, artificially saturating all shared divisions to `u32::MAX` and effectively burying the `mutant_2` signature. As a result, the `mutant_2` check requires `not(feature = "mutant_7")`.
