# The `NonZeroFixed` Typestate in `bcinr`

In the `bcinr` architecture, the `NonZeroFixed` struct leverages Rust's type system to enforce strict domain invariants at compile time, eliminating the need for redundant runtime zero-checks in branchless execution paths.

## 1. Encapsulation and Sealed Representation
The `NonZeroFixed` typestate encapsulates a non-zero Q16.16 fixed-point number alongside its `NumericFaultSet`:

```rust
pub struct NonZeroFixed {
    val: u32,
    faults: NumericFaultSet,
}
```

The fields are deliberately private (a "sealed" representation). Consumers are entirely prevented from manually instantiating or modifying the internal `val` or `faults`. The type can only be constructed via trusted constructors (e.g., `try_from`, `ONE`) or as the output of mathematically guaranteed non-zero operations. This guarantees that:
1. The value strictly represents a non-zero magnitude ($x > 0$ or $x \neq 0$).
2. The value and its execution fault history (e.g., `OVERFLOW`, `RANGE_VIOLATION`) are strictly bound together and cannot be decoupled.

## 2. Formal Encoding of State Invariants
By restricting the domain at instantiation, the invariant $x \neq 0$ is baked into the type itself.

In the broader API, `bcinr-cmca` utilizes typestates to mathematically type-check operations that would otherwise require runtime masking. For instance, creating a denominator for fixed-point division maps from a generic numerical domain to a guaranteed non-zero domain. 
Functions enforcing domain-specific invariants can emit `NonZeroFixed` to prove to the compiler that the value is structurally safe for division.

## 3. Mathematically Bypassing Branchless Zero Checks
Because the non-zero invariant is enforced statically, the arithmetic logic within `NonZeroFixed` mathematically bypasses runtime branchless checks (like generating canonical boolean masks for zero detection) that would otherwise burn cycles on the hot path.

**Example: The Division Denominator Check**
In standard `NonNegativeFixed` arithmetic, `saturating_div` must guard against divide-by-zero by creating canonical masks and manipulating the denominator fallback:

```rust
// Standard division in NonNegativeFixed
let den_is_zero = const_eq_u32(other.val, 0);
let d = den_is_zero.select_u32(1, other.val);

// ... fault accumulation ...
let e = CanonicalMask::select_faults(
    den_is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    // ...
);
```

By binding the denominator to `NonZeroFixed`, the compiler mathematically guarantees $x \neq 0$. The implementation completely omits the `const_eq_u32(other.val, 0)` mask generation and the subsequent fallback substitution, reducing the operation's complexity:

```rust
impl NonNegativeFixed {
    pub const fn saturating_div_nonzero(self, other: NonZeroFixed) -> Self {
        // Zero-check mathematically elided
        let d = other.val;
        
        let lz = d.leading_zeros();
        let d_norm = d << lz;
        // ... proceeds without DIVIDE_BY_ZERO fault accumulation ...
    }
}
```

**Example: The Logarithm Domain Check**
The logarithm function requires $x > 0$. By binding `log2` to a strictly positive `NonZeroFixed`, the function omits the zero-check mask and its associated invalid domain fault logic.

```rust
impl NonZeroFixed {
    pub fn log2(self) -> SignedFixed {
        // Zero-check elided entirely
        let x = self.val as u64;
        let lz = x.leading_zeros();
        // ...
        
        // No CanonicalMask::select_faults for DIVIDE_BY_ZERO needed
        SignedFixed::from_parts(computed, self.faults)
    }
}
```

## Summary
The `NonZeroFixed` typestate embodies the core `bcinr` design philosophy: "Rich semantics upstream, fixed deterministic mechanics downstream." By shifting the $x \neq 0$ invariant to the type system, it mathematically strips away the need for runtime zero-bounds masking, accelerating branchless execution paths while preserving the rigorous `CC=1` and allocation-free runtime laws.
