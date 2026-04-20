# BCINR — BranchlessCInRust (v26.4.19)
## Project Mandate: The Deterministic Substrate

`bcinr` is a civilizational-scale systems library providing an axiomatic calculus for branchless algorithmics. It is designed to be the "hard substrate" for AGI, where timing side-channels are physically impossible and logic is expressed as arithmetic.

### Core Architectural Laws
- **The Radon Law ($CC=1$)**: No public primitive shall contain a single `if`, `match`, or data-dependent `loop`. Logic must be expressed as bitwise polynomials.
- **The Zero-Allocation Boundary**: Hot-path execution must be `#![no_std]` and perform 0 heap allocations. Memory is managed via `BumpArena` and `LockFreeSlab`.
- **The Contract with Teeth**: Every primitive is an executable specification. If the implementation deviates from the axiomatic reference by even 1 bit, the verification matrix MUST fail.
- **The Substrate Integrity Score (SIS)**: A file is only "PhD-Verified" if it scores 100/100 on the maturity matrix (Proof + Oracle + Hostile Tests).

### MAPE-K Autonomic Loop
All self-managing components must utilize the `AutonomicSubstrate` building blocks to implement:
1. **Observe**: Collect bit-level telemetry.
2. **Infer**: Calculate `RlState` using branchless metrics.
3. **Propose**: Generate `AutonomicAction` masks.
4. **Accept**: Filter through the `PolicyGuard`.
5. **Execute**: Advance state via constant-time transitions.
