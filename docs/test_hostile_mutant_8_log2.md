Here is the documentation regarding `mutant_8` found in the codebase:

### `mutant_8` Implementation
Located in `crates/bcinr-cmca/src/fixed.rs` (lines 323-326), `mutant_8` alters the `is_zero` mask in the `NonNegativeFixed::log2` method. 

**Original Code:**
```rust
#[cfg(not(feature = "mutant_8"))]
let is_zero = const_eq_u32(self.val, 0);
```

**Mutated Code:**
```rust
#[cfg(feature = "mutant_8")]
let is_zero = const_eq_u32(0, 0); // Mutated: always true
```

### Mathematical Law Broken
The mutant violates the **admissible domain and mathematical contract** of the base-2 logarithm function. 
Mathematically, $\log_2(x)$ is only undefined for $x = 0$ (within the non-negative domain), which is the only case that should trigger an invalid domain/divide-by-zero fault. By hardcoding the `is_zero` mask to `true` (`const_eq_u32(0, 0)`), the mutant forces the algorithm to falsely conclude that *any* mathematically valid input ($x > 0$) evaluates to $0$. It breaks the domain logic and forces a constant fallback value rather than computing the true logarithm.

### Expected Outcome / Refusal
Located in `crates/bcinr-cmca/tests/hostile_mutants.rs` (in the `kill_mutant_8_log2_false_zero` test), the adversarial test supplies a strictly positive, valid input (`NonNegativeFixed::from_value_bits(100)`). 

Because of the forced mask, the function incorrectly accumulates error state. The test asserts that the resulting fixed-point value's fault mask explicitly contains the following typed refusals:
- `NumericFaultSet::DIVIDE_BY_ZERO`
- `NumericFaultSet::INVALID_DOMAIN`

This test successfully kills the mutant by proving that the mutated implementation improperly rejects a mathematically sound input and triggers a false refusal.
