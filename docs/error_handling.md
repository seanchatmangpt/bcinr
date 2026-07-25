# Error Handling and Typed Refusals in BCINR

The `bcinr` (BranchlessCInRust) codebase is a deterministic, allocation-free execution substrate. Under its strict constitutional mandates (e.g., the Radon Law $CC=1$, zero heap allocation), standard Rust error handling paradigms like panics, unwinding, `?` operator early-returns, and string-based error logs are completely prohibited in the hot path. 

Instead, failure modes and contract violations are handled via **branchless accumulation** and **Typed Refusals**.

## 1. Typed Refusals over Text and Panics

Every anomalous condition—whether out-of-envelope numeric states, mismatched certificates, or logic contract violations—is represented by a discrete, bounded enum or bitflag. By utilizing typed error codes instead of human-readable text strings, the substrate completely avoids allocations and variable-length graph traversals.

### Key Refusal Types
The architecture distributes refusals into tight, domain-specific enums to ensure exact mapping of failures without cross-domain pollution:

- **`StabilityRefusal`** (`crates/bcinr-cmca/src/allocator.rs`): The primary legacy-compatible error enum representing runtime control loop anomalies. Variants include `CertificateMissing`, `ContractViolation`, `ContractionMarginInsufficient`, `ModeDwellTimeViolated`, `LearningFrozen`, and numeric threshold breaches.
- **`RefusalSet`** (`crates/bcinr-cmca/src/allocator.rs`): A `u32` bitmask representing a set of refusals that can co-occur. It allows accumulating errors concurrently without short-circuiting branches. Bits include `NO_LEAVES`, `CERTIFICATE_MISSING`, `CERTIFICATE_STALE`, `ROUND_MISMATCH`, `DIGEST_MISMATCH`, etc.
- **`StabilityDerivationRefusal`** (`crates/bcinr-cmca/src/stability.rs`): Specific to witness validation (e.g., `WitnessNotPositive`, `ContractionMarginInsufficient`).
- **`CertificationRefusal`** (`crates/bcinr-cmca/src/certification.rs`): Strict binding verification checks (e.g., `RoundIdentityMismatch`, `ControlModeMismatch`).
- **`ProposalRefusal`** (`crates/bcinr-cmca/src/proposal.rs`): Rejections for proposed control deltas (e.g., `ProposalDigestMismatch`, `UnsupportedDelta`, `TelemetryStandingBlocked`).
- **`ModeSwitchRefusal`** (`crates/bcinr-cmca/src/mode_switch.rs`): Errors encountered during state actuation (e.g., `CertificateDigestMismatch`, `StaleAdmittedState`).
- **`GeneratedProfileRefusal`** (`crates/bcinr-cmca/src/artifact.rs`): Used in offline/test-time artifact verification, representing schema anomalies or digest discrepancies.

## 2. Branchless Composition and Error Accumulation

Standard Rust error handling relies on branching (`if err.is_some() { return ... }`). Because authoritative functions in `bcinr` are mandated to execute in exactly the same number of CPU instructions regardless of input (`CC=1`), errors are instead composed via boolean algebra and bitwise masks.

### Outcome Aggregation
Internal functions do not return `Result` types directly on the hot path. Instead, they output exhaustive outcome structures (like `AllocationOutcome`) which carry the speculative result *alongside* any accrued faults:
```rust
pub struct AllocationOutcome {
    candidate: [NonNegativeFixed; N],
    numeric_faults: NumericFaultSet,
    refusals: RefusalSet,
}
```

### Masked Refusal Selection
`RefusalSet` aggregates anomalies via branchless bitwise unions. If a validation check fails, the respective error flag is merged into the refusal set using `masked` selection rather than a branch:
```rust
// Accumulate refusal bitwise without jumping
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```
Errors are only mapped out to standard `Result` adapters (e.g., `wrap_result` via a lookup array `REFUSALS[(err_code as usize) & 31]`) at the very outer edge of the authoritative boundary.

## 3. Masked State Commitment (No Speculative State Pollution)

According to the constitution (AGENTS.md §10), persistent state must never be mutated speculatively before all admission predicates are verified. 

If a refusal condition is flagged, the runtime executes a branchless masked selection that simply overwrites the state with its existing values. 
```rust
// State candidate computed fully without branch
let candidate = ModeState { ... };

// Branchless select based on accumulated valid condition
let next = if admitted { candidate } else { *persistent };
*persistent = next;
```
A rejected operation leaves the persistent state bit-for-bit unchanged. The codebase explicitly prohibits fixing up bad inputs, falling back to simpler algorithms, or silently applying defaults. The refusal strictly blocks the state transition and yields the exact typed anomaly code.
