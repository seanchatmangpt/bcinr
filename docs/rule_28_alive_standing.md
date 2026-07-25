# `ALIVE` Standing (Rule 28 of AGENTS.md)

According to Section 28 ("Standing vocabulary") in the BCINR Deterministic Substrate Constitution, the **`ALIVE`** standing is defined as:

> **"The implementation executes and passes all declared gates in the pinned environment."**

### What It Means
The `ALIVE` standing indicates that a piece of code is fully functional, verified, and actively compliant with the repository's strict constitutional requirements. It has progressed beyond being merely theoretical or partially tested and is proven to be safe for the deterministic substrate. 

### Conditions for the `ALIVE` Label
For a piece of code to be labeled `ALIVE`, it must satisfy the following conditions:

1. **Executable State:** The implementation must successfully compile and execute. It cannot be dead code, a stub, or cause the build to break (`BUILD_BROKEN`).
2. **Passing all Declared Gates:** The code must successfully pass the repository's comprehensive verification gates (outlined in Section 23). This includes, but is not limited to:
   - Source-level audits for $CC=1$ (Cyclomatic Complexity of 1).
   - Zero-allocation limits and panic-path analysis.
   - Cheat-scanner validations to ensure no evasion or prohibited structures are present.
   - Successful completion of all hostile mutation protocols (where all plausible mutants are killed).
   - Object-code disassembly audits proving absolute branchlessness (no conditional jumps or loop backedges) in the authoritative call graph.
3. **Pinned Environment Validation:** The successful execution and verification must occur in the project's exact, reproducible "pinned environment" (specific release target, architecture, and feature configuration), rather than merely passing tests on an unverified or local setup.

**Contrast with other standings:** 
A piece of code will *not* be labeled `ALIVE` if:
- Verification is incomplete (`PARTIAL_ALIVE`).
- Complete object-code standing isn't established despite the source appearing branchless (`SOURCE_BRANCHLESS_PARTIAL`).
- An agent claims success but it hasn't been independently reproduced (`REPORTED_ALIVE`).
