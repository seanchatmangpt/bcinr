## 2026-06-13T02:48:04Z

You are a Worker agent. Your working directory is `/Users/sac/bcinr/.agents/worker_m2_m3_m4/`.
Your task is to implement Milestones 2, 3, and 4:

1. **Milestone 2: Codebase Warnings & Doctest Fixes**:
   - Check and resolve the 22 compiler/lint warnings in `crates/bcinr-logic/` (such as `unused_parens`, `unused_mut`, and ignored `#[inline(always)]`).
   - Fix the workspace doctest compilation failures caused by library linkage/rlib conflicts (such as `ciborium_ll` and `criterion_plot` conflicts). Make sure `cargo test --workspace` can run successfully, including all doctests.

2. **Milestone 3: Tool Admissibility & AST Migration**:
   - In `tools/bcinr-contract-gate/src/main.rs`, migrate substring check logic (e.g. `.contains("Branchless Contract")` or `.contains("BRANCHLESS CONTRACT")`) to AST-based comments/attribute checks using `syn`. Avoid calling `.contains` on raw file content strings.
   - In `tools/bcinr-bench-auditor/src/main.rs`, migrate substring check logic (e.g. `.contains("#[cfg(test)]")` and `.contains(&signature_call)`) to AST-based parsing and traversal using `syn` (e.g. visit all `syn::Ident` inside the Criterion benchmark files, and skip traversing modules annotated with `#[cfg(test)]` by overriding `visit_item_mod`).
   - Clean up `Cargo.lock` and project config version diagnostics (`ANTI-LLM-VERSION-001`). Since `prettytable-rs` depends on `encode_unicode` version `"1.0.0"`, copy the cached `encode_unicode` source from `/Users/sac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/encode_unicode-1.0.0` to `crates/encode_unicode_patch/` in the workspace, modify its version to `"1.0.1"` in its `Cargo.toml`, and patch it in the workspace root `Cargo.toml` using `[patch.crates-io]` so that version `"1.0.0"` is completely eliminated from `Cargo.lock`.
   - Resolve `ANTI-LLM-SURFACE-001` diagnostics (plain `tower\_lsp` found in codebase) by renaming/replacing all occurrences of literal `"tower\_lsp"` in any metadata/documentation files under `.agents/` with `"tower\_lsp"` or `"tower\_lsp"`.
   - To be absolutely safe from any regex matching on `.contains` by the `anti-llm-cheat-lsp` scanner, avoid using the method name `.contains(...)` on strings in all `tools/` source files. If you need substring checks, write a helper function (e.g., matching byte slices with `.windows().any(...)`).

3. **Milestone 4: Benchmark Coverage**:
   - Limit `bcinr-bench-auditor` check directory to only `crates/bcinr-logic/src/algorithms` instead of scanning the entire `crates/bcinr-logic/src`, as helper/internal modules are not part of public primitives. Verify that `bcinr-bench-auditor` compiles and exits with 0 (success).

4. **Verification**:
   - Run the LSP scan command:
     `cargo run --manifest-path /Users/sac/lsp-max/Cargo.toml --package anti-llm-cheat-lsp -- scan --dir /Users/sac/bcinr`
     and verify that it exits with 0 diagnostics.
   - Verify that all checks, tests, clippy, and gates compile and pass cleanly.
   - Write a detailed handoff report to `/Users/sac/bcinr/.agents/worker_m2_m3_m4/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## 2026-06-13T03:00:21Z

**Context**: Important coordination update: E2E test runner must be rewritten in Rust.
**Content**: The E2E Testing Track Orchestrator has published `TEST_INFRA.md` and `TEST_READY.md`. However, they implemented the E2E test runner as a Python script (`tests/e2e_test_runner.py`). To comply with our hard constraint prohibiting Python usage for code verification, you must implement/rewrite the E2E test runner as a Rust integration test (e.g. `bcinr/tests/e2e.rs` so it runs via `cargo test --test e2e` or similar cargo test target) that executes the exact same validations (F1-F5 workspace checks, contract gates, formatting, bench audits, and anti-llm-cheat-lsp scans).
**Action**: Please implement this Rust-based E2E test runner in the workspace (e.g., as `bcinr/tests/e2e.rs`), ensure it performs all 60 checks described in `tests/e2e_test_runner.py` using `std::process::Command`, set `CARGO_TARGET_DIR` to `/tmp/bcinr-e2e-target`, handle cleanup/tearDown properly (such as using a wrapper or a custom Drop structure to clean up temp files), run it via cargo test, verify that all E2E tests pass, and report back when finished.
