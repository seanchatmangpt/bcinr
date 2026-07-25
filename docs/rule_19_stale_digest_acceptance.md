# Research on Rule 19: Meaningful Laws and "Stale Digest Acceptance"

Based on the `AGENTS.md` Constitution, **Rule 19 (Hostile mutation protocol)** mandates that every implementation file must identify at least three load-bearing laws and produce hostile mutants against them. These mutants must be killed by triggering a precise typed refusal or an independent oracle mismatch.

Under **Rule 4 (`@armstrong_fault` — Master of Failure Law)**, **"stale digest acceptance"** is explicitly listed as a meaningful law for counterfactual mutant design.

### What is "Stale Digest Acceptance" in Context?
In the BCINR deterministic substrate, state transitions and adaptive mutations are strictly governed by certificates, receipts, and masks (as defined in **Rule 11: ReceiptSound law**). A digest represents a cryptographic or deterministic proof of state, influence, or authorization (e.g., a certificate digest, envelope receipt, or outcome receipt). 

A **"stale digest acceptance" mutant** is an adversarial modification to the source code that artificially bypasses the freshness or validity check of these digests. Specifically, it modifies the logic to force the runtime to incorrectly accept an old, expired, or previously processed digest instead of safely masking it out or rejecting it.

### Why is Testing Against it Required?
1. **Ensures Strict Refusal Paths (Rule 18):** BCINR requires that invalid operations yield bounded typed refusals, not panics or silent fallbacks. A mutant that forces stale digest acceptance must be caught by a specific typed refusal like `CertificateStale` or `DigestMismatch`. If the test suite simply fails with `assert_ne!` rather than explicitly catching the typed refusal, the suite is considered defective.
2. **Protects State Transition Integrity (Rule 10 & 11):** The constitution strictly forbids "mutation before complete admission." If a stale digest successfully bypasses the policy guard, it would result in unauthorized structural mutations, directly violating the fixed conservation and monotonicity laws of the runtime.
3. **Proves Load-Bearing Structural Law (Rule 24):** "Stale certificate acceptance" is listed as an absolute failure that reduces the Substrate Integrity Score (SIS) to 0. Testing against a stale digest acceptance mutant mathematically proves that the mechanisms enforcing digest freshness are actually structural, load-bearing, and functioning correctly, rather than being "dead-path compliance" (CHEAT-007).
