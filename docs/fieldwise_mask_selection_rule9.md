# Branchless Fieldwise State Selection (Rule 9)

Under Rule 9 (**Mask-based execution law**) and the absolute $CC=1$ strictures, `bcinr` prohibits the use of conditional branches (`if`/`else`) to decide whether to commit or rollback state mutations. Instead, logic is translated into full-width bitwise masks, and state updates are applied using fieldwise structured selection polynomials.

This guarantees a **deterministic, constant-time transaction shape** where the execution cost and instruction path are identical regardless of whether a semantic operation is accepted or refused.

## 1. The Core `CanonicalMask`

The foundation of the branchless selection is expanding a binary condition into a full-width bitmask (`0x00000000` for `FALSE` and `0xFFFFFFFF` for `TRUE`):

```rust
// from crates/bcinr-cmca/src/fixed.rs
pub const fn from_lsb(lsb: u32) -> Self {
    Self(0u32.wrapping_sub(lsb & 1))
}

pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```

The mathematical contract for this operation is:
$$
\operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b)
$$

## 2. Fieldwise Structured Selection

When dealing with complex structured states—such as `NonNegativeFixed` which contains both value bits and a sealed `NumericFaultSet`—`bcinr` cannot simply discard faults on the unchosen path or accidentally merge them. 

The framework handles this by explicitly distributing the selection mask across **all inner fields** simultaneously, rather than wrapping the struct in an `if`. For instance, `select_nnf` ensures that only the numeric faults corresponding to the selected execution path survive:

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

This structural propagation prevents "phantom faults" from an unselected branch from polluting the admitted state, while satisfying the requirement that no jumps or branches are emitted.

## 3. The Commit Transaction Shape

This mechanism completely replaces traditional control-flow validation. The mutation of persistent multi-dimensional state follows a strict 5-step transaction:

1. **Current Immutable State:** Read the existing canonical state.
2. **Derive Candidate State:** Calculate all proposed updates unconditionally into local stack variables.
3. **Verify Predicates:** Accumulate bounds checks, policy rules, and numeric faults into a single, branchless indicator bit (`has_refusal`).
4. **Derive Refusal Mask:** Create a full-width mask from the refusal indicator.
5. **Fieldwise Masked Commit:** Unconditionally write back to the persistent structure by selecting field-by-field.

### Example: Masked Commit of Multi-Dimensional State

```rust
// 1. Compute single admission/refusal bit branchlessly
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

// 2. Fieldwise masked commit for multi-dimensional state
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],       // 'a' is selected if refused (Rollback)
            local_weights[v & 7][e & 7], // 'b' is selected if admitted (Commit)
        );
    });
});

// 3. Fieldwise masked commit for scalar state
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
```

By removing standard branching, the system guarantees that the same $O(N)$ work is done regardless of the semantic outcome, avoiding timing side-channels and providing physical determinism in the hot path.
