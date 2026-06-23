# Project: Branchless Process Intelligence Library Suite

## Architecture
We implement a branchless, zero-allocation, `#![no_std]` process intelligence suite inside the `playground` crate.
It consists of four primary layers:
1. **petri**: Bitmask-based Petri Net token replay engine.
2. **yawl**: YAWL routing semantics engine supporting OR/AND/XOR splits and joins, cancelling discriminators, and interleaved parallel routing.
3. **powl**: POWL ontology matrix compiler which flattens process trees into non-recursive flat `Powl64Op` array execution via static masks.
4. **wasm**: `#![no_std]` WASM API boundary wrapping the Petri Net and YAWL engines without dynamic allocations.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | E2E Testing Track | Design E2E test infra, write Tiers 1-4 tests, publish `TEST_READY.md` | none | IN_PROGRESS (Conv: 4ec3934d-896b-4d9c-9169-cbf93bab5cbe) |
| 2 | Petri Net Engine | Implement branchless, zero-alloc Petri Net token replay (`petri`) | none | IN_PROGRESS (Conv: 2a11a9ca-8e2d-49ae-949f-1027432776de) |
| 3 | YAWL Routing Engine | Implement branchless YAWL routing semantics (`yawl`) | M2 | IN_PROGRESS (Conv: 2a11a9ca-8e2d-49ae-949f-1027432776de) |
| 4 | POWL Compiler | Implement flat non-recursive POWL execution (`powl`) | M3 | IN_PROGRESS (Conv: 2a11a9ca-8e2d-49ae-949f-1027432776de) |
| 5 | WASM API Boundary | Implement no_std WASM C-interface wrappers (`wasm`) | M2, M3 | IN_PROGRESS (Conv: 2a11a9ca-8e2d-49ae-949f-1027432776de) |
| 6 | Final Integration & Hardening | Pass 100% E2E tests, add adversarial tests (Tier 5) | M1, M2, M3, M4, M5 | IN_PROGRESS (Conv: 2a11a9ca-8e2d-49ae-949f-1027432776de) |

## Interface Contracts
- **CC = 1**: All public primitives must avoid data-dependent branching (no `if`, `match`, or loops checking data). Use bitwise polynomials and mask selection.
- **Zero-Allocation**: No dynamic heap allocations (`Vec`, `Box`, etc. from `alloc`) on the hot execution paths.
- **no_std**: The playground crate must compile with `#![no_std]`.

### Module APIs:
- **petri**:
  - `pub fn petri_fire_transition(marking: &mut u64, in_mask: u64, out_mask: u64, missing: &mut u32, consumed: &mut u32, produced: &mut u32)`
  - `pub fn petri_fire_invisible(marking: &mut u64, inv_in_masks: &[u64], inv_out_masks: &[u64])`
- **yawl**:
  - `BYawlEngine::execute_task_branchless(&mut self, task: &BYawlTask) -> u64`
- **powl**:
  - `pub fn powl64_execute_step(state: &mut PowlState, op: &Powl64Op, input_choice: u64, loop_repeat: u64)`
- **wasm**:
  - `pub unsafe extern "C" fn wasm_petri_replay(...) -> i32`
  - `pub unsafe extern "C" fn wasm_yawl_execute_task(...) -> i32`

## Code Layout
- `playground/src/lib.rs`: Library root containing module exports.
- `playground/src/petri.rs`: Petri net replayer.
- `playground/src/yawl.rs`: YAWL routing engine.
- `playground/src/powl.rs`: POWL compiler/executor.
- `playground/src/wasm.rs`: WASM C-API.
- `playground/tests/`: Integration, differential, and property-based tests.
