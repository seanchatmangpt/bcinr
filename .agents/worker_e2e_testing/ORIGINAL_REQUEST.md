## 2026-06-13T02:31:00Z
You are a Worker subagent (`teamwork_preview_worker`) under the E2E Testing Track Orchestrator.
Your working directory is `/Users/sac/bcinr/.agents/worker_e2e_testing/`.
Your mission is to design, implement, and run a comprehensive, opaque-box E2E test suite for the `bcinr` project, and publish `TEST_INFRA.md` and `TEST_READY.md` at the project root.

### Mandatory Integrity Check
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

### Step 1: Read Project Context
Read `/Users/sac/bcinr/PROJECT.md` and `/Users/sac/bcinr/.agents/ORIGINAL_REQUEST.md`.

### Step 2: Create `TEST_INFRA.md`
Create `/Users/sac/bcinr/TEST_INFRA.md` following the requirements in the E2E Testing Track guidelines. Include:
1. Test Philosophy: Opaque-box, requirement-driven, interface-compatible.
2. Feature Inventory:
   - F1: Workspace Health (Compile, Unit, Doc, Boundary Tests).
   - F2: Contract Gate (Branchless CC=1, Zero-Alloc, Doc Comment).
   - F3: Substrate Integrity Score (SIS via `maturity_auditor.py`).
   - F4: Bench Auditor (Benchmark Coverage).
   - F5: LSP Canary Compliance (`anti-llm-cheat-lsp`).
3. Test Architecture: E2E runner details, input/output formats, directory layout.
4. Real-World Application Scenarios (Tier 4 list).
5. Coverage Thresholds (Tier 1-4 minimum test counts).

### Step 3: Implement E2E Test Suite
Create `/Users/sac/bcinr/tests/e2e_test_runner.py` containing a Python-based E2E test suite.
The suite must cover the 5 features above across 4 tiers with at least the following counts:
- Tier 1 (Feature Coverage): >= 5 test cases per feature (total >= 25)
- Tier 2 (Boundary & Corner Cases): >= 5 test cases per feature (total >= 25)
- Tier 3 (Cross-Feature Combinations): >= 5 test cases total
- Tier 4 (Real-World Application Scenarios): >= 5 test cases total
Total test cases must be at least 60.

To implement Tiers 2-3 effectively:
- Tier 2/3 cases should verify how tools handle invalid files or configurations (e.g. create a temporary file with a branch or missing contract comment in a temp subdirectory or in `crates/bcinr-logic/src/algorithms/` temporarily, run `bcinr-contract-gate` on it, verify it fails, and clean it up).
- Verify how tools handle missing benchmarks (e.g. temporarily add a function to `crates/bcinr-logic/src/int.rs` and verify `bcinr-bench-auditor` reports it).
- Verify how tools handle LSP violations (e.g. temporarily add a file with plain `tower\_lsp` or default version 1.0.X, run the scan, verify it fails, and clean it up).
- Ensure that the tests are completely clean and do not leave any side effects or temporary files in the repository.

For E2E tests that run on the whole repository (like Tier 4), design them to verify the tool's execution and output format. If a tool fails on the current codebase, the test should assert the expected exit code and check for specific error messages (e.g. verifying that contract-gate reports `MISSING_U64_CONTRACT` or CC > 1, or that the LSP scanner reports specific diagnostics). This verifies the tool's functionality dynamically without being blocked by the implementation track's progress.

### Step 4: Run the Test Suite & Verify
Run the E2E test suite. Document the execution command and results in your handoff report. Fix any issues you find. All test cases in the test runner must pass.

### Step 5: Publish `TEST_READY.md`
Publish `/Users/sac/bcinr/TEST_READY.md` at project root with the checklist and results.

### Step 6: Write handoff
Write `/Users/sac/bcinr/.agents/worker_e2e_testing/handoff.md` and finish your task.
