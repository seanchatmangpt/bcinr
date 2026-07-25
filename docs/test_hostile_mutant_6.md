Here is the documentation regarding `mutant_6` in `crates/bcinr-cmca`:

### Implementation Details of `mutant_6`

In `crates/bcinr-cmca/src/fixed.rs`, `mutant_6` targets the `saturating_add` method of `NonNegativeFixed`. It conditionally alters the logic used to detect an addition overflow:

```rust
#[inline(always)]
pub const fn saturating_add(self, other: Self) -> Self {
    let sum = self.val.wrapping_add(other.val);
    
    // Correct condition: overflow occurs if the wrapped sum is less than a summand
    #[cfg(not(feature = "mutant_6"))]
    let overflow = const_lt_u32(sum, self.val);
    
    // Mutated condition: inverted overflow comparison
    #[cfg(feature = "mutant_6")]
    let overflow = const_lt_u32(self.val, sum);
    
    // ...
}
```

### Mathematical Law Broken

The mathematical law for unsigned fixed-point (or integer) addition states that for values $a$ and $b$, an overflow occurs if and only if $a + b < a$ (due to modulo arithmetic wrap-around). 

`mutant_6` breaks this law by inverting the inequality, checking instead if $a < a + b$. Since the addition of two positive, non-overflowing numbers will *always* result in a sum greater than the original number, this mutant completely breaks the mathematical property of overflow detection, causing it to falsely register an overflow on almost all valid additions.

### Expected Outcome / Refusal

When `mutant_6` is active, safe and non-overflowing arithmetic incorrectly triggers a numeric fault. 

In `crates/bcinr-cmca/tests/hostile_mutants.rs`, the test `kill_mutant_6_saturating_add_false_overflow` explicitly verifies this consequence. The test adds two small values (10 and 20) that do not overflow. It then asserts that the result's internal fault mask correctly (or rather, expectedly due to the mutant) raises the `OVERFLOW` and `SATURATION` bits in its `NumericFaultSet`:

```rust
#[cfg(feature = "mutant_6")]
#[test]
fn kill_mutant_6_saturating_add_false_overflow() {
    let a = NonNegativeFixed::from_value_bits(10);
    let b = NonNegativeFixed::from_value_bits(20);
    let c = a.saturating_add(b);
    assert_eq!(
        c.faults().bits(),
        bcinr_cmca::fixed::NumericFaultSet::OVERFLOW
            .union(bcinr_cmca::fixed::NumericFaultSet::SATURATION)
            .bits(),
        "Mutant 6 (inverted overflow comparison) should falsely report OVERFLOW|SATURATION \
         for 10 + 20, which does not actually overflow"
    );
}
```
This demonstrates the typed refusal: instead of silently computing a wrong value, the corrupted logic raises a trackable, deterministic fault bit.
