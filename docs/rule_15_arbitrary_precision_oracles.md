Based on the `AGENTS.md` Constitution, here is an explanation of why "arbitrary-precision implementations" are listed as a permitted independent form for the Oracle under Rule 15, and how they differ from the production implementation:

### 1. Guaranteed Structural and Logical Distinction
Rule 15 mandates that an oracle must be "structurally and logically distinct" from the production code to prevent circular validation (such as a line-by-line translation or reuse of production helpers). 

The authoritative production implementation is governed by strict, absolute runtime laws (Rule 3 and 14). It must be:
- Allocation-free (`zero heap allocation`, `#![no_std]`)
- Branchless (`CC=1`, `no data-dependent branches` or loops)
- Fixed-width (fixed memory access, fixed-width arithmetic)

Arbitrary-precision implementations fundamentally contrast with these rules. They require dynamic memory allocation (the heap) to accommodate growing digit arrays and rely heavily on data-dependent branching and loops (e.g., for carry propagation and normalization). Because an arbitrary-precision algorithm inherently violates the production runtime's structural laws, it physically cannot be a copy of the production code. This guarantees true structural independence.

### 2. Establishing an Exact Mathematical Reference
According to Rule 14 (Numeric-law requirements), authoritative arithmetic is fixed-width and bounded by a "declared error envelope" (often employing approximations, fixed-point math, and specific saturation limits). 

An arbitrary-precision implementation evaluates the algorithm in a pure, unconstrained mathematical space, free from fixed-width truncation, architecture-specific rounding, or bitwise limitations. It computes the "exact" answer. This exactness provides an unquestionable ground truth (the Oracle) to independently verify that the production code's SWAR, bitwise polynomials, and fixed-width approximations stay safely within their certified error envelopes and obey their Hoare contracts.
