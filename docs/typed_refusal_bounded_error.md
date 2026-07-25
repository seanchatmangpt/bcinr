# Rule 18: Bounded Typed Refusals without Panicking

I explored the `bcinr` codebase, specifically the `bcinr-cmca` crate, to investigate how `StabilityRefusal` and `ObservatoryFlag` are structured to guarantee bounded typed refusals while complying with the strict branchless and zero-panic runtime laws (Rule 18).

Here is a breakdown of their structural approach:

## 1. Bounded Enumerations over Text
Both types (`StabilityRefusal` and `ObservatoryFlag`) are defined as simple `#[derive(Copy, Clone, Debug, PartialEq, Eq)]` enumerations containing precise failure categories (e.g., `StabilityRefusal::CertificateMissing`, `ObservatoryFlag::NumericallyUncertain`, `StabilityRefusal::ContractViolation`). This completely avoids allocating human-readable strings, panicking, or returning plausible default states in the hot path. 

## 2. Branchless Instantiation via SWAR Lookups
Instead of `match` statements or `if-else` branches, both types implement a `from_u32(val: u32) -> Option<Self>` method that operates in $O(1)$ constant time without any branches:
- They use constant-time bounded checking primitives like `const_lt_u32` and `const_select_u32`.
- They read from a fixed-size array lookup table (e.g., `const REFUSALS: [StabilityRefusal; 32]`), safely indexing it with a bitwise mask (e.g., `idx & 31` or `idx & 7`) to guarantee bounded execution without a panic-inducing bounds check.

## 3. Branchless Accumulation via Bitsets
To prevent control flow interruptions (like "first-error-wins" early returns), these typed refusals are never returned directly during intermediate computations. Instead, conditions map to specific bits in opaque bitset wrappers (`NumericFaultSet`, `RefusalSet`, and `ObservatoryFlagSet`).
- Multiple failure conditions can co-occur. They are accumulated completely branchlessly using a `union` method: `pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }`
- Using `CanonicalMask` and `select_faults`, the substrate applies condition selections securely without control flow divergence. 

## 4. API Boundary Projection (`primary_flag` and `into_result`)
After the branchless hot path completes execution, the outcome struct (e.g., `AllocationOutcome` or `ObservatoryOutcome`) bundles the results and the refusal/fault sets together. 
- For callers needing a single standard typed refusal, projection methods such as `RefusalSet::primary_reason()` or `ObservatoryFlagSet::primary_flag()` are used to collapse the full sets down into a single `StabilityRefusal` or `ObservatoryFlag`.
- A wrapper method `into_result(self) -> Result<T, StabilityRefusal>` is provided at the API boundary, bridging the strictly branchless operations with standard, typed idiomatic Rust error handling, completely avoiding panics, side-effects, or speculative partial state mutation.
