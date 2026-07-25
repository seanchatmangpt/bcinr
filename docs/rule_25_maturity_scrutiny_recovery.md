# Rule 25: MaturityScrutiny Protocol

In the BCINR Deterministic Substrate, the **MaturityScrutiny protocol** is an emergency recovery procedure triggered whenever the Substrate Integrity Score (SIS) drops below 100 (`SIS < 100`). A drop in SIS can be caused by accumulated violations or a single absolute failure (such as a hidden branch, allocation in the hot path, or a surviving mutant).

When triggered, agents must execute the following strict 9-step recovery process:

1. **Freeze feature development**: All ongoing feature work must halt immediately.
2. **Quarantine affected code**: Isolate the defective components.
3. **Identify all reachable authoritative symbols**: Trace the full transitive call graph originating from or touching the defect.
4. **Rerun proofs, scans, mutants, and disassembly**: Re-establish the mathematical and structural baseline of the affected area.
5. **Produce a root-cause report**: Formally document the exact constitutional law that was violated and the reason it occurred.
6. **Repair the structural defect**: Fix the underlying codebase so that it complies with all laws (e.g., ensuring branchlessness and zero-allocation).
7. **Regenerate all dependent artifacts**: Rebuild all downstream proofs, generated source code, and serialized artifacts to reflect the repair.
8. **Rerun the complete gate matrix**: Validate the corrected implementation against all authoritative repository gates across every supported target and feature combination.
9. **Issue a new standing receipt**: Formally restore the repository's verified standing.

## Prohibition on Feature Relocation

Rule 25 explicitly dictates: *"Agents may not work around a failed gate by moving the feature elsewhere."*

Agents cannot simply relocate code to bypass a failed gate because doing so violates the core principles of the BCINR repository:
- **Evasion of Jurisdiction:** Moving code to avoid gate failures constitutes "Gate-jurisdiction theater" (CHEAT-010) or "Scanner evasion" (CHEAT-006). Gates define strict mathematical boundaries, and attempting to hide code from the structural auditor (`@turing_machine`) is fundamentally unlawful.
- **Mandatory Defect Repair:** Step 6 of the protocol mandates that the structural defect itself must be *repaired*. Relocating a non-compliant feature does not fix the defect; it merely conceals it. 
- **Constitutional Precedence:** Substrate rules override implementation convenience. Bypassing a gate to deliver a feature breaks the foundational mandate that "the authoritative instruction shape must not depend on semantic input." Code must be mathematically and structurally correct, regardless of where it lives in the project.
