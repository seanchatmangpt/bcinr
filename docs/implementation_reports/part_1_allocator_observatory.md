# Implementation Report: Cascade Allocation, Stable Projectors, and Observatory Telemetry

This report details the implementation of the Cascade Allocation, Stable Projectors, and Observatory Telemetry logic within the `bcinr-cmca` crate. The systems outlined here are core components of the Chatman Multifractal Consequence Allocation (CMCA) substrate — "Covariance Monitoring and Calibration Assessment" was this report's earlier, unreconciled expansion (see `../CMCA_EXPLANATION.md` for the canonical name) — adhering to the strict architectural laws of BCINR: zero heap allocations (`no_alloc`) and constant-time execution with no branches (`CC=1`).

---

## 1. Cascade Allocation

**File Path:** `/Users/sac/bcinr/crates/bcinr-cmca/src/allocator.rs`

The Cascade Allocator distributes resource flows hierarchically down a forest structure. The resource allocation algorithm executes in distinct phases without any input-dependent branches or variable loops:

### 1.1 Initial Allocation and Propagation
The initial resource flow is distributed to the roots of the forest based on policy lenses and clipped semantic masses. To achieve `CC=1` and $O(1)$ constant-time execution, the propagation uses fixed unrolled macro iterations (`unroll_8_static!`) rather than `while` or `for` loops.

```rust
// Snippet from `/Users/sac/bcinr/crates/bcinr-cmca/src/allocator.rs`
/// Performs a single straight-line flow propagation step down the node forest.
#[inline(never)]
fn flow_step(
    parent: &[i32; N],
    is_leaf: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    rho: &[NonNegativeFixed; N],
    child_w: &[[NonNegativeFixed; N]; N],
    cw_sum: &[NonNegativeFixed; N],
    leaf_w: &[[NonNegativeFixed; N]; N],
    lw_sum: &[NonNegativeFixed; N],
    alloc_flow: &mut [NonNegativeFixed; N],
    flat_alloc: &mut [NonNegativeFixed; N],
) {
    unroll_8_static!(v, {
        let has_children = !is_leaf[v & 7];
        
        let flat_part = NonNegativeFixed::from_bits(const_select_u32(has_children as u32, ((NonNegativeFixed::ONE - rho[v & 7]) * alloc_flow[v & 7]).val, 0));
        let desc_part = NonNegativeFixed::from_bits(const_select_u32(has_children as u32, (rho[v & 7] * alloc_flow[v & 7]).val, 0));
        
        // ... (denominators computation)
        
        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7] & has_children;
            let flat_addition = flat_part * leaf_w[v & 7][x & 7].saturating_div(NonNegativeFixed::from_bits(lw_denom));
            flat_alloc[x & 7] += NonNegativeFixed::from_bits(const_select_u32(is_sub as u32, flat_addition.val, 0));
            
            let is_child = (parent[x & 7] == v as i32) & has_children;
            let flow_addition = desc_part * child_w[v & 7][x & 7].saturating_div(NonNegativeFixed::from_bits(cw_denom));
            alloc_flow[x & 7] += NonNegativeFixed::from_bits(const_select_u32(is_child as u32, flow_addition.val, 0));
        });
        
        alloc_flow[v & 7] = NonNegativeFixed::from_bits(const_select_u32(has_children as u32, 0, alloc_flow[v & 7].val));
    });
}
```
**`no_alloc` & `CC=1` mechanisms:** The function exclusively uses fixed-size stack arrays (`[T; N]`). Conditional logic like "if this is a subtree leaf" is converted into boolean masks (`is_sub`) and evaluated branchlessly using `const_select_u32`. This avoids `if` statements and branch misprediction side-channels.

---

## 2. Stable Projectors and Explore Floors

After the flow is computed across all models and lenses, it is aggregated into a combined allocation vector (`pi_combined`). The Stable Projector then scales leaf allocations based on resource prices $\mu_x$ and operational costs $c_x$. A minimum explore floor ($\eta$) is also mixed in.

