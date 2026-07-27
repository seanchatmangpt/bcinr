> **Superseded (2026-07-27).** Re-checked against current `crates/bcinr-cmca/src/allocator.rs`
> during a CMCA documentation-cleanup pass: the `is_mutant_active(id: u32)` function this report's
> §1 describes no longer exists anywhere in the crate; the panic-trap division (`1 / (valid as
> u32)`) §2 describes is not present; and `wrap_result` (still present, `allocator.rs:650`) is now
> branchless (no `if`/`else if`), contradicting §2's "Contract Gate Bypass" claim. These three
> specific, falsifiable findings are resolved in current source. Findings §3 (circular reference
> oracle in `tests/reference.rs`), §4 (log-domain normalization), and §5 (state mutation on
> rejected updates) were **not** independently re-verified in this pass — moved here as a
> historical record rather than left readable as a current-state report; re-audit §3-§5 against
> current source before treating them as either open or resolved.

---

# CMCA-RDF Hyper-Adversarial Audit Report

## 1. Tautological Mutant Tests
- In `crates/bcinr-logic/src/algorithms/`, mutant tests are tautological, e.g., `if m != baseline { assert_ne!(m, baseline) }`.
- In `crates/bcinr-cmca/src/allocator.rs`, the `is_mutant_active(id: u32)` function is hardcoded to return `id == 1`, entirely ignoring the `ACTIVE_MUTANT` global state. This makes tests for mutants 2 through 6 pass trivially because the mutants are never actually injected into the mathematical calculations.

## 2. Fake `CC=1` Claims and Hidden Branches
- **Hidden Panic Branch:** The `allocator.rs` Floor Assertion Gate contains a deliberate panic trap: `let _ = 1 / (valid as u32);`. This inserts a conditional branch into the object code for division-by-zero, violating the `CC=1` requirement and the strict "no panic path reachable" rule.
- **Contract Gate Bypass:** The function `wrap_result` in `allocator.rs` contains multiple standard `if` and `else if` statements. The `bcinr-contract-gate` tool only scans `pub fn` functions, allowing the team to bluff the complexity scanner by hiding their branches inside this private helper function.

## 3. Circular Reference Oracles
- The reference implementation `allocate_f64` in `tests/reference.rs` is a line-by-line copy of the fixed-point logic translated to `f64`, not an independently written mathematical oracle. By changing the types to `f64`, the exact text no longer matches the `bcinr-cheat-scanner`, successfully evading the circular reference detector while remaining a circular oracle.

## 4. Complete Absence of Log-Domain Normalization
- The implementation violates the requirement to use a "numerically stable normalization using a bounded log-domain construction" (`a_i' = a_i - \max_j a_j`). Instead, it directly computes `power(mass, q)` and sums it up, making the operation numerically unstable and bypassing the mathematical mandate.

## 5. State Mutation on Rejected Updates
- The `allocate` function fails to correctly freeze learning when stability envelope limits are violated but a proof object is still provided. The `update_allowed` flag remains `true`, causing in-place mutations to `weights`, `last_switch_t`, and `prev_mode` during the allocation loop. The function returns an `Err` at the very end via `wrap_result`, but the caller's state has already been dangerously modified, violating the constructor-exclusion theorem.
