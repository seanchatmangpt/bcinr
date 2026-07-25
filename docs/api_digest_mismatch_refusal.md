# Branchless Implementation of `DigestMismatch` in `bcinr-cmca`

While `DigestMismatch` refusal variants exist in several files (e.g., `artifact.rs`, `mode_switch.rs`, `proposal.rs`), they often use standard `if` branches there. The mathematically strict, purely branchless enforcement mandated by the BCINR constitution (`CC=1`, no `if`/`match` control flow) is located in the authoritative allocator's hot path: `crates/bcinr-cmca/src/allocator.rs`.

Here is the structural and mathematical pipeline that triggers `DIGEST_MISMATCH` branchlessly:

## 1. Algebraic Equality (`const_eq_u32`)
Instead of using the `==` operator which could compile to a conditional jump, the allocator relies on a bitwise polynomial to evaluate equality.
```rust
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}
```
- **XOR (`^`)**: Produces `0` only if `a` and `b` are identical.
- **Two's Complement Negation (`wrapping_neg`)**: If `x` is non-zero, either `x` or `-x` will have the Most Significant Bit (MSB) set.
- **Sign Bit Extraction (`>> 31`)**: Shifts the MSB down to the 1s place, resulting in `1` if they differ, and `0` if they match.
- **Inversion (`1u32.wrapping_sub`)**: Flips the logic so `1` means equal and `0` means mismatch.

## 2. Structural Loop Unrolling
To avoid control-flow branches from loop termination, the 32-byte array comparison is statically generated using an unrolling macro:
```rust
let mut digest_match = 1u32;
unroll_32_static!(i, {
    digest_match &= const_eq_u32(
        digest[i & 31] as u32,
        crate::generated::stability_profile::CERTIFICATE_DIGEST[i & 31] as u32,
    );
});
```
The result of each branchless byte-comparison is accumulated via bitwise `&`. `digest_match` remains `1` only if all 32 bytes are exactly equal.

## 3. Translation to Error Mask State
The success state (`digest_match == 1`) is mathematically inverted into an error state (`digest_err`):
```rust
let digest_err = const_eq_u32(digest_match, 0) != 0;
```
If `digest_match` is `0`, `const_eq_u32` returns `1`. (Note: The compiler can safely turn the `!= 0` into a branchless conditional boolean mapping without branching).

## 4. Branchless Mask Application
The boolean error indicator is mathematically expanded into a full-width mask and conditionally applied to `RefusalSet::DIGEST_MISMATCH`:
```rust
let gated_refusals = RefusalSet::EMPTY
    .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32))
```
This relies on the structural mask selection contract defined in the `masked` function on the `RefusalSet`:
```rust
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```
- If `condition` is `1` (mismatch), `0u32.wrapping_sub(1)` underflows to `0xFFFFFFFF`, and the bitwise `&` preserves the refusal flag bit.
- If `condition` is `0` (match), it evaluates to `0x00000000`, wiping out the flag.

Ultimately, this sequence strictly maps an invalid input domain to a `StabilityRefusal::CertificateDigestMismatch` typed refusal using bounded $O(1)$ constant time execution, fulfilling the project's branchless computing laws.
