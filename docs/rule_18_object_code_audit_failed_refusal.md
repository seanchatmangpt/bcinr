Based on the `AGENTS.md` constitution of the BCINR repository, here is the detailed breakdown of the `ObjectCodeAuditFailed` typed refusal and its relationship to the object-code audit and the CI pipeline:

### What is the `ObjectCodeAuditFailed` Typed Refusal?
Under **Rule 18 (Typed refusals)**, the authoritative branchless runtime must use bounded typed refusal codes to reject unsupported inputs or operations, rather than resorting to human-readable text, panicking, falling back, or returning defaults in the hot path. 

`ObjectCodeAuditFailed` is one of these explicitly required refusal categories. It represents a structural compliance failure at the machine-code level. In a substrate where mathematical determinism and branchless execution are strictly enforced, `ObjectCodeAuditFailed` indicates that a primitive violated the strict structural laws after compilation, invalidating its standing.

### Relationship to Rule 20 (Object-code audit)
**Rule 20** mandates that every supported release target undergo an exact production-profile disassembly audit. Source-level checks (like cyclomatic complexity `CC=1`) are considered necessary but insufficient. The object-code audit directly inspects the compiled symbols for:
- Conditional jumps
- Loop backedges
- Indirect calls
- Floating-point or division instructions
- Allocator, panic, or bounds-check symbols
- Unexpected runtime library calls

If the compiled object code for an authoritative symbol contains any of these prohibited structures, it triggers an `ObjectCodeAuditFailed`. This condition represents a failure to maintain branchless determinism when translating from source code to machine code.

### Relationship to the CI Pipeline
The `ObjectCodeAuditFailed` condition is directly integrated into the CI pipeline's strict repository gates:
1. **Mandatory CI Gate (Rule 23):** The CI pipeline requires the execution of `cargo make audit-object-code` (alongside tasks like `scan-cheats` and `test-mutants`). 
2. **Substrate Integrity Score Drop (Rule 24):** If the CI audit uncovers a hidden branch, allocation, or any structural violation, it is classified as an absolute failure, dropping the Substrate Integrity Score (SIS) to `0` instantly.
3. **MaturityScrutiny Protocol (Rule 25):** The failure triggers the `MaturityScrutiny` protocol which completely freezes and blocks all feature development. The affected code is quarantined, and developers must repair the defect (e.g., replace the hidden branch with mask-based arithmetic), regenerate all artifacts, and rerun the complete CI gate matrix before a new standing receipt is issued and merges can resume. 

In summary, `ObjectCodeAuditFailed` is the formal, typed representation of a machine-code level structural violation, rigorously enforced by the `audit-object-code` step in the CI pipeline to guarantee absolute execution determinism.
