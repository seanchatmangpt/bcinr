# Auto Select Structural Enforcement and Audit Plan

**Owner:** `@turing_machine`
**Jurisdiction:** `mfw-auto-select`
**Objective:** Guarantee absolute adherence to the BCINR deterministic substrate constitution, ensuring $CC=1$, zero heap allocation, and no runtime loop backedges in the authoritative call graph.

## 1. Source Audit Plan

### 1.1 Cyclomatic Complexity ($CC=1$) Enforcement
* **Target:** `/Users/sac/mfw/mfw-auto-select/src/lib.rs` (and any associated generated modules or internal helpers).
* **Criteria:** The `bcinr-cheat-scanner` must run across the entire AST of `mfw-auto-select`. The audit must confirm:
  * Zero instances of `if`, `if let`, `match`, `else`, `while`, `loop`, `break`, `continue`, `?`, or early `return`.
  * Zero `unwrap`, `expect`, `unwrap_or_else`, or branch-bearing checked arithmetic.
  * Zero iterator short-circuiting or variable bounds.
* **Mask-based Execution Verification:**
  * Validate that `select_u32` (or its equivalent) relies solely on bitwise operators (`&`, `|`, `!`, `^`) and two's complement underflow (`wrapping_sub`).
  * Ensure the admission mask is strictly full-width (i.e., `0x00000000` or `0xFFFFFFFF`).

### 1.2 Loop Backedge Elimination
* **Criteria:** The evaluation over the fixed set of 8 candidates must be statically unrolled. No runtime loop structures are permitted. The scanner will verify that the implementation cascades `evaluate_candidate` calls sequentially.

### 1.3 Anti-Cheat and Gate Jurisdiction
* Ensure `mfw-auto-select` is explicitly included in the gate matrix.
* **Cheat Detection:** Scan for `CHEAT-001` (self-canceling operations), `CHEAT-004` (artificial inflation), and `CHEAT-006` (scanner evasion through macros or aliases).
* Confirm that any dependencies brought into the scope of `mfw-auto-select` are rigorously scanned for hidden branches.

## 2. Object-Code Disassembly Audit Plan

### 2.1 Release Profile Targets
* The disassembly audit must target `x86_64-unknown-none` and `aarch64-unknown-none` using the strict release profile (LTO enabled, `opt-level = 3`).

### 2.2 Instruction-Level Restrictions
* **Conditional Jumps:** There must be exactly zero `jxx` instructions (x86_64) or `b.xx` instructions (AArch64) within the `select` and `evaluate_candidate` symbols.
* **Loop Backedges:** Confirm that the assembly exhibits a straight-line block or static unrolling with no backward jumps to earlier addresses.
* **Floating Point / Division:** Verify the absence of any floating-point or hardware division instructions (unless explicitly admitted, which is prohibited for this primitive).

### 2.3 Reachability and Symbols
* **No Allocator:** The final binary must contain zero symbols linking to the global allocator (e.g., `__rust_alloc`, `malloc`).
* **No Panics:** Ensure no panic landing pads (`core::panicking::panic`, bounds-check panics) are present or reachable within the call graph.

## 3. Standing Execution Protocol

1. **Invoke Cheat Scanner:** 
   `cargo make scan-cheats` explicitly targeting `mfw-auto-select` crate paths.
2. **Compile to Target Assembly:** 
   Extract `.s` object output for the overarching `select` function.
3. **Execute Disassembly Auditor:**
   Mechanically parse the assembly output to verify `Conditional jumps = 0`, `Loop backedges = 0`, `Panic path = No`, `Allocator = No`.
4. **Substrate Integrity Score:** 
   Any verified branch or allocation immediately zeroes the SIS and blocks merge. The primitive reaches PhD-Verified status only when all criteria in this audit plan are mechanically satisfied.
