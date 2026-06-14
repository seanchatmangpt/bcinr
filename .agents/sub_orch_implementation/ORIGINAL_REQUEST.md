# Original User Request

## Initial Request — 2026-06-12T19:29:45-07:00

You are the Implementation Track Orchestrator. Your working directory is `/Users/sac/bcinr/.agents/sub_orch_implementation/`.
Your mission is to coordinate all codebase correctness and compliance fixes to achieve complete release readiness for v26.6.12.
1. Read `/Users/sac/bcinr/PROJECT.md`, `/Users/sac/bcinr/.agents/ORIGINAL_REQUEST.md`, `/Users/sac/bcinr/.agents/teamwork_preview_explorer_init_audit/handoff.md`, and `/Users/sac/bcinr/.agents/teamwork_preview_explorer_init_audit/analysis.md` to understand codebase problems.
2. Create your own `SCOPE.md` under your working directory specifying the implementation scope, architecture, and milestones.
3. Coordinate the execution of these milestones by spawning workers/reviewers/challengers to:
   - Fix `tools/u64_audit.py` line length assertions (from 36/39 to 34).
   - Run `tools/u64_audit.py` to update the references.
   - Refactor the public implementations in `crates/bcinr-logic/src/algorithms/` to match the new oracles branchlessly (specifically category F select, ensuring Radon Law CC=1 is maintained).
   - Add `"Branchless Contract"` comment to doc comments of all algorithm public functions to satisfy contract gate.
   - Resolve 22 compiler/lint warnings in `crates/bcinr-logic/`.
   - Fix workspace doctest failures due to rlib / dependency conflicts.
   - Resolve missing benchmark coverage for the 59 helper functions in `crates/bcinr-logic/src/` (add benchmarks to the Criterion suite or adjust filter logic in `bcinr-bench-auditor` if appropriate).
   - Migrate substring check logic (`.contains(...)`) in `tools/bcinr-contract-gate/src/main.rs` and `tools/bcinr-bench-auditor/src/main.rs` to AST checking using `syn`/parsing tools to eliminate the `ANTI-LLM-STRANGE-007` LSP scan diagnostics.
   - Resolve `ANTI-LLM-VERSION-001` and `ANTI-LLM-SURFACE-001` LSP scan diagnostics (clean `Cargo.lock` version / references).
4. Mandatory Integrity Warning: Always include the verbatim integrity warning in your workers' dispatch prompts:
   "DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected."
5. Monitor `TEST_READY.md`. Once the E2E test suite is published and ready, verify that your implementation passes 100% of the E2E tests.
6. Spawn a Forensic Auditor (`teamwork_preview_auditor`) to verify implementation integrity. If any violation is found, roll back and iterate.
7. Send a message to parent (`dc5fade1-56cc-48e4-a95b-67093600ad13`) with your handoff.md path when done.
