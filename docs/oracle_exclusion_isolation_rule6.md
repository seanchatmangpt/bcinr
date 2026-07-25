# Rule 6 (Oracle Exclusion) Implementation in BCINR

Under **Rule 6** of the `AGENTS.md` deterministic constitution, a "Test-only oracle" is an independent mathematical specification used to verify the branchless production implementation. While mandatory for correctness, it must be physically and structurally prevented from being linked into the `#![no_std]`, zero-allocation authoritative hot path. 

Based on a review of the `bcinr` workspace (particularly `crates/bcinr-cmca` and `crates/bcinr-logic`), here is how the physical isolation boundary is rigidly enforced using Rust's compile-time semantics:

## 1. The `tests/` Directory Isolation (Integration Crate Boundary)
The most robust boundary is the use of Cargo's standard `tests/` directory. Files in this directory are treated by `rustc` as entirely separate crates that link *against* the target library. This structural isolation guarantees the oracle code can never be linked into the production binary.

**Example:** In `crates/bcinr-cmca/tests/fault_union_production_oracle.rs`, an independent mathematical oracle is defined to verify the behavior of `NumericFaultSet::union` (ensuring bitwise-OR idempotence).
* Because it lives in `tests/`, it can only interact with the crate's public API. 
* It can safely use `std` or heavy test infrastructure without violating the production library's strict `#![no_std]` requirement.

## 2. Conditional Compilation (`#[cfg(test)]`)
For internal verification and unit-level oracles that need access to internal crate state, the codebase relies on Rust's `#[cfg(test)]` attribute. This guarantees the compiler completely drops the Abstract Syntax Tree (AST) of the oracle logic during a standard build.

* **Module-level gating:** In `crates/bcinr-cmca/src/lib.rs`, entire artifact verification modules are explicitly conditionally compiled:
  ```rust
  // Test-time-only Gamma_CMCA artifact verification (VerifyGeneratedProfile).
  // Gated to #[cfg(test)] because its dependencies (blake3, serde) are dev-dependencies.
  #[cfg(test)]
  pub mod artifact;
  ```
* **Inline module grouping:** Oracles co-located with production logic (e.g., in `crates/bcinr-cmca/src/fixed.rs` or `crates/bcinr-cmca/src/allocator.rs`) are strictly encapsulated under:
  ```rust
  #[cfg(test)]
  mod tests {
      // Independent oracle proofs and tests are fully erased 
      // from the final binary unless built with `cargo test`.
  }
  ```

## 3. Strict `[dev-dependencies]` Segregation
Independent oracles often require heavy external libraries (like `blake3`, `serde`, or `chicago-tdd-tools`) to compute exact mathematical references or digest comparisons.
* In `crates/bcinr-cmca/Cargo.toml`, these are exclusively listed under `[dev-dependencies]`.
* This ensures that the dependency graph of the `#![no_std]` binary remains completely untainted. If production hot-path code accidentally referenced an oracle dependency, the standard `cargo build` would hard-fail at resolution.

## 4. Cargo Feature Gating for Hostile Mutants
The `bcinr` framework verifies the effectiveness of these independent oracles by injecting deliberate faults (mutants) to ensure the oracle triggers a failure.
* This is isolated via explicit default-off Cargo features (`mutant_1` through `mutant_11` defined in `Cargo.toml`).
* The testing pipeline invokes these exclusively during verification:
  ```bash
  cargo test -p bcinr-cmca --features mutant_N --test hostile_mutants <dedicated_oracle_name> -- --exact
  ```
* Because these features are disabled by default and the code is often tested via `tests/` oracles, neither the corrupted implementation nor the oracle assertions can leak into the deterministic branchless runtime.
