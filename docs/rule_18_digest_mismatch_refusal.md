# Research: DigestMismatch Typed Refusal

## What is `DigestMismatch`?
According to Rule 18 ("Typed refusals") of the BCINR `AGENTS.md` constitution, `DigestMismatch` is a mandatory bounded typed refusal category. It is returned when an authoritative operation is rejected because an input's cryptographic or integrity digest fails to match the expected, certified value. Rule 18 strictly prohibits handling invalid inputs by panicking, silently clamping, or falling back to simpler algorithms.

## Why is it necessary?
1. **Enforces the `ReceiptSound` Law (Rule 11):** To mutate persistent adaptive state, mutations must satisfy a strict logical conjunction (e.g., `AcceptedCertificate`, `AcceptedEnvelopeReceipt`). `DigestMismatch` acts as the primary gatekeeper, ensuring that no forged, malicious, or corrupted certificates/receipts can alter the system's state.
2. **Preserves Determinism and `CC=1` (Rules 3, 8, 9, 10):** BCINR is an allocation-free, branchless substrate that requires zero data-dependent branches (`CC=1`). By using a typed refusal instead of `panic!` or early returns (`?`), the system avoids hidden branches and unwinding paths. The digest comparison evaluates into a full-width boolean mask used to branchlessly select between the `candidate` state and the `current` state. If mismatched, it selects the unmodified `current` state and the `DigestMismatch` refusal code—ensuring fixed bounded execution work without timing side-channels.

## Scenarios that trigger it
`DigestMismatch` is triggered during adaptive state transitions when processing external inputs whose digests do not match the expected values. Specific implementations found in the codebase include:
- **`CertificateDigestMismatch`** (e.g., in `mode_switch.rs`, `allocator.rs`): Triggered during state actuations when a supplied certificate does not match the expected certificate, such as when attempting to use a stale or superseded certificate.
- **`PayloadDigestMismatch`** (e.g., in `artifact.rs`): Triggered when a generated artifact or payload digest fails validation.
- **`ProposalDigestMismatch`** (e.g., in `proposal.rs`): Triggered when a proposed control delta provides an invalid digest.
- **`CurrentModeDigestMismatch`** (e.g., in `proposal.rs`): Triggered when a proposal's expected mode digest does not match the system's actual current mode digest.
