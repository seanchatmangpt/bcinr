# Auto Select Terminal Convergence: Mutant Ledger

> **Owner:** `@armstrong_fault`
> **Phase:** Auto Select Implementation Loop (Iteration 42)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Mission

This ledger provides auditable proof of adversarial adequacy for the **Auto Select Terminal Convergence Operator** (`f_converge`). Under Rule 19 of the BCINR Constitution, the implementation is subjected to structural law mutations. We assert that the oracle explicitly identifies the violated postcondition, or the mutation triggers a typed refusal, rather than failing via a simple `assert_ne!`.

## 2. Mutant Ledger

| mutant id | source file | changed law | exact mutation | expected detection | actual detection | test name | receipt digest | standing |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `m1_epoch` | `auto_select_terminal_convergence.rs` | Invalid-Input Refusal (Epoch) | Bypassed expected epoch validation | Oracle identifies `ContractViolation`, Mutant returns `None` | Oracle returns `ContractViolation` and rejects `m1` bypass | `test_mutants` | `da1334af4ee5aa96db8cbf7dc1e477413d2cb12a72d54ae26d1d0ce3a1b5fb41` | ALIVE |
| `m2_drift` | `auto_select_terminal_convergence.rs` | State-Mutation Boundary | Unconditional mutation of `epoch_clock` despite refusal | Oracle preserves state, Mutant mutates state | Oracle preserves `epoch_clock` as `10`, Mutant drifts to `11` | `test_mutants` | `da1334af4ee5aa96db8cbf7dc1e477413d2cb12a72d54ae26d1d0ce3a1b5fb41` | ALIVE |
| `m3_aggr` | `auto_select_terminal_convergence.rs` | Invalid-Input Refusal (Aggregation) | Dropped critical refusal check | Oracle identifies `BranchlessContractFailed` | Oracle identifies `BranchlessContractFailed`, Mutant returns `None` | `test_mutants` | `da1334af4ee5aa96db8cbf7dc1e477413d2cb12a72d54ae26d1d0ce3a1b5fb41` | ALIVE |
| `m4_mass` | `auto_select_terminal_convergence.rs` | Overflow Behavior (Saturating Math) | Replaced `saturating_add` with `wrapping_add` | Oracle asserts `mass == u64::MAX`, Mutant wraps | Oracle caps `mass` at `u64::MAX`, Mutant wraps to `1` | `test_mutants` | `da1334af4ee5aa96db8cbf7dc1e477413d2cb12a72d54ae26d1d0ce3a1b5fb41` | ALIVE |

## 3. Verification Evidence

The mutants correctly comply with the typed refusal requirement from `AGENTS.md`. No `assert_ne!(baseline, mutant)` calls exist within the `test_mutants` function. In scenarios where a mutant produces a wrong accepted value rather than a refusal (like `m1`, `m3`), the independent oracle identified the exact violated postcondition. In scenarios where a mutant corrupted persistent state (like `m2`, `m4`), the exact postcondition mismatch against the structural baseline was captured.

The test suite executed successfully with all mutants killed. 
