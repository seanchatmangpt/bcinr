# DigestMismatch in BCINR

## Definition
`DigestMismatch` is a bounded typed refusal variant emitted when the integrity bindings of digest validations fail during authoritative execution. It guarantees that an operation is rejected when a provided digest does not bit-for-bit match the expected digest or certificate. Because of strict constitutional laws against speculative mutation (AGENTS.md §10), a rejected operation leaves the persistent state field-for-field unchanged.

Several variants of this refusal exist across the deterministic pipeline:
- `StabilityRefusal::CertificateDigestMismatch` (`crates/bcinr-cmca/src/allocator.rs`)
- `ModeSwitchRefusal::CertificateDigestMismatch` (`crates/bcinr-cmca/src/mode_switch.rs`)
- `GeneratedProfileRefusal::PayloadDigestMismatch` (`crates/bcinr-cmca/src/artifact.rs`)
- `ProposalRefusal::ProposalDigestMismatch` (`crates/bcinr-cmca/src/proposal.rs`)
- `ProposalRefusal::CurrentModeDigestMismatch` (`crates/bcinr-cmca/src/proposal.rs`)

## Branchless Mathematical Condition
In accordance with the BCINR constitution (Cyclomatic Complexity $CC=1$, no data-dependent branches), the authoritative hot path (`crates/bcinr-cmca/src/allocator.rs`) implements the mismatch trigger mathematically, entirely avoiding conditional jumps (`if`, `match`, or early returns) that could introduce timing side-channels. 

The pipeline that realizes `DIGEST_MISMATCH` is defined as follows:

### 1. Algebraic Equality (`const_eq_u32`)
The fundamental building block replaces the conditional `==` operator with an arithmetic polynomial using two's complement and bit manipulation:
```rust
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}
```
* **$x = a \oplus b$**: Evaluates to `0` only if `a` and `b` match exactly.
* **$nonzero = (x \lor (-x)) \gg 31$**: For any non-zero value, either the value or its negation will have the MSB (sign bit) set. Shifting it down extracts a `1` if they differ, or `0` if they match.
* **$1 - nonzero$**: Inverts the logic so `1` means equal and `0` means mismatch.

### 2. Structural Loop Unrolling
Instead of a branching loop, 32-byte array comparisons are flattened via a static unrolling macro. The branchless boolean results of `const_eq_u32` are accumulated using bitwise `&`:
```rust
let mut digest_match = 1u32;
unroll_32_static!(i, {
    digest_match &= const_eq_u32(
        digest[i & 31] as u32,
        crate::generated::stability_profile::CERTIFICATE_DIGEST[i & 31] as u32,
    );
});
```
`digest_match` remains `1` only if all 32 bytes perfectly align.

### 3. Error Inversion & Branchless Mask Generation
The success state (`digest_match = 1`) is inverted into an error boolean (`digest_err`), where `1` indicates a mismatch. 
```rust
let digest_err = const_eq_u32(digest_match, 0) != 0;
```
This error boolean is algebraically expanded into a 32-bit mask:
```rust
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```
* If `condition` is `1` (Mismatch): $0 - 1 = \text{0xFFFFFFFF}$, activating the typed refusal flag bit via bitwise `&`.
* If `condition` is `0` (Match): $0 - 0 = \text{0x00000000}$, erasing the refusal bit entirely.

### 4. Deterministic Application
The resulting refusal flags are unioned via purely bit-parallel execution:
```rust
let gated_refusals = RefusalSet::EMPTY
    .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32));
```
By mapping structural unrolling and mathematical polynomials directly to state logic, the entire equality check fulfills the substrate's zero-allocation and bounded $O(1)$ constant-time constitutional mandate.
