# Independent Oracle Test Architecture in BCINR

The oracle test architecture rigorously enforces the **Independent Oracle Law (Rule 15)** and the **Hostile Mutation Protocol (Rule 19)** to provide deterministic, mathematically proven behavior for branchless execution. 

## 1. Structural and Algorithmic Distinctness (`tests/reference.rs`)
To comply with Rule 15 and avoid the **Circular Oracle** anti-pattern (CHEAT-002), the oracle models deliberately take a fundamentally different execution path than the production code. 
- **Type Segregation:** While production code utilizes allocation-free, branchless fixed-point/integer arithmetic (`NonNegativeFixed`), the oracle uses standard `f64` transliterations. For example, `crates/bcinr-cmca/tests/reference.rs` computes exact float arrays (`compute_measures_f64`, `allocate_f64`) against which the fixed-point structures are bounded.
- **Control Flow Deviation:** The production code strictly enforces `CC=1` (no branches, loops, or panics). In contrast, the `f64` oracles employ traditional control-flow constructs like explicit loops (`for i in 0..N`) and manual `if`/`else` clamps. 
- **Linter Suppression:** The `reference.rs` explicitly suppresses linters (e.g., `#![allow(clippy::needless_range_loop, clippy::manual_clamp, clippy::too_many_arguments)]`) to preserve its purely mathematical translation shape and prevent it from being refactored into production-style idiomatic Rust.

## 2. Bounded Execution \u0026 Hostile Mutants (e.g., `fault_union_production_oracle.rs`)
Under Rule 19, the oracle must act as the ultimate judge for "hostile mutants."
- **Counterfactual Verification:** The test suite defines explicit "mutants" (intentionally flawed behaviors) to demonstrate that the oracle successfully identifies algorithmic corruption. 
- **Example — Fault Unions:** In `crates/bcinr-cmca/tests/fault_union_production_oracle.rs`, the suite ensures the union law (`a | b`) is strictly distinct from a bitwise-XOR mutation (`a ^ b`). By testing idempotence (`a | a == a`), an XOR mutation will fail (`a ^ a == EMPTY`). It structurally validates the three-way accumulation (`faults_out = faults_left UNION faults_right UNION faults_local`) to ensure faults are never silently erased (e.g., testing the "local-only" mutant).

## 3. Hoare-logic Test Architecture (`rl_state.rs`)
In logic-layer tests (like `crates/bcinr-logic/src/autonomic/rl_state.rs`), the architecture follows the "Contract with Teeth: Oracle, Boundaries, 3 Mutants" paradigm:
1. **Positive Oracle (Reference Implementation):** A distinct `checksum_reference` function that establishes the baseline mathematical truth.
2. **Negative Mutants (Flawed Logic):** Inline distinct mutant functions (e.g., replacing `^` with `!`, `&`, or `|`) that represent possible implementation regressions.
3. **Equivalence Assertions:** The test matrix iterates through the mutants to assert that they definitively mismatch the Positive Oracle baseline.

## Summary
The Oracles are implemented as exact mathematical `f64` translations or structurally distinct operations that avoid the `CC=1` and `no_std` zero-allocation constraints of the hot-path. They serve as the isolated Truth models to certify that the tightly bit-packed, zero-branch production code behaves identically (within defined numeric error envelopes) to the literal algorithmic specifications.
