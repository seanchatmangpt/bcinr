Here is the documentation regarding `mutant_8`:

### `mutant_8` Documentation

**File Locations:**
- Test file: `crates/bcinr-cmca/tests/hostile_mutants.rs`
- Implementation file: `crates/bcinr-cmca/src/fixed.rs`

**Mutation Details:**
In `crates/bcinr-cmca/src/fixed.rs` within the `log2()` function, the original zero-check mask is replaced with a forced true mask.
- Original: `let is_zero = const_eq_u32(self.val, 0);`
- Mutated: `let is_zero = const_eq_u32(0, 0); // Mutated: always true`

**Mathematical Law Broken:**
The implementation violates the domain validity law for logarithmic functions. Specifically, `log2(x)` must only report domain faults when $x = 0$ (or $x < 0$, which is impossible for `NonNegativeFixed`). By forcing `is_zero` to always be true, the mutant breaks the law that `log2()` of a valid, nonzero value must NOT report `DIVIDE_BY_ZERO` or `INVALID_DOMAIN` faults.

**Expected Outcome/Refusal:**
Because the `is_zero` mask evaluates to true regardless of the input, the bounded arithmetic correctly applies the masked fault. As a result, when evaluating a valid expression like `NonNegativeFixed::from_value_bits(100).log2()`, the resulting object incorrectly carries the combined fault bits for `DIVIDE_BY_ZERO` and `INVALID_DOMAIN` (which triggers a typed refusal/fault when inspected), falsely rejecting a valid nonzero operand. The hostile test `kill_mutant_8_log2_false_zero` asserts this exact incorrect fault set is generated.
