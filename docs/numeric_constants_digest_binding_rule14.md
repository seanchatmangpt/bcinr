# Rule 14: Digest Binding for Numeric Constants

According to **Rule 14 (Numeric-law requirements)** of the BCINR Constitution, every smoothing or clamp constant must be strictly: **Named**, **Derived**, **Admitted**, and **Included in the influence digest**. 

Silently inserting arbitrary "magic numbers" (like an unexplained epsilon) is prohibited. By statically binding these constants into the system's influence digest (often tracked via a `numeric_profile` digest), any undocumented numerical drift or tweaking immediately alters the digest. This ensures that any change in numerical boundaries automatically triggers a strict refusal (e.g. `CertificateStale` or digest mismatch), protecting the hot path from silent operational changes.

### The Branchless Digestion Process (`mix64`)

In the `bcinr-cmca` hot path, digesting the context cannot rely on variable-length byte buffers or standard cryptographic hash functions, as these violate the $CC=1$ rule and Zero-Allocation laws. 

Instead, execution context is sealed iteratively using **`mix64`**, a deterministic, branchless, and allocation-free avalanche mixer based on SplitMix64:

```rust
// crates/bcinr-cmca/src/proposal.rs
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

### Sealing the Constants into the Digest

The system seals the influence state by sequentially passing fixed-width 64-bit state variables through `mix64`. As seen in `seal_certificate` / `seal_digest`, the constants—represented in the `numeric_profile`—along with the `influence_state` and `candidate_digest` are all chained together:

```rust
// crates/bcinr-cmca/src/certification.rs
#[inline(always)]
fn seal_digest(candidate: &StabilityCandidate, bindings: &CertificateBindings) -> u64 {
    // 1. Initialize digest with the candidate's own structural digest
    let mut d = mix64(candidate.candidate_digest(), bindings.admitted_graph);
    
    // ... Mix other domain-specific bindings ...
    d = mix64(d, bindings.generated_payload);
    d = mix64(d, bindings.kernel_specialization_identity);
    
    // 2. The clamp and smoothing constants are bound here (numeric_profile)
    d = mix64(d, bindings.numeric_profile); 
    
    // ...
    d = mix64(d, bindings.pricing_law);
    d = mix64(d, bindings.floor_law);
    d = mix64(d, bindings.control_mode);
    
    // 3. The influence state is bound here
    d = mix64(d, bindings.influence_state); 
    
    d = mix64(d, bindings.comparison_derivation);
    d = mix64(d, bindings.round_identity);
    d
}
```

### Why This Fulfills The Laws
- **Zero Allocation:** The entire digest accumulates in fixed-size registers. There is no heap-backed `Vec<u8>` or stream accumulation.
- **$CC=1$ Control Flow:** The iterative hashing involves no conditional branches (`if`/`match`), only sequential XORs and multiplications.
- **Deterministic Sealing:** Hashing the sequence of state variables guarantees deterministic outputs that fulfill the rigorous proofs required by `@hoare_oracle` and the strict enforcement rules mandated by `@turing_machine`.
