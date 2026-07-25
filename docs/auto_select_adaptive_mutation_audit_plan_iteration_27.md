# Auto Select Adaptive Mutation Operator: Structural Enforcement and Audit Plan

> **Owner:** `@turing_machine`
> **Phase:** Auto Select Implementation Loop (Iteration 27)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Executive Summary

This is the required structural audit plan for the **Auto Select Adaptive Mutation Operator** ($f_{adapt}$). This operator mathematically updates the `AdmittedControlState` based on accumulated integrated receipt telemetry, effectively closing the MAPE-K loop.

The objective is to guarantee absolute adherence to the BCINR deterministic substrate constitution, ensuring $CC=1$, zero heap allocation, fixed bounded execution work, and no runtime loop backedges in the authoritative call graph, thereby satisfying the **ReceiptSound law** (Rule 11) structurally.

## 2. Source Audit Plan

### 2.1 Cyclomatic Complexity ($CC=1$) Enforcement
* **Target:** The exact implementation module (e.g., `crates/bcinr-powl/src/adaptive_mutation.rs` or the module defining $f_{adapt}$).
* **Criteria:** The `bcinr-cheat-scanner` must run across the entire AST of the implementation. The audit must confirm:
  * Zero instances of `if`, `if let`, `match`, `else`, `while`, `loop`, `break`, `continue`, `?`, or early `return`.
  * Zero `unwrap`, `expect`, `unwrap_or_else`, or branch-bearing checked arithmetic.
  * Zero iterator short-circuiting or variable-bound loops.
* **Mask-based Execution Verification (Rule 9):**
  * Validate that the computation of the final admission mask ($M_{admit} = M_{learning} \land M_{cert} \land M_{env} \land M_{outcome}$) strictly uses bitwise `&`.
  * Ensure state mutation is achieved via a fieldwise masked transaction: `(m & new_val) | (!m & old_val)`.
  * Validate that mathematical refusal mechanisms (like clamping or falling back to a safe distribution on a zero denominator) are enacted completely branchlessly.

### 2.2 Mathematical Operators Verification (Rule 14)
* **Saturating Arithmetic:** Ensure all probability mappings and weight summations explicitly utilize `saturating_add` and `saturating_sub` to prevent overflow wrap-around.
* **Branchless Division:** Verify that any division normalization uses the explicitly admitted branchless division replacements (Rule 14).
* **Fixed-Point Arithmetic:** Verify all gradients strictly use fixed-width bit-shifts and bounds. Floating point is completely prohibited.

### 2.3 Anti-Cheat and Gate Jurisdiction
* Ensure the target module is explicitly included in the gate matrix.
* **Cheat Detection:** Scan for `CHEAT-001` (self-canceling operations), `CHEAT-004` (artificial inflation), and `CHEAT-006` (scanner evasion through macros or aliases).
* Inspect all private helper functions and macro expansions reachable from the authoritative call graph for hidden branches or panics.

## 3. Object-Code Disassembly Audit Plan

### 3.1 Release Profile Targets
* The disassembly audit must target `aarch64-unknown-none` and `x86_64-unknown-none` using the strict release profile (LTO enabled, `opt-level = 3`).

### 3.2 Instruction-Level Restrictions
* **Conditional Jumps:** There must be exactly **zero** `jxx` instructions (x86_64) or `b.xx` instructions (AArch64) within the $f_{adapt}$ symbol.
* **Loop Backedges:** Confirm that the assembly exhibits a straight-line block or static unrolling with no backward jumps to earlier addresses.
* **Floating Point / Division:** Verify the absence of any floating-point or hardware division (`div`, `udiv`, `sdiv`) instructions, as they are non-deterministic or panic-inducing.

### 3.3 Reachability and Symbols
* **No Allocator:** The final binary must contain zero symbols linking to the global allocator (e.g., `__rust_alloc`, `malloc`).
* **No Panics:** Ensure no panic landing pads (`core::panicking::panic`, bounds-check panics, overflow/underflow panics) are present or reachable within the call graph.

## 4. Standing Execution Protocol

1. **Invoke Cheat Scanner:** 
   `cargo make scan-cheats` explicitly targeting the `adaptive_mutation` implementation.
2. **Compile to Target Assembly:** 
   Extract `.s` object output for the overarching adaptive mutation function (`audit_execute_adaptive_mutation`).
3. **Execute Disassembly Auditor:**
   Mechanically parse the assembly output to verify `Conditional jumps = 0`, `Loop backedges = 0`, `Panic path = No`, `Allocator = No`.
4. **Substrate Integrity Score:** 
   Any verified branch or allocation immediately zeroes the SIS and blocks merge. The primitive reaches PhD-Verified status only when all criteria in this audit plan are mechanically satisfied.
