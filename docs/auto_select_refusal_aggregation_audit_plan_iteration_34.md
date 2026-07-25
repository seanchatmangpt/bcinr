# Auto Select Refusal Aggregation Operator: Structural Enforcement and Audit Plan

> **Owner:** `@turing_machine`
> **Phase:** Auto Select Implementation Loop (Iteration 34)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Constitutional Mission

This document establishes the formal Structural Enforcement and Audit Plan for the **Auto Select Refusal Aggregation Operator** ($f_{refuse}$), ensuring it complies with the BCINR Deterministic Substrate Constitution.

As `@turing_machine` (Enforcer of Determinism), I mandate that the implementation composed by `@von_neumann_bypass` for $f_{refuse}$ must preserve strict $CC=1$ invariants and zero allocations at both the Rust source level and the final object-code disassembly level.

## 2. Structural Requirements

To satisfy the **Radon Law ($CC=1$)** and the **Zero-Allocation Boundary**, the implementation must adhere to the following structural constraints:

### 2.1 Branchless Execution (CC=1)
* **Prohibited Constructs:** The implementation must contain exactly zero `if`, `match`, `while`, `loop`, `?`, or early `return` statements.
* **Refusal Accumulation:** The reduction of all intermediate refusal codes ($r_{base}, r_{adapt}, r_{epoch}, r_{conv}, r_{dispatch}, r_{receipt}, r_{ocel}$) into a single `FullMapekRefusal` code must be performed using bitwise `|` (OR).
* **Masked Masking:** Masking downstream refusals ($r_{dispatch}, r_{receipt}, r_{ocel}$) based on the transaction admission mask ($m_{update\_mask}$) must be implemented using bitwise `&` (AND). Conditional assignment is completely prohibited.

### 2.2 Memory Management
* **Zero Allocation:** The aggregation step must operate entirely in CPU registers or on the fixed-size stack. 
* **No `alloc` Calls:** The generated assembly must have zero references to `malloc`, `free`, `cmca_allocate`, or any global allocator symbols.

### 2.3 Mathematical Folding
* The operations must avoid sequential dependency stalls where possible. Instruction-level parallelism (ILP) should be maximized by accumulating independent refusals concurrently before folding them into the final `$r_{final}$`.

## 3. Object-Code Audit Execution Plan

Once `@von_neumann_bypass` implements $f_{refuse}$ within `execute_full_mapek_loop` (or equivalent integration point), the following steps must be executed to certify the artifact:

### Step 1: Rust Source-Level Scan
Execute the BCINR Cheat Scanner and Contract Gate to statically verify the absence of structural cheats and hidden branches.
```bash
cargo make scan-cheats
cargo make contract-gate
```
**Expected Outcome:** `CC=1` is verified. No cheat patterns detected.

### Step 2: Target Object-Code Disassembly
Generate the raw production-profile assembly dump for the `bcinr-cmca` crate (and `bcinr-logic` if applicable).
```bash
cargo make audit-object-code
```

### Step 3: Per-Symbol Assembly Classification
Manually or mechanically inspect the assembly of the authoritative root symbol (e.g., `execute_full_mapek_loop`) and its transitive helpers, specifically focusing on the instructions generated for $f_{refuse}$.

* **Required Invariants:**
  1. `jxx` (Conditional Jumps): **0** (No branching based on intermediate refusal values).
  2. `jmp` (Loop Backedges): **0** (No dynamic traversal of the refusal set).
  3. Allocator Calls: **0**.
  4. Panic Symbols: **0**.

### Step 4: Mutant Detection Verification
Execute the test suite against the hostile mutants designed by `@armstrong_fault`. The mutants must test early-return bypasses and conditional refusal aggregation.
```bash
cargo make test-mutants
```
**Expected Outcome:** All mutant oracle tests strictly fail, demonstrating that non-branchless deviations cause verifiable timing, behavior, or result changes.

## 4. Substrate Integrity Score (SIS) Certification

The implementation of the Refusal Aggregation Operator will only be classified as **PhD-Verified** ($SIS = 100$) if it seamlessly passes all steps of this structural and object-code audit plan. A single conditional jump or hidden branch in the final assembly reduces the score to $SIS = 0$ and triggers `MaturityScrutiny`.
