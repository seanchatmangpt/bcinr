# Research on Rule 18 (Typed Refusals) in `bcinr`

In the BCINR (BranchlessCInRust) deterministic substrate, **Rule 18** dictates that all rejected authoritative operations must produce a bounded typed refusal code without panicking, unconditionally clamping, or mutating partial state. Additionally, to comply with **Rule 8 (Absolute CC=1 law)**, control flow branching (like `Result::Err` short-circuiting or the `?` operator) is strictly prohibited in the hot path. 

To achieve this, the codebase implements a two-layered taxonomy for typed refusals: a branchless internal bitflag set (`RefusalSet`) used in the hot path, and an outward-facing enum (`StabilityRefusal`) used on the slow rail.

## Taxonomy of Typed Refusals

### 1. The Internal Hot-Path Representation: `RefusalSet`
Because early returns are illegal in $CC=1$ code, errors are aggregated using a bitflag structure called `RefusalSet` (a wrapped `u32`).
- **Structure**: Each specific error state corresponds to a bit in the `u32` value (e.g., `NO_LEAVES = 1 << 0`, `DIGEST_MISMATCH = 1 << 4`, `PROPOSAL_REJECTED = 1 << 6`).
- **Implementation**: Instead of branching, the allocator evaluates *both* success and failure conditions. Errors are aggregated using a branchless bitwise OR (`union`) and conditionally instantiated using bitwise arithmetic (`masked(condition)`).
- **Totality**: The hot-path function unconditionally evaluates and returns an `AllocationOutcome` struct, which contains the tentatively computed candidate alongside any accumulated `RefusalSet` and `NumericFaultSet` flags. 

### 2. The External Slow-Rail Representation: `StabilityRefusal` Enum
Outside the strictly audited $CC=1$ authoritative roots (or when mapped explicitly branchlessly via arrays), errors are represented by the `StabilityRefusal` enum. 
- **Structure**: A standard Rust `#[derive(Copy, Clone, Debug, PartialEq, Eq)] enum` mapping to the required categories dictated by the constitution (e.g., `CertificateMissing`, `CertificateDigestMismatch`, `ContractViolation`, `ContractionMarginInsufficient`).
- **Implementation**: The bridging method `AllocationOutcome::into_result()` is permitted to use `if`/`else` control flow to inspect the `RefusalSet` and return a standard `Result<T, StabilityRefusal>`. Alternatively, an array mapping (`wrap_result`) allows extracting standard variants purely via constant-time index lookups.

## Handling of Specific Conditions

The mapping from the internal co-occurring `RefusalSet` bitflags to the legacy single-variant `StabilityRefusal` enum prioritizes specific errors down to a `primary_reason`.

### `DigestMismatch`
- **Representation (Internal)**: Represented as the `RefusalSet::DIGEST_MISMATCH` bitflag or `RefusalSet::CERTIFICATE_STALE`.
- **Handling**: These bits are unioned branchlessly into the outcome if the provided cryptographic digest does not match the expected state. 
- **Mapping**: When transitioning to the slow rail, both map to `StabilityRefusal::CertificateDigestMismatch`. 

### `ContractionMarginInsufficient`
- **Representation (Internal)**: Tracked natively via the `RefusalSet::PROPOSAL_REJECTED` bitflag.
- **Handling**: Emitted when numeric gradient descent bounds fail (e.g., learning rate out of bounds, step destabilized).
- **Mapping**: If `PROPOSAL_REJECTED` is present in the `RefusalSet` and no higher-priority certificate-related issues exist, it translates to `StabilityRefusal::ContractionMarginInsufficient`.

### `ContractViolation`
- **Representation (Internal)**: Can stem from mismatches like `RefusalSet::ROUND_MISMATCH`.
- **Handling**: Acts as the ultimate fallback. 
- **Mapping**: If the `RefusalSet` is non-empty but none of the specific higher-priority cases (`DIGEST_MISMATCH`, `CERTIFICATE_MISSING`, `DWELL_UNSATISFIED`, `PROPOSAL_REJECTED`) match, `primary_reason()` defaults to `StabilityRefusal::ContractViolation`. It is also explicitly returned for `ROUND_MISMATCH`.

In conclusion, BCINR handles Rule 18 by computing all mathematical states branchlessly and accumulating semantic failures via bitwise logic, preserving the hardware-level timing guarantees before eventually serializing the faults into strict typed enums for downstream consumers.
