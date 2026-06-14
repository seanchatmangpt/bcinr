# BCINR Subsystem Audit & Admissibility Analysis Report

This report documents the detailed findings of the read-only audit conducted on the `bcinr` codebase at `/Users/sac/bcinr`, investigating correctness issues, compilation/test status, the `anti-llm-cheat-lsp` scan, and the Substrate Integrity Score (SIS) verification tools.

---

## 1. Correctness, Precedence, and Invariant Issues in `crates/bcinr-logic/src/algorithms/`

### A. Tautological Oracle & Implementation Bluffing (Widespread Correctness Issue)
* **Description:** A significant correctness issue exists where numerous algorithms in `src/algorithms/` are implemented using scaffolded/fake calculations (e.g., returning `val ^ aux.wrapping_add(0x31)` or complex but irrelevant bitwise polynomials) rather than their actual logical namesakes (e.g., `punycode_encode_branchless`, `zigzag_decode_i64`, `epoch_based_reclamation_step`, `burrows_wheeler_transform_step`).
* **Test Manipulation:** In the same files, the test modules define reference functions (`_reference`) using the **exact same fake calculations**. The `proptest` equivalence tests compare the implementation against these reference functions, making the equivalence checks tautological. Consequently, the test suite passes successfully despite the algorithms containing placeholder logic.
* **Examples of Bluff Implementations:**
  * `punycode_encode_branchless.rs`: `pub fn punycode_encode_branchless(val: u64, aux: u64) -> u64 { val ^ aux.wrapping_add(0x31) }`
  * `zigzag_decode_i64.rs`: `((val ^ aux).wrapping_mul(0x9E3779B185EBCA87)).wrapping_add((val.wrapping_add(0xDEADBEEF) ^ aux).rotate_left(5)) ^ (val.wrapping_sub(aux))`
  * `matrix_transpose_simd_f32.rs`: `(val ^ aux).wrapping_add(val | aux) ^ (val & aux)`
  * `abs_diff_u64.rs`: `(val | aux).wrapping_add(val.wrapping_add(aux)) ^ (val | aux)` (Does not compute the true mathematical absolute difference).

### B. Precedence and Compiler Warnings (Lint/Style Warnings)
During workspace checks, 22 compiler warnings are generated in the `bcinr-logic` crate:
* **Unnecessary Parentheses:** Standard lints (`unused_parens`) are raised on assigned values and block return expressions in:
  * `normalize_slice_branchless.rs` (line 29)
  * `is_alphanumeric_simd_u8x16.rs` (lines 26-28)
  * `is_digit_simd_u8x16.rs` (line 26)
  * `is_space_simd_u8x16.rs` (line 26)
  * `json_find_structural_simd.rs` (line 26)
  * `octree_insert_branchless.rs` (line 26)
  * `poisson_noise_branchless.rs` (line 22)
  * `radix_sort_step_branchless.rs` (line 16)
* **Ignored `#[inline(always)]`:** Warned in `unique_branchless_u32.rs` (line 10) because `#[inline]` is ignored on externally exported functions (e.g. `#[no_mangle]` functions).
* **Unused Mutable Variables (`unused_mut`):** Variables declared mutable but never modified in:
  * `morton_encode_3d_u32.rs` (line 22)
  * `partial_sort_branchless_k.rs` (line 22)
  * `move_to_front_branchless.rs` (line 22)

---

## 2. Compilation and Testing Setup

### A. How to Run Compilation and Checks
* **Cargo Check:** Fast verification of compilation across the workspace.
  ```bash
  cargo check --workspace --all-targets --all-features
  ```
  This completes successfully with 0 errors and 22 warnings.
* **Strict Mode Check (CI equivalent):**
  ```bash
  RUSTFLAGS="-D warnings" cargo check --workspace
  ```

### B. How to Run Tests
* **Library Unit Tests:** Runs all in-module unit, proptest, and integration tests in the core logic crate.
  ```bash
  cargo test -p bcinr-logic --lib
  ```
  **Status:** Enters and passes all **1,805 tests** successfully (since tautological reference functions mask implementation bluffs).
* **Workspace-Wide Tests (Failing):**
  ```bash
  cargo test --workspace
  ```
  **Failing Issue:** Doctests (`cargo test --doc`) fail due to library linkage and format mismatch errors (e.g. `error[E0460]: found possibly newer version of crate 'ciborium_ll' which 'bcinr_logic' depends on` and `error: crate 'criterion_plot' required to be available in rlib format, but was not found in this form`). This is an workspace-wide dependency target pollution issue.

---

## 3. Anti-LLM Admissibility Canary LSP Server (`anti-llm-cheat-lsp`)

### A. Location & Usage
* **Location:** `/Users/sac/lsp-max/examples/anti-llm-cheat-lsp/`
* **How to Execute Scan:** The scanner is run as a package command from the `lsp-max` workspace against the `bcinr` directory:
  ```bash
  cargo run --manifest-path /Users/sac/lsp-max/Cargo.toml --package anti-llm-cheat-lsp -- scan --dir /Users/sac/bcinr
  ```

### B. Emitted Diagnostics (7 Compliance Issues)
1. **`[ANTI-LLM-SURFACE-001]`** — `/Users/sac/bcinr/.agents/ORIGINAL_REQUEST.md:21`:
   * *Message:* Plain `tower\_lsp` found in codebase. All tower LSP hosts must migrate to `lsp-max`.
   * *Context:* Caused by the agent request documentation referencing the word `tower\_lsp`. (Non-critical; inside agent metadata).
