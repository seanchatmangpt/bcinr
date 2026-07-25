# The `CertificateMissing` Typed Refusal in BCINR

In the `bcinr` (BranchlessCInRust) codebase, the deterministic execution substrate is bound by the **Radon Law ($CC=1$)**, which strictly prohibits data-dependent branches, panics, early-returns, and string-based error logs in the hot path. Consequently, all anomalies—such as missing certificates or numeric limit breaches—are managed via branchless accumulation and **Typed Refusals**. 

## 1. The `CertificateMissing` Disposition

There are two primary ways `CertificateMissing` is expressed in the `bcinr-cmca` crate:
- **`StabilityRefusal::CertificateMissing`**: A legacy-compatible, discrete error enum returned at the edge of the authoritative boundary.
- **`RefusalSet::CERTIFICATE_MISSING`**: A single bitflag (`1 << 1`) inside the `RefusalSet` `u32` mask, which is used for branchless, concurrent error accumulation during structural evaluation.

According to the `REFUSAL_REALIZATION_REPORT.md`, `RefusalSet::CERTIFICATE_MISSING` currently holds a disposition of **`RESERVED_WITH_EXPLICIT_NONCLAIM`**. 

No code path inside the crate's authoritative root (like `allocate()`) actually constructs this bit—not even as a masked-to-zero attempt. This is due to the strict fixed-shape input mandate of the substrate:
- Core hot-path APIs like `allocate()` take a mandatory `digest: [u8; 32]`, never an `Option<[u8; 32]>`. 
- Similarly, `apply_mode_switch()` takes a mandatory `certificate: CertificateReceipt`, never an `Option`.

Introducing an `Option` to dynamically distinguish "missing" from "mismatched" would necessitate a branch in the hot path (violating $CC=1$) or an upstream signature change. However, the bit is *not* a dead variant; "no certificate was ever obtained" is a meaningful domain condition (e.g., if a caller legitimately fails to seal a certificate upstream). It is kept reserved for a future API shape that can branchlessly handle missing certificates at a checked boundary, and it is currently read by `RefusalSet::primary_reason()` to collapse into `StabilityRefusal::CertificateMissing`.

*(Note: State mismatches for presented certificates are actively handled by `CERTIFICATE_STALE` and `DIGEST_MISMATCH` bitflags, which have `REACHABLE` and component-owned dispositions.)*

## 2. Structural Evaluation and Branchless Triggering

When the system structurally evaluates a domain condition to trigger refusals, it relies on branchless bitwise logic rather than `if err.is_some() { return ... }`.

Refusals are accumulated into a `RefusalSet` using the `masked` and `union` methods, ensuring deterministic $O(1)$ constant-time execution:

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

If an error condition (such as `digest_err` for `DIGEST_MISMATCH`) evaluates to `1`, its corresponding bitmask is unioned into the active `RefusalSet`. 

## 3. Masked State Commitment

Under BCINR's constitution (AGENTS.md §10), a persistent state must never be mutated speculatively before all admission predicates are verified. 

If an operation demands a verified state context and a refusal like `CertificateMissing` or `DigestMismatch` is flagged, the runtime executes a branchless masked selection (e.g., `select_nnf` utilizing `CanonicalMask`) that structurally overwrites the candidate state with the pre-existing state. 

```rust
// Conceptual representation of branchless admission:
let mask = valid_mask(...);
// If not admitted, candidate is dropped and current state is re-selected
let next = State::select(mask, candidate, current);
*persistent = next;
```

Through this mechanism, a rejected operation leaves the persistent state bit-for-bit unchanged without ever invoking a conditional jump, honoring the branchless contract while strictly refusing unverified execution.
