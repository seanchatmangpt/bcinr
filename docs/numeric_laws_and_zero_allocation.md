# Relationship Between Numeric-Law Requirements and the Zero-Allocation Boundary

This document explores the fundamental connection between Rule 14 (Numeric-law requirements) and Rule 3 (Absolute runtime laws / Zero-Allocation Boundary) in the `bcinr` deterministic substrate.

## The Constitutional Mandate

The `bcinr` substrate mandates "bounded, branchless, allocation-free execution" to serve as a physically predictable, constant-time foundation for AGI. The intersection of Rule 14 and Rule 3 enforces that all mathematics and memory handling operate in total lockstep to prevent timing side-channels, data-dependent variability, and runtime unpredictability.

### 1. Predictability and Determinism
Dynamic allocation (heap) introduces non-deterministic execution times, fragmentation, and potential Out-Of-Memory (OOM) panics. By strictly enforcing fixed-width arithmetic (Rule 14) and `#![no_std]`, zero heap allocation (Rule 3), the substrate guarantees that math operations take a precise, constant number of cycles. Variable execution work is structurally impossible.

### 2. Fixed-Width State and Error Envelopes
Rule 14 requires that authoritative arithmetic be bounded by a declared error envelope and be mathematically deterministic. Fixed-width structures guarantee that:
- Precision limits are known at compile-time.
- Overflows/underflows are handled structurally (e.g., via bitwise masks and SWAR techniques, satisfying Rules 9 and 14).
- Formal verification (via `@hoare_oracle`) can exhaustively prove properties over finite, bounded domains (e.g., bit-vector solver certificates). Dynamic data types would make the state space functionally infinite and impossible to formally bound with fixed error envelopes.

### 3. Allocation-Free Transactions
Rule 10 mandates that "clone the state" cannot mean a heap-backed copy. It must involve fixed-size stack values or fixed-size scratch structures. To execute complex fixed-point math, algorithms must be able to hold intermediate state. Because they cannot allocate, they must rely on:
- Pre-allocated constant-time allocators like `BumpArena` or `LockFreeSlab` (as defined in `GEMINI.md`).
- Stack-based fixed-width structures (e.g., SIMD lanes or SWAR registers).
This ensures memory access remains bounded and execution work remains fixed (Rule 3).

### 4. Bounded and Branchless Evaluation
Numeric error envelopes frequently dictate how bounds and clamps are applied to values. Rule 14 prohibits hidden `epsilon` injection and mandates explicit clamps. Because dynamic allocations could fail and require control flow (branches) to handle OOM, they inherently violate the $CC=1$ rule. Using `BumpArena` and fixed-width types means capacity and data shapes are known a priori, so no branches are needed for bounds checking or allocation failure handling on the hot path.

## Conclusion

The relationship is one of mutual dependency: mathematical determinism (Rule 14) is impossible to guarantee if the underlying memory model is non-deterministic (violating Rule 3). Fixed-point math and error envelopes must be modeled via fixed-width structures and static memory regions (`BumpArena`/`LockFreeSlab`) because they physically constrain the execution trace to a mathematically provable, constant-time shape without hidden branches or allocation panics.
