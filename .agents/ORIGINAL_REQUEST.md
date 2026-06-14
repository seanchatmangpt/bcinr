# Original User Request

## Initial Request — 2026-06-13T02:22:13Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Finish v26.6.12 Release with Anti-Cheat

Audit the `bcinr` codebase, resolve all remaining correctness/precedence/invariant issues, verify compliance using the `anti-llm-cheat-lsp` canary scanner, and prepare `v26.6.12` for release.

Working directory: `/Users/sac/bcinr`

## Requirements

### R1. Complete Release Readiness for v26.6.12
Perform comprehensive audits across all 300 branchless algorithms. Verify that the codebase compiles with zero warnings under `-D warnings` on nightly, and that the unit, doc, and boundary tests all pass.

### R2. Anti-Cheat Canary Admissibility
Use `anti-llm-cheat-lsp` to scan the codebase and ensure it conforms to the admissibility laws:
- No plain `tower\_lsp` usage.
- No victory/overclaim language.
- Resolve any substring checks used as laws.
- Resolve any default template versions (like 1.0.0).

## Acceptance Criteria

### Execution & Soundness
- [ ] Core logic in `crates/bcinr-logic/` compiles with zero warnings and runs all tests successfully.
- [ ] No `if` or `match` blocks or data-dependent loops exist in the public primitive logic (`crates/bcinr-logic/src/algorithms/`).
- [ ] Substrate Integrity Score (SIS) matches 100/100 across the algorithm index.

### Admissibility Criteria
- [ ] The `anti-llm-cheat-lsp` scanner exits with 0 diagnostics when run against the codebase.
- [ ] Version and routing checks adhere to the inverted LSP laws.

## Follow-up — 2026-06-13T04:01:10Z

The user requests to wrap up and finish all remaining tasks now. Please finalize the remaining milestones, execute the final E2E Rust validation suite and anti-llm-cheat-lsp scans, ensure all acceptance criteria are met, and prepare the final handoff report for the v26.6.12 release.

## Follow-up — 2026-06-13T04:38:59Z

Important feedback from user: The point of this release is to provide real, functional branchless implementations of the algorithms, not facade dummy hashes or self-certifying tests. You must instruct the Orchestrator and Implementer subagents to write real implementations that comply with the Radon Law, Hoare invariants, and compile cleanly.

## Follow-up — 2026-06-13T04:39:54Z

Important requirement from user: You must add falsification tests (hostile mutant tests / counterfactual checker tests) to each algorithm module. The test suite must actively falsify and reject incorrect/dummy/fake implementations so that it is impossible to pass the validation gate with fake or facade code. Propagate this immediately.

## Follow-up — 2026-06-13T04:55:30Z

Important user instruction: You must launch 10 parallel subagents immediately within your orchestration hierarchy to split the work of rewriting the 234 algorithm files with real branchless implementations and adding the falsification tests. Coordinate this so that they work concurrently to close all gaps.
