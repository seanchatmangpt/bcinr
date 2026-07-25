# Rule 12: No Runtime Theorem Discovery

## Overview
In the BCINR framework, **Rule 12** of the Constitution (`AGENTS.md`) strictly prohibits "runtime theorem discovery." The authoritative hot path is allowed to verify a mathematical witness, but it must never dynamically search for or compute one.

Algorithms such as spectral-radius estimation, power iteration, Jacobian derivation, optimization over weighting vectors, Lyapunov search, and adaptive threshold discovery are explicitly banned at runtime.

## Why is Runtime Theorem Discovery Prohibited?
The deterministic substrate relies on absolute predictability, enforced by core architectural laws:

1. **The Radon Law ($CC=1$)**: Authoritative functions must contain exactly zero data-dependent control flow branches (`if`, `match`, or dynamic `loop` operations). Logic must be expressed as bitwise polynomials.
2. **Bounded Execution**: The system requires strictly bounded execution work and fixed memory access. 
3. **Zero Allocation**: Hot-path execution must be `#![no_std]` and perform absolutely zero heap allocations.

Theorem discovery intrinsically violates these requirements. Mathematical algorithms like power iteration or eigenvalue search involve:
- **Unbounded Iteration**: Iterative solvers and theorem discovery require loops that terminate based on data-dependent convergence criteria. This directly violates bounded execution and $CC=1$.
- **Floating-Point Operations**: Standard numerical analysis heavily depends on floating-point arithmetic, which is explicitly banned on the hot path to prevent architecture-dependent rounding and non-determinism.
- **Dynamic Allocations**: Generating and manipulating intermediary matrices and search states typically requires dynamic memory allocations.

If the hot path attempted theorem discovery, it would lose its mathematically rigid, deterministic, and constant-time execution guarantees.

## The Solution: Slow Rail Discovery and Branchless Verification

To achieve adaptive stability without breaking architectural constraints, BCINR separates responsibilities between the **Slow Rail** and the **Hot Path**.

### 1. Slow Rail Theorem Discovery
The Slow Rail is a non-authoritative execution environment permitted to use allocations, loops, and floating-point math. When the system state changes, the Slow Rail handles discovery:
- It constructs the Jacobian and Gain matrices.
- It performs spectral decomposition and eigenvalue search to prove the system is strictly contractive ($\rho(G) < 1$).
- It derives a strictly positive eigenvector/witness vector ($d$) and a contraction margin ($\delta$).
- It packages the comparison matrix ($G_{\mathrm{certified}}$), the witness ($d$), the margin ($\delta$), and drift bounds into a fixed-point, cryptographically digested structure.
- The slow rail explicitly derives $G,\ d,\ \delta,\ R_{\mathrm{noise}},\ \text{and } R_{\mathrm{switch}}$.

### 2. Branchless Hot Path Verification (Fixed Witness Domination)
The authoritative Hot Path takes the pre-calculated, fixed-point witness and merely **verifies** static domination in strictly $O(1)$ time:
$$ \widehat G \leq G_{\mathrm{certified}} $$
$$ G_{\mathrm{certified}} d \leq (1-\delta)d $$

This verification is performed with absolute structural determinism (comparing packed values only):
- **Static Unrolling**: Matrix multiplications are statically unrolled using fixed-dimension arrays or macros. There are no dynamic loop backedges in the generated machine code.
- **Bitwise Evaluation**: Instead of conditionally branching (`if G * d > bound { return Err(...) }`), elementwise comparisons yield boolean conditions that are accumulated bitwise.
- **Masked Refusals**: If the mathematical inequality fails in any dimension, the error state is converted branchlessly into a mask using two's complement arithmetic. This mask is then bitwise AND-ed with a typed refusal flag (such as `ContractionMarginInsufficient`) to construct the final refused state.

By forcing the Slow Rail to perform the iterative *discovery* and the Hot Path to only perform the $O(1)$ packed-value *verification*, BCINR guarantees civilizational-scale stability without compromising its zero-branch deterministic substrate.
