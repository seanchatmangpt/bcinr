# Rule 25: MaturityScrutiny Protocol

The `MaturityScrutiny` protocol is the emergency lockdown and remediation sequence triggered automatically when the Substrate Integrity Score (SIS) falls below 100. In the deterministic context of BCINR, a drop in SIS indicates a constitutional violation of the authoritative runtime (e.g., hidden branches, allocation, or surviving mutants). 

Under Rule 25, agents are strictly forbidden from evading a failed gate by moving the feature elsewhere. Instead, they must execute the following 9-step mechanical protocol:

## The 9-Step Execution Sequence

1. **Freeze feature development:** All ongoing product or feature work is immediately halted. The absolute priority is restoring the deterministic integrity of the substrate.
2. **Quarantine affected code:** The specific module, crate, or file responsible for the violation is isolated to prevent the structural defect from contaminating the wider authoritative hot path.
3. **Identify all reachable authoritative symbols:** This requires a complete transitive traversal of the call graph from all authoritative roots. It includes not just direct public functions, but all transitive callees, trait monomorphizations, compiler intrinsics, linked runtime symbols, macros, and generated code.
4. **Rerun proofs, scans, mutants, and disassembly:** The core verification matrix is executed against the quarantined state. This includes Hoare contract verification (`@hoare_oracle`), `CC=1` cheat scanning (`@turing_machine`), hostile mutants (`@armstrong_fault`), and an exact production-profile object-code disassembly audit.
5. **Produce a root-cause report:** A formal explanation of exactly how and why the constitutional law was violated (e.g., a hidden branch in a trait implementation, an unexpected bounds-check panic, or a stale certificate).
6. **Repair the structural defect:** The implementation owner (`@von_neumann_bypass`) replaces the defective code with branchless, allocation-free, mathematically proven constructs (e.g., SWAR, fixed-width masks, or bounded execution logic).
7. **Regenerate all dependent artifacts:** This involves executing a clean rebuild and regeneration of all associated source graphs, proofs, digests, and generated code to ensure mechanical reproducibility.
8. **Rerun the complete gate matrix:** The entire repository's test matrix is run across all combinations of features, release profiles, and supported architectures to confirm the repair.
9. **Issue a new standing receipt:** A verified artifact digest is produced, certifying that the codebase has returned to an $SIS = 100$ and feature development may resume.

---

## Deep Dive: Reachable Authoritative Symbols (Step 3)

Identifying all reachable authoritative symbols is a critical step because the BCINR constitution strictly mandates **whole-call-graph branchlessness** (Rule 7) and **object-code auditing** (Rule 20). 

It is not enough to inspect a function's Rust syntax. The protocol must map out the entire disassembly landscape that the defective code touches, auditing it for:
- Conditional jumps or loop backedges (violations of $CC=1$).
- Panic or bounds-check symbols.
- Allocator symbols (`malloc`, etc.).
- Floating-point or division instructions.

Without mapping out *every* transitive dependency (including generic monomorphizations and compiler intrinsics), a hidden branch introduced deep within a private helper or standard library abstraction could silently compromise the allocation-free, deterministic bounds of the hot path.

## Deep Dive: Regenerating All Dependent Artifacts (Step 7)

Why is manual patching insufficient without a complete, verified tree regeneration?

According to **Rule 21 (Generated-code law)**, generated code and derived artifacts must be completely reproducible and byte-identical across generations. Hand-editing or manually patching a structural defect is explicitly prohibited because:
- **It Breaks the Chain of Trust:** Manual patches bypass the deterministic `clean generation -> digest output -> regenerate` pipeline.
- **It Invalidates Standing:** Hand-edited fixes leave unexplained drift between the source graph and the runtime execution, automatically invalidating standing.
- **It Hides Further Defects:** A manual fix might resolve a single object-code branch but fail to align with the overarching Hoare proofs or bit-vector solver certificates.

A complete tree regeneration mathematically proves that the repair is structurally sound from the axiomatic contracts down to the release object code, ensuring that the final execution remains a pure reflection of admitted laws rather than developer convenience.
