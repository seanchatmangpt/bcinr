# Rule 1 (Mission) - Feature Completion Criteria

According to Rule 1 (Mission) in `AGENTS.md`, BCINR is a deterministic computational substrate designed for bounded, branchless, allocation-free execution. The authoritative runtime must mathematically preserve a strict pipeline: `admitted input -> fixed instruction shape -> deterministic output`.

To ensure this absolute structural integrity, every authoritative primitive must satisfy the following 7 criteria before a feature is considered complete:

1. **A mathematical contract:** A precise Hoare contract defining valid input domains, output ranges, conservation and monotonicity laws, overflow behaviors, and deterministic state-mutation boundaries.
2. **A structurally lawful implementation:** Code that strictly adheres to the core architectural laws, such as zero heap allocations and bit-parallel mechanics with no data-dependent branches ($CC=1$).
3. **An independent oracle or proof:** A mathematically independent specification, formal proof, or bit-vector solver certificate to verify full-domain correctness, structurally distinct from the production implementation.
4. **Hostile mutants:** A set of syntactically plausible, independent mutants (e.g., dropped factors, sign inversions) designed to prove that the test suite and safeguards can trigger a specific typed refusal or oracle mismatch.
5. **Source-level verification:** Automated scans of the syntax tree (including macros and generated Rust) verifying that the source code contains no hidden control flow, prohibited operations, or rule evasions.
6. **Object-code verification:** Exact production-profile disassembly audits for all supported targets, confirming that the compiled machine code is completely free of conditional jumps, loop backedges, allocator symbols, and panic paths.
7. **Reproducible evidence:** Verifiable artifacts and receipts proving that all required tests, bounds, scans, proofs, and audits passed across all supported configurations and feature matrices.

### Why "Appears Correct in Tests" is Insufficient
The repository explicitly rejects implementations that merely "appear correct in tests." This is because standard empirical testing only validates semantic correctness (output) for a finite set of inputs; it does not guarantee structural compliance (the execution path). 

In BCINR, the *shape of the execution* is just as critical as the result. A function might pass all unit tests, but its underlying code could still contain data-dependent branches, heap allocations, or rely on compiler optimizations to flatten logic. This violates the foundational Radon Law and exposes the system to timing side-channels and non-deterministic overhead. The 7 criteria mandate that the code is not just functionally correct on the surface, but mathematically proven, structurally enforced down to the assembly level, and systematically verified to be invariant across the entire input domain.
