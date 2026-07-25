Here are the findings regarding how "Stale Certificate Acceptance" is handled and how the hot path branchlessly rejects certificates, based on the codebase in `bcinr-cmca`.

### 1. How is "Stale Certificate Acceptance" detected?
According to the `crates/bcinr-cmca/REFUSAL_REALIZATION_REPORT.md` (and inline comments in `allocator.rs` and `mode_switch.rs`), the `CERTIFICATE_STALE` bit is marked as **`OWNED_BY_DIFFERENT_COMPONENT`**.

The hot path (`allocate()`) does not directly check for or construct the `CERTIFICATE_STALE` bit. Instead, "stale certificate acceptance" is realized and detected when a previously-valid certificate is no longer current across two other modules:
- **`mode_switch::ModeSwitchRefusal::CertificateDigestMismatch`**: The `apply_mode_switch` function refuses when the presented `certificate != expected_certificate` (which is exactly what a superseded certificate produces against a freshly re-derived expectation).
- **`certification::CertificationRefusal::RoundIdentityMismatch`**: For the specific sub-case where a certificate is sealed against a superseded round, `seal_certificate` independently re-verifies the `round_identity` binding.

In `mode_switch.rs`, this is enforced through the "masked-commit law" (AGENTS.md §10): the candidate next-state is structurally computed *unconditionally* before any checks run. All admission predicates (including certificate validity and dwell) are collapsed into a single boolean, and the persistence write acts as a masked select that leaves the state field-for-field unchanged upon rejection, rather than branching out early.

### 2. How does the hot path reject stale certificates without `if` statements?
Because the hot path (`allocator::allocate()`) delegates the conceptual "stale" logic upstream to `mode_switch` and `certification`, it only needs to branchlessly reject generic **Digest Mismatches** and aggregate other numeric/state errors. 

It accomplishes this via fixed-width bitwise arithmetic and macro unrolling rather than control-flow branches (`if` statements):

**Step A: Branchless Digest Comparison**
It unrolls a loop to compare the 32-byte digest array bit-by-bit against the expected profile using `const_eq_u32` (which returns a bitmask rather than a boolean). 
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

**Step B: Branchless Refusal Accumulation**
Instead of using `if digest_err { return Err(...) }`, it uses a bitwise `masked()` function to zero out a refusal bit unless the condition is `1`.
```rust
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```

The refusals are then unconditionally unioned together without a single branch:
```rust
let gated_refusals = RefusalSet::EMPTY
    .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32))
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    // ...
```
This preserves the absolute $CC=1$ rule and the constant-time masked-commit laws of the deterministic substrate, fully avoiding early-return branching while still correctly yielding the refusal state.
