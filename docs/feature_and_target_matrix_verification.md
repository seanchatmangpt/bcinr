# Feature and Target Matrix Verification

According to **Rule 22** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), passing structural gates in only a single configuration is strictly insufficient to establish repository standing.

## Why a Single Configuration is Insufficient

Different feature flags, compilation profiles, or target architectures can significantly alter the generated machine code and control flow graph. A codebase that is branchless and allocation-free under one set of features or on one architecture might introduce hidden branches, panic paths, or unapproved fallbacks under another. Therefore, passing a single feature configuration does not prove that the universal, absolute structural laws of the substrate are upheld globally. 

## Requirements for Structural Gates

To establish true repository standing, all gates must run across **every supported combination**. This combinatorial verification matrix must include:

* `default features`
* `no default features`
* `all features`
* `release profile`
* `supported architectures`
* `test profile where relevant`
* `generated clean tree`

Every configuration in this matrix must independently pass all required repository gates (e.g., source and object-code audit, mutant testing, `CC=1` enforcement) without exception.

## Architecture-Dependent Instructions (PDEP/PEXT)

When utilizing architecture-specific instructions like `PDEP` (Parallel Bits Deposit) or `PEXT` (Parallel Bits Extract), the implementation must strictly adhere to the following fallback and evidence requirements:

1. **Admitted Target Capability:** The use of the hardware instruction must be explicitly gated by an admitted target capability.
2. **Lawful Fallback or Typed Refusal:** The system must either provide a fallback implementation for targets lacking the capability or issue a typed refusal. 
3. **Same Structural Laws:** If a fallback implementation is provided, it is not exempt from the constitution. It **must satisfy the exact same structural laws** as the primary implementation (e.g., strictly `CC=1`, zero branching, constant-time execution).
4. **Separate Disassembly Evidence:** Disassembly evidence must be generated and audited independently for *both* the hardware-accelerated target and the fallback target to definitively prove that neither compilation path introduces a branch or loop backedge.
