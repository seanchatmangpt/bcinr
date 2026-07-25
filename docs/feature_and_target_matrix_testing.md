# Feature and Target Matrix Testing (Rule 22 Analysis)

In accordance with **Rule 22 (Feature and target matrix)** of the `AGENTS.md` BCINR Deterministic Substrate Constitution, validating a single configuration is strictly insufficient to establish repository standing. The authoritative runtime must prove its deterministic, branchless execution across all possible compilation and deployment topologies.

## Exhaustive Combination Matrix Requirement

All constitutional gates (including mutant testing, cheat scanning, contract enforcement, and object-code auditing) must be executed across every supported combination in the matrix. The mandatory permutations include:

*   **Feature Flags**: `default features`, `no default features`, and `all features`.
*   **Compilation Profiles**: `release profile` (essential for final object-code evidence) and `test profile where relevant`.
*   **Execution Environments**: All `supported architectures`.
*   **Source States**: A `generated clean tree`.

**Why this is required:** 
In Rust, conditional compilation (`#[cfg(...)]`), varying feature flags, and target-specific compiler optimizations can radically alter the generated object code. A function might cleanly compile into a branchless execution path on one architecture but unexpectedly generate conditional jumps or loop backedges on another. By enforcing the execution of *all gates* across *all combinations*, the substrate guarantees that no hidden branches, allocations, or structural violations (such as breaking the `CC=1` Radon Law) are introduced under specific build configurations. 

## Handling Architecture-Specific Instructions (e.g., PDEP/PEXT)

When utilizing hardware-accelerated instructions like `PDEP` (Parallel Bits Deposit) or `PEXT` (Parallel Bits Extract)—which are highly effective for branchless bit manipulation—Rule 22 imposes stringent requirements to maintain the integrity of the substrate:

1.  **Admitted Target Capability:** 
    The use of architecture-specific instructions must be gated by an explicitly admitted target capability. It cannot rely on implicit assumptions about the runtime hardware.

2.  **Lawful Fallback or Typed Refusal:** 
    If a target environment lacks the hardware capability, the system cannot silently fall back to an unverified software loop. It must provide either:
    *   A **lawful fallback implementation**, OR
    *   A **typed refusal** (e.g., explicitly rejecting execution due to unsupported hardware).

3.  **Strict Adherence for Fallbacks:** 
    If a fallback implementation is provided, it is granted zero leniency. It must satisfy the exact same structural laws as the accelerated path. This means the fallback must remain entirely branchless (`CC=1`), fixed-width, and devoid of hidden runtime algorithm searches or data-dependent loops. 

4.  **Separate Disassembly Evidence:** 
    The object-code audit cannot be generalized. The implementer must produce separate disassembly evidence for *both* the hardware-accelerated target and the fallback target. This ensures the compiler has successfully generated `CC=1` straight-line machine code for both variants, proving that neither pathway relies on hidden control flow.
