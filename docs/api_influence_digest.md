Here is the requested documentation regarding branchless digestion.

```markdown
# Branchless Execution Context Hashing (`mix64`)

> **Note on requested paths:** I checked `crates/bcinr-api/src/` for `digest.rs`, `crypto.rs`, or `hash.rs`, but no such files exist in that directory (the API only exposes `sketch.rs` for fast hashing). The authoritative branchless hashing mechanism is instead implemented in `crates/bcinr-cmca/src/proposal.rs` and `crates/bcinr-cmca/src/certification.rs`.

## How `InfluenceDigest` / `mix64` Hashes Context Branchlessly

In the `bcinr` (BranchlessCInRust) authoritative hot path, deriving digests must strictly follow the repository's $CC=1$ rule and the Zero-Allocation Boundary. This means no conditional branches (`if`/`match`), no dynamic memory allocations, and no variable-length buffers (like `Vec<u8>`). 

To achieve this, the execution context is sealed iteratively using a deterministic, branchless avalanche mixer named `mix64` (a SplitMix64 variant).

### 1. The Core Branchless Mixer (`mix64`)
The `mix64` function provides a fixed, allocation-free mix operation that only uses bitwise XOR and wrapping multiplication. 

```rust
// In crates/bcinr-cmca/src/proposal.rs
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

### 2. Iterative Hashing Without Variable-Length Buffers
Instead of collecting bindings into a byte buffer and hashing them at once, `bcinr` seals the `InfluenceDigest` (and other state bindings) by iteratively chaining fixed-width 64-bit state variables through `mix64`. 

```rust
// In crates/bcinr-cmca/src/certification.rs
#[inline(always)]
fn seal_digest(candidate: &StabilityCandidate, bindings: &CertificateBindings) -> u64 {
    let mut d = mix64(candidate.candidate_digest(), bindings.admitted_graph);
    d = mix64(d, bindings.generated_payload);
    d = mix64(d, bindings.kernel_specialization_identity);
    d = mix64(d, bindings.numeric_profile);
    d = mix64(d, bindings.q_registry);
    d = mix64(d, bindings.pricing_law);
    d = mix64(d, bindings.floor_law);
    d = mix64(d, bindings.control_mode);
    d = mix64(d, bindings.influence_state);         // <-- Influence Digest bound here
    d = mix64(d, bindings.comparison_derivation);
    d = mix64(d, bindings.round_identity);
    d
}
```

**Why this satisfies the constitutional laws:**
- **No Allocation / Fixed Memory:** Every identity is already represented as a fixed-size `u64`. There is no need to dynamically allocate or manage arrays; the compiler can aggressively map this to a pipeline of register operations.
- **$CC=1$:** The iterative hashing process purely consists of sequential arithmetic and bitwise operations. The control flow is unconditionally linear. 
- **Deterministic:** The same sequence of state IDs always produces the exact same avalanche digest predictably across targets, fulfilling the rigid testing requirements of the repository.
```
