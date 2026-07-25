# Rule 11 (ReceiptSound Law): Cryptographic Receipts and Validation

Under **Rule 11 (ReceiptSound Law)** of the BCINR Deterministic Substrate Constitution, adaptive mutation of persistent state represents a critical boundary. It requires the strict conjunction of five structural proofs. No alternate constructor or API is permitted to bypass this requirement:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

## The Role of `AcceptedEnvelopeReceipt` and `AcceptedOutcomeReceipt`

1. **`AcceptedEnvelopeReceipt` (The Stability Proof):**
   This receipt acts as the structural guarantee that the current execution is safe and bounded. It proves that the execution stays within the mathematical stability envelope (numeric error bounds, admissible domains) established by the `AcceptedCertificate`. If an execution threatens to exceed boundaries, a bounded typed refusal (e.g., `StabilityRefusal::EnvelopeViolated`) is triggered, dropping the selection mask to 0.

2. **`AcceptedOutcomeReceipt` (The Yield Proof):**
   This receipt provides empirical evidence (telemetry/computational yields) to justify an adaptive update within the autonomic (MAPE-K) loop. It proves that any proposed mutation to the adaptive state is backed by verified telemetry and prevents mutations based on speculative or "unwitnessed" operations.

## Structural and Cryptographic Binding in the Hot Path

Both `EnvelopeReceipt` and `OutcomeReceipt` are designed as zero-cost proof tokens, acting as axiomatic witnesses of execution. Structurally, they encapsulate a crate-visible digest:
```rust
pub struct EnvelopeReceipt { pub(crate) digest: u64 }
pub struct OutcomeReceipt { pub(crate) digest: u64 }
```
Because the `digest` field is restricted, external modules cannot trivially construct or mutate them without possessing the necessary verified state.

### Branchless Cryptographic Binding ($CC=1$)

To comply with the constitution's Radon Law ($CC=1$, zero branches), the receipts are structurally bound in the hot path without conditional statements (no `if` or `match`). In `crates/bcinr-cmca/src/allocator.rs`, the `admit_adaptive_update` function strictly mandates these receipts and evaluates their cryptographic alignment using straight-line bitwise operations:

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

**Mechanism Breakdown:**
1. **Cryptographic Verification (`XOR` & `OR`):** The code computes bitwise XOR (`^`) differences between the state digest and each receipt's digest (including the envelope and outcome receipts). A valid cryptographic match produces `0`. All differences are accumulated with bitwise OR (`|`). The combined result is `0` if and only if **all** digests align perfectly.
2. **Boolean to Mask:** The equality check to `0` is converted to an integer mask (`as u32`), outputting `1` (valid) or `0` (invalid).
3. **Data-Oriented Array Selection:** The `digests_ok` mask is combined with other parameter validations (`&`) to generate a safe index of `0` or `1`. This index selects from a static array `[None, Some(Self)]`. 

If any receipt is missing, unverified, or misaligned, the mask deterministically evaluates to `0`. During state application, this `0` mask enforces a fallback behavior (frozen learning) via constant-time bit-level selection:
$$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$
This guarantees that all adaptive state fields remain bit-for-bit mathematically unchanged without ever introducing control-flow branching.
