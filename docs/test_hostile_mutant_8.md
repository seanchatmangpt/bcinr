Here is the requested analysis for `mutant_8` in the `bcinr` codebase.

### Locations
- **Implementation**: `crates/bcinr-cmca/src/fixed.rs` (around lines 323-326) inside the `NonNegativeFixed::log2` method.
- **Hostile Test**: `crates/bcinr-cmca/tests/hostile_mutants.rs` (lines 618-642) as `kill_mutant_8_log2_false_zero()`.

### Code Context
In `src/fixed.rs`, the unmodified codebase checks if the input to `log2` is `0` by constructing a mask using `const_eq_u32(self.val, 0)`. `mutant_8` replaces this with an always-true constant:
```rust
#[cfg(not(feature = "mutant_8"))]
let is_zero = const_eq_u32(self.val, 0);
#[cfg(feature = "mutant_8")]
let is_zero = const_eq_u32(0, 0); // Mutated: always true
```

### Mathematical Law Broken
**Logarithmic Domain Adherence**: The mathematical domain for `log2(x)` is strictly `x > 0`. A bounded, deterministic computational substrate must strictly differentiate between values in the valid domain and `0` to apply correct refusal semantics. 

By forcing `is_zero` to always be true, `mutant_8` breaks the mathematical distinction between valid positive numbers and zero. It essentially tells the runtime that *every* input is zero, meaning even valid, strictly positive numbers will be treated as domain violations.

### Expected Outcome / Refusal
When testing `mutant_8` with the hostile fixture `kill_mutant_8_log2_false_zero()`, the adversarial test feeds in a strictly positive, mathematically valid value (`NonNegativeFixed::from_value_bits(100)`). 

The expected outcome is that the system **falsely refuses** this valid input by producing the typed refusal faults:
```rust
NumericFaultSet::DIVIDE_BY_ZERO | NumericFaultSet::INVALID_DOMAIN
```
The test explicitly asserts that these specific faults are raised for `log2(100)`, proving that the mutant destroyed the domain-checking law.
