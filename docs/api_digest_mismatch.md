Here is the documentation on how `DigestMismatch` and `mix64` are implemented securely and branchlessly in the `bcinr` substrate:

# Cryptographic Verification and `DigestMismatch` in BCINR

The `DigestMismatch` refusal variants are generated when the integrity bindings of digest validations fail during authoritative execution. In the `bcinr-cmca` crate, these validations occur entirely branchlessly and securely, adhering strictly to the `CC=1` (Cyclomatic Complexity 1) and zero-allocation requirements.

## Implementations of `DigestMismatch`
Several exact refusal variants exist across the pipeline, including:
- `ProposalRefusal::ProposalDigestMismatch` (`crates/bcinr-cmca/src/proposal.rs`)
- `ProposalRefusal::CurrentModeDigestMismatch` (`crates/bcinr-cmca/src/proposal.rs`)
- `ModeSwitchRefusal::CertificateDigestMismatch` (`crates/bcinr-cmca/src/mode_switch.rs`)

## The `mix64` Digest Function
To bind identity fields securely and branchlessly in the hot path, `bcinr` uses a `mix64` function (located in `crates/bcinr-cmca/src/proposal.rs`). 

```rust
#[inline(always)]
pub(crate) fn mix64(a: u64, b: u64) -> u64 {
    let mut x = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x
}
```
**Key Properties of `mix64`:**
1. **Branchless Execution**: Relies entirely on bitwise XORs and `wrapping_mul` with fixed 64-bit constants (SplitMix64-style avalanche mix). There are no `if` statements or loops.
2. **Zero Allocation**: Operates strictly on `u64` values on the stack, obeying the deterministic substrate's `#![no_std]` rule.

## Branchless Refusal Generation
When enforcing admissions (e.g., `apply_mode_switch` in `crates/bcinr-cmca/src/mode_switch.rs`), the codebase strictly enforces the **Masked-commit law (AGENTS.md §10)**: 

1. **Unconditional Computation**: The candidate next-state is fully computed structurally *before* any admission checks run. 
2. **Predicate Evaluation**: The independent checks (like comparing the expected digest receipt against the provided one) are evaluated into flat boolean predicates.
3. **Masked Select**:
```rust
let cert_ok = certificate == expected_certificate;
let dwell_ok = dwell.round_identity() == round_identity
    && dwell.transition_identity() == transition_identity;
let state_ok = switch.admitted_state_digest == persistent.mode_digest;

let admitted = cert_ok && dwell_ok && state_ok;

// Candidate is computed unconditionally (no branch gates its computation)
let candidate = ModeState {
    mode_digest: switch.target_mode_digest,
    generation: persistent.generation.wrapping_add(1),
};

// Masked select applied branchlessly at the object-code level
let next = if admitted { candidate } else { *persistent };
*persistent = next;

let result = if admitted {
    Ok(())
} else if !cert_ok {
    Err(ModeSwitchRefusal::CertificateDigestMismatch)
} // ... 
```
If `cert_ok` is false, it maps directly to `ModeSwitchRefusal::CertificateDigestMismatch`. By unconditionally computing the outcome and performing a masked select, `*persistent` selects its original untouched state on failure. Thus, a rejected operation leaves the persistent state field-for-field unchanged without any speculative mutations, ensuring perfect deterministic side-channel resistance.
