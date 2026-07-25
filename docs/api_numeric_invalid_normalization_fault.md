Here is the markdown documenting how `NumericFaultSet::INVALID_NORMALIZATION` is set branchlessly and what it represents:

```markdown
# `NumericFaultSet::INVALID_NORMALIZATION`

The `INVALID_NORMALIZATION` bitmask is defined in `crates/bcinr-cmca/src/fixed.rs` and is utilized within the resource allocator (`crates/bcinr-cmca/src/allocator.rs`) to denote a mathematically undefined normalization state.

## Mathematical Condition

The fault represents a scenario where the total sum of weights or probabilities required for normalization evaluates exactly to `0`. 

Specifically, in `allocator.rs`, this occurs when evaluating the exponential cost penalty for resource allocation across a set of leaf nodes. The substrate computes a partial weight `p` for each leaf:
`p = pi_combined[x & 7] * neg_mu_cost.exp()`

It accumulates these into `priced_sum`. If `priced_sum == 0`, normalizing the individual leaf probabilities `p / priced_sum` mathematically requires a division by zero, which is undefined.

## Branchless Enforcement

To comply with the strict branchless $CC=1$ totality laws (where the function must execute without `if`, panics, or early returns), the fault is recorded and mitigated using bitwise polynomials:

1. **Condition Evaluation:** 
   The code checks if the sum is zero using a constant-time bit-parallel equality function which returns a `u32` (1 if true, 0 if false):
   ```rust
   let priced_sum_is_zero = const_eq_u32(priced_sum.value_bits(), 0);
   ```

2. **Fault Accumulation:** 
   The 0 or 1 is expanded into a full bitmask (`CanonicalMask`) using `from_lsb`. It then branchlessly selects either `INVALID_NORMALIZATION` or `EMPTY` (using bitwise AND/OR) and accumulates it into the fault lattice via `.union()` (bitwise OR):
   ```rust
   local_numeric_faults = local_numeric_faults.union(
       CanonicalMask::from_lsb(priced_sum_is_zero).select_faults(
           NumericFaultSet::INVALID_NORMALIZATION,
           NumericFaultSet::EMPTY,
       )
   );
   ```

3. **Safe Substitution:** 
   Instead of early-returning or panicking on division-by-zero, the substrate substitutes a safe denominator (`1.0` represented as `NonNegativeFixed::ONE`) using a branchless ternary selection (`select_nnf`). This allows the execution path to complete unconditionally while carrying the fault flag:
   ```rust
   let psd = select_nnf(priced_sum_is_zero, NonNegativeFixed::ONE, priced_sum);
   ```
```
