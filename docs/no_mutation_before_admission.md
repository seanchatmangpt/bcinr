# Rule 10: No Mutation Before Admission

Rule 10 of the BCINR Deterministic Substrate Constitution states that **persistent state must never be mutated speculatively**. A rejected operation must leave persistent state bit-for-bit unchanged. 

The constitution requires the following specific transaction shape:
`current immutable state -> fixed-size candidate state -> verify all predicates -> derive admission mask -> fieldwise masked commit`

Based on researching `crates/bcinr-cmca`, here is how this pattern is implemented in practice:

## Example 1: Matrix and Scalar State Commit (`allocator.rs`)
In the main allocation hot-path (`src/allocator.rs`), the candidate topology and allocations are computed into local scratch arrays (`local_weights`) and local scalars (`local_last_switch_t`, `local_prev_mode`) without modifying the persistent references.

At the very end of the function, the state is committed by deriving an aggregate mask (`has_refusal`) and using mask-based selection functions like `select_nnf` and `const_select_u32`:

```rust
// 1. Derive admission mask (`has_refusal` acts as the inverted admission mask)
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

// 2. Fieldwise masked commit of the `weights` matrix in bounded iterations
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],        // Keep current immutable state if refused (mask is 1)
            local_weights[v & 7][e & 7],  // Commit candidate state if admitted (mask is 0)
        );
    });
});

// 3. Masked commit of other fixed-size scalar state
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

## Example 2: Mode State Switch (`mode_switch.rs`)
In `src/mode_switch.rs`, the `apply_mode_switch` function explicitly documents its adherence to the "Masked-commit law (AGENTS.md §10)". 

It verifies predicates to compute an `admitted` boolean, computes the candidate next-state unconditionally (no branch gates its computation), and then performs the masked commit.

```rust
// 1. Verify all predicates
let cert_ok = certificate == expected_certificate;
let dwell_ok = dwell.round_identity() == round_identity
    && dwell.transition_identity() == transition_identity;
let state_ok = switch.admitted_state_digest == persistent.mode_digest;

// 2. Derive admission mask
let admitted = cert_ok && dwell_ok && state_ok;

// 3. Compute fixed-size candidate state structurally (unconditionally)
let candidate = ModeState {
    mode_digest: switch.target_mode_digest,
    generation: persistent.generation.wrapping_add(1),
};

// 4. Fieldwise masked commit
// (Selects between candidate and the untouched current state `*persistent`)
let next = if admitted { candidate } else { *persistent };
*persistent = next;
```

This pattern ensures that any failure or refusal condition evaluates entirely off-state, and the final state is only advanced in constant time by selecting the candidate buffers over the original buffers.
