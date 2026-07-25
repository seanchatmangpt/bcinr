# Analysis of `AllocationOutcome` in `allocator.rs`

Based on my research of `crates/bcinr-cmca/src/allocator.rs`, the `AllocationOutcome` struct is a core component of the Covariance Monitoring and Calibration Assessment (CMCA) substrate's branchless allocation engine. It strictly adheres to the "Radon Law" ($CC=1$) by entirely avoiding control-flow branches, panics, and the use of Rust's `Result` in the hot path.

Here is a breakdown of how it safely packages deterministic results, numeric faults, and typed refusals branchlessly:

## 1. Always Constructible and Total
Unlike standard Rust functions that use `Result` for early returns on failure, the authoritative root allocation functions are **total**. They always execute in constant time and produce an `AllocationOutcome` for any admitted input. Anomalies do not interrupt control flow; instead, they are recorded as flags within the outcome.

## 2. Structure of `AllocationOutcome`
The `AllocationOutcome` struct encapsulates three private fields:
- `candidate: [NonNegativeFixed; N]`: The computed resource allocation distribution array.
- `numeric_faults: NumericFaultSet`: A set of bitflags capturing any numerical bounds/overflow issues encountered.
- `refusals: RefusalSet`: A set of typed refusal bitflags capturing domain or stability violations (e.g., missing certificates, envelope violations).

## 3. Branchless Aggregation via `new_internal`
The struct can only be instantiated through the `new_internal` constructor. This method enforces an aggregation invariant entirely without branches.
```rust
pub(crate) fn new_internal(
    candidate: [NonNegativeFixed; N],
    local_faults: NumericFaultSet,
    refusals: RefusalSet,
) -> Self
```
Within `new_internal`, `NumericFaultSet` aggregation is performed using the `unroll_8_static!` macro to branchlessly bitwise-union the local faults with the `.faults()` intrinsic to each of the computed `NonNegativeFixed` candidates.

## 4. `RefusalSet` and Bitwise Unions
Refusals (e.g., `CERTIFICATE_MISSING`, `PROPOSAL_REJECTED`) are represented in the `RefusalSet` struct. Rather than a lossy single-variant enum, `RefusalSet` uses bitwise unions (`|`) to accumulate multiple concurrent refusal reasons branchlessly. Masks are applied to conditionally zero out or retain these flags without `if` statements.

## 5. Bridging to `Result` Outside the Hot Path
To interface ergonomically with code outside the authoritative hot path, `AllocationOutcome` provides an `into_result` method.
```rust
pub fn into_result(self) -> Result<[NonNegativeFixed; N], StabilityRefusal>
```
Only at this non-authoritative boundary does the outcome check if the `refusals` set is empty and branch to return a standard `Ok(candidate)` or an `Err` containing a legacy `StabilityRefusal` code (derived via `primary_reason()`).
