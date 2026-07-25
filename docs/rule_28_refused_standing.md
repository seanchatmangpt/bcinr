# The `REFUSED` Standing in BCINR

According to **Rule 28 (Standing vocabulary)** of the `AGENTS.md` constitution, the `REFUSED` standing is defined as:

> **`REFUSED`**: The input or configuration is outside the admitted domain.

## What it Means
The `REFUSED` standing indicates that the runtime has successfully intercepted and explicitly rejected an input, state, or configuration because it violates the mathematical contracts, bounds, or supported policies of the authoritative primitive. In the BCINR deterministic substrate, reaching a `REFUSED` state is a safe and required failure mode when presented with invalid data.

## Conditions Triggering a `REFUSED` Standing
Code is expected to produce a `REFUSED` standing (in the form of a typed refusal code) when it encounters configurations outside the admitted domain. Based on the broader constitution (specifically **Rule 18: Typed refusals**), these conditions include:

1. **Mathematical and Domain Violations:**
   - The input is outside the supported domain (`UnsupportedDomain`).
   - A mathematical contract or stability bound is breached (`ContractViolation`, `ContractionMarginInsufficient`).
   - Numeric constraints are exceeded (`NumericRangeExceeded`).

2. **State and Lifecycle Violations:**
   - Adaptive mutations are attempted when learning is disabled (`LearningFrozen`).
   - Unadmitted control states or violated mode dwellings (`ControlStateUnadmitted`, `ModeDwellViolated`).

3. **Cryptographic and Integrity Validation Failures:**
   - Digests, certificates, or receipts are missing, stale, or mismatched (`DigestMismatch`, `CertificateMissing`, `CertificateStale`, `ReceiptMissing`, `ReceiptRejected`).
   - Envelopes are violated (`EnvelopeViolated`).

4. **Environment and Verification Guardrails:**
   - Target architecture mismatches, such as missing hardware instruction support with no valid fallback (`SupportMismatch`).
   - Structural audits fail or scanner evasion is found (`ObjectCodeAuditFailed`, `BranchlessContractFailed`, `CheatDetected`).

### Mandatory Laws of Refusal
When an input causes a `REFUSED` condition, the authoritative runtime must obey strict invariants:
- **No Side Effects:** The system must leave persistent state bit-for-bit unchanged (Rule 10: "No mutation before complete admission").
- **No Panics or Fallbacks:** The code may not panic, drop a factor, silently clamp outside of policy, fall back to a simpler algorithm, or return a plausible default (Rule 18).
- **No Human Text:** The rejection must be a bounded, fixed-width typed code rather than a human-readable string in the hot path.
