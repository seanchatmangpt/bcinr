# The `@von_neumann_bypass` Protocol

The `@von_neumann_bypass` protocol is a core component of the BCINR (BranchlessCInRust) deterministic substrate constitution, representing the **Architect of Arithmetic Logic** and authoritative implementation owner. 

Its governing principle is: **"Bit-parallel mechanics over byte-sequential control flow."**

The protocol enforces the **Radon Law ($CC=1$)**, dictating that all authoritative runtimes must contain zero data-dependent branches (`if`, `match`, `while`), zero heap allocations, and zero panic paths. These laws apply transitively across the entire call graph, including private helpers and macro expansions.

To achieve this, the protocol relies on three core implementation strategies:

## 1. Transforming Sequential Decisions into Arithmetic Selection
In a branchless architecture, semantic execution decisions cannot be conditionally executed. They must be transformed into arithmetic selection:

* **Mask-based Execution**: All runtime predicates must evaluate to full-width boolean masks ($m \in \{0, 2^w-1\}$). 
* **Arithmetic Selection**: Instead of `if valid { candidate } else { current }`, code must execute both paths and select the result using bitwise arithmetic:
  `select(m, a, b) = (m & a) | (!m & b)`
* **Transactional Masked Commits**: Speculative mutation before admission is strictly prohibited. The required operational shape is:
  1. Read current immutable state.
  2. Compute a fixed-size candidate state structurally.
  3. Verify all predicates to derive an admission mask.
  4. Perform a fieldwise masked commit.
  A rejected operation leaves the persistent state bit-for-bit unchanged.
* **Typed Refusals**: Invalidation conditions must result in a bounded typed refusal (e.g., `ContractViolation`, `DigestMismatch`) rather than panicking or clamping silently.

## 2. Fixed Lookup Tables
To avoid branch logic for semantic evaluations, conditions are mapped to integers and resolved via constant-time lookups:

* **Table-driven Dispatch**: Operations that would typically require `match` ladders are instead converted to state indices that map into fixed-size `const` lookup table (LUT) arrays.
* **O(1) Evaluation**: Lookup tables are precomputed (e.g., evaluating all 256 byte configurations or PDDL8 bounding patterns) to ensure execution latency is strictly bounded and independent of the input data.
* **Bounded Data Structures**: Due to strict limits like the PDDL8 bounds (arity ≤ 8, depth ≤ 64), the state space fits entirely within fixed memory footprints, enabling exhaustive $O(1)$ LUT-based admission gating.

## 3. SWAR State Transitions
SWAR (SIMD Within A Register) is fundamental to the protocol for processing multiple data items simultaneously in standard integer registers without relying on loops or hardware SIMD imports:

* **Bitwise Boolean Polynomials**: Operations like evaluating the capacity state for the scheduler (`ready & ~active & ~guards_mask`) must be resolved strictly through branchless bitwise boolean polynomials.
* **PDEP / PEXT Masking**: The architecture leverages `PDEP` (Parallel Bit Deposit) and `PEXT` (Parallel Bit Extract) to gather dispersed numeric bounds or boolean flags into contiguous registers for bulk capacity evaluation. 
* **Hacker's Delight Fallbacks**: For architectures without native BMI2 instruction support for `PDEP`/`PEXT`, the protocol requires fully unrolled 6-stage constant-time parallel-prefix algorithms (avoiding simulated loops that would trigger JCC—Jump Conditional Code—violations).
* **Zero-Detection and Validation**: SWAR relies heavily on arithmetic tricks like zero-byte detection (e.g., `v.wrapping_sub(0x0101..) & !v & 0x8080..`) to validate strings, search text, or verify bounds without introducing branch prediction penalties.
* **SIMD Shuffles**: Where stabilized, SIMD shuffle instructions align resource state vectors against numeric capacities for multi-way ($N$-way) evaluation seamlessly.

By adhering to this protocol, `bcinr` ensures execution times are strictly deterministic, provably free of timing side-channels, and fully auditable by independent static analyzers.
