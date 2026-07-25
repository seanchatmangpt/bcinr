# PDEP/PEXT Hardware Instructions in BCINR

## Admission by `@von_neumann_bypass`

Under the `@von_neumann_bypass` protocol (Architect of Arithmetic Logic), architecture-specific bit manipulation instructions like `PDEP` (Parallel Bit Deposit) and `PEXT` (Parallel Bit Extract) are explicitly admitted for the purpose of enabling high-performance, branchless bit-parallel logic. 

Specifically, they are admitted for:
* **Branchless Arithmetic Design & SWAR Construction**: Translating sequential semantic decisions into $O(1)$ constant-time bit-parallel execution.
* **State Gathering (`PEXT`)**: Taking a selector mask and gathering dispersed boolean flags, numeric bounds, or specific bit fields scattered across a state vector into contiguous registers for bulk capacity evaluation without iterative loops.
* **State Scattering (`PDEP`)**: Taking densely packed configuration bits and depositing them into sparse, mask-specified positions. This enables transactional, masked commits and state updates securely without speculative mutation or branch prediction penalties.

## Strict Rules for Architecture-Specific Instructions

According to the **Feature and Target Matrix Law** (Section 22 of the BCINR Constitution), the use of architecture-specific instructions (like BMI2's `PDEP`/`PEXT`) requires strict adherence to the following rules:

### 1. Lawful Fallback Targets
If a native instruction is absent on the target architecture, a simpler branching equivalent (e.g., a variable `while` loop) is strictly prohibited. Such fallbacks would introduce Jump Conditional Code (JCC) violations and break the project's zero-branch Radon Law.
* **Same Structural Laws**: The fallback implementation must satisfy the exact same structural laws as the primary path (bounded, branchless, zero allocation, $CC=1$).
* **Parallel-Prefix Algorithms**: Fallbacks for `PDEP`/`PEXT` must be implemented as fully unrolled, 6-stage constant-time parallel-prefix algorithms (such as those adapted from *Hacker's Delight*). These must operate entirely using primitive constant-time arithmetic and bitwise operations (`^`, `|`, `&`, `<<`, `>>`).

### 2. Typed Refusals
If a lawful fallback target cannot be constructed for an architecture-specific instruction, the implementation cannot panic, silently truncate, degrade in correctness, or fall back to an unverified branching path. Instead, it must yield a bounded, deterministic **Typed Refusal** (e.g., returning a strict error enum variant such as `SupportMismatch` or `UnsupportedDomain`).

### 3. Disassembly Evidence
The use of hardware-specific intrinsics and their fallbacks must be proven at the object-code level, not just the source level.
* **Separate Disassembly Evidence**: An exact production-profile disassembly audit must be produced for the specific target architecture.
* **Object-Code Scrutiny**: The audit must inspect all authoritative root symbols and transitive helper symbols to ensure zero conditional jumps, zero loop backedges, and zero panic paths remain in the final machine code. The results must list each symbol individually to maintain the required Substrate Integrity Score (SIS).
