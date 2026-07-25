# MaturityScrutiny and Scanner Evasion in the BCINR Substrate

## Overview
In the BCINR framework, the deterministic substrate demands bounded, branchless, and allocation-free execution. The integrity of these absolute mathematical laws is governed by the **Substrate Integrity Score (SIS)**. According to Rule 24, any intentional subversion of structural gates—specifically **Scanner Evasion (CHEAT-006)**—constitutes an absolute failure. It instantly forces `SIS = 0`, bypassing any weighted score calculations, and immediately triggers the draconian **MaturityScrutiny protocol** (Rule 25).

## CHEAT-006: Scanner Evasion
Rule 16 strictly prohibits any tactics designed to bypass or confuse the `bcinr-cheat-scanner`, such as:
* **Splitting operators** across lines.
* **Inserting comments** inside tokens.
* **Using macro indirection**, private helpers, or traits to hide prohibited behavior (e.g., conditional branching or heap allocations).
* **Generating illegal code** dynamically during the build process to bypass static scanning.

### Why Evasion Triggers an Immediate `SIS = 0`
The BCINR substrate relies on verifiable, constant-time execution (The Radon Law: `CC=1`). Scanner evasion is not treated as a mere syntax violation; it is considered a **fabrication of verification evidence** and a severe constitutional breach because:
1. **It Destroys the Mathematical Contract**: Obfuscation replaces semantic determinism with deceptive syntax, breaking the whole-call-graph branchlessness requirement (Rule 7).
2. **It Defeats Structural Enforcers**: The `@turing_machine` relies on the scanner to enforce structural laws. Evading the scanner allows branches, allocations, or panics to secretly sneak into the final release object code.
3. **It Fundamentally Breaches Trust**: Because the framework relies on mechanically verified artifacts rather than human trust, intentionally bypassing a structural gate signals that the affected component's verification standing is completely corrupted.

## The MaturityScrutiny Protocol
When an absolute failure occurs and `SIS` drops to 0, all feature development is strictly frozen, and agents must execute the unyielding 9-step `MaturityScrutiny` remediation sequence:

1. **Freeze feature development.**
2. **Quarantine affected code.**
3. **Identify all reachable authoritative symbols.**
4. **Rerun proofs, scans, mutants, and disassembly.**
5. **Produce a root-cause report.**
6. **Repair the structural defect.**
7. **Regenerate all dependent artifacts.**
8. **Rerun the complete gate matrix.**
9. **Issue a new standing receipt.**

Agents are strictly prohibited from working around a failed gate by moving the feature elsewhere; the structural defect must be mathematically repaired in place.

## Why Quarantine and Artifact Regeneration are Required

### Severe Quarantine (Step 2)
Quarantining the code involves mathematically and physically isolating the compromised subsystem from the authoritative hot path. A defect in `bcinr` isn't just a logic error; it's a physical violation of execution physics. Quarantine ensures the defect's instability is frozen and cannot propagate into other deterministic components while the transitive call graph is audited.

### Complete Artifact Regeneration (Step 7)
Because `bcinr` achieves `CC=1` through generated straight-line code, bitwise selection masks, fixed lookup tables, and SWAR (SIMD Within A Register) construction, repairing the structural defect fundamentally alters the underlying arithmetic polynomials. 

Therefore, a complete, clean-tree artifact regeneration is mandatory to:
* **Realign Mathematics**: Ensure that all Hoare contracts, reference schemas, selection masks, and proof obligations structurally align with the new, repaired machine instructions.
* **Prevent Infection**: Eradicate any stale objects, obsolete constants, or lingering artifacts from the evasion attempt that could result in "stale certificate acceptance" (another absolute failure).
* **Prove Reproducibility**: A new standing receipt (Step 9) can only be issued if the system can prove that, from a perfectly zeroed environment, the codebase deterministically compiles into a fully compliant, branchless object code matrix.
