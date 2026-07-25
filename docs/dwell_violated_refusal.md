# Mode Dwell Violated Refusal (Branchless Evaluation)

In `bcinr`, the deterministic Hot Path avoids standard timer conditionals (e.g., `if elapsed < min { return Err(...); }`) to strictly adhere to the project's $CC=1$ cyclomatic complexity mandate (the Radon Law). Standard conditional logic creates execution branches and data-dependent jumps, which are explicitly forbidden in the authoritative hot path.

Instead of branching, the substrate uses pure bitwise polynomials and canonical masks to evaluate if an adaptive state is attempting to switch modes before fulfilling the required dwell time. This mathematically prevents adaptive state thrashing while maintaining fixed-bound, allocation-free execution.

## 1. Bitwise Dwell Time Comparison

The hot path (`bcinr-cmca/src/allocator.rs`) evaluates whether the elapsed dwell rounds (`tau_d`) satisfy the `MODE_DWELL_ROUNDS_MIN` defined by the active stability profile. This is evaluated using the branchless `const_lt_u32` polynomial comparator:

```rust
let dwell_err = const_lt_u32(
    tau_d,
    crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
);
```

Under the hood, `const_lt_u32` determines if `a < b` using a proven bitwise polynomial over two's complement arithmetic, isolating the sign bit of the difference without generating an assembly branch:

```rust
// Polynomial branchless `<` comparison
((a_bb ^ ((a_bb ^ b_bb) | (a_bb.wrapping_sub(b_bb) ^ b_bb))) >> 31) & 1
```

This returns a deterministic `1` (if violated) or `0` (if satisfied).

## 2. Refusal Masking via Canonical Masks

Once the `1` or `0` violation flag (`dwell_err`) is derived, it is expanded into a full-width canonical mask (where `0` becomes `0x00000000` and `1` becomes `0xFFFFFFFF`) using two's complement wrapping arithmetic: `0u32.wrapping_sub(condition & 1)`. 

While `CanonicalMask` explicitly handles this abstraction in `fixed.rs`, the `RefusalSet::masked` implementation in `allocator.rs` directly implements this canonical masking contract to apply the `DWELL_UNSATISFIED` refusal:

```rust
let gated_refusals = RefusalSet::EMPTY
    // ... other refusals
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    // ...
```

If `dwell_err` is `1`, the mask is `0xFFFFFFFF`, and the `DWELL_UNSATISFIED` bit is logically OR'd (`union`) into `gated_refusals`. If `0`, the mask is `0x00000000`, contributing nothing.

## 3. Mask-Based State Transition

Ultimately, the mode switch candidate state must be either committed or safely dropped. True to the mask-based execution law, this is executed using a bitwise multiplexer (`const_select_u32`) rather than an `if !has_refusal` block:

```rust
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

If the dwell time was violated, the resulting `has_refusal` bit generates an active canonical mask. The `select` multiplexer then mathematically enforces `(mask & current) | (!mask & candidate)`. The state transition is gracefully reverted to the persistent state bit-for-bit, and the refusal surfaces upstream structurally as `StabilityRefusal::ModeDwellTimeViolated`.
