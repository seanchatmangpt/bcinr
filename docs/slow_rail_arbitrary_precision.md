# Slow Rail's Arbitrary Precision Oracle in BCINR

## 1. The Discrepancy: `BigInt` vs. `f64`
According to the **Independence Protocol** (`docs/independent_oracles.md`), an **Arbitrary-Precision Implementation** (using types like `BigInt` or `BigRational`) is explicitly listed as one of the permitted forms for `@hoare_oracle` to construct an independent mathematical specification. 

However, a direct inspection of the `bcinr` codebase reveals that **neither `rug` nor `num-bigint` are actually used** for this purpose. 

Instead, the Slow Rail's "arbitrary precision" oracle for fixed-point differential testing is implemented exclusively using **`f64` (double-precision floating-point) arithmetic**. While `f64` is not strictly "arbitrary precision" in the computer science sense, the project terminology (e.g., `docs/numeric_error_envelope.md`) explicitly refers to it as the *"arbitrary-precision `f64` reference oracle"* because its 53-bit mantissa vastly exceeds the precision of the Q16.16 fixed-point math used on the deterministic Hot Path.

## 2. Construction of the Independent Specification
The baseline truth for the fixed-point allocation engine is constructed in `crates/bcinr-cmca/tests/reference.rs`. Here is how `@hoare_oracle` establishes it without violating the Mandatory Decomposition Protocol:

### A. Explicit Structural Isolation (`CHEAT-002` Prevention)
The reference oracle (e.g., `allocate_f64`) is written as a direct, unoptimized mathematical formula on the **Slow Rail**. It intentionally retains explicit index loops (`for i in 0..N`) and manual clamps, and strictly avoids using any of the production normalization tables, bit-level masks, or SWAR (SIMD Within A Register) fixed-point helpers that are mandatory on the branchless Hot Path. This satisfies the rule that the oracle must be structurally and logically distinct.

### B. Floating-Point Baseline
It takes standard `f64` inputs (scaled from the Q16.16 `PackedSemanticState`) and performs all aggregation, exponentiation, and q-norm math using standard library float operations (`.powf()`, `.exp()`, `.log2()`). This provides the baseline representing the true mathematical intent.

## 3. Fixed-Point Differential Testing
The baseline truth is enforced against the production code in `crates/bcinr-cmca/tests/differential.rs`. 

1. **Hostile Proptesting**: The `proptest` framework generates thousands of structurally complex, valid permutations of input parameters (e.g., factors, lens exponents, parent structures).
2. **Parallel Execution**: The exact same inputs are fed into both the Hot Path branchless Q16.16 allocator (`allocate`) and the Slow Rail `f64` reference oracle (`allocate_f64`).
3. **Error Envelope Verification**: The Q16.16 outputs are converted to `f64` and compared against the reference. The test asserts that the absolute difference $\max |E_{\text{abs}}|$ is bounded by the exact **Numeric Error Envelope**. For example:
   - For complex tree allocations, the differential test asserts the mismatch is $< 0.22$.
   - For the Real Conformance Metric Estimators (RCME), the deviation from the `f64` oracle is bounded by $\le 1.5 \times 2^{-16}$ (as defined in `docs/innovations/real_conformance_metric_estimators.md`).

By using `f64` on the Slow Rail, `@hoare_oracle` maintains strict logical independence from the branchless bitwise polynomials of the Hot Path. This fulfills the requirement of the Mandatory Decomposition Protocol (proving that the fixed-point logic correctly models the continuous mathematical formula) without incurring the heavy build dependency cost of true arbitrary-precision crates like `rug`.
