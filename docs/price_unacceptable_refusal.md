# PriceUnacceptable (PriceGainUnsafe) Typed Refusal

In the `bcinr` deterministic substrate, resource pricing bounds must be verified without violating the $CC=1$ (Cyclomatic Complexity = 1) Radon Law. The hot path cannot use control flow (like `if`, `match`, or early returns) to reject an unacceptable price.

Instead, the pricing bounds are structurally evaluated and integrated into the phase admission and state-commit boundary using straight-line boolean arithmetic and fixed-width masks.

## Branchless Evaluation of Pricing Bounds

In the cascade allocator (`crates/bcinr-cmca/src/allocator.rs`), resource prices are provided as an array `mu`. The maximum permitted price `mu_max` is defined structurally via bit representation:
```rust
let mu_max = NonNegativeFixed::from_value_bits(6553600);
```

The system evaluates whether any provided price exceeds this maximum by unrolling the check over the fixed node array bounds. It accumulates a boolean `price_err` flag using `const_lt_u32`:

```rust
let mut price_err = false;
unroll_8_static!(i, {
    price_err |= const_lt_u32(mu_max.value_bits(), mu[i & 7].value_bits()) != 0;
});
```

Because `const_lt_u32` operates on the underlying bit representations branchlessly, the CPU executes exactly the same instructions regardless of whether the price is bounded properly or excessively high.

## Generation of the Refusal Set

The `price_err` is then bitwise OR'd with other control-plane assertions (such as learning rate bounds, mode dwell time violations, and divergence errors) to form a unified `has_error` bit:

```rust
let has_error =
    !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

This error bit serves as a structural multiplexer to generate the typed refusal. Specifically, the system uses mask-based accumulation into a bitfield (`RefusalSet`):

```rust
let gated_refusals = RefusalSet::EMPTY
    // ...
    .union(
        RefusalSet::PROPOSAL_REJECTED
            .masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32),
    )
    .masked(has_refusal as u32);
```

*(Note: While explicitly tracked in the legacy 1D enum as `StabilityRefusal::PriceGainUnsafe`, the modernized `RefusalSet` bitfield intentionally collapses unacceptable pricing into the broader `PROPOSAL_REJECTED` mask branchlessly.)*

## Constant-Time State Commit

If the price bounds are violated, an early `return Err(...)` is strictly prohibited by the substrate constitution. Instead, the runtime completes the entire cascading allocation projection and MWU calculation as if the proposal were valid.

The refusal is enforced exclusively at the state-commit boundary using `has_refusal`. The runtime uses a field-wise masked assignment (`select_nnf`) to discard the rejected mutations:

```rust
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],       // Keep previous state if refused
            local_weights[v & 7][e & 7], // Commit new state if accepted
        );
    });
});
```

Because `select_nnf` simply uses bitwise masks to pick between the prior valid state and the newly computed candidate state, the system successfully filters out the `PriceUnacceptable` scenario deterministically, yielding exactly the same instruction sequence and execution time footprint in both the accepted and rejected code paths.
