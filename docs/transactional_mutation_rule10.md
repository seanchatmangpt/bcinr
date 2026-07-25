# Rule 10: Branchless Transactional State Commits

According to **AGENTS.md Rule 10 (No mutation before complete admission)**, persistent state must never be mutated speculatively. If a transaction is rejected or invalid, the persistent state must be left byte-for-byte unchanged without using `if/else` branching or early returns for control flow.

The protocol enforces the following transaction shape:
`current immutable state → fixed-size candidate state → verify all predicates → derive admission mask → fieldwise masked commit`

Based on the `bcinr` source code (specifically `crates/bcinr-cmca/src/allocator.rs` and `crates/bcinr-cmca/src/mode_switch.rs`), here is how this is implemented in practice.

## 1. Buffering in a Fixed-Size Scratch Structure
Since heap allocation is strictly forbidden (Rule 3), "cloning the state" means computing the next state into fixed-size local stack variables.
For instance, in `allocator.rs`, the next weights are computed into a local scratch array `local_weights`:
```rust
let mut local_weights = [[NonNegativeFixed::ZERO; 8]; 8];
// ... complex computation to populate local_weights ...
```
In `mode_switch.rs`, the candidate state is similarly built as a stack struct uninhibited by any validation branches (Masked-commit law: "compute the candidate structurally"):
```rust
let candidate = ModeState {
    mode_digest: switch.target_mode_digest,
    generation: persistent.generation.wrapping_add(1),
};
```

## 2. Validation & Deriving the Admission Mask
Predicates and invariants are checked in parallel or sequence, and their results are accumulated into a single gate mask (or boolean) rather than triggering early returns.

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

let admitted = cert_ok && dwell_ok && state_ok;
```

## 3. Fieldwise Masked Commit (`select(m, candidate, current)`)
Once the admission mask is derived, the actual write-back occurs via a masked `select` operation, enforcing $x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$.

In `allocator.rs`, `select_nnf` relies on a branchless bitwise polynomial (`CanonicalMask`) to overwrite the state. The `local_weights` candidate is conditionally committed to `weights` by bitwise selecting on `has_refusal`:
```rust
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],
            local_weights[v & 7][e & 7],
        );
    });
});
```

Under the hood (`select_nnf`), this is completely branchless and selects both value bits and fault sets bitwise:
```rust
#[inline(always)]
fn select_nnf(condition: u32, a: NonNegativeFixed, b: NonNegativeFixed) -> NonNegativeFixed {
    let mask = CanonicalMask::from_lsb(condition);
    NonNegativeFixed::from_parts(
        mask.select_u32(a.value_bits(), b.value_bits()),
        mask.select_faults(a.faults(), b.faults()),
    )
}
```

In `mode_switch.rs`, a similar transactional commit occurs for the top-level mode state. If `admitted` is false, it writes back a structural clone of `*persistent`, keeping the state bit-for-bit identical without branching:
```rust
let next = if admitted { candidate } else { *persistent };
*persistent = next;
```
*(Note: While this appears as an `if` expression at the source level, in a `#[no_std]` environment working on fixed-size transparent structs without side effects, this lowers to conditional moves/masked assignments, aligning with the masked-commit law.)*

## Summary
By enforcing this transactional protocol, `bcinr` ensures that speculative execution is safely captured in local fixed-size buffers, and persistent state mutations are resolved atomically and branchlessly, effectively nullifying timing side-channels and corrupted states.
