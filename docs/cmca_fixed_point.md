Here is the documentation for the branchless fixed-point implementation in `crates/bcinr-cmca/src/fixed.rs`.

# Branchless Q16.16 Fixed-Point Arithmetic (`fixed.rs`)

The `crates/bcinr-cmca/src/fixed.rs` file implements a mathematically rigorous, branchless Q16.16 fixed-point arithmetic system. It is specifically designed to comply with the project's deterministic substrate laws (e.g., $CC=1$, zero allocation, no panics, no data-dependent control flow).

Here is an analysis of how the core mechanics are implemented:

## 1. Sealed Representation and Fault Sets
The core numeric types (`NonNegativeFixed` and `SignedFixed`) tightly couple their scalar magnitude with an opaque `NumericFaultSet`:

```rust
pub struct NonNegativeFixed {
    val: u32,
    faults: NumericFaultSet,
}
```

By keeping the value and its faults bundled, operations naturally propagate errors through the computation chain without needing `Result`, `Option`, or early returns (all of which would introduce prohibited control flow).

- **Fault Accumulation**: `NumericFaultSet` operates as a branchless join-semilattice. Faults are merged via a bitwise union (`a.0 | b.0`). The system strictly avoids "first-error-wins" or "last-error-wins" logic. If multiple invalid domains are breached in sequence, the final fault set correctly contains the union of all faults encountered.
- **Fault Types**: Supported fault bits include `OVERFLOW`, `UNDERFLOW`, `DIVIDE_BY_ZERO`, `INVALID_DOMAIN`, `SATURATION`, `RANGE_VIOLATION`, and `APPROX_ENVELOPE`.

## 2. Branchless Control Flow (`CanonicalMask`)
Since conditionals like `if`, `match`, and early `return` are strictly forbidden under the $CC=1$ rule, logic gates are translated into arithmetic masks using `CanonicalMask`. 

- A `CanonicalMask` holds a `u32` value that is strictly enforced to be either `0` (`FALSE`) or `u32::MAX` (`TRUE`).
- **Selection Logic**: To choose between two values branchlessly, the mask uses bitwise AND/OR operations: 
  ```rust
  // select_u32 implementation:
  (a & self.0) | (b & !self.0)
  ```
- **Comparisons**: Comparisons like `<` and `==` are evaluated purely through bitwise math. For example, `const_lt_u32(a, b)` calculates differences and analyzes sign bits (`>> 31`) to produce a `CanonicalMask` of `TRUE` or `FALSE` without any CPU jump instructions.

## 3. Handling Bounds and Saturation
Rather than using `if value > max { value = max }`, the code utilizes wrapping arithmetic mapped through the `CanonicalMask` to forcefully clamp bounds and inject correct faults dynamically.

### Example: Saturating Addition
```rust
pub const fn saturating_add(self, other: Self) -> Self {
    let sum = self.val.wrapping_add(other.val);
    let overflow = const_lt_u32(sum, self.val); 
    
    // Branchlessly inject faults if an overflow mask is true
    let e = CanonicalMask::select_faults(
        overflow,
        NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
        NumericFaultSet::EMPTY,
    );
    
    Self {
        // Force the value to u32::MAX on overflow, otherwise use the wrapped sum
        val: overflow.select_u32(u32::MAX, sum),
        faults: self.faults.union(other.faults).union(e),
    }
}
```

## 4. Complex Mathematical Operations
High-order operations like division and logarithms achieve unbounded execution time equivalence by avoiding looping or dynamic estimation:

- **Division (`saturating_div`)**: 
  - Replaces denominator zeros with `1` dynamically via `d = den_is_zero.select_u32(1, other.val)`. This prevents actual hardware division traps (panics) from being triggered while processing invalid paths.
  - Computes the inverse branchlessly using four fixed, fully-unrolled iterations of a Newton-Raphson approximation sequence (`x0`, `x1`, `x2`, `x3`).
  - Conditionally merges `DIVIDE_BY_ZERO` and `INVALID_DOMAIN` into the fault mask if the input denominator mask was triggered.
- **Log2 and Exp (`log2`, `exp2`, `exp`)**: 
  - `log2` calculates leading zeros natively and executes a static set of unrolled polynomial corrections for the mantissa logic.
  - `exp2` unrolls four successive scaled multiplications (`res1` through `res4`) to evaluate the fractional piece, processing bitwise shifts via masked assignments (`ip_neg.select_u32(val_shr, val_shl)`) instead of conditional blocks.

This architecture ensures that the execution path, CPU cycle consumption, and memory overhead are strictly identical regardless of whether an input is well-formed or highly corrupt, making it mathematically deterministic and side-channel safe.
