# Hostile Mutation Protocol & Typed Refusals (Rule 19)

### 1. The Rule 19 Mandate
According to `AGENTS.md` Rule 19 (and Rule 4 `@armstrong_fault`), adversarial testing in BCINR forbids generic inequality assertions like `assert_ne!(baseline, mutant)` as the sole proof of detection. A surviving mutant invalidates the project standing. The protocol demands that tests prove a mutant triggers a **specific typed refusal** or a known deterministic mathematical corruption state. 

### 2. Implementation of Mutants
`hostile_mutants.rs` injects faults via two primary mechanisms:
- **Build-Time Corruptions (via `cfg(feature="mutant_X")`)**: These test lower-level production logic directly (e.g., inverting an overflow check, swapping operations, breaking normalization). 
- **Argument-Level Substitutions**: Functions like `evaluate_m01` test independent measurement rejection by explicitly feeding in corrupted artifacts (e.g., passing a point estimate instead of a lower bound, zeroing out drift).

### 3. Structural Trapping via Typed Refusals
The tests enforce the protocol by verifying bit-exact responses to faults rather than panicking or checking for generic differences. For example:

**Numeric Level Typed Refusals (`NumericFaultSet`)**
- **Mutant 6 (False Overflow)**: Inverts the overflow check in `saturating_add`. The test does not just assert that the addition was wrong; it verifies that the returned bitmask precisely reports an overflow and saturation refusal: `c.faults().bits() == NumericFaultSet::OVERFLOW | NumericFaultSet::SATURATION`.
- **Mutant 7 (False Zero Div)**: Breaks the zero-denominator check in `saturating_div`. It is structurally trapped by explicitly checking for the `DIVIDE_BY_ZERO | INVALID_DOMAIN` typed refusal.
- **Mutant 8 (False Log2 Zero)**: Trapped by `DIVIDE_BY_ZERO | INVALID_DOMAIN` when `is_zero` is forced to true.

**Logic / State Typed Refusals (`ObservatoryFlag`)**
- **Mutants 9, 10, 11 (False Conditionals)**: Invert logic checks (e.g., drift or under-bound checks). The tests assert that specific flags like `ObservatoryFlag::Drifting`, `ObservatoryFlag::NumericallyUncertain`, or `ObservatoryFlag::GramDegenerate` are falsely set.
- **Mutants M01-M07**: Substitute invalid parameters. The tests assert that the exact flag expected is erased because the true conditional failed due to the corruption.

**Deterministic State Corruptions**
For mathematical routines like allocation mixing where a single bit-flag isn't the primary output (meaning the mutant produces a wrong accepted value rather than a refusal), the tests compare against pre-computed, deterministic corruption arrays rather than relying only on `assert_ne!`:
- **Mutant 1 (Measure Collapse)**: Must exactly equal `WRONG_M1_MEASURE_COLLAPSE`.
- **Mutant 2 (Sign Inversion)**: Must exactly equal `WRONG_M2_Q_SIGN_INVERSION`.
- **Mutant 3 (Broken Normalization)**: Must exactly equal `WRONG_M3_BROKEN_NORMALIZATION`.
- **Mutant 4 (Identity Skew)**: Must exactly equal `WRONG_M4_RDF_IDENTITY_SKEW`.

### Conclusion
By mapping structural faults to exact bit-level refusals (`NumericFaultSet`, `ObservatoryFlag`) or explicit error states, `hostile_mutants.rs` guarantees that the system's deterministic, branchless mechanics fail exactly as predicted when underlying mathematical laws are violated.
