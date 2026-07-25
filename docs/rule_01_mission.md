# Rule 1: Mission

**BCINR as a Deterministic Computational Substrate:**
BCINR is designed to be a deterministic computational substrate specifically for bounded, branchless, and allocation-free execution. Being a deterministic computational substrate means that the authoritative runtime strictly preserves the following mathematical pipeline without exception:

`admitted input → fixed instruction shape → deterministic output`

The execution path and behavior must be entirely rigid and perfectly predictable, avoiding data-dependent branches, variable execution work, or runtime allocations. 

**The 7 Existence Requirements for an Authoritative Primitive:**
The repository mandates that implementations cannot just "appear correct" in tests. A feature is not considered complete until all seven of the following artifacts exist:

1. A mathematical contract
2. A structurally lawful implementation
3. An independent oracle or proof
4. Hostile mutants
5. Source-level verification
6. Object-code verification
7. Reproducible evidence
