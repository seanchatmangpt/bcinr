# Research: Typed Refusal Architecture (Rule 18)

Under **Rule 18 (Typed refusals)** of the BCINR `AGENTS.md` Constitution, all rejected authoritative operations must produce a bounded typed refusal code. Human-readable text, variable-length strings, panics, partial state mutations, and unconditional default clamps are strictly prohibited in the hot path to preserve $CC=1$ deterministic branchlessness and zero-allocation execution.

## Exhaustive List of Typed Refusal Categories

The constitution explicitly mandates the following bounded refusal categories (represented externally via the `StabilityRefusal` enum):

1. `ContractViolation`
2. `UnsupportedDomain`
3. `NumericRangeExceeded`
4. `DigestMismatch`
5. `CertificateMissing`
6. `CertificateStale`
7. `EnvelopeViolated`
8. `ContractionMarginInsufficient`
9. `LearningFrozen`
10. `ReceiptMissing`
11. `ReceiptRejected`
12. `ModeDwellViolated`
13. `ControlStateUnadmitted`
14. `SupportMismatch`
15. `DistinguishabilityInsufficient`
16. `BranchlessContractFailed`
17. `ObjectCodeAuditFailed`
18. `CheatDetected`

## How They Function Without Standard Error Handling

Because BCINR enforces an absolute **$CC=1$ law (Rule 8)**, standard Rust error handling operators like `?`, `if let Err`, `match`, and early returns are banned inside authoritative code. Instead of halting execution when a fault occurs, the architecture uses a two-layered, completely branchless approach:

### 1. Internal Hot-Path Representation (`RefusalSet`)
Inside the authoritative hot path, errors are tracked using a zero-cost bitflag structure called `RefusalSet` (a wrapped `u32`).
*   **Branchless Accumulation:** Rather than branching on failures, both success and failure computations are fully executed in constant time. Errors are aggregated seamlessly using a branchless bitwise OR (`union`), e.g., `folded_faults = folded_faults.union(candidate.faults())`.
*   **Masked Selection:** Error conditionals are converted to bitmasks. A refusal is only registered via bitwise arithmetic (`masked(condition)`), where `condition` is evaluated mathematically without jumps (e.g., `RefusalSet::PROPOSAL_REJECTED.masked(err_condition_mask)`).
*   **Totality:** Authoritative functions always return a structured outcome unconditionally (e.g., `AllocationOutcome`), which couples the speculatively computed state with any accumulated `RefusalSet` flags. Because of **Rule 10**, this state is only formally committed if the resulting refusal mask evaluates to 0 (no errors); otherwise, the system is left bit-for-bit unchanged.

### 2. External Slow-Rail Representation (`StabilityRefusal` Enum)
Outside the strict $CC=1$ boundaries (on the "slow rail"), the system translates the `RefusalSet` bits into the standard `StabilityRefusal` enum variants via deterministic index lookups or branchless mappings.
*   Methods like `AllocationOutcome::into_result()` or static translation arrays allow downstream consumers to interpret failures canonically (e.g., mapping `DIGEST_MISMATCH | CERTIFICATE_STALE` down to `StabilityRefusal::CertificateDigestMismatch`) without compromising the upstream authoritative calculation block. 

By pushing semantic failure aggregation strictly into bitwise polynomials, BCINR respects the constraints of bounded execution and guarantees mathematical preservation of fixed-instruction pathways regardless of input validity.
