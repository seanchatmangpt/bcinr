# CertificateStale Refusal in BCINR

### Exact Definition
Mandated by **Rule 18** of the BCINR constitution, `CertificateStale` is a bounded typed refusal raised when an authoritative operation requiring a verified state context is invoked with a certificate that is no longer valid for the current system state. 

Contrary to conventional systems, "staleness" is a cryptographic and structural property, not a chronological timestamp. A certificate becomes stale when its structural binding to the current system state drifts. This triggers if any upstream parameters (such as generated tables, target rounds, kernel implementations, or bounds) have been superseded, causing the certificate's cryptographic digest ($H_a$) to fail its structural binding. 

The conceptual refusal is mapped to `RefusalSet::CERTIFICATE_STALE = Self(1 << 2)` in the allocator, but its realization is delegated to specific structural checks like `ModeSwitchRefusal::CertificateDigestMismatch` and `CertificationRefusal::RoundIdentityMismatch`.

### Branchless Mathematical Condition
Under the **Radon Law ($CC=1$)**, data-dependent branching (`if`, `match`, early returns) is strictly prohibited. The `CertificateStale` condition is enforced structurally through a branchless equality check on the cryptographic bindings.

The structural evaluation relies on a parallel XOR cascade across the digest fields:
```rust
let is_valid = (((state.digest ^ cert.digest) | ...) == 0) as u32;
```

Rather than using control flow to abort the operation, the condition is mathematically unioned into a numeric `RefusalSet`. The runtime derives a full-width admission mask ($m \in \{0, 2^w-1\}$) from this set. 

If a `CertificateStale` condition accumulates (e.g., the XOR cascade yields a non-zero difference), the admission mask evaluates to `0`. The runtime then calculates the final state using masked selection:
```rust
next_state = State::select(admission_mask, candidate_state, current_state)
```
This guarantees the persistent state remains bit-for-bit unchanged without a single control-flow branch, cleanly preserving the substrate's branchless contract (Rule 10).
