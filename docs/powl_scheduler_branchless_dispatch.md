# POWL VM Scheduler Branchless Dispatch

The POWL VM scheduler (`crates/bcinr-powl/src/scheduler.rs`) achieves deterministic, branchless instruction dispatch in compliance with Rule 3 (no dynamic dispatch, no indirect calls) and the rule of zero data-dependent branches (`CC=1`).

It implements a SWAR (SIMD-within-a-register) execution loop where control-flow transitions are computed using bitwise arithmetic and masks, rather than control flow statements.

## 1. Branchless Type Identification
Instead of using `match` or `if` statements on `OpKind` enums, the scheduler uses a `kind_mask()` mathematical function. This returns `u64::MAX` when the operation matches a target kind and `0` otherwise:
```rust
fn kind_mask(kind: OpKind, target: OpKind) -> u64 {
    let diff = (kind as u8) ^ (target as u8);
    let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
    nz.wrapping_sub(1) // u64::MAX when equal, 0 otherwise
}
```

## 2. Predicate Evaluation
Whether an instruction fires is determined by creating a `fire_mask` using bitwise logic, preventing the need for a branching execution path. 

```rust
// Compute effective_pred by selecting behavior based on whether the op is a Join
let is_join = kind_mask(op.kind, OpKind::Join);
let join_effective = op.pred_mask & state.choice_taken;

// Equivalent to `if is_join { join_effective } else { op.pred_mask }`
let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

let sat = pred_satisfied(new_done, effective_pred);
let sat_bit = sat & 1;

// Convert 1 or 0 into `u64::MAX` or `0`, masking it with the op's bit index
let fire_mask = u64::wrapping_sub(0, sat_bit) & bit;

fired |= fire_mask;
new_done |= fire_mask;
```
If an instruction isn't ready to fire, `fire_mask` is `0`, causing the subsequent bitwise additions/updates for execution to become no-ops.

## 3. Dispatch & State Mutations
Specific side-effects like XOR branching or Loop counter modifications are applied sequentially and unconditionally to all instructions. The `fire_mask` and `kind_mask` are used to mask out (nullify) the effects for inappropriate instruction types or ones that didn't fire.

- **`apply_xor_dispatch`**:
  Computes `active = is_xor & fire_nz`, selectively extracting and storing the branch selection mask without conditionals.
  
- **`apply_loop_redo`**:
  Evaluates `active = is_redo & fire_nz & limit_ok`. The per-slot loop counter uses saturating addition instead of an `if` block, incrementing only if the instruction fired and is of the correct type:
  ```rust
  *loop_iter = loop_iter.saturating_add((active & 1) as u8);
  ```

All state mutation is performed using bit-parallel masked selection matching the required equation from the project's rules: $\text{select}(m,a,b) = (m \land a) \lor (\neg m \land b)$.
