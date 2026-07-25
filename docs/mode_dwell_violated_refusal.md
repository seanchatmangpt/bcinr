# Mode Dwell Violated Refusal (Branchless Evaluation)

In `bcinr`, the deterministic Hot Path avoids standard timer conditionals (e.g., `if elapsed < min { return Err(...); }`) to strictly adhere to the project's $CC=1$ cyclomatic complexity mandate (the Radon Law). Standard conditional logic creates execution branches and data-dependent jumps, which are explicitly forbidden in the authoritative hot path.

Instead of branching, the substrate uses pure bitwise polynomials and canonical masks to evaluate if an adaptive state is attempting to switch modes before fulfilling the required dwell time. This mathematically prevents adaptive state thrashing while maintaining fixed-bound, allocation-free execution.

## 1. What is Mode Dwell Time?

A mode dwell time is the minimum number of rounds (`MODE_DWELL_ROUNDS_MIN` as defined by the stability profile) that an adaptive state or learning mode must persist before it is allowed to transition to a new mode. Enforcing a dwell time guarantees that the MAPE-K autonomic loop does not oscillate or thrash between different control states, ensuring thermodynamic and numerical stability across transitions.

## 2. Bitwise Dwell Time Comparison

The hot path evaluates whether the elapsed dwell rounds (`tau_d`) satisfy the `MODE_DWELL_ROUNDS_MIN` constraint without any control-flow branches. This is evaluated using the branchless `const_lt_u32` polynomial comparator in `bcinr-cmca`:

```rust
let dwell_err = const_lt_u32(
    tau_d,
    crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
);
```

Under the hood, `const_lt_u32` computes `a < b` using a proven bitwise polynomial over two's complement arithmetic, isolating the sign bit of the difference without generating an assembly branch:

```rust
// Polynomial branchless `<` comparison
let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
```

This returns a deterministic `1` (if violated, i.e., `tau_d < MODE_DWELL_ROUNDS_MIN`) or `0` (if satisfied).

## 3. Refusal Masking via Canonical Masks

Once the `1` or `0` violation flag (`dwell_err`) is derived, it is expanded into a full-width canonical mask (where `0` becomes `0x00000000` and `1` becomes `0xFFFFFFFF`) using two's complement wrapping arithmetic: `0u32.wrapping_sub(condition & 1)`. 

The `RefusalSet::masked` implementation uses this canonical mask to inject the `DWELL_UNSATISFIED` (Mode Dwell Violated) refusal entirely through bitwise logic:

```rust
let gated_refusals = RefusalSet::EMPTY
    // ... other refusals
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    // ...
```

If `dwell_err` is `1`, the mask is `0xFFFFFFFF`, and the `DWELL_UNSATISFIED` bit is logically OR'd (`union`) into `gated_refusals`. If `0`, the mask is `0x00000000`, leaving the refusal bit untouched.

## 4. Mask-Based State Transition

Ultimately, the candidate state for the mode switch must be either committed or safely dropped. True to the mask-based execution law (Rule 9/10), this is executed using a bitwise multiplexer (`const_select_u32` / `CanonicalMask::select_u32`) rather than an `if !has_refusal` block:

```rust
*persistent = const_select_u32(has_refusal as u32, *persistent, candidate);
```

If the dwell time was violated, the resulting `has_refusal` bit generates an active canonical mask. The `select` multiplexer then mathematically enforces `(mask & current) | (!mask & candidate)`. The state transition is gracefully reverted to the persistent state bit-for-bit, and the refusal is surfaced upstream as `ModeSwitchRefusal::DwellIdentityMismatch` or `ModeDwellViolated`, successfully avoiding any architectural branching.
