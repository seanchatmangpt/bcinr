# Analysis of `mutant_3` in the `bcinr` Codebase

Based on the investigation across the repository, `mutant_3` refers to several distinct adversarial mutations designed to test the robustness of the deterministic substrate against mathematical and logic deviations. The primary focus is on the `bcinr-cmca` crate, with other variants existing in the `powl` and `logic` crates.

## 1. `crates/bcinr-cmca/tests/hostile_mutants.rs` (Primary)

### **Mathematical Law Broken**
In the CMCA allocator (`src/allocator.rs`), `mutant_3` breaks the **flat-share normalization law**. The law dictates that each leaf's flat allocation share must be correctly normalized by the true sum of sibling leaf weights (`lw_sum`).
The mutant breaks this by bypassing the condition (`c_cond`) and forcing the leaf-weight-sum denominator (`lw_denom`) to a constant `NonNegativeFixed::ONE` regardless of the actual computed `lw_sum`.

### **Expected Outcome & Refusal**
The test function `kill_mutant_3_broken_normalization` enforces the typed refusal by running the allocation tree and verifying its corrupted output. 
- The mutated allocation array must **exactly match** a specific predefined corrupted baseline named `WRONG_M3_BROKEN_NORMALIZATION`. 
- It must explicitly fail to match the correct baseline (`CORRECT_TREE`) via an `assert_ne!` guard. 
- The `MUTANT_KILL_MATRIX.md` confirms this mutant is successfully `KILLED_BY_INTENDED_ORACLE`.

---

## 2. Other Instances in the Codebase

### `crates/bcinr-powl/src/admit.rs` (Admission Control)
- **Law Broken**: "Off-by-One Comparison Offset". It mutates the greater-than-or-equal-to comparison (`ge_mask`) by dropping a `-1` wrapping subtraction offset (`wrapping_sub(x as i64)`). This corrupts the threshold boundaries for process admission topologies (e.g., dropping priority or standard tenants incorrectly).
- **Expected Outcome**: Defeated by the `kill_mutant_3_off_by_one_offset` test, which triggers `verify_mutant_failure(admit_dpag_mutant_3)`, asserting a strict deviation from the deterministic policy.

### `crates/bcinr-powl/src/compiler.rs` (Powl Execution Tape)
- **Law Broken**: "LoopRedo Admittance Corruption". The mutant omits an essential `is_not_redo` activity check (`let active = in_bounds` instead of properly checking the instruction boundaries) when calculating the reachability mask for Powl execution tapes. This breaks the law stating only non-redo active ops must be strictly reachable from the entry point.
- **Expected Outcome**: Defeated in tape generation tests. The mutation causes the reachability output `m3` to mismatch the expected mask, allowing the test harness to assert `killed_mutant_3 = true`.

### `crates/bcinr-logic/src/algorithms/*` (Math Primitives)
- **Law Broken**: Counterfactual low-byte XOR-mutation (`reference(val, aux) ^ 0xFF`). Breaks bitwise equivalence for primitive fixed-point, hash, and SIMD mathematical functions (e.g., `sigmoid_sat_u32`, `wyhash_64`, `crc32c`).
- **Expected Outcome**: Proptest suites checking mathematical determinism catch the deviation across the exhaustive `u64`/`u32` domains against an independent oracle.
