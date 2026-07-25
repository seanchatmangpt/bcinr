Here is the documentation on how `NumericFaultSet::INVALID_DOMAIN` is set branchlessly and what it represents:

```markdown
# Branchless Setting of `NumericFaultSet::INVALID_DOMAIN`

In the `bcinr-cmca` crate (`crates/bcinr-cmca/src/fixed.rs`), the `NumericFaultSet::INVALID_DOMAIN` bitmask is set branchlessly to flag mathematical operations that are invoked on values outside their valid mathematical domain. 

It is triggered in two specific operations:

### 1. Mathematical Conditions
- **Division by Zero (`saturating_div`)**: 
  Triggered when the denominator (`other.val`) is exactly 0. Mathematically, division by zero is undefined, which violates the operation's domain.
- **Logarithm of Zero (`log2`)**:
  Triggered when the input value (`self.val`) is exactly 0. The logarithm is undefined for $x \le 0$ (for these non-negative fixed-point numbers, the out-of-bounds value is precisely 0).

In both scenarios, the `INVALID_DOMAIN` fault is explicitly combined with the `DIVIDE_BY_ZERO` fault via a bitwise union.

### 2. Branchless Implementation Mechanics
The implementation perfectly adheres to the $CC=1$ architectural law by avoiding all `if` or `match` statements. The bitmask is evaluated and assigned using purely bitwise mask logic:

**Step A: Mask Generation**  
A condition is evaluated into a `CanonicalMask` (which structurally guarantees a value of either all 1s `0xFFFFFFFF` or all 0s `0x00000000`) using the branchless `const_eq_u32` primitive.
```rust
// In saturating_div:
let den_is_zero = const_eq_u32(other.val, 0);

// In log2:
let is_zero = const_eq_u32(self.val, 0);
```

**Step B: Branchless Selection**  
The mask is passed to `CanonicalMask::select_faults`, which uses bitwise logic (equivalent to `(mask & true_val) | (~mask & false_val)`) to choose the appropriate fault set without control-flow divergence.
```rust
// From log2:
let e = CanonicalMask::select_faults(
    is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    NumericFaultSet::EMPTY,
);
```

**Step C: Fault Accumulation**  
The newly calculated fault set (`e`) is accumulated into the state's existing faults via `.union()` (a bitwise OR operation). The struct is then reconstructed unconditionally.
```rust
faults: self.faults.union(e),
```

This strict composition of polynomial/bitwise instructions ensures that the CPU takes the exact same execution time and path whether the domain condition is valid or invalid.
```
