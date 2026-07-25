Here is the analysis of `mutant_7` as requested:

### Location
The mutant is implemented in `crates/bcinr-cmca/src/fixed.rs` within the `const_eq_u32` function, and its oracle test is located in `crates/bcinr-cmca/tests/hostile_mutants.rs`.

### Implementation of `mutant_7`
In `crates/bcinr-cmca/src/fixed.rs`, the `const_eq_u32` function is designed as a branchless equality check. `mutant_7` introduces a sign inversion when checking if the XOR difference of the two numbers is non-zero.

**Original:**
```rust
let nonzero = (x | x.wrapping_neg()) >> 31;
```

**Mutated (`mutant_7`):**
```rust
let nonzero = (!x & !x.wrapping_neg()) >> 31; // Mutated: sign inversion
```

### Mathematical Law Broken
This mutation breaks the mathematical law that **dividing a number by a nonzero denominator must NOT report a division-by-zero or invalid-domain fault**. By corrupting the `const_eq_u32` check, the zero-denominator check used by `saturating_div` evaluates incorrectly, causing nonzero denominators to be treated as zero.

### Expected Outcome / Refusal
The expected refusal is explicitly tested by `kill_mutant_7_saturating_div_false_zero` in `crates/bcinr-cmca/tests/hostile_mutants.rs`. 

When performing `100.saturating_div(20)`, the corrupted zero-denominator check causes the system to falsely report faults. The expected outcome is that the resulting value's fault mask exactly equals:
```rust
bcinr_cmca::fixed::NumericFaultSet::DIVIDE_BY_ZERO
    .union(bcinr_cmca::fixed::NumericFaultSet::INVALID_DOMAIN)
    .bits()
```
