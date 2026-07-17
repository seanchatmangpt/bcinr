# Implementation Requirements for bcinr, praxis, and mfact v26.7.15
**Owner:** `@von_neumann_bypass` (Lead Implementer - Architect of Arithmetic Logic)

## 1. V2 Tape Integration & Scheduler Bridging (Branchless)
- **Objective:** Bridge `CompiledPowlV2` output natively into `scheduler_tick_guarded_v2` without any translation layer to legacy 32-byte `PowlTape`.
- **Arithmetic Constraint:** The scheduler must accept the `v2::PowlTape` (64-byte `Powl64Op`) layout. Control flow branching based on operation type (`entry_op`, `exit_op`) must be completely eliminated.
- **Implementation Strategy:** 
  - Use bitwise arithmetic (e.g., SWAR techniques) to evaluate scheduler conditions over the `v2::Powl64Op` array in parallel.
  - No `if` statements or `match` blocks for operation dispatch within the hot path. 

## 2. EventSet Masking and Numeric-Fluent Extraction
- **Objective:** Compute the `ready` mask dynamically and efficiently handle numeric-fluent extraction for the `PddlConcurrencyAnalyzer`.
- **Arithmetic Constraint:** Evaluating capacity nonfaces and updating the `ready` `EventSet` must use branchless bit manipulation.
- **Implementation Strategy:**
  - **PDEP/PEXT & SWAR:** Utilize `PDEP` (Parallel Bits Deposit) and `PEXT` (Parallel Bits Extract) instructions to gather dispersed numeric bounds or boolean flags from the state vector into contiguous registers for bulk capacity evaluation.
  - **SIMD Shuffles:** Use SIMD shuffle instructions (e.g., via `core::arch` intrinsics or `std::simd` if stabilized/permitted) to align multiple resource state vectors against the required numeric capacities for the $N$-way nonface evaluation.
  - **Branchless Masking:** The computation of `ready & ~active & ~guards_mask` must occur entirely as bitwise boolean polynomials.

## 3. Downstream API Alignment in Praxis (`chatman/engine.rs`)
- **Objective:** Ensure the consuming loops in the Praxis engine seamlessly invoke the new branchless V2 scheduler API.
- **Arithmetic Constraint:** The engine loop must not rebuild masks sequentially.
- **Implementation Strategy:**
  - Update `chatman/engine.rs` to initialize and pass `CompiledPowlV2` artifacts.
  - When the engine processes an `ExecutionReceipt`, it must utilize the newly explicit `ready` `EventSet` for stateless, constant-time validation (`(fired.bits() & !ready.bits()) == 0`) rather than sequential or stateful replay.
  - Strip all legacy `&[crate::tape::Powl64Op]` bindings from the Praxis interface.

## Success Criteria (The Radon Law $CC=1$)
- **Zero Branches:** The core `scheduler_tick_guarded_v2` must have cyclomatic complexity of exactly 1.
- **Zero Allocations:** No heap allocation inside the execution tick.
- **Hostile Verification:** Passes all `@armstrong_fault` mutational tests ensuring that any broken masking directly fails the receipt validation.
