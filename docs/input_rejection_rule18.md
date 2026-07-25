Based on the codebase analysis, here is the research regarding Rule 18 (Unsupported Input Rejection) and how the runtime handles typed refusals, avoids silent fallbacks, and prevents partial state mutations.

### 1. Bounded Typed Refusal Codes Instead of Strings
To avoid allocations and branching in the hot path, any failure to meet stability invariants is mapped to a bounded `enum` or bitwise `RefusalSet` instead of human-readable error text. 

For example, `bcinr-cmca/src/allocator.rs` defines `StabilityRefusal`:
```rust
pub enum StabilityRefusal {
    CertificateMissing,
    BlockGainBoundExceeded,
    ContractionMarginInsufficient,
    LearningRateOutsideEnvelope,
    ModeDwellTimeViolated,
    QRangeDestabilizing,
    MassClampUnsafe,
    PriceGainUnsafe,
    NumericRangeExceeded,
    UnsupportedDomain,
    ContractViolation,
    // ...
}
```
Similarly, `crates/bcinr-cmca/src/mode_switch.rs` specifies failures explicitly in `ModeSwitchRefusal`:
- `CertificateDigestMismatch`
- `DwellIdentityMismatch`
- `StaleAdmittedState`

### 2. No Early Returns or Short-Circuiting (Branchless Rejection)
When an unsupported input or domain violation is found (like out-of-bounds `q` lens parameters or a bad certificate digest), the runtime does *not* immediately use `return Err(...)`. Instead, errors are collected via bitwise polynomials to satisfy the $CC=1$ rule. 

In `allocator.rs`, all errors are verified in a branchless sequence:
```rust
let lr_err = const_lt_u32(zeta_w_max_q16, zeta.value_bits()) != 0;
let dwell_err = const_lt_u32(tau_d, MODE_DWELL_ROUNDS_MIN) != 0;

let mut q_err = false;
unroll_4_static!(q_idx, {
    let q_val = lenses[q_idx & 3].q.value_bits();
    q_err |= !(-131072..=131072).contains(&q_val);
});

// Accumulate all typed refusals using boolean or bitwise OR:
let has_error = !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

### 3. Avoiding Silent Clamps, Fallbacks, and Dropped Factors
If a numeric bounds violation or division-by-zero occurs during fixed-point calculations, the runtime doesn't silently clamp values or skip the calculation. Instead, any numeric violations are unioned into a `NumericFaultSet` that bubbles up into the refusal check.

For instance, the arithmetic pathways in `allocator.rs` thread numeric faults precisely:
```rust
let mass_log = node_masses[...].log2();
path_faults = path_faults.union(mass_log.faults()); // Do not drop faults
```
When `select_nnf` (NonNegativeFixed selection) is used, it explicitly selects the correct fault states rather than silently discarding them:
```rust
fn select_nnf(condition: u32, a: NonNegativeFixed, b: NonNegativeFixed) -> NonNegativeFixed {
    let mask = CanonicalMask::from_lsb(condition);
    NonNegativeFixed::from_parts(
        mask.select_u32(a.value_bits(), b.value_bits()),
        mask.select_faults(a.faults(), b.faults()), // Preserves faults across alternatives
    )
}
```

### 4. Preventing Partial State Mutation (Masked Commits)
Rule 18 forbids mutating partial state. The runtime calculates the full sequence of execution and candidate next-states structurally, totally ignoring whether a refusal condition is active.

Only at the very end of the calculation does it apply the bitwise error mask to decide whether to write the result or keep the original state, leaving it *byte-for-byte unchanged* upon refusal (as asserted in `tests/jtbd_refusal_invariance_regression.rs`).

For example, in `allocator.rs`'s authoritative root:
```rust
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

// The commit is masked: if has_refusal == 1, it writes back the original untouched variable
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

### 5. Returning Typed Outcome Wrappers
To avoid using `Result` as control flow (which generates hidden panic paths and branches), the system wraps both the final numeric result and the accumulated refusal flags into an `AllocationOutcome` via a constructor that aggregates all failures:
```rust
let final_refusals = gated_refusals.union(RefusalSet::NO_LEAVES.masked(nl_is_zero));
AllocationOutcome::new_internal(pi_res, local_numeric_faults, final_refusals)
```
Downstream agents or callers can then inspect `outcome.is_refused()` and `.refusals().primary_reason()` to find the mapped typed refusal code, satisfying the requirement to formally and strictly reject all unsupported parameters without branching or partial mutations.
