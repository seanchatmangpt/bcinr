# Enforcement of the `ReceiptSound` Law (Rule 11)

Based on the exploration of `crates/bcinr-cmca/src/allocator.rs`, the `ReceiptSound` law is structurally enforced through a cryptographic authorization token and branchless state masking. 

## Structural Binding of the 5 Required Types

Rule 11 mandates that adaptive mutation cannot occur without all 5 certified receipts. In the codebase, this is structurally bound through the `AdaptiveUpdate<CertifiedLearning>::admit_adaptive_update` constructor function.

To obtain an `AdaptiveUpdate` authorization token, the caller must provide all 5 required types by value (not by reference or option):

1. `state: AdmittedControlState` (AdmittedControlState)
2. `cert: CertificateReceipt` (AcceptedCertificate)
3. `env: EnvelopeReceipt` (AcceptedEnvelopeReceipt)
4. `outcome: OutcomeReceipt` (AcceptedOutcomeReceipt)
5. `_mode: CertifiedLearning` (CertifiedLearningMode)

Before mutation is authorized, the system structurally binds these pieces by asserting they all originate from the exact same transaction. This is done branchlessly by XOR-ing their digests:

```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;
```

The function only returns `Some(AdaptiveUpdate)` if all receipts match and stability thresholds (temperature, distinguishability) are satisfied.

## Mask-Based Mutation Guarding

The hot-path mutation function, `allocate()`, enforces the final mutation boundary by taking the authorization token as an argument:
`proof: Option<&AdaptiveUpdate<CertifiedLearning>>`

In compliance with Rule 11 ("When learning is frozen... deterministic selection may continue; all adaptive state fields remain unchanged"), `allocate()` uses the proof to branchlessly control the learning state:

1. **Proof Extraction:** 
   ```rust
   let proof_some = proof.is_some();
   let degrade_to_certified_selection = proof.is_none();
   ```

2. **Branchless Freezing:** 
   The `proof_some` flag is logically ANDed with the state update conditions:
   ```rust
   let update_allowed = !(switch_wanted & !can_switch) & !freeze_learning & proof_some;
   let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;
   ```
   If no valid proof is provided, both `update_allowed` and `did_switch` evaluate to `0`. 

3. **Byte-for-byte Conservation:**
   Because `update_allowed` is zero, the intermediate weight matrices (`local_weights`) are never updated with new payoffs. When the function performs its final branchless selection to commit the state back to memory, it effectively writes back the exact original state bit-for-bit:
   ```rust
   weights[v & 7][e & 7] = select_nnf(
       has_refusal as u32,
       weights[v & 7][e & 7],
       local_weights[v & 7][e & 7],
   );
   ```

This guarantees that without structurally presenting all 5 bound receipts at once, learning is strictly frozen without introducing any runtime panics, early returns, or conditional branches.
