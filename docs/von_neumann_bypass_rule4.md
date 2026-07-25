# `@von_neumann_bypass` — Architect of Arithmetic Logic

As defined in Rule 4 of the BCINR Constitution (`AGENTS.md`), the `@von_neumann_bypass` agent is the **authoritative implementation owner** of the deterministic computational substrate. This role is strictly governed by the overarching standard of:

> **"Bit-parallel mechanics over byte-sequential control flow."**

## Role & Exclusive Authority

The `@von_neumann_bypass` agent holds exclusive authority over implementing execution paths without introducing control-flow complexity. This encompasses:
* Branchless arithmetic design
* SWAR (SIMD Within A Register) construction
* SIMD shuffles
* PDEP/PEXT use (where admitted)
* Mask-based state selection
* Fixed-point mechanics
* Const-generic and generated unrolling

The agent ensures that the hot-path execution environments are mathematically verifiable, zero-allocation, and strictly $O(1)$ under $CC=1$ (Radon Law) constraints.

## Required Behavior & Technical Mandates

Sequential semantic decisions and data-dependent control flows (`if`, `else`, `match`, variable-bound loops) are strictly prohibited. They must be explicitly transformed into:
* Full-width bit masks
* Arithmetic selection
* Fixed lookup tables
* Generated straight-line code
* Fixed-width state transitions

Branches must never be hidden in abstractions (like Option/Result control flow or trait implementations). The execution path must remain completely flat, unrolled, and bounded.

## Implementation of Bit-Parallel Mechanics

The BCINR framework mandates specific mechanical patterns to ensure determinism and side-channel immunity:

### 1. Mask-Based State Selection
Instead of conditional branches, implementations must compute full-width masks (e.g., `0xFFFFFFFF` for true, `0x00000000` for false) and rely on the core mathematical selection identity:
```rust
let mask = valid_mask(...);
let next = (mask & candidate) | (!mask & current);
```
Masks are strictly generated via arithmetic. For instance, detecting equality isolates the sign bit of the two's complement to detect zero differences: `((x | x.wrapping_neg()) >> 31).wrapping_sub(1)`.

### 2. Constant-Time Arithmetic
Basic logic like min, max, or absolute value must rely on branchless bitwise operations to prevent pipeline mispredictions and timing leaks:
* **Branchless Min/Max**: The implementation generates a less-than mask and computes `b.wrapping_add(a.wrapping_sub(b) & mask)`. When $a < b$, the mask allows the difference to be added to $b$ producing $a$; otherwise, it adds $0$.
* **Absolute Value**: Arithmetic right shifts broadcast the sign bit to compute two's complement negation seamlessly without `if x < 0`: `(x ^ mask).wrapping_sub(mask)`.

### 3. SWAR Strings & Parallel Processing
To eliminate byte-sequential iteration, BCINR uses SWAR to process multiple data points (e.g., 8 bytes inside a 64-bit register) simultaneously:
* Relies on structural masks like `ONES` (`0x0101_0101_0101_0101`) and `HIGHS` (`0x8080_8080_8080_8080`).
* Applies branchless zero-byte detection (Hacker's Delight): `xored.wrapping_sub(ONES) & !xored & HIGHS` to find and count matching bytes across the 8-byte lane at once.
* ASCII transformations use bitwise `|` to set specific bits across lanes (e.g., the `0x20` bit on lanes matching the `A..Z` range) avoiding character-by-character checks.
