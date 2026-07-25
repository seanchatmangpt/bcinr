### `mutant_5` Inspection Report

**Location:** 
- Implementation: `crates/bcinr-cmca/src/allocator.rs` (lines 1901-1904, 1958-1961)
- Test/Verification: `crates/bcinr-cmca/tests/hostile_mutants.rs` (`kill_mutant_5_consequence_truncation` function)

**Broken Mathematical Law:**
`mutant_5` breaks the **consequence-mass truncation law**. 
In the authoritative implementation, the consequence array `mu` must be strictly bounded/clamped to its admitted range `[0, mu_max]` before it is used to price leaf costs. When `mutant_5` is enabled, it explicitly skips this mathematical boundary enforcement and uses the raw, unclamped `mu` values:

```rust
// In bcinr-cmca/src/allocator.rs:
#[cfg(feature = "mutant_5")]
let mu_actual = mu[x & 7];

#[cfg(not(feature = "mutant_5"))]
let mu_actual = clip(mu[x & 7], NonNegativeFixed::ZERO, mu_max);
```

**Expected Outcome / Refusal:**
The hostile test fixture (`kill_mutant_5_consequence_truncation`) supplies a deliberately negative/out-of-bounds `mu`. 

Because `mutant_5` skips the clipping procedure, the pricing logic computes based on invalid parameters, leading to a specific, mathematically compromised outcome: **a corrupted allocation array**. 
- It fails to compute the intended clamped baseline (`CORRECT_MU_COST`). 
- Instead, the unhandled input precisely forces the calculation to yield the `CORRECT_BASELINE` array under these specific fixture conditions.

The test kills the mutant by ensuring that the generated allocation state stringently equals this known-corrupt signature (`CORRECT_BASELINE`) and strictly diverges from the correct mathematical law outcome (`CORRECT_MU_COST`).
