Here is the requested markdown document:

```markdown
# Analysis of `NumericFaultSet::INEXACT`

After thoroughly searching `crates/bcinr-cmca/src/fixed.rs` and the broader `bcinr` codebase using `grep_search` and `view_file`, I can confirm that **`NumericFaultSet::INEXACT` does not exist**. 

### Existing `NumericFaultSet` Definitions
The `NumericFaultSet` is defined in `crates/bcinr-cmca/src/fixed.rs` (lines 13-28). The currently defined bitmasks are:

```rust
impl NumericFaultSet {
    pub const EMPTY: Self = Self(0);

    pub const OVERFLOW: Self = Self(1 << 0);
    pub const UNDERFLOW: Self = Self(1 << 1);
    pub const DIVIDE_BY_ZERO: Self = Self(1 << 2);
    pub const INVALID_DOMAIN: Self = Self(1 << 3);
    pub const INVALID_NORMALIZATION: Self = Self(1 << 4);
    pub const SUPPORT_MISMATCH: Self = Self(1 << 5);
    pub const SATURATION: Self = Self(1 << 6);
    pub const APPROX_ENVELOPE: Self = Self(1 << 7);
    pub const RANGE_VIOLATION: Self = Self(1 << 8);
}
```

While "Inexact" is a standard IEEE 754 floating-point exception flag (raised when an operation's result cannot be represented exactly and requires rounding), it is intentionally absent from this deterministic fixed-point substrate. Instead, bounded mathematical approximations rely on designated fault flags like `APPROX_ENVELOPE` or `SATURATION`.

### Branchless Manipulation of Fault Bitmasks
Even though `INEXACT` does not exist, the mathematical condition for generating and propagating any `NumericFaultSet` is strictly bound to the `$CC=1$` (Cyclomatic Complexity 1) Radon Law in the BCINR framework. Here is exactly how this is achieved branchlessly:

1. **Bitwise Accumulation (`union`)**:
   Faults are never checked via short-circuiting control flow (such as "first fault wins"). Instead, they are accumulated using a total, branchless bitwise OR operation.
   ```rust
   // From crates/bcinr-cmca/src/fixed.rs
   #[inline(always)]
   pub const fn union(self, other: Self) -> Self {
       Self(self.0 | other.0)
   }
   ```

2. **Branchless State Selection (`CanonicalMask`)**:
   When algorithmic rules require conditionally selecting between fault outcomes, the framework uses a `CanonicalMask`. A canonical mask is mathematically constrained to be either all `0`s (`0x00000000`) or all `1`s (`0xFFFFFFFF`), generated without branches using wrapping negation from an LSB.
   
   The mask applies bitwise AND/NOT-AND operations to select the correct state structure:
   ```rust
   // From crates/bcinr-cmca/src/fixed.rs
   #[inline(always)]
   pub const fn select_faults(self, a: NumericFaultSet, b: NumericFaultSet) -> NumericFaultSet {
       NumericFaultSet::from_bits_raw(self.select_u32(a.0, b.0))
   }
   
   #[inline(always)]
   pub const fn select_u32(self, a: u32, b: u32) -> u32 {
       (a & self.0) | (b & !self.0)
   }
   ```
   This approach guarantees constant-time execution without data-dependent jumps, unconditionally mapping the mathematical domain properties directly into bitwise polynomials.
```
