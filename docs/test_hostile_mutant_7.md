Here is the documentation regarding `mutant_7` based on the exploration of the codebase:

### 1. Location of `mutant_7`
- **Codebase Injection**: The fault is injected in the branchless helper function `const_eq_u32` in `crates/bcinr-cmca/src/fixed.rs` (lines 114-121).
- **Test Implementation**: The dedicated oracle test `kill_mutant_7_saturating_div_false_zero` is located in `crates/bcinr-cmca/tests/hostile_mutants.rs` (lines 602-616).

### 2. Mathematical Law Broken
The mutant breaks the zero-denominator check in division. The valid branchless implementation isolates the sign bit to determine if a value is non-zero using `(x | x.wrapping_neg()) >> 31`. The `mutant_7` feature actively flips the bitwise operators to `(!x & !x.wrapping_neg()) >> 31`, causing a sign inversion. 

This corrupts the equality check `const_eq_u32`, effectively breaking the mathematical law that dividing a nonzero value (e.g., 100) by another nonzero value (e.g., 20) must NOT report a `DIVIDE_BY_ZERO` or `INVALID_DOMAIN` error. 

### 3. Expected Outcome & Typed Refusal
Because `saturating_div` branchlessly accumulates faults on its return struct (adhering to the zero-branching rule in the hot path), the operation `100 / 20` under `mutant_7` falsely rejects the operation. 

The test explicitly asserts that the typed refusal (`c.faults().bits()`) matches the exact combined refusal mask of two fault flags:
- `bcinr_cmca::fixed::NumericFaultSet::DIVIDE_BY_ZERO`
- `bcinr_cmca::fixed::NumericFaultSet::INVALID_DOMAIN`
