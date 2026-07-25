Here are the findings regarding the `CertificateStale` refusal and how it is evaluated in the BCINR substrate:

### How `CertificateStale` is Evaluated

Contrary to conventional systems, **the age of a certificate is NEVER evaluated against an expiration threshold or chronological timestamp.** In the deterministic BCINR substrate, "staleness" is a cryptographic and structural property, not a measure of time. 

As detailed in `docs/rule_18_certificate_stale_refusal.md` and the implementations in `crates/bcinr-cmca/src/`, a certificate becomes stale when its structural binding to the current system state drifts. It triggers if any upstream parameters (e.g., generated tables, target rounds, target digests) have been superseded. 

The conceptual `CertificateStale` refusal (mapped to `RefusalSet::CERTIFICATE_STALE = Self(1 << 2)` in `allocator.rs`) is not constructed directly in the allocator, but is instead realized downstream by two specific typed refusals:

1. **`ModeSwitchRefusal::CertificateDigestMismatch` (`mode_switch.rs`):**
   When applying a mode switch, the system checks if the presented certificate is stale by comparing its cryptographic digest against an independently generated `expected_certificate`'s digest. If the expected state has moved on, the digests will not match, inherently identifying the certificate as a stale, superseded proof.
   *Branchless execution:* This is validated by a structural equality check (e.g., `let cert_ok = certificate == expected_certificate;`), which under the strict $CC=1$ Radon Law is verified using a parallel XOR cascade (`(((state.digest ^ cert.digest) | ...) == 0) as u32`). The boolean result determines the masked selection for the next state, cleanly bypassing data-dependent branching.

2. **`CertificationRefusal::RoundIdentityMismatch` (`certification.rs`):**
   When sealing a certificate, the runtime compares 11 domain-specific bindings (such as `round_identity`, `admitted_graph`, `numeric_profile`) between the `actual` and `expected` structs. A certificate bound to a past round is exactly a stale certificate, and this mismatch directly triggers the refusal.

### Branchless Enforcement & State Mutation (Rule 10)
If the digest or round identity checks fail, the conjunctive condition for a valid certificate breaks. Rather than utilizing `if/else` control flow to abort, the operation calculates the `admitted` mask. The final state update is applied using a masked select operation (`let next = if admitted { candidate } else { *persistent };` which compiles to branchless selection). 

When `CertificateStale` is triggered, the admission mask evaluates to `0`, ensuring that the persistent state is left bit-for-bit unchanged, and the requested adaptive mutation safely fails over to the explicit typed refusal.
