Here is the documentation on how the branchless mode switch masks execution based on certificate state in `bcinr-cmca`:

# The Branchless Mode Switch: Missing vs. Invalid Certificates

In compliance with the BCINR constitution (laws #10 and #11), the hot-path authoritative allocator (`bcinr_cmca::allocator::allocate`) implements adaptive learning freeze and refusal write-back isolation purely through bitwise polynomial masking, without a single `if`, `match`, or early return.

The allocator handles missing and invalid stability certificates (`proof: Option<&AdaptiveUpdate<CertifiedLearning>>`) using two distinct masked execution strategies:

### 1. Missing Certificate: Graceful Degradation (Frozen Learning)
When no certificate is provided (`proof.is_none()`):
* **State Identification**: `degrade_to_certified_selection` is set to `true`, and `proof_some` is set to `false`.
* **Execution Masking**: Because `proof_some` is false, `update_allowed` and `did_switch` both branchlessly evaluate to `0`. This structurally bypasses the adaptive mutations for `local_weights`, `local_last_switch_t`, and `local_prev_mode`. The candidate state remains bit-for-bit identical to the input state.
* **Refusal Suppression**: The `has_refusal` commit gate is defined as `(has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection`. Since `degrade_to_certified_selection` is true, the gate's condition evaluates to `0`, forcing `has_refusal` to `0`.
* **Write-back Selection**: The state commit `select_nnf(has_refusal as u32, weights, local_weights)` observes a `0` mask and commits `local_weights` (which are exactly the preserved original weights).
* **Return Semantics**: The `.masked(has_refusal as u32)` filter applied to `gated_refusals` erases `RefusalSet::AUTHORITY_MISSING`. As documented in the source code, this bit is `UNREACHABLE_BY_PROOF`. The allocator successfully returns an `AllocationOutcome` without generating a typed refusal, allowing *deterministic selection to continue* while learning is frozen, exactly as mandated by the `ReceiptSound` law.

### 2. Invalid Certificate: Speculative Execution & Refusal Isolation
When a certificate is provided but fails a stability invariant (e.g., `digest_err`, `beta_err`, `dwell_err`):
* **State Identification**: `degrade_to_certified_selection` is `false`, and `proof_some` is `true`.
* **Execution Masking**: Because `proof_some` is true, `update_allowed` may evaluate to `true`. The allocator *speculatively mutates* the `local_weights` in its fixed-size scratch space along the execution path.
* **Refusal Activation**: The invariant violation causes `has_error` to evaluate to `1`. Since `!degrade_to_certified_selection` is also `1`, the `has_refusal` commit gate evaluates to `1`.
* **Write-back Selection**: The state commit `select_nnf(1, weights, local_weights)` observes a `1` mask and discards the speculatively mutated `local_weights`, writing back the original, unmodified `weights` to ensure the persistent state is untouched (satisfying the "No mutation before complete admission" law).
* **Return Semantics**: The `.masked(has_refusal as u32)` filter applied to `gated_refusals` preserves the constructed error flags. The specific typed refusal (`DIGEST_MISMATCH`, `PROPOSAL_REJECTED`, etc.) is successfully embedded in the `AllocationOutcome`.

*(Note: The legacy enum `StabilityRefusal::CertificateMissing` / `RefusalSet::CERTIFICATE_MISSING` remains in the codebase for downstream module compatibility and `primary_reason()` backwards compatibility. However, the hot-path allocator does not directly emit it as a refusal, deferring to the correct semantic state of degraded, frozen learning.)*
