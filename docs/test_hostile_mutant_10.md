Here is the documentation regarding `mutant_10` and the mathematical law it breaks:

# Analysis of `mutant_10`

### Location
- **Test File:** `crates/bcinr-cmca/tests/hostile_mutants.rs`
- **Implementation File:** `crates/bcinr-cmca/src/observatory.rs`

### Mathematical Law Broken
The system utilizes branchless bitwise operations to check bounds. The valid arithmetic logic for `kappa_under_off` expects `kappa_under` to be strictly less than `epsilon_on`:
```rust
#[cfg(not(feature = "mutant_10"))]
let kappa_under_off = const_lt_u32(kappa_under.value_bits(), epsilon_on.value_bits());
```

`mutant_10` corrupts this mathematical bound by deliberately inverting the operand order for the `const_lt_u32` comparison:
```rust
#[cfg(feature = "mutant_10")]
let kappa_under_off = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits()); // Mutated: inverted
```
By inverting this comparison, the mutant checks if `epsilon_on` is strictly less than `kappa_under` instead of the reverse.

### Expected Outcome & Refusal
Because the bound comparison is inverted, the mutant will **falsely set** the `ObservatoryFlag::NumericallyUncertain` flag when `kappa_under` is actually strictly greater than `epsilon_on` (a state which correctly indicates numerical certainty, not uncertainty). 

The test `kill_mutant_10_false_numerically_uncertain` verifies this exact expected corruption. It provides a fixture where `kappa_hat == kappa_under == 131072` and `epsilon_on == 65536`. Since `kappa_under` is NOT below `epsilon_on`, the true execution path should NOT set the `NumericallyUncertain` flag. However, the test successfully ensures that the mutant will falsely trigger this flag on these valid inputs:

```rust
assert!(
    result
        .flags
        .contains(bcinr_cmca::observatory::ObservatoryFlag::NumericallyUncertain),
    "Mutant 10 (inverted kappa_under_off comparison) should falsely set \
     NumericallyUncertain when kappa_under is not below epsilon_on"
);
```
