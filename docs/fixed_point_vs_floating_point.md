# Fixed-Point Mechanics vs. Floating Point in BCINR

According to Rule 14 of the BCINR `AGENTS.md` constitution, the project imposes strict deterministic and structural laws on numerical computations. This document explains the rationale behind the absolute ban on floating-point operations and details how `bcinr` implements real-number approximations securely using branchless fixed-point mechanics.

## The Ban on Floating-Point Operations

Floating-point arithmetic is strictly prohibited in the authoritative hot-path of `bcinr` for several critical reasons:

1. **Non-Determinism and Architecture Variance:** Floating-point operations can yield slightly different results depending on the underlying CPU architecture, compiler optimizations, and hardware rounding modes. BCINR mandates bit-for-bit reproducible output across all supported architectures.
2. **NaN and Infinity:** IEEE 754 floating-point introduces `NaN` (Not-a-Number) and `Infinity` states. These represent undefined or hardware-variable numeric states. Rule 14 explicitly requires arithmetic to be free of `NaN` and `Infinity`.
3. **Violation of the Radon Law ($CC=1$):** The core rule of BCINR is that the authoritative call graph must have a Cyclomatic Complexity of 1 (no branches, loops, or panics). Handling floating-point edge cases (like subnormals or exceptions) often introduces hidden control-flow branches at the hardware or compiler level.
4. **Timing Side-Channels:** Floating-point instructions can have variable execution times depending on the operand values (e.g., handling subnormal numbers), violating the constitutional requirement for execution work to be strictly bounded and constant-time.

## Deterministic Fixed-Point Mechanics

To support real-number arithmetic without floating-point, `bcinr` employs **Q16.16 Fixed-Point Arithmetic** (16 bits for the integer part, 16 bits for the fractional part), representing a value `v` as `v * 65536`. This is implemented entirely with standard integer types (e.g., `u32`, `i32`) and branchless bitwise polynomials.

### Handling Approximations and Explicit Bounds

When exact arithmetic is mathematically impossible (such as for reciprocals, logarithms, or exponentials), `bcinr` allows mathematical approximations, provided they strictly follow Rule 14's rigorous criteria:

1. **Declared Error Envelopes:** 
   An approximation primitive cannot simply be "close enough." It must declare a rigorous error envelope:
   - **Domain & Codomain**: The exact bounds of valid inputs and their corresponding outputs.
   - **Error Bounds**: Explicitly quantified maximum absolute error and maximum relative error.
   - **Behavioral Proofs**: Formal guarantees of the function's monotonicity, saturation behavior, and edge-case/boundary handling.

2. **Branchless Algorithms:**
   Functions are implemented using pure integer mathematics and bit-parallel masking, replacing control flow with arithmetic selection:
   - **Reciprocal / Square Root:** Uses fixed iterations of the Newton-Raphson method starting from bit-shift seeds. Bounded loops are completely macro-unrolled to eliminate data-dependent loop termination.
   - **Logarithm:** Extracts the integer component directly from the position of the leading bit (`leading_zeros`) and computes the fractional part using a linear interpolation on the normalized mantissa.
   - **Exponentiation:** Shifts the integer part and uses unrolled polynomial approximations for the fractional part.

3. **Explicit Constants and Clamps:**
   "Magic numbers" or silent epsilons (`CHEAT-003`) are constitutional violations. Any constant used for smoothing or clamping must be strictly named, derived mathematically, formally admitted, and included in the influence digest to cryptographically track any algorithmic tuning parameters.

4. **Saturating/Wrapping Contracts and Fault Propagation:**
   Errors such as overflow or division-by-zero do not panic or use `Result`-based short-circuiting.
   - Arithmetic explicitly wraps or saturates via bitwise selections based on mathematical contracts.
   - Anomalies are branchlessly recorded in a `NumericFaultSet` that unions faults (e.g., `OVERFLOW`, `DIVIDE_BY_ZERO`) deterministically alongside the output values. This ensures that a rejected operation leaves the persistent state byte-for-byte unchanged.

### Verification (The Substrate Integrity Score)
Every approximation must score 100/100 on the maturity matrix by including:
* An **independent mathematical oracle** (not derived from the implementation itself).
* **Hostile mutants** to verify typed refusals and error propagation.
* A **production-profile disassembly audit** to guarantee that no conditional branch, panic path, or floating-point instruction was linked into the hot-path object code.
