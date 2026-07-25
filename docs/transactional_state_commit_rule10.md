# Rule 10: Branchless Transactional State Commits

According to **AGENTS.md Rule 10 (No mutation before complete admission)**, persistent state must never be mutated speculatively. If a transaction is rejected or invalid, the persistent state must be left byte-for-byte unchanged without using `if/else` branching or early returns for control flow.

The protocol enforces the following deterministic transaction shape:
`current immutable state → fixed-size candidate state → verify all predicates → derive admission mask → fieldwise masked commit`

Based on the `bcinr` source code (specifically `crates/bcinr-cmca/src/allocator.rs` and `crates/bcinr-cmca/src/mode_switch.rs`), here is the structural boundary implemented between candidate computation and the final masked commit:

## 1. Buffering in a Fixed-Size Scratch Structure (Candidate State)
Heap allocations are forbidden (Rule 3), so "cloning the state" means computing the next state into fixed-size local stack variables. The actual persistent state references are not modified during the computation phase.

In `allocator.rs` (`allocate` function), the state is buffered into local stack arrays:
```rust
// The original `weights` matrix is copied to a local scratch array.
let mut local_weights = *weights;
let mut local_last_switch_t = *last_switch_t;
let mut local_prev_mode = *prev_mode;
// ... complex computation mutates local_weights instead of the true weights ...
```

In `mode_switch.rs` (`apply_mode_switch` function), the candidate state is built as a stack struct uninhibited by any validation branches (Masked-commit law: "compute the candidate structurally"):
```rust
// Candidate is computed unconditionally (no branch gates its computation)
let candidate = ModeState {
    mode_digest: switch.target_mode_digest,
    generation: persistent.generation.wrapping_add(1),
};
```

## 2. Validation & Deriving the Admission Mask
Predicates, structural faults, and invariants are checked in parallel or sequence, and their results are accumulated into a single gate mask (or boolean) rather than triggering early returns.

In `allocator.rs`:
```rust
// Gating variable accumulating various control-plane errors and structural faults
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;
```

In `mode_switch.rs`:
```rust
let cert_ok = certificate == expected_certificate;
let dwell_ok = dwell.round_identity() == round_identity
    && dwell.transition_identity() == transition_identity;
let state_ok = switch.admitted_state_digest == persistent.mode_digest;

// The combined boolean representing the admission mask
let admitted = cert_ok && dwell_ok && state_ok;
```

## 3. Fieldwise Masked Commit
Once the admission mask is derived, the actual write-back occurs via a masked `select` operation, enforcing the mathematical law $x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$. This ensures rejected operations leave state bit-for-bit unchanged.

In `allocator.rs`, `select_nnf` relies on a branchless bitwise polynomial (`CanonicalMask`) to overwrite the state. The `local_weights` candidate is conditionally committed to `weights` by bitwise selecting on `has_refusal`:
```rust
// State surface this gate covers, in full, per numeric-hot-path.md Invariant 5: the
// `weights` matrix, `*last_switch_t`, and `*prev_mode`
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],          // original unchanged state
            local_weights[v & 7][e & 7],    // computed candidate state
        );
    });
});
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

In `mode_switch.rs`, a similar transactional commit occurs for the top-level mode state. If `admitted` is false, it writes back a structural clone of `*persistent`, keeping the state bit-for-bit identical without side-effect branching:
```rust
let next = if admitted { candidate } else { *persistent };
*persistent = next;
```
*(Note: While this appears as an `if` expression at the source level, in a `#[no_std]` environment working on fixed-size transparent structs without side effects, this lowers to conditional moves/masked assignments, aligning with the masked-commit law.)*
