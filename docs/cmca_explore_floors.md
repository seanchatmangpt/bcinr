Here is the documentation on how the "Explore Floors" (`eta / n_L`) mechanism is implemented branchlessly in `allocator.rs` to guarantee minimum search thresholds:

# Cascade Resource Allocator - Explore Floors Mechanism

The "Explore Floors" (`eta / n_L`) mechanism guarantees minimal search and prevents numerical singularity by mixing a uniform exploration floor into the final allocation vector. Under the strict mandates of the Radon Law (`CC=1`, zero branching), this implementation is completely branchless and uses an exact budget conservation scheme to avoid precision loss.

## 1. Branchless Base/Residual Calculation
Instead of relying on a precomputed reciprocal table which might fail to sum exactly to 1.0 (Q16.16 `65536`) when `n_L` doesn't divide evenly, the allocator computes an exact base `q_floor` and residual `r_floor` at runtime.

```rust
let nl_safe = const_select_u32(nl_is_zero, 1, nl);
let q_floor = 65536u32 / nl_safe;
let r_floor = 65536u32 - q_floor * nl_safe;
```
Nodes are assigned a `leaf_rank` branchlessly via `running_rank += is_leaf[x] as u32`.

## 2. Mixing the Explore Floor
For each node in the unrolled loop, if the node's `leaf_rank` is less than `r_floor`, it receives an extra unit of allocation (`gets_extra = 1`). This forms `nl_recip` (the exact uniform share).

The exploration floor is mixed with the price-normalized allocation (`p_mu`) branchlessly. `select_nnf` ensures that the newly calculated value is only committed if the current node is a leaf:

```rust
let gets_extra = const_lt_u32(leaf_rank[x & 7], r_floor);
let nl_recip = NonNegativeFixed::from_value_bits(q_floor + gets_extra);

let val = (eta_actual * nl_recip) + ((NonNegativeFixed::ONE - eta_actual) * p_mu);
pi_res[x & 7] = select_nnf(is_leaf[x & 7] as u32, val, pi_val);
```

## 3. Post-Mixing Truncation Correction (Conservation Law)
Fixed-point operations (`saturating_div` and `saturating_mul`) naturally truncate towards zero, meaning the sum of `pi_res` across all leaves will often under-count the target `1.0` total allocation. The allocator addresses this branchlessly without a magic constant.

1. It sums the actual array values into `leaf_sum` using masked additions.
2. It determines if there is a deficit or excess relative to `NonNegativeFixed::ONE.value_bits()`, computing the absolute `gap`.
3. It breaks `gap` into `gap_q` and `gap_r` exactly as it did for the uniform floor.
4. It distributes `gap_q + gets_extra_unit` correction factors using the same `leaf_rank`.

```rust
let gap = const_select_u32(is_deficit, target_bits.wrapping_sub(leaf_sum), leaf_sum.wrapping_sub(target_bits));
let gap_q = gap_safe / nl_safe;
let gap_r = gap_safe - gap_q * nl_safe;

unroll_8_static!(x, {
    let gets_extra_unit = const_lt_u32(leaf_rank[x & 7], gap_r);
    let bump = gap_q + gets_extra_unit;
    
    let bumped_up = NonNegativeFixed::from_parts(pi_res[x & 7].value_bits().wrapping_add(bump), pi_res[x & 7].faults());
    let bumped_down = NonNegativeFixed::from_parts(pi_res[x & 7].value_bits().wrapping_sub(bump), pi_res[x & 7].faults());
    
    let corrected = select_nnf(is_deficit, bumped_up, select_nnf(is_excess, bumped_down, pi_res[x & 7]));
    pi_res[x & 7] = select_nnf(is_leaf[x & 7] as u32, corrected, pi_res[x & 7]);
});
```
This final phase uses `from_parts` to guarantee the retention of any accumulated `NumericFaultSet` (satisfying numeric-hot-path Invariant 2). No conditional control-flow is used throughout the entire finalization process.