```rust
// Snippet from `/Users/sac/bcinr/crates/bcinr-cmca/src/allocator.rs`
// Inside `allocate` function:
    let psd = NonNegativeFixed::from_bits(const_select_u32(const_eq_u32(priced_sum.val, 0), NonNegativeFixed::ONE.val, priced_sum.val));
    let mut nl = 0u32;
    unroll_8_static!(i, { nl += is_leaf[i & 7] as u32; });
    unroll_8_static!(x, {
        let mu_actual = clip(mu[x & 7], NonNegativeFixed::ZERO, mu_max);
        
        let p_mu = (pi_combined[x & 7] * SignedFixed::from_bits(0i32.wrapping_sub((mu_actual * costs[x & 7]).val as i32)).exp()).saturating_div(psd);
        
        let eta_actual = eta;
        
        // Use lookup table to avoid division
        let mut nl_recip = NonNegativeFixed::ZERO;
        unroll_9_static!(idx, {
            let matches = const_eq_u32(nl, idx as u32);
            nl_recip = NonNegativeFixed::from_bits(const_select_u32(matches, LEAF_RECIP[idx].val, nl_recip.val));
        });
        
        let val = (eta_actual * nl_recip) + ((NonNegativeFixed::ONE - eta_actual) * p_mu);
        let pi_val = pi_res[x & 7];
        pi_res[x & 7] = NonNegativeFixed::from_bits(const_select_u32(is_leaf[x & 7] as u32, val.val, pi_val.val));
    });
```
**`no_alloc` & `CC=1` mechanisms:** Rather than computing `1 / nl` which might trigger division panic or non-constant latency, a fixed array `LEAF_RECIP` is scanned fully. A `const_select_u32` accumulates the correct value. The final projection `val` is committed only if `is_leaf` is true using `const_select_u32`.

---

## 3. Observatory Telemetry Engine

**File Path:** `/Users/sac/bcinr/crates/bcinr-cmca/src/observatory.rs`

The Observatory evaluates calibration metrics (Numerical Uncertainty, Gram Degeneracy, Drifting, Scale Inertia) and computes safety indicators in constant time. 

### 3.1 Branchless Evaluation
Instead of returning early on the first failure (which creates a timing side-channel), it calculates all conditions concurrently using masks, then resolves the priority queue branchlessly.

```rust
// Snippet from `/Users/sac/bcinr/crates/bcinr-cmca/src/observatory.rs`
pub fn evaluate_calibration(
    artifact: &MeasurementArtifact,
    epsilon_on: NonNegativeFixed,
    epsilon_gram: NonNegativeFixed,
    epsilon_drift: NonNegativeFixed,
    s_meas: NonNegativeFixed,
    s_leaf: NonNegativeFixed,
) -> Result<CertificateReceipt, ObservatoryFlag> {
    // ... extracting metrics ...
    
    // Evaluate all conditions branchlessly using mask bits
    let is_drift = const_lt_u32(epsilon_drift.val, d_js.val);
    let is_scale_inert = const_eq_u32(s_meas.val, s_leaf.val);
    
    let kappa_hat_on = const_lt_u32(epsilon_on.val, kappa_hat.val) | const_eq_u32(epsilon_on.val, kappa_hat.val);
    let kappa_under_off = const_lt_u32(kappa_under.val, epsilon_on.val);
    let is_numerically_uncertain = kappa_hat_on & kappa_under_off;
    
    let kappa_under_on = const_lt_u32(epsilon_on.val, kappa_under.val) | const_eq_u32(epsilon_on.val, kappa_under.val);
    let gamma_under_off = const_lt_u32(gamma_min_plus_under.val, epsilon_gram.val);
    let is_gram_degenerate = kappa_under_on & gamma_under_off;
    
    let is_unadmitted = const_eq_u32(artifact.proposal as u32, ModeDelta::Retain as u32);
    let is_recert = kappa_under_on & (!gamma_under_off) & (!is_unadmitted);
    
    // Overwrite the flag to reflect priority queue of errors
    let mut flag = 5u32; // Default to Ok
    flag = const_select_u32(is_recert, 5, flag);
    flag = const_select_u32(is_unadmitted, 4, flag);
    flag = const_select_u32(is_gram_degenerate, 1, flag);
    flag = const_select_u32(is_numerically_uncertain, 0, flag);
    flag = const_select_u32(is_scale_inert, 3, flag);
    flag = const_select_u32(is_drift, 2, flag);
    
    wrap_observatory_result(flag, artifact.control_mode_digest)
}
```

### 3.2 Branchless Result Wrapping
The return uses `Result`, but instantiating it cannot use an `if` statement. A branchless memory selection is employed:

```rust
// Snippet from `/Users/sac/bcinr/crates/bcinr-cmca/src/observatory.rs`
pub fn wrap_observatory_result(
    flag_code: u32,
    digest: u64,
) -> Result<CertificateReceipt, ObservatoryFlag> {
    let flag = FLAGS[(flag_code as usize) & 7];
    let is_recert = const_eq_u32(flag_code, 5);
    let outcomes = [Err(flag), Ok(CertificateReceipt::new(digest))];
    
    // Selects Err or Ok based on whether flag_code matched 5
    outcomes[is_recert as usize]
}
```
**`no_alloc` & `CC=1` mechanisms:** The function creates an array `outcomes` holding both the `Err` and `Ok` variants on the stack. The memory representation handles selection using a branchless index.

---

## 4. Conclusion
Both `allocator.rs` and `observatory.rs` demonstrate highly disciplined adherence to the BCINR rules. Control flows such as dynamic length loops, variable memory requests, and branch evaluation conditions have been explicitly transformed into fixed-loop macro unrolling, mask arrays, and bitwise boolean logic with constant-time selection mechanisms.
