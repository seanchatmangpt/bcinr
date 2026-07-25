# BCINR Rule 11 (ReceiptSound Law) - Branchless Admission Logic

Under the `ReceiptSound` law (Rule 11), adaptive mutation requires the simultaneous presence of five specific conditions:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

In the `bcinr` substrate (specifically `crates/bcinr-cmca/src/allocator.rs`), these five conditions are evaluated structurally and mathematically to construct a branchless admission mask, strictly adhering to the $CC=1$ cyclomatic complexity requirement.

### 1. Structural Binding
The five components are structurally bound as required inputs to the `AdaptiveUpdate::admit_adaptive_update` authorization constructor. The fifth condition, `CertifiedLearningMode`, is statically proven via the type system by requiring the `CertifiedLearning` marker type:

```rust
pub fn admit_adaptive_update(
    state: AdmittedControlState,
    cert: CertificateReceipt,
    env: EnvelopeReceipt,
    outcome: OutcomeReceipt,
    temperature: NonNegativeFixed,
    distinguishability: NonNegativeFixed,
    _mode: CertifiedLearning, // Condition 5
) -> Option<Self> {
```

### 2. Branchless Digest Verification
To mathematically prove that the four runtime state receipts originated from the exact same verified transaction, their cryptographic digests are cross-validated. This is achieved branchlessly by combining bitwise XOR and OR operations. A perfect match resolves to `0`, which is evaluated to a `1` bit via equality comparison:

```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;
```

### 3. Admission Mask Derivation
The digest validation mask (`digests_ok`) is bitwise ANDed with the stability policy validations (`temp_ok` for temperature ceiling, `dist_ok` for distinguishability floor) to create the unified admission mask (`ok`):

```rust
let ok = temp_ok & dist_ok & digests_ok;
```

### 4. Branchless Selection (The Execution)
The `ok` admission mask is then used to branchlessly index into an array of possible outcomes, avoiding any conditional `if` statements or `match` blocks. If any condition fails, the mask collapses to `0`, returning `None`. If all conditions hold, the mask evaluates to `1`, returning the authorized `AdaptiveUpdate` token:

```rust
let outcomes = [
    None,
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

### 5. Masked Commit Gate
In the core authoritative hot path (`allocate` function), the presence of this authorization token is extracted and mathematically combined to construct the final masks that gate state mutation (`update_allowed` and `did_switch`):

```rust
let proof_some = proof.is_some();
// ...
let update_allowed = !(switch_wanted & !can_switch) & !freeze_learning & proof_some;
let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;
```

If the 5 required conditions were not met (making `proof_some = 0`), the mutation masks mathematically collapse to `0`. The selection function then perfectly falls back, leaving the persistent adaptive state bit-for-bit unchanged.
