# Branchless Refusal Propagation in `bcinr-cmca`

The BCINR project strictly adheres to the **Radon Law ($CC=1$)**, which forbids early returns (`?`), panics, and data-dependent control flow (`match`, `if`). To comply with this constraint, the authoritative root function `allocate` in `crates/bcinr-cmca/src/allocator.rs` replaces standard Rust error-handling with deterministic, bitwise accumulation.

Here is exactly how `StabilityRefusal` errors are detected, propagated, and applied branchlessly.

## 1. Representation: `RefusalSet`
While the legacy `StabilityRefusal` is a standard Rust `enum`, enums fundamentally require branching to evaluate and pattern match. In the hot path, refusals are instead tracked via `RefusalSet`—a struct wrapping a `u32` bitmask where each bit maps to a distinct failure mode (e.g., `DIGEST_MISMATCH`, `DWELL_UNSATISFIED`, `PROPOSAL_REJECTED`).

## 2. Eager Condition Evaluation
Instead of branching when a bounds violation is detected, `allocate` eagerly evaluates every mathematical constraint into an integer flag (`0` or `1`) using constant-time helper functions like `const_lt_u32`:

```rust
let lr_err = const_lt_u32(zeta_w_max_q16, zeta.value_bits()) != 0;
let eta_err = const_lt_u32(eta.value_bits(), eta_g_min_q16) != 0;

// Aggregate error flags bitwise
let has_error = !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

## 3. Masked Accumulation
Refusals are accumulated using `.masked(condition)` and `.union()` rather than `if` statements. The `masked` function applies a bitwise multiplier to select the error bit if the condition is `1`, or `0` if the condition is `0`:

```rust
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```

The error flags are unioned together into a final `RefusalSet`:

```rust
let gated_refusals = RefusalSet::EMPTY
    .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32))
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    .union(
        RefusalSet::PROPOSAL_REJECTED
            .masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32),
    )
    .union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
    .masked(has_refusal as u32);

let final_refusals = gated_refusals.union(RefusalSet::NO_LEAVES.masked(nl_is_zero));
```

## 4. The Commit Gate (State Protection)
According to the absolute runtime laws, an error must leave the system in an unchanged state. Because early returns are forbidden, `allocate` computes all candidate updates in local memory. Before returning, it writes state changes back to the caller's mutable pointers—but gates this write-back with constant-time selection (`select_nnf` and `const_select_u32`) based on the aggregated `has_refusal` bit:

```rust
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],       // Keep original on refusal
            local_weights[v & 7][e & 7], // Commit update on success
        );
    });
});
```

## 5. Returning `AllocationOutcome`
The `allocate` function strictly avoids using `Result` natively to ensure totality. Instead, it returns an `AllocationOutcome` struct that bundles:
1. The allocated candidate array
2. Any accumulated `NumericFaultSet` 
3. The aggregated `RefusalSet` 

Callers outside of the $CC=1$ authoritative bounds (on the "slow rail") can convert this structured data back into standard Rust idioms using `.into_result()`, which inspects the `RefusalSet` and projects it down to a legacy `Err(StabilityRefusal::...)`.
