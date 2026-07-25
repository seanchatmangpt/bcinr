Based on `AGENTS.md`, here are the details regarding the "Feature and target matrix" under Rule 22, specifically focusing on architecture-specific instructions like PDEP/PEXT:

### Architecture-Specific Instructions Requirements (e.g., PDEP/PEXT)

For any architecture-specific instructions, the following three requirements must be met:
1. **Admitted target capability:** The capability must be explicitly admitted.
2. **Lawful fallback or typed refusal:** There must be a lawful fallback target provided or a typed refusal returned.
3. **Separate disassembly evidence:** The specific instruction requires its own separate disassembly evidence.

### Fallback Rules

If a fallback implementation is used, it **must satisfy the exact same structural laws** as the primary implementation. This means the fallback must remain strictly compliant with the repository's rules, such as being completely branchless (`CC=1`), allocation-free, deterministic, and bounded, verified via the same gate requirements across the feature and target matrix.
