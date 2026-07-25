# Numeric-Law Requirements for Division Approximations

Based on the rules outlined in `AGENTS.md` (specifically **Rule 14. Numeric-law requirements**), here is the research regarding the special scrutiny applied to `reciprocal` and `fixed-point division replacement`:

## Why a Declared Error Envelope is Required
In the BCINR framework, authoritative arithmetic must be strictly deterministic, fixed-width, and entirely free of floating-point operations, NaN, infinity, and architecture-dependent rounding. 

Operations like `reciprocal` and `fixed-point division replacement` are inherently **approximations** within fixed-point integer mathematics. Because this substrate enforces bit-for-bit determinism and uses branchless arithmetic to execute logic, any error from approximation must be strictly contained so it doesn't break algebraic invariants or cause divergent state transitions. 

To prove that the approximation is safe, it must provide a strict Hoare contract and a **declared error envelope** consisting of:
* The valid domain and codomain
* The maximum absolute error
* The maximum relative error
* A monotonicity result
* Explicit saturation and boundary behavior

This envelope acts as an executable specification. It guarantees that the maximum mathematical drift is known, bounded, and verified by an independent oracle without relying on hardware-specific rounding.

## How Epsilon is Handled
In many traditional codebases, an "epsilon" (a tiny constant) is silently added to denominators to prevent division-by-zero or to smooth out numerical instability. 

Under Rule 14, **"No epsilon may be inserted silently."** 

If a smoothing constant, clamp constant, or epsilon is required for the approximation to be safe, it must be explicitly defined and tracked. Specifically, the constant must be:
1. **Named** 
2. **Derived** (proven mathematically rather than guessed)
3. **Admitted** (accepted into the formal contract)
4. **Included in the influence digest** (so it is cryptographically/structurally accounted for in state changes)

This ensures that arbitrary magic numbers are never hidden inside arithmetic abstractions, keeping the implementation mathematically sound and strictly verified.
