# Auto Select Execution Dispatch Operator Structural Audit (Iteration 26)

**Owner:** `@turing_machine`
**Jurisdiction:** `bcinr-powl` execution scheduler and Auto Select dispatch boundary
**Objective:** Guarantee absolute adherence to the BCINR deterministic substrate constitution for the Auto Select Execution Dispatch Operator ($f_{dispatch}$).

## 1. Source Audit Plan

### 1.1 Cyclomatic Complexity ($CC=1$) Enforcement
* **Target:** The $f_{dispatch}$ function bridging the Auto Select $T_{mask}$ into the branchless POWL VM scheduler.
* **Criteria:** The `bcinr-cheat-scanner` must run across the entire AST of the dispatch layer. The audit must confirm:
  * Zero instances of `if`, `match`, early `return`, `unwrap`, or `?`.
  * Zero loop structures (`for`, `while`, `loop`). Any tape selection must use static unrolling or masked selection.
* **Transactional Masking Verification:** 
  * Validate that tape firing is determined solely using bitwise boolean logic: `T_mask & V_state.ready`.
  * Verify that the final VM state update strictly uses branchless selection: `select_u64(T_mask, V_candidate, V_state)`.

### 1.2 Pipeline Composition Boundary
* **Criteria:** The $f_{dispatch}$ implementation must pass the execution state sequentially as stack-allocated fixed-width structs between components without allocating intermediate memory on the heap.
* **Cheat Detection:** Scan for `CHEAT-001` (self-canceling operations) within the masking operations, ensuring no dead-path compliance (`CHEAT-007`) exists when handling a zeroed $T_{mask}$.

## 2. Object-Code Disassembly Audit Plan

### 2.1 Target Profiles
* **Targets:** `x86_64-unknown-none` and `aarch64-unknown-none`.
* **Profile:** Strict release profile (`opt-level = 3`, LTO enabled).

### 2.2 Instruction-Level Restrictions
* **Zero Conditional Jumps:** There must be zero `jxx` (x86) or `b.cond` (AArch64) instructions linking the tape dispatch. The compiler must produce a monolithic straight-line instruction block.
* **Loop Backedges:** The assembly must be confirmed to have zero backward jumps across the entire call graph of $f_{dispatch}$.
* **No Dynamic Dispatch:** Must verify the absence of `dyn Trait` calls (`call [reg]`) for tape execution.
* **Arithmetic Hardware Traps:** Verify that loop limits and tape iteration counters use strict branchless saturating arithmetic without division traps.

### 2.3 Reachability and Symbols
* **No Allocator:** The dispatch must resolve exactly zero symbols matching `__rust_alloc` or `malloc`.
* **No Panics:** Ensure no panic handlers (`core::panicking::panic`, bounds-check panics) are reachable.

## 3. Standing Execution Protocol

1. **Invoke Cheat Scanner:** Execute `cargo make scan-cheats` explicitly targeting the dispatch file.
2. **Compile to Assembly:** Generate the `.s` file for the exact $f_{dispatch}$ symbol.
3. **Mechanical Disassembly Verification:** Automatically parse the generated `.s` to enforce `Conditional jumps = 0`, `Loop backedges = 0`, `Allocator = No`, `Panic path = No`.
4. **SIS Enforcement:** If any step fails, the Substrate Integrity Score drops to 0, immediately triggering `MaturityScrutiny` and blocking the dispatch merge.
