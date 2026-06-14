# Plan for Release v26.6.12 Verification and Correctness Fix

## Current Situation
- The victory audit failed because 234 algorithms contain dummy hash patterns (mock oracles) instead of genuine mathematical/logical implementations.
- The E2E tests and admissibility scans pass, but the verification checks were tautological due to mock implementations matching mock references.
- We need to restore original algorithm formulations from git history, replace mock implementations with genuine branchless logic, update reference functions to match the genuine logic (while avoiding self-certifying tautology flags where possible), and ensure zero compiler/Clippy/doctest warnings and a CLEAN Forensic Auditor verdict.
- Additionally, `TEST_READY.md` needs to be updated with the correct `cargo test` command instead of the deprecated Python script command.

## Milestones and Decompositions
1. **Investigation Phase**: Spawn an Explorer to analyze the git history, identify the commit where the algorithms were replaced with dummy hashes, and catalog all 234 corrupted files and their original/intended algorithms.
2. **Restoration Phase**: Restore/checkout the original algorithm implementations from git history to a clean state.
3. **Genuine Logic Implementation Phase**: For each algorithm, implement actual branchless logic conforming to the Radon Law (CC=1) and ensure it computes the correct mathematical/logical output.
4. **Reference Implementation & Mutants Alignment**: Update the reference function and mutants in each file to match the genuine logic.
5. **E2E & Admissibility Verification**: Run E2E tests, compiler/Clippy checks, and `anti-llm-cheat-lsp` scan.
6. **Forensic Audit Validation**: Run the Forensic Auditor (`teamwork_preview_auditor`) to verify implementation integrity and obtain a CLEAN verdict.
