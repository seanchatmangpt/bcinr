# The `NonNegativeFixed` Typestate in `bcinr`

In the `bcinr` architecture, the `NonNegativeFixed` struct leverages Rust's type system to enforce strict domain invariants at compile time, eliminating the need for redundant runtime bounds checks in branchless execution paths.

## 1. Encapsulation and Sealed Representation
The `NonNegativeFixed` typestate encapsulates a non-negative Q16.16 fixed-point number (stored as an unsigned `u32`) alongside its `NumericFaultSet`:

```rust
pub struct NonNegativeFixed {
    val: u32,
    faults: NumericFaultSet,
}
```

The fields are deliberately private (a "sealed" representation). As demonstrated by UI compiler tests like `fail_field_construct_non_negative_fixed.rs` and `fail_struct_update_non_negative_fixed.rs`, consumers are entirely prevented from manually instantiating or modifying the internal `val` or `faults`. The type can only be constructed via trusted constructors (e.g., `ZERO`, `from_value_bits`, `from_num`) or as the output of defined arithmetic operations. This guarantees that:
1. The value cannot natively represent a negative number.
2. The value and its execution fault history (e.g., `OVERFLOW`, `INVALID_DOMAIN`) are strictly bound together and cannot be decoupled.

## 2. Formal Encoding of State Invariants
By utilizing an unsigned `u32` rather than an `i32`, the invariant $x \ge 0$ is baked into the type itself. *(Note: It guarantees the number is non-negative rather than strictly positive, as `ZERO` is a valid state).*

In the broader API, `bcinr-cmca` heavily utilizes this to type-check mathematical transformations. For example, exponential operations map the domain of all real numbers to non-negative reals. In code, `SignedFixed` implements:
```rust
pub fn exp2(self) -> NonNegativeFixed
pub fn exp(self) -> NonNegativeFixed
```
By mapping the return type to `NonNegativeFixed`, the compiler formally understands that the output of an exponentiation is guaranteed non-negative. 

## 3. Mathematically Bypassing Branchless Bounds Checks
Because the non-negative invariant is enforced statically, the arithmetic logic within `NonNegativeFixed` mathematically bypasses runtime checks that would otherwise burn cycles on branchless mask generation. 

**Example: The Logarithm Domain Check**
The logarithm function has a domain of $x > 0$. If `log2` were implemented on `SignedFixed`, it would need to generate branchless masks for two failure modes: $x < 0$ and $x == 0$.
```rust
impl NonNegativeFixed {
    pub fn log2(self) -> SignedFixed { ... }
}
```
By binding `log2` exclusively to `NonNegativeFixed`, the compiler mathematically guarantees $x \ge 0$. The implementation inside `src/fixed.rs` completely omits the masking logic for negative inputs, keeping the hot path lean. It only generates a single fault mask for `const_eq_u32(self.val, 0)`:
```rust
let is_zero = const_eq_u32(self.val, 0);
let e = CanonicalMask::select_faults(
    is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    NumericFaultSet::EMPTY,
);
```

**Example: Saturation Bounding**
When performing addition in fixed-point math, an overflow must saturate to the maximum or minimum bound. For `SignedFixed`, `saturating_add` must compute `is_neg = const_lt_i32(self.val, 0)` to dynamically select between `i32::MAX` and `i32::MIN`.

For `NonNegativeFixed`, the state is guaranteed non-negative. Its `saturating_add` bypasses the sign check entirely and statically saturates to `u32::MAX`:
```rust
let overflow = const_lt_u32(sum, self.val);
Self {
    val: overflow.select_u32(u32::MAX, sum),
    // ...
}
```

## Summary
The `NonNegativeFixed` typestate embodies the `bcinr` design philosophy: "Rich semantics upstream, fixed deterministic mechanics downstream." By shifting the $x \ge 0$ invariant to the type system, it mathematically strips away the need for runtime negative-bounds masking, accelerating branchless execution paths while preserving the rigorous `CC=1` and `no panic` constraints.
