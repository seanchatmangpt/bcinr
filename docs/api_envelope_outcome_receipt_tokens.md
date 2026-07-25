# Receipt Tokens in the Deterministic Substrate

**Location:** `crates/bcinr-cmca/src/allocator.rs`

## Structural Layout
`EnvelopeReceipt` and `OutcomeReceipt` are designed as zero-cost proof tokens, acting as axiomatic witnesses of execution. Their structural layout is minimal and identical:

```rust
/// Proof token certifying receipt of a valid envelope.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EnvelopeReceipt {
    pub(crate) digest: u64,
}

/// Proof token certifying receipt of a valid outcome.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OutcomeReceipt {
    pub(crate) digest: u64,
}
```

Both structs enforce encapsulation by keeping the `digest` field restricted to crate visibility (`pub(crate)`). The public APIs (`admit_envelope`, `admit_outcome`) merely consume and re-expose the digest, meaning you can't mutate or trivially construct them in external modules without possessing the necessary state. 

## Satisfying the ReceiptSound Law via Branchless Verification
The **ReceiptSound law** (`AGENTS.md` Invariant 11) dictates that adaptive mutation requires all of:
- `AdmittedControlState`
- `AcceptedCertificate`
- `AcceptedEnvelopeReceipt`
- `AcceptedOutcomeReceipt`
- `CertifiedLearningMode`

The implementation satisfies this law exactly via the signature of the `admit_adaptive_update` function in `AdaptiveUpdate<CertifiedLearning>`. It unconditionally requires all of these receipts as arguments:

```rust
pub fn admit_adaptive_update(
    state: AdmittedControlState,
    cert: CertificateReceipt,
    env: EnvelopeReceipt,
    outcome: OutcomeReceipt,
    temperature: NonNegativeFixed,
    distinguishability: NonNegativeFixed,
    _mode: CertifiedLearning,
) -> Option<Self>
```

### Branchless Verification ($CC=1$)
To comply with the strict branchless execution mandates of the substrate, the allocator avoids standard control-flow constructs like `if` or `match`. Instead, it uses straight-line bitwise operations to verify the alignment of all component digests:

```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;

let ok = temp_ok & dist_ok & digests_ok;

let outcomes = [
    None,
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

**How it works:**
1. **Bitwise XOR (`^`)**: Computes differences between the state digest and each receipt's digest. A valid match yields `0`.
2. **Bitwise OR (`|`)**: Accumulates the differences. The aggregated result is `0` if and only if **all** digests match perfectly.
3. **Boolean to Mask**: Converts the equality check into a `0` or `1` integer mask (`as u32`).
4. **Data-Oriented Array Selection**: Combines all verification constraints using Bitwise AND (`&`), generating an index (`0` or `1`). This safely selects `None` or `Some(Self)` from a static array, proving total programmatic correctness without relying on speculative branching.