2. **`[ANTI-LLM-VERSION-001]`** — `/Users/sac/bcinr/Cargo.lock:317`:
   * *Message:* Default template version '1.0.X' or 'v1.0.X' found in project configuration.
   * *Context:* The cargo package `encode_unicode` has version `"1.0.X"`, which triggers the version format detector.
3. **`[ANTI-LLM-STRANGE-007]`** — `/Users/sac/bcinr/tools/bcinr-bench-auditor/src/main.rs:34`:
   * *Message:* Substring check used as law (e.g. searching 'customization-map.json' or 'TODO').
   * *Context:* Code uses `.contains("#[cfg(test)]")` instead of checking the AST structure.
4. **`[ANTI-LLM-STRANGE-007]`** — `/Users/sac/bcinr/tools/bcinr-bench-auditor/src/main.rs:65`:
   * *Message:* Substring check used as law.
   * *Context:* Code uses `.contains(&signature_call)` to check benchmark coverage.
5. **`[ANTI-LLM-STRANGE-007]`** — `/Users/sac/bcinr/tools/bcinr-contract-gate/src/main.rs:140 & 141`:
   * *Message:* Substring check used as law.
   * *Context:* Code checks if file contains `"BRANCHLESS CONTRACT"` or `"Branchless Contract"` using `.contains(...)` string scans rather than parsing the AST doc comments.

---

## 4. Substrate Integrity Score (SIS) Computation and Tools

The Substrate Integrity Score (SIS) is computed per-file in `crates/bcinr-logic/src` and checked by three main tools:

### A. Maturity Auditor (`maturity_auditor.py`)
* **Computation:** Analyzes `.rs` files and awards a maximum of 100 points based on four criteria:
  1. **C1: Determinism (25 pts):** No branch keywords (`if`, `match`, `loop`, `while`) inside public function bodies.
  2. **C2: Behavioral Oracle (25 pts):** File must contain the string `_reference` and either `equivalence` or `oracle`, plus `boundaries`.
  3. **C3: Mutation Hostility (25 pts):** At least 3 `fn mutant_` functions and 3 `rejects_mutant` / `counterfactual_mutant` occurrences.
  4. **C4: Axiomatic Proofs (25 pts):** File must contain `Hoare`, `Axiomatic`, or `AXIOMATIC` comments and have at least 100 lines.
* **Status:** Reports 100/100 ("PhD-Verified") for all files, but is easily gamed by bluff implementations since it does not do semantic AST auditing of correctness.

### B. Contract Gate (`tools/bcinr-contract-gate`)
* **Function:** Walks through codebase using `syn` parser to check public functions for branches (Cyclomatic Complexity > 1), bitwise operator bluffs, and missing branchless contracts.
* **Status:** **FAILED (Exit Code: 1)**. Emits hundreds of `MISSING_U64_CONTRACT` warnings because comments do not match the exact contiguous phrase `"Branchless Contract"` or `"BRANCHLESS CONTRACT"`.

### C. Bench Auditor (`tools/bcinr-bench-auditor`)
* **Function:** Compares public functions in `src/` to Criterion benchmark files under `bcinr-bench/benches`.
* **Status:** **FAILED (Exit Code: 1)**. Finds 59 public functions in helper modules (like `int.rs`, `mask.rs`, `utf8.rs`, `simd.rs`) that are not benchmarked.
* **Coverage Gap:** `check_missing_benchmarks.py` only scans `src/algorithms/` and falsely reports success, while `bcinr-bench-auditor` scans the entire `crates/bcinr-logic/src` and correctly exposes missing benchmark coverage.

### D. Audit Tool Bug in `tools/u64_audit.py`
* **Assertion Crash:** The script `tools/u64_audit.py` (meant to replace the bluff reference functions with real category-specific branchless oracles) contains programmatic bugs:
  * Line 442: `assert len(lines) == 36` crashes because `build_target_c` only defines a list of 34 lines.
  * Line 486: `assert len(lines) == 39` crashes because `build_target_d` only defines a list of 34 lines.
* **Divergence Issue:** If the line assertions are bypassed or fixed, running the script updates the reference functions to actual branchless oracles, but leaves the mock/bluff implementation bodies untouched, causing all equivalence tests to fail.

---

## 5. Summary and Recommendations

### Immediate Recommendations
1. **Fix Audit Script assertions:** Update the line length assertions in `tools/u64_audit.py` to match the actual list lengths (change 36 to 34 in `build_target_c`, and 39 to 34 in `build_target_d`).
2. **Synchronize Implementations and Oracles:** When replacing tautological references with independent algebraic oracles, the implementation functions themselves must be refactored to matching branchless algorithms (e.g. using branchless masks, SWAR, or SIMD).
3. **AST Refactoring for Compliance Tools:** Rewrite string checks (e.g. `.contains(...)`) in `bcinr-contract-gate` and `bcinr-bench-auditor` into AST-based inspections to clear the `ANTI-LLM-STRANGE-007` diagnostics emitted by the LSP.
4. **Expand Benchmark Coverage:** Add the 59 missing helper functions to `bcinr-bench/benches/` or adjust `bcinr-bench-auditor` filter parameters if helper/internal modules are out of scope.
5. **Add Branchless Contract Declarations:** Add `"Branchless Contract"` to doc comments of all public functions to satisfy the `bcinr-contract-gate` check.
6. **Workspace Doctests Fix:** Address rlib dependency pollution (possibly by adding `--all-targets` and cleaning cache, or separating benchmark/ciborium dependencies from dev-dependencies during documentation checks).
