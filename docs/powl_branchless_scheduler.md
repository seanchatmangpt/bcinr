# POWL Branchless Scheduler Tick Execution

The POWL scheduler evaluates state transitions using branchless SIMD-within-a-register (SWAR) arithmetic, entirely eliminating data-dependent control flow (like `if` statements). It operates on bitmasks representing up to 64 operations in a `PowlTape`. 

Here is the exact step-by-step bitwise logic used in a tick, as implemented in `crates/bcinr-powl/src/scheduler.rs` and `const_scheduler.rs`:

## 1. Candidate Selection
Instead of iterating over all operations and checking their state, the scheduler isolates only the active operations that have not yet completed:
```rust
let mut candidates = state.check_mask & !state.done_mask;
```
It iterates through `candidates` using `trailing_zeros()` to isolate each candidate bit.

## 2. Branchless Precondition Check (`pred_mask`)
For a candidate operation to fire, all its required predecessors must be done.
The scheduler checks `pred_mask` against `new_done` (or a `done_snapshot` in `const_scheduler.rs`):
```rust
let unmet = op.pred_mask & !new_done;
```
If `unmet` is 0, the preconditions are satisfied. To map this boolean state into a full-width mask (`0` or `u64::MAX`) without branching, it uses the following bitwise arithmetic:
```rust
// (unmet == 0) is 1 if satisfied, 0 otherwise.
// wrapping_sub maps 0 -> 0, 1 -> u64::MAX
let sat_mask = 0u64.wrapping_sub((unmet == 0) as u64); 
```

## 3. Branchless Firing (`fire_mask`)
The full-width `sat_mask` is then masked down to the specific bit for this operation (`bit = 1u64 << i`):
```rust
let sat_bit = sat_mask & 1;
let fire_mask = u64::wrapping_sub(0, sat_bit) & bit;
```
If the operation fires, `fire_mask` equals the operation's `bit`. If it doesn't, `fire_mask` equals `0`. 

## 4. State Updates
The `fire_mask` directly drives state mutations using bitwise `OR` operations:
```rust
fired |= fire_mask;
new_done |= fire_mask;
```

## 5. Successor Enablement (`succ_mask`)
If the operation fired, its successors must be added to the `check_mask` for the next tick. The scheduler shifts `fire_mask` down to bit 0 to get a `1` or `0`, expands it to a full-width mask, and applies it to `succ_mask`:
```rust
let fired_this = fire_mask >> i; // 1 or 0
new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);
```

## 6. Complex Control Flow via Masks (`OpKind` Check)
For operations that dictate control flow (like XOR choices or loops), the scheduler checks the `OpKind` branchlessly:
```rust
let diff = (op.kind as u8) ^ (OpKind::XorDispatch as u8);
let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
let is_xor_mask = nz.wrapping_sub(1); // u64::MAX if equal, 0 if not
```
All side-effects (like recording an XOR choice or incrementing a loop counter) are executed universally. However, the final state mutation is conditionally applied by bitwise `AND`-ing the result with `is_xor_mask` and `fire_mask`. Incorrect mutations are safely zeroed out, strictly maintaining deterministic, branchless execution.

## Fully Unrolled Compile-Time Tick (`const_scheduler.rs`)
In `const_scheduler.rs`, when the POWL topology is statically known, the `while candidates != 0` loop overhead is entirely eliminated. The scheduler uses a `ConstTopology` to precompute the firing order, allowing the compiler to unroll the loop into purely sequential arithmetic instructions (e.g., `AND`, `SUBS`, `CSINV`, `ORRS`), executing the tick in fractions of a nanosecond with strict zero-branching.
