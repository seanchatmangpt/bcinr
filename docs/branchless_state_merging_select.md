# Branchless State Merging (`select_nnf` / B-Calculus) in `bcinr`

Under the strict mandates of the Radon Law ($CC=1$) and the B-Calculus execution model, `bcinr` is prohibited from using traditional control flow (e.g., `if admitted { commit(candidate) } else { rollback() }`) for state mutation. 

Instead, the execution follows a deterministic, constant-time transaction shape governed by **Mask-Based Execution** and **No Mutation Before Complete Admission** rules. Sequential semantic decisions are transformed into bitwise polynomials and full-width mask selections.

Here is how the hot path commits candidate state to persistent memory without branches.

## 1. Full-Width Masks (`CanonicalMask`)

The foundation of branchless selection is the `CanonicalMask`, which ensures condition flags are expanded into full-width bitmasks (`0x00000000` for `FALSE` and `0xFFFFFFFF` for `TRUE`).

```rust
// crates/bcinr-cmca/src/fixed.rs
pub const fn from_lsb(lsb: u32) -> Self {
    Self(0u32.wrapping_sub(lsb & 1))
}

pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```

The mathematical contract for this selection is equivalent to:

$$
\operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b)
$$

This performs a bit-parallel choice without a jump instruction.

## 2. Structured Selection (`select_nnf`)

For fixed-point operations, `bcinr` uses structured types like `NonNegativeFixed` and `SignedFixed`, which seal a numeric value alongside its `NumericFaultSet`. 

When selecting between a candidate state and a fallback state, the system cannot silently discard faults from the chosen path, nor can it let faults from the unchosen path leak into the result. The `select_nnf` function distributes the mask selection over both the value bits and the fault sets simultaneously:

```rust
#[inline(always)]
fn select_nnf(condition: u32, a: NonNegativeFixed, b: NonNegativeFixed) -> NonNegativeFixed {
    let mask = CanonicalMask::from_lsb(condition);
    NonNegativeFixed::from_parts(
        mask.select_u32(a.value_bits(), b.value_bits()),
        mask.select_faults(a.faults(), b.faults()),
    )
}
```

This enforces the numeric-hot-path invariant: the selected alternative's faults survive, the unselected alternative's faults are dropped, and no new faults are silently masked away.

## 3. The Hot Path Transaction Shape

In `allocate()` (`crates/bcinr-cmca/src/allocator.rs`), the mutation of the allocator's state (`weights`, `last_switch_t`, `prev_mode`) strictly follows the constitutional transaction shape:

1. **Current Immutable State:** Read the original state (e.g., `weights[v][e]`).
2. **Derive Candidate State:** Compute all updates locally into stack-allocated candidate variables (`local_weights`, `local_last_switch_t`, `local_prev_mode`) branchlessly.
3. **Verify Predicates:** Accumulate all numeric faults, bounds checks, and policy rules into boolean flags.
4. **Derive Refusal Mask:** Calculate a single, branchless `has_refusal` bit (`1` if rejected, `0` if admitted).
5. **Fieldwise Masked Commit:** Unconditionally write to the persistent reference by selecting between the original and candidate state using `select_nnf`.

```rust
// crates/bcinr-cmca/src/allocator.rs

// 1. Compute single admission/refusal bit
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

// 2. Fieldwise masked commit for multi-dimensional state (weights)
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],       // a (selected if has_refusal == 1, aka rollback)
            local_weights[v & 7][e & 7], // b (selected if has_refusal == 0, aka commit)
        );
    });
});

// 3. Fieldwise masked commit for scalar state
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

### Why this is deterministic

Because there is no `if !has_refusal { ... }` block:
- The candidate state `local_weights` is always computed.
- The original state `weights` is always read.
- The persistent `&mut` reference is *always* overwritten.
- The data written back is governed purely by the output of the bitwise selection polynomial. 

If the state is refused, the function writes back the exact, byte-for-byte identical original state. If admitted, it writes the candidate state. This fulfills the `BCINR` mandate of complete call-graph branchlessness while maintaining mathematical state safety.
