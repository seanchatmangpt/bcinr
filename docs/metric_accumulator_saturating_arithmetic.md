# MetricAccumulator: Branchless Aggregation in the MAPE-K Observe Phase

Within the BCINR MAPE-K loop, the **Observe** phase is responsible for continuously aggregating bit-level telemetry (e.g., system health and integrity scores) with strict guarantees around determinism and execution branching. The `MetricAccumulator` structure serves as the branchless utility to achieve this.

## Aggregating Bit-Level Telemetry

The `MetricAccumulator` aggregates telemetry through constant-time primitives that strictly follow the **Radon Law ($CC=1$)**, ensuring that structural integrity calculations remain allocation-free and branchless. 

The primary integer-based aggregation method is `saturating_sum(current: u64, val: u64) -> u64`, which serves as a primitive entry point that delegates to `metric_accumulator_sat_add(current, val)`. This function is implemented via Rust's intrinsic `u64::saturating_add`, which guarantees branchless execution on modern ISAs.

## Reliance on Saturating Arithmetic (Rule 14 Compliance)

**Rule 14 (Numeric-law requirements)** mandates that authoritative arithmetic must be deterministic, explicitly bounded by an explicit contract, and safe from panic-inducing bounds checks. 

`MetricAccumulator` relies on saturating arithmetic to safely accumulate state by ensuring:

1. **No Silent Wrapping**: If the accumulated metrics hit the upper boundary, they gracefully clamp to `u64::MAX`. If standard wrapping addition were used, reaching the maximum capacity would wrap back to zero, severely undermining monotonic accumulations like health metrics or error counts.
2. **No Panic-Inducing Bounds Checks**: Standard checked arithmetic operations (`checked_add`) inherently introduce hidden conditional branches and potential unwinding paths on failure, explicitly violating **Rule 8 (Absolute CC=1 law)** and **Rule 3 (Absolute runtime laws)**. Saturating addition acts as a zero-cost abstraction that relies on hardware-level saturating instructions rather than software bounds-checking.
3. **Hoare-logic Provability**: Saturating addition adheres to a rigorous, explicitly verifiable contract:
   * **Precondition:** `{ current, val ∈ U64 }`
   * **Postcondition:** `{ result = min(current + val, U64_MAX) }`

To guarantee this behavior remains structurally protected, the repository utilizes the "Contract with Teeth." The `MetricAccumulator` includes hostile negative mutants (such as deliberately substituting `wrapping_add` or `saturating_sub`) which the test suite must provably reject against an independent reference oracle.
