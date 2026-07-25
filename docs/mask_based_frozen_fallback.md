# Mask-Based Frozen Fallback in `bcinr`

## Overview
Under Rule 11 (ReceiptSound Law), `bcinr` mandates that adaptive mutations to control state are strictly gated by valid receipts (such as `CertifiedLearningMode`). If learning mode is missing or "frozen" (e.g., due to missing proofs or stability boundary violations), deterministic selection must continue, but all adaptive state fields must remain strictly unchanged.

Critically, Rule 11 requires that this fallback must be implemented using **masked state selection, not branching**. This adheres to the project's absolute $CC=1$ law (no `if`, `match`, or data-dependent execution paths), ensuring deterministic execution continues without structural variations.

## Implementation Details

### 1. Branchless Derivation of the "Frozen" Mode
In `crates/bcinr-cmca/src/allocator.rs`, the authoritative hot-path function `allocate()` receives its learning mode receipt as an `Option`: `proof: Option<&AdaptiveUpdate<CertifiedLearning>>`. 

Instead of branching on `proof.is_some()`, the function eagerly evaluates it into boolean values that drive downstream masks:
```rust
let proof_some = proof.is_some();
let degrade_to_certified_selection = proof.is_none();
```
Other control-plane errors (price/learning rate bounds, digest mismatches) are aggregated via pure bitwise `|` operations into a `has_error` boolean flag. This is then used to gate whether updates are allowed:
```rust
let update_allowed = !(switch_wanted & !can_switch) & !freeze_learning & proof_some;
```
If `degrade_to_certified_selection` is true, variables like `update_allowed` and `did_switch` safely evaluate to false (0).

### 2. Unconditional Speculative Execution
Regardless of whether learning is frozen or an error has occurred, the runtime **unconditionally computes the entire candidate state**. The full algorithm—calculating normalized weights, dominator limits, and candidate mode transitions—is executed. No early returns or `else` blocks skip logic. This ensures the CPU performs a fixed bounded execution trace every time, completely eliminating timing side-channels.

### 3. Mask-Based State Selection
The actual commit or fallback happens at the very end of the function using fixed-width bitwise selection functions like `select_nnf` (for `NonNegativeFixed` types) and `const_select_u32`.

Rather than executing a mutating `if` block, the substrate constructs a full-width mask and applies it to the "current" and "candidate" states.
```rust
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],           // Fallback (Current State)
            local_weights[v & 7][e & 7],     // Candidate State
        );
    });
});

*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

### 4. CanonicalMask Machinery
Under the hood, these functions rely on `CanonicalMask` defined in `crates/bcinr-cmca/src/fixed.rs`. A boolean-equivalent condition (like `has_refusal as u32` or `update_allowed as u32`) is translated into a full 32-bit mask (`0x00000000` or `0xFFFFFFFF`) via wrapping negation:
```rust
#[inline(always)]
pub const fn from_lsb(lsb: u32) -> Self {
    Self(0u32.wrapping_sub(lsb & 1))
}
```
The selection operates purely via bitwise arithmetic without branching jumps:
```rust
#[inline(always)]
pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```
*(Also distributing faults safely via `select_faults` for `bcinr`'s numeric tracking).*

## Summary Guarantee
When `proof` is `None` (frozen fallback), `update_allowed` becomes false. The bitwise masks force the write-back phase to select the **original pre-call state variables**, rewriting `weights`, `last_switch_t`, and `prev_mode` with their byte-identical historical values. 

As highlighted in the documentation and verified in test matrices (e.g. `tests/jtbd_sequential_state_evolution.rs`), this structural implementation strictly fulfills the ReceiptSound Law. The system achieves a robust degraded fallback purely through boolean logic gates and bitwise masked overwrites, adhering faithfully to the overarching $CC=1$ deterministic substrate constraints.
