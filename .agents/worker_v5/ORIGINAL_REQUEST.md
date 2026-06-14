## 2026-06-12T21:44:51-07:00
You are teamwork_preview_worker. Your working directory is `/Users/sac/bcinr/.agents/worker_v5/`.
Your mission is to restore and refactoring all 307 algorithm files in `crates/bcinr-logic/src/algorithms/` to remove the category-specific dummy hashes (Patterns 1-4) and replace them with genuine branchless implementations and decoupled mathematically/logically correct reference functions.

### MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

### Instructions:
1. Examine the Python files starting with `implement_` and `implement_batch_` (such as `implement_1_30.py`, `implement_batch_2.py`, `implement_batch_6.py`, etc.) in `/Users/sac/bcinr/`. These scripts contain the genuine logic mapping and reference mappings for the algorithms.
2. Do NOT run Python scripts to modify the codebase (the Zero Python Rule prohibits Python for codebase modifications/audits). Instead, write a Rust utility in the workspace (you can create it under `tools/rust_audit` or a new cargo crate) that parses these Python files as text, extracts the algorithm mappings, and refactors each corresponding Rust file in `crates/bcinr-logic/src/algorithms/`.
3. For each algorithm file:
   - Replace the `pub fn` implementation body with the genuine branchless logic.
   - Replace the `_reference` function body with the correct mathematical/logical reference logic.
   - Ensure the reference function is mathematically correct (i.e. not a dummy hash) so that the proptest equivalence checks are decoupled, independent, and act as real validation gates.
   - Preserve all existing doc headers (making sure the doc comments contain the literal phrase "Branchless Contract" to satisfy the contract gate), formal proof/Hoare comment blocks, and mutant tests.
   - Verify that each file has at least 100 lines (adding academic padding comments at the end of the file if needed) to satisfy the maturity check.
   - Ensure the mutant functions `mutant_` are distinct from the reference function so they are rejected by the counterfactual tests.
4. Clean up any compiler or Clippy warnings. Ensure the codebase compiles with zero warnings under `#![deny(warnings)]` on nightly.
5. Update `TEST_READY.md` to show the correct E2E test verification command `cargo test -p bcinr --test e2e -- --test-threads=1` instead of the deprecated Python script runner command.
6. Run the admissibility scan using the pre-compiled binary at `/tmp/bcinr-e2e-target/debug/anti-llm-cheat-lsp` to ensure exactly 0 diagnostics are emitted. If needed, configure `.anti-llm-ignore` or adjust imports/versions.
7. Run the Rust E2E integration test suite via `cargo test -p bcinr --test e2e -- --test-threads=1` and ensure all 60 tests pass.
8. Document all your changes, compile/test results, and layout verification details in `/Users/sac/bcinr/.agents/worker_v5/handoff.md`, and complete your task. Send a message to parent (`dc5fade1-56cc-48e4-a95b-67093600ad13`) with the path to your handoff when done.
