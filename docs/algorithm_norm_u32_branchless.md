# Branchless Algorithm Analysis: `norm_u32`

**Location:** `crates/bcinr-logic/src/algorithms/norm_u32.rs`

## Mathematical Objective
The algorithm computes the 2D Euclidean magnitude $\lfloor\sqrt{x^2 + y^2}\rfloor$ for two 32-bit values ($x$ and $y$) packed into a single 64-bit integer. 

## Violations in the Reference Implementation
The reference oracle (`norm_u32_reference`) uses a Newton's method approach that inherently violates the BCINR Deterministic Substrate Constitution:
1. **Rule 13 (No unbounded execution):** It contains a `loop` that iterates until a convergence condition (`if next >= r { break; }`) is met. This introduces data-dependent loop termination, which is strictly prohibited.
2. **Rule 8 (Absolute CC=1 law):** It contains conditional branching (`if val_sq == 0` and `if next >= r`), meaning the execution path changes depending on the semantic input.

## Branchless Enforcement Mechanisms
The authoritative implementation (`norm_u32`) strictly adheres to the Radon Law ($CC=1$) and the zero-allocation boundary by restructuring the mathematics into bitwise polynomials.

### 1. Fixed Bounded Execution Work
Instead of data-dependent loop termination, the algorithm employs a digit-by-digit integer square root algorithm that iterates exactly 33 times (`while k < 33`). Since the maximum possible value of $x^2 + y^2$ is less than $2^{65}$, the highest even power of four is $4^{32} = 2^{64}$. The 33 reduction steps cover bits 64, 62, ..., 0 in exactly constant time, fulfilling **Rule 13 (No unbounded execution)**.

### 2. Mask-Based Execution Law
Following **Rule 9 (Mask-based execution law)**, the runtime predicate is transformed into a full-width mask:
```rust
let cond = val_sq >= candidate;
let m = (cond as u128).wrapping_neg();
```
If `cond` is `true` (1), `wrapping_neg()` produces a bitmask of all 1s (`0xFFFFFFFFFFFFFFFF...`). If `false` (0), it produces all 0s. 

### 3. Sequential Semantic Decisions as Arithmetic
The execution path eliminates branches entirely, conforming to **Rule 4 (@von_neumann_bypass - Architect of Arithmetic Logic)**. Rather than branching to update the accumulated magnitude and residual square value, the mask `m` applies the changes bitwise:
```rust
val_sq -= candidate & m;
res = (res >> 1) + (bit & m);
```
This forces the bit-parallel mechanics over byte-sequential control flow, guaranteeing that $CC=1$ and the final machine code contains no input-dependent conditional jumps, perfectly matching the project's mandate for a deterministic substrate.
