Here is the documentation for `RefusalSet::PROPOSAL_REJECTED` in `crates/bcinr-cmca/src/allocator.rs`:

### Branchless Bitmask Application

The `RefusalSet::PROPOSAL_REJECTED` bitmask is defined as `Self(1 << 6)`. It is accumulated into the final `gated_refusals` set using purely branchless bitwise operations, in adherence to the $CC=1$ rule:

```rust
let gated_refusals = RefusalSet::EMPTY
    // ...
    .union(
        RefusalSet::PROPOSAL_REJECTED
            .masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32),
    )
    // ...
    .masked(has_refusal as u32);
```

The operation uses two constant-time methods:
1. `masked(condition)`: Performs a bitwise filter without branching, returning the `PROPOSAL_REJECTED` bit if `condition == 1` and `0` otherwise, using `self.0 & 0u32.wrapping_sub(condition & 1)`. 
2. `union(other)`: Executes a bitwise OR (`self.0 | other.0`).

At the end, it is globally gated by `has_refusal`, which ensures the refusal is only surfaced when `!degrade_to_certified_selection` (i.e. a proof was provided).

### Structural Condition Represented

`PROPOSAL_REJECTED` represents a **stability-envelope violation**. It fires when a proposed candidate parameter set fails to adhere to the absolute numeric bounds certified in the authoritative stability profile. 

It acts as a catch-all structural boundary for multiple parameter-validation predicates. The bit is set to `1` if *any* of the following branchlessly computed boolean flags indicate an error:

- `!gd_ok`: The proposed stability gain matrix and weight vector fail the contraction condition $G d \leq d - \delta d$ (where $\delta$ is the `CONTRACTION_MARGIN`).
- `lr_err`: The proposed learning rate `zeta` exceeds `ZETA_W_MAX`.
- `beta_err`: The bounded momentum/update factor `beta` exceeds `BETA_M_MAX`.
- `eta_err`: The mixing weight `eta` falls below the minimum bound `ETA_G_MIN`.
- `q_err`: Any lens $q$ value falls outside the absolute hardware/quantization range `[-131072, 131072]`.
- `price_err`: Any leaf price `mu[i]` exceeds the allowed `mu_max`.

When this flag is triggered, the operation completes unconditionally, but the masked state-commit process acts as a no-op, preserving the pre-call weights and mode byte-for-byte.
