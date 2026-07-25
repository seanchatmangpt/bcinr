# Rule 10: No Mutation Before Complete Admission in `allocator.rs`

In accordance with Rule 10 of the BCINR Deterministic Substrate Constitution, `crates/bcinr-cmca/src/allocator.rs` rigorously enforces that persistent state is never mutated speculatively. The transaction boundary is sealed using a branchless, fieldwise masked commit that strictly follows the required invariant:

$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

## Transaction Boundary and Masked Commit

In `allocator.rs` (around line 2054+), the state-commit gate evaluates the complete set of stability envelopes and constraints before applying any changes to the mutable state surface. 

### 1. Fixed-Size Candidate State
Instead of modifying the persistent state during the step, the allocator computes updates into fixed-size local stack variables (`local_weights`, `local_last_switch_t`, `local_prev_mode`).

### 2. Derivation of the Admission Mask
The implementation derives a single unified `has_refusal` mask that folds all potential errors and structural checks (like an empty leaf set) into a single bit-mask, making sure to bypass rejections during gracefully degraded selection modes:
```rust
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;
```

### 3. Fieldwise Masked Commit
The actual state mutation is performed using a branchless selection function. If `has_refusal` is true (non-zero), the selection function falls back to the original unmodified state byte-for-byte; otherwise, it commits the candidate values. 

This commit spans the entire mutable surface that `allocate()` is allowed to persist (the `weights` matrix, `*last_switch_t`, and `*prev_mode`):

```rust
// State surface this gate covers, in full, per numeric-hot-path.md Invariant 5:
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],       // Original pre-call state
            local_weights[v & 7][e & 7], // Fixed-size candidate state
        );
    });
});
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

### Conclusion
By utilizing `select_nnf` and `const_select_u32` combined with loop unrolling (`unroll_8_static!`), the allocator ensures a $CC=1$ deterministic output. A rejected operation behaves exactly as a zero-mask application, strictly satisfying the Rule 10 requirement that unadmitted operations leave persistent state bit-for-bit unchanged.
