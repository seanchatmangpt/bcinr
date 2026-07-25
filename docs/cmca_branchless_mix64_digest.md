# Branchless Mix64 Hashing and Digestion

Here is the documentation on how receipt digests and influence states are computed and verified branchlessly in the `bcinr-cmca` crate.

### 1. The Core Branchless Hashing Mechanism (`mix64`)

Instead of standard cryptographic hashes (which are reserved for the "slow rail"), the authoritative hot path derives digests using a deterministic, branchless, and allocation-free avalanche mixer based on **SplitMix64**. 

This is defined in `crates/bcinr-cmca/src/proposal.rs`:

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

### 2. Sealing the Influence State and Receipts

The `mix64` function is used sequentially to chain multiple identities and state variables into a single digest. For instance, in `crates/bcinr-cmca/src/certification.rs`, the `influence_state` (which acts as the influence digest) and other bindings are sealed branchlessly by accumulating the mix:

```rust
#[inline(always)]
fn seal_digest(candidate: &StabilityCandidate, bindings: &CertificateBindings) -> u64 {
    let mut d = mix64(candidate.candidate_digest(), bindings.admitted_graph);
    // ...
    d = mix64(d, bindings.control_mode);
    d = mix64(d, bindings.influence_state);
    d = mix64(d, bindings.comparison_derivation);
    d = mix64(d, bindings.round_identity);
    d
}
```

### 3. Branchless Digest Verification

Validating these hashed digests strictly adheres to the repository's $CC=1$ rule (no conditional branches). The runtime verifies digests in two primary ways:

**A. Byte-by-Byte Certificate Digest Matching (`const_eq_u32`)**
When comparing a 32-byte array digest against a precompiled `CERTIFICATE_DIGEST` (in `allocator.rs`), the code unrolls the loop and computes a `digest_match` mask using a constant-time equality check:

```rust
let mut digest_match = 1u32;
unroll_32_static!(i, {
    digest_match &= const_eq_u32(
        digest[i & 31] as u32,
        crate::generated::stability_profile::CERTIFICATE_DIGEST[i & 31] as u32,
    );
});
let digest_err = const_eq_u32(digest_match, 0) != 0;
```

The underlying `const_eq_u32` achieves this using a bitwise arithmetic trick rather than `if` statements. It leverages the sign bit behavior of two's complement negation:
```rust
#[inline(always)]
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}
```

**B. Receipt Digest Pooling (XOR/OR Strategy)**
When validating that multiple token receipts (Certificate, Envelope, Outcome) belong to the same state transition (as seen in `allocator.rs`), the engine validates them simultaneously via bitwise XOR and OR operations:

```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    // ...
```
Using XOR (`^`), identical digests yield `0`. Using OR (`|`), any mismatch will accumulate a non-zero bit. If the final bitwise evaluation is `0`, all digests match perfectly without requiring a single conditional branch.
