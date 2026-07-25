# Bounded Error Envelope Requirements for Numeric Approximations

Under Rule 14 of the BCINR Deterministic Substrate Constitution, authoritative arithmetic must be strictly fixed-width, deterministic, bounded by a declared error envelope, and free of non-deterministic artifacts (such as NaN, infinity, or architecture-dependent rounding). 

## Required Declarations for Every Approximation

To satisfy the constitution, every numeric approximation must explicitly declare all of the following ten properties to guarantee a structurally lawful implementation:

1. **Domain**: The valid input range for the approximation.
2. **Codomain**: The exact output range (range of possible values).
3. **Maximum Absolute Error**: The strictly bounded absolute deviation from the true mathematical value.
4. **Maximum Relative Error**: The strictly bounded relative deviation from the true value.
5. **Monotonicity Result**: Proof or enforcement that the approximation preserves order.
6. **Saturation Behavior**: Explicit contract for how the function behaves when overflowing or hitting bounds.
7. **Boundary Behavior**: The exact behavior at the edges of the specified domain.
8. **Independent Reference**: An independent oracle or mathematical contract (e.g., Hoare specification, SAT/SMT bit-vector model) verifying the approximation.
9. **Mutants**: Hostile mutants to ensure tests can structurally catch deviations and trigger Typed Refusals.
10. **Object-code Audit**: Disassembly evidence proving the final generated machine code lacks branches, loops, or hidden floating-point instructions.

## The Ban on Silent Epsilon Insertion

Rule 14 explicitly states: **"No epsilon may be inserted silently."**

Silent epsilon insertion—often used casually in standard software to prevent division by zero or to handle numerical instability—is strictly banned in BCINR for several core constitutional reasons:

1. **Unwitnessed State Changes**: Adding an arbitrary constant without derivation circumvents the `@hoare_oracle` axiomatic proof requirements. The constitution demands that every semantic decision be structurally audited.
2. **Violation of Constant Visibility**: The constitution mandates that every smoothing or clamp constant must be explicitly **named, derived, admitted, and included in the influence digest**. A silent epsilon violates all four criteria.
3. **Mathematical Obfuscation**: Hiding an epsilon in code acts as an undocumented bound. It masks underlying mathematical limitations rather than explicitly defining the required valid input domain or implementing proper refusal conditions.
4. **Invalidation of the Hoare Contract**: A silently injected epsilon undermines the structural equivalence between the strict mathematical law, the independent reference, and the bounded runtime code, nullifying the validity of the exhaustive domain proof.

All edge cases must instead be handled explicitly via masked fixed-point selection, bounded numeric contracts, or bounded Typed Refusals.
