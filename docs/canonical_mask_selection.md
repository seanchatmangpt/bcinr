# CanonicalMask and Branchless Selection in bcinr

In the deterministic `bcinr` substrate, `CanonicalMask` is a core building block that enables completely branchless execution of conditional logic, satisfying the absolute runtime laws defined in `AGENTS.md` (no data-dependent branches, $CC=1$).

## Secure Generation of CanonicalMask

A `CanonicalMask` is an abstraction representing a boolean predicate as a full-width bitmask. Rather than using Rust's `bool` (which can induce branching), it uses a sealed `u32` inner value where:
- `TRUE` is represented by all ones (`0xFFFFFFFF` or `u32::MAX`).
- `FALSE` is represented by all zeros (`0x00000000`).

### Design Security Features
1. **Private Representation:** `pub struct CanonicalMask(u32);` hides the inner `u32`. It can only be constructed using the safe, branchless constructors provided in the same module, guaranteeing that a `CanonicalMask` will *only* ever contain `0xFFFFFFFF` or `0x00000000` (Invariant 3).
2. **LSB Expansion (`from_lsb`):** The simplest constructor takes a 1-bit value (0 or 1) and expands it securely via wrapping arithmetic:
   ```rust
   pub const fn from_lsb(lsb: u32) -> Self {
       Self(0u32.wrapping_sub(lsb & 1))
   }
   ```
   If `lsb & 1` is `1`, `0 - 1` underflows to `0xFFFFFFFF`. If it's `0`, `0 - 0` is `0x00000000`. No branches are needed.
3. **Branchless Comparators:** More complex masks are built using bit-parallel comparator functions without any `if` statements. For example, `const_eq_u32` generates a mask strictly using bitwise arithmetic and sign extraction:
   ```rust
   pub const fn const_eq_u32(a: u32, b: u32) -> CanonicalMask {
       let x = a ^ b;
       // Any non-zero bits will set the MSB (sign bit) when OR'ed with its negation
       let nonzero = (x | x.wrapping_neg()) >> 31;
       CanonicalMask(0u32.wrapping_sub(1u32.wrapping_sub(nonzero)))
   }
   ```

## Branchless Selection: `.select(a, b)`

Once a `CanonicalMask` is securely generated, it replaces `if-else` blocks entirely using bitwise polynomial multiplexing.

### The Selection Logic
The method `select_u32` evaluates condition branches concurrently and merges the results purely through bit-wise AND (`&`) and OR (`|`):

```rust
pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```

- **If the mask is `TRUE` (`0xFFFFFFFF`):**
  - `self.0` is `0xFFFFFFFF`.
  - `!self.0` is `0x00000000`.
  - The equation becomes `(a & 0xFFFFFFFF) | (b & 0x00000000)`.
  - This simplifies to `a | 0 = a`.

- **If the mask is `FALSE` (`0x00000000`):**
  - `self.0` is `0x00000000`.
  - `!self.0` is `0xFFFFFFFF`.
  - The equation becomes `(a & 0x00000000) | (b & 0xFFFFFFFF)`.
  - This simplifies to `0 | b = b`.

### Ecosystem Integration
`CanonicalMask` extends beyond base types, operating on fixed-point numbers and state representations. For example, when performing arithmetic that might trigger an error or underflow, the system seamlessly applies the same mask to select faults, tracking success or failure without violating branchlessness:
```rust
pub const fn select_faults(self, a: NumericFaultSet, b: NumericFaultSet) -> NumericFaultSet {
    NumericFaultSet::from_bits_raw(self.select_u32(a.0, b.0))
}
```
This guarantees that control flow is completely flattened into a constant sequence of mathematical operations, rendering the algorithm immune to timing side-channels and structurally compliant with the project's rigid $CC=1$ deterministic mandate.
