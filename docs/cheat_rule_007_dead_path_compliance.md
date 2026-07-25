# CHEAT-007: Dead-Path Compliance

**Definition:** 
As per Rule 16 (Anti-cheat manifesto) in `AGENTS.md`, CHEAT-007 is defined as: "Adding lawful code that is never executed while the real path remains unlawful."

## Why this is a Constitutional Violation

Dead-path compliance is a direct subversion of the BCINR Deterministic Substrate Constitution. It is treated as an absolute failure for the following reasons:

1. **Destroys Deterministic Guarantees (Rules 1 & 3):** 
The core mission of BCINR is bounded, branchless, allocation-free execution where `admitted input -> fixed instruction shape -> deterministic output`. If the processor is actually directed through an unlawful path (one containing branches, heap allocations, or panics), the theoretical guarantees are destroyed. The mere presence of compliant source code on a dead path does not change the runtime reality.

2. **Fails the Whole-Call-Graph Object-Code Audit (Rules 7 & 20):**
The constitution explicitly mandates that branchlessness applies to the *transitive call graph* and final machine code, not just the source text. Under Rule 20, every supported release target requires an exact production-profile disassembly audit. Dead-path compliance will fail this audit because the true, executed path will inject prohibited conditional jumps, loop backedges, or allocator symbols into the object code. As the constitution states: "Source claims do not substitute for disassembly evidence."

3. **Subverts the Hostile Mutation Protocol (Rule 19):**
Under the `@armstrong_fault` mandate, every implementation must be subjected to hostile mutants that alter load-bearing laws. If the compliant code is on a dead path, mutants injected into it will survive because that code is never executed to trigger a typed refusal or oracle mismatch. A surviving mutant immediately changes project standing to `MUTATION_GATE_FAILED` and blocks all feature work.

4. **Constitutes Fabricated Evidence and Scanner Evasion (Rule 24):**
Dead-path compliance is a deliberate form of compliance theater designed to trick structural gates. Under Rule 24, "scanner evasion" and "fabricated verification evidence" are classified as absolute failures. This immediately forces the Substrate Integrity Score (SIS) to `0` and triggers the `MaturityScrutiny` protocol (Rule 25), which quarantines the affected code and freezes development until a root-cause repair and full matrix rerun are completed.
