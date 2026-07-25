Here is the requested documentation regarding `mutant_7` in `crates/bcinr-cmca/tests/hostile_mutants.rs` and the codebase:

### `mutant_7` Analysis

**1. The Mathematical Law Broken**
The underlying law being tested is the correctness of the branchless bit-parallel equality check, specifically used for the **zero-denominator invariant** in operations like `saturating_div`.

The function `const_eq_u32(a, b)` is contractually bound to return `TRUE` if and only if `a == b`, and `FALSE` otherwise. The original implementation correctly determines if the XOR of the two values (`x`) is non-zero by using `(x | x.wrapping_neg()) >> 31`.

Under `mutant_7` in `crates/bcinr-cmca/src/fixed.rs`, this law is broken via a sign inversion:
```rust
// Original branchless equality check
let nonzero = (x | x.wrapping_neg()) >> 31;

// mutant_7: Sign inversion
let nonzero = (!x & !x.wrapping_neg()) >> 31; 
```
This corruption fundamentally breaks the `const_eq_u32` primitive, causing it to falsely return true when inputs differ (falsely flagging non-zeros as zeros). 

**2. Expected Outcome / Refusal**
The expected outcome of this mutation is that mathematical operations relying on zero-checks (like division) will falsely flag valid, non-zero operations as domain violations. 

In `crates/bcinr-cmca/tests/hostile_mutants.rs`, the dedicated test oracle `kill_mutant_7_saturating_div_false_zero` proves this by running `100 / 20`. Because `mutant_7` corrupts the zero-denominator check, `saturating_div` incorrectly believes the denominator `20` is zero. 

The typed refusal/fault expected in the test assertion is:
```rust
bcinr_cmca::fixed::NumericFaultSet::DIVIDE_BY_ZERO
    .union(bcinr_cmca::fixed::NumericFaultSet::INVALID_DOMAIN)
```
The test explicitly verifies that the faulty implementation triggers `DIVIDE_BY_ZERO | INVALID_DOMAIN` for a perfectly valid mathematical operation, demonstrating that the test suite successfully traps the mutated logic.
