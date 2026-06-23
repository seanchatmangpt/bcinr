# Scope: E2E Testing Track

## Architecture
We design a comprehensive, opaque-box E2E and differential test suite for the 4 process intelligence layers:
- `petri`: Petri net token replay.
- `yawl`: YAWL routing semantics.
- `powl`: POWL flat array execution.
- `wasm`: WASM API boundary.

For differential testing, we implement branching references under `playground/tests/reference/` matching the semantics from the original reference repositories. The E2E tests will compare the branchless implementation against these branching references.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Test Infra & References | Design `TEST_INFRA.md` and implement branching reference logic in `playground/tests/reference/` | none | DONE (Conv: 2386d4a3-9fb0-4151-98f4-ded46b4eca49) |
| 2 | Tiers 1 & 2 Tests | Implement Tier 1 (Feature Coverage) and Tier 2 (Boundary & Corner Cases) tests | M1 | IN_PROGRESS |
| 3 | Tiers 3 & 4 Tests | Implement Tier 3 (Cross-Feature Combinations) and Tier 4 (Real-World Application Scenarios) tests | M2 | PLANNED |
| 4 | Final Integration & Gate | Verify full test suite execution, publish `TEST_READY.md` | M3 | PLANNED |

## Interface Contracts
- **Test methodology**: 4-Tier test methodology.
- **Run Command**: `cargo test` in `/Users/sac/bcinr/playground`.
- **Zero-Allocation**: Tests must execute without performing allocations in the library hot path.
