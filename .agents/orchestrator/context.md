# Context

## Workspace
- Root: `/Users/sac/bcinr`
- Primary code locations: `crates/bcinr-logic/` and workspace members

## Key Goals
1. Audit the `bcinr` codebase, resolve correctness/precedence/invariant issues.
2. Verify compliance using `anti-llm-cheat-lsp` canary scanner.
3. Prepare `v26.6.12` for release.

## Current Audit Blockers
- **Victory Rejected**: 234 files in `crates/bcinr-logic/src/algorithms/` have dummy category-specific hash implementations (Patterns 1-4) instead of genuine mathematical logic.
- We must restore genuine implementations from git history and ensure they compute correct values without any dummy/facade logic or self-certifying tests.
- `TEST_READY.md` has an outdated E2E command reference (`python3 tests/e2e_test_runner.py`).

## Acceptance Criteria
- Core logic in `crates/bcinr-logic/` compiles with zero warnings and runs all tests successfully.
- No `if` or `match` blocks or data-dependent loops exist in the public primitive logic (`crates/bcinr-logic/src/algorithms/`).
- Substrate Integrity Score (SIS) matches 100/100 across the algorithm index.
- The `anti-llm-cheat-lsp` scanner exits with 0 diagnostics.
- Version and routing checks adhere to the inverted LSP laws.
- No plain `tower_lsp` usage.
- All algorithms use genuine, functional branchless implementations.
