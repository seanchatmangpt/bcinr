I have investigated how the `bcinr-cmca` crate handles the `ContractionMarginInsufficient` refusal branchlessly on the hot path. Here is the documentation formatted as Markdown:

### Branchless Computation and Refusal of Insufficient Contraction Margin

In accordance with the Radon Law ($CC=1$) and the strict requirements for branchless, zero-allocation execution described in the `AGENTS.md` and `GEMINI.md` architecture mandates, the `bcinr-cmca` hot path allocator (in `crates/bcinr-cmca/src/allocator.rs`) handles stability failures without conditionals. 

Specifically, the hot path must mathematically enforce the static domination law:
$$ G \cdot d \le (1 - \delta) \cdot d $$

Here is how the calculation and refusal are executed branchlessly:

#### 1. Branchless Matrix Multiplication and Inequality Check
Around lines 1552–1565 of `allocator.rs`, the domination law is verified using fully unrolled, straight-line arithmetic via the `unroll_5_static!` macro. No `for` loops or `if` statements are present in the compiled code:

```rust
let mut gd_ok = true;
unroll_5_static!(i, {
    let mut sum_g_d = 0u128;
    unroll_5_static!(j, {
        let g_raw = crate::generated::stability_profile::GAIN_MATRIX[i][j].raw as u128;
        let d_raw = crate::generated::stability_profile::WEIGHT_VECTOR[j].raw as u128;
        sum_g_d += g_raw * d_raw;
    });
    let lhs = sum_g_d / 1_000_000_000;
    
    let d_i_raw = crate::generated::stability_profile::WEIGHT_VECTOR[i].raw as u128;
    let delta_raw = crate::generated::stability_profile::CONTRACTION_MARGIN.raw as u128;
    let rhs = d_i_raw - (delta_raw * d_i_raw / 1_000_000_000);
    
    // Branchlessly accumulate the boolean result
    gd_ok &= lhs <= rhs;
});
```

#### 2. Bitwise Aggregation of Errors
Instead of early returning if `gd_ok` is false, it gets incorporated into a single unified error bitmask using bitwise operations:

```rust
let has_error = !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

#### 3. Refusal Mapping
The `allocator.rs` maps bit-flags inside a fixed `RefusalSet`. When `gd_ok` is false, it raises the internal `RefusalSet::PROPOSAL_REJECTED` flag, driven branchlessly by the `masked` function.

```rust
let gated_refusals = RefusalSet::EMPTY
    // ...
    .union(
        RefusalSet::PROPOSAL_REJECTED
            .masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32),
    )
    // ...
```
When bridging back to legacy `Result` shapes, `RefusalSet::primary_reason()` maps this internally to `StabilityRefusal::ContractionMarginInsufficient`.

#### 4. Constant-Time State Reversion
Because the authoritative hot path is forbidden from mutating persistent state speculatively (Rule 10: "No mutation before complete admission"), the implementation constructs a candidate state and then applies it via a constant-time masked commit `select_nnf`. 

```rust
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],        // original state if refused
            local_weights[v & 7][e & 7],  // new state if admitted
        );
    });
});
```

By substituting branches with arithmetic masks and data unrolling, the hot path effectively audits the contraction margin while maintaining its perfect $CC=1$ cyclomatic complexity constraint.
