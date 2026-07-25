# CertificateMissing in BCINR

## Exact Definition

`CertificateMissing` represents a domain condition where an upstream caller fundamentally failed to supply or seal a certificate receipt (i.e., "no certificate was ever obtained"). It is expressed in the `bcinr-cmca` crate in two primary ways:

1. **`StabilityRefusal::CertificateMissing`**: A legacy-compatible, discrete error enum variant used at the edge of the authoritative boundary (mapped from `u32` error code `0`).
2. **`RefusalSet::CERTIFICATE_MISSING`**: A single bitflag (`1 << 1`) inside the branchless `RefusalSet` `u32` mask, with a disposition of **`RESERVED_WITH_EXPLICIT_NONCLAIM`**.

## Branchless Mathematical Condition that Triggers It

Paradoxically, **there is currently no mathematical condition that triggers it in the hot path**, because it has no representable trigger given the current API surface.

Due to BCINR's strict **Radon Law ($CC=1$)** and fixed-shape input mandates, the authoritative root must remain total over a fixed-shape domain:
- Core hot-path APIs like `allocate()` take a mandatory `digest: [u8; 32]`, never an `Option<[u8; 32]>`.
- APIs like `apply_mode_switch()` take a mandatory `certificate: CertificateReceipt` by value, never an `Option`.

Introducing an `Option` to dynamically distinguish a "missing" certificate from a "mismatched" one (e.g. `DIGEST_MISMATCH` or `CERTIFICATE_STALE`) would necessitate a branch in the hot path, violating $CC=1$. Therefore, **no code path inside the crate's authoritative root actually constructs the `RefusalSet::CERTIFICATE_MISSING` bit—not even as a masked-to-zero attempt.**

### How it behaves in theory
The bitflag is intentionally kept because the domain condition is meaningful (e.g., if a caller legitimately fails to seal a certificate upstream). It is reserved for a future API shape that can branchlessly handle missing certificates at a checked boundary. 

If it were possible to trigger, the condition would be evaluated into a numeric value (`1` for true, `0` for false) and passed into constant-time bitwise accumulation without branching:

```rust
// Zeros `self` unless `condition` is `1` (no branch)
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}

// Accumulates the refusal into the set (no branch)
#[inline(always)]
pub const fn union(self, other: Self) -> Self {
    Self(self.0 | other.0)
}
```

This would create an admission mask where the runtime executes a branchless masked selection (e.g., `next = State::select(mask, candidate, current)`) to structurally reject the operation, leaving the persistent state bit-for-bit unchanged.
