# POWL Execution & Radon Law (CC=1) Documentation

The `crates/bcinr-powl` crate executes partially ordered workflow tapes using `Powl64Op`. Under the BCINR Constitutional Radon Law ($CC=1$), the hot-path execution substrate is required to be fully deterministic and allocation-free, and absolutely forbids branch instructions (e.g., `if`, `else`, `match`, or early returns) to prevent timing side-channels and enforce constant-time semantic transitions.

Here is how opcode dispatch and instruction evaluation are strictly structured using bitwise masks:

## 1. Branchless Structs & `Powl64Op`
`Powl64Op` is a 64-byte flat structure aligned to cache lines that represents a partially ordered workflow node. Its static layout avoids pointer traversal.

```rust
pub struct Powl64Op {
    pub pred_mask: u64,
    pub succ_mask: u64,
    pub ctrl: u64,
    pub op_kind: OpKind,
    // padding for exact 64 bytes cache-line
}
```

## 2. Mask-Based `OpKind` Resolution
Pattern matching (`match op.kind`) is prohibited. Instead, the runtime evaluates tags via bitwise spread operations that resolve to full-width masks (either `u64::MAX` or `0`).
```rust
#[inline(always)]
fn kind_mask(kind: OpKind, target: OpKind) -> u64 {
    let diff = (kind as u8) ^ (target as u8);
    let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
    nz.wrapping_sub(1) // u64::MAX when equal, 0 otherwise
}
```
If an opcode matches, the mask is `u64::MAX`, enabling bitwise selection formulas over `Powl64Op` structures.

## 3. Predicate Evaluation
Readiness checks skip branching (`if required & !done == 0`) and instead mathematically cast the unsatisfied requirements to a `0` or `1`, followed by `wrapping_sub` to generate a full firing mask.
```rust
#[inline(always)]
pub(crate) fn pred_satisfied(done: u64, required: u64) -> u64 {
    let unmet = required & !done;
    0u64.wrapping_sub((unmet == 0) as u64)
}
```

## 4. Branchless Instruction Dispatch
Specific opcode logic (like XOR Choice and Loop Redo) operates by computing bitmask mutations that are conditionally applied via bitwise `AND`, omitting jumps entirely.

### `XorDispatch` Evaluation
A branch in the workflow is chosen deterministically using lowest-set-bit isolation, masking the rest as suppressed.
```rust
let is_xor = kind_mask(op.kind, OpKind::XorDispatch);
let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
let active = is_xor & fire_nz; // Evaluates to true bitmask ONLY if kind matches and firing

// Branchlessly isolate chosen branch (lowest bit set)
let chosen = op.branch_mask & op.branch_mask.wrapping_neg(); 
let suppressed = op.branch_mask & !chosen;

// Mutate state with mask applied
*choice_taken |= chosen & active;
```

### Instruction Specific Override (`Join` Opcode)
Conditionals inside loops (e.g. `if op.kind == Join { ... } else { ... }`) are flattened. Using `is_join`, alternative properties are selected using a boolean equation.
```rust
let is_join = kind_mask(op.kind, OpKind::Join);
let join_effective = op.pred_mask & state.choice_taken;

// Branchlessly mux effective preconditions based on Join mask vs everything else
let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);
```

### `LoopRedo` Logic
Saturating counters handle cyclic jumps by incrementing conditionally. Limits use branchless overflow detection via bit 15.
```rust
let limit_ok = iter_under_limit(*loop_iter, op.branch_count);
let active = is_redo & fire_nz & limit_ok;

// Exactly 0 or 1 increment. No if statement.
*loop_iter = loop_iter.saturating_add((active & 1) as u8);
```

By ensuring every logic branch resolves to an algebraic formula that updates the 64-bit state masks (`done_mask`, `check_mask`), the workflow engine advances states symmetrically for every tick without altering the PC control-flow structure.
