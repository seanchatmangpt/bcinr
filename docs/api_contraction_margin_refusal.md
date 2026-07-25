Here is the documentation on how the `ContractionMarginInsufficient` typed refusal is produced branchlessly:

The `ContractionMarginInsufficient` typed refusal enforces the system's stability contraction margin invariant. Mathematically, it requires that the weighted sum of the gain matrix over the weight vector respects the contraction margin $\delta$:

$$ \widehat{G} d \le (1 - \delta)d $$

### Mathematical Implementation

In the authoritative hot path (`crates/bcinr-cmca/src/allocator.rs`), this check is calculated over a $5 \times 5$ dimension structure using exact 128-bit unsigned integer arithmetic on $10^9$-scaled fixed-point values from the statically generated profile:

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
        gd_ok &= lhs <= rhs;
    });
```

### Branchless Structure ($CC=1$)

This check achieves $O(1)$ constant-time execution with strictly zero branches (`CC=1`), adhering to the "Radon Law", through the following structural guarantees:

1. **Static Unrolling:** The nested loops over the vector and matrix dimensions use the `unroll_5_static!` macro, completely eliminating loop bounds-checking and loop backedges in the generated machine code.
2. **Boolean Accumulation:** The domain bounds check (`lhs <= rhs`) produces a boolean value that is safely bitwise-ANDed into the `gd_ok` flag (`gd_ok &= lhs <= rhs`), rather than relying on short-circuiting or `if` statements.
3. **Masked Accumulation into a RefusalSet:** The negation of `gd_ok` (`!gd_ok`) is bitwise-ORed with other proposal validity flags to form an error boolean mask. This boolean expression is then branchlessly mapped to a fixed-width `RefusalSet` operation:
   ```rust
   .union(
       RefusalSet::PROPOSAL_REJECTED
           .masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32),
   )
   ```
   The `masked(condition)` method applies a bitwise arithmetic mask, guaranteeing that `PROPOSAL_REJECTED` is strictly unioned into the refusal bitfield without branching.

*(Note: When `RefusalSet` is collapsed into a single `StabilityRefusal` enum via `primary_reason()` for legacy callers, the `PROPOSAL_REJECTED` bit explicitly maps to `StabilityRefusal::ContractionMarginInsufficient`. Additionally, `crates/bcinr-cmca/src/stability.rs` implements this same mathematical bounds-check in the slow rail for certificate derivation, which legitimately uses standard `if` branching as permitted by the `AGENTS.md` guidelines).*
