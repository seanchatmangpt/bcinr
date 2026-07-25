# OCEL and SRBCG in `bcinr-powl`

Based on the contents of `crates/bcinr-powl/src/ocel.rs`, here is the documentation on Object-Centric Event Logs (OCEL) and Symmetric Run-Bounded Conformance Gating (SRBCG).

## Object-Centric Event Logs (OCEL)
The Object-Centric Event Log (`OcelLog`) is a deterministic, zero-allocation (`no_std` compatible) mechanism for recording execution traces of Partially Ordered Workflow Language (POWL) workflows. 
- **Fixed-Capacity:** It uses a static array that can hold up to 512 discrete events without any heap allocation, guaranteeing $O(1)$ time complexity for appending events.
- **Event Types:** It primarily records two types of activities:
  - `op_fired`: Marks the execution of a specific operation (`op_idx`) within a workflow run (`run_id`).
  - `run_sealed`: Seals a `run_id` with an `op_trace` bitmask, declaring the set of operations that have been fired.

## Symmetric Run-Bounded Conformance Gating (SRBCG)
Because heap allocation is forbidden within this substrate, mapping dynamic `run_id`s to state vectors must be done in bounded stack memory. SRBCG provides this by statically allocating an array of 64 slots.
- **Run Limit:** It restricts conformance tracking to a maximum of 64 concurrent or unique workflow runs.
- **Overflow Handling:** If a trace exceeds the 64-run boundary, it sets an overflow mask rather than panicking or failing silently, and rejects the trace with a typed refusal (`ConformanceResult::RunLimitExceeded`).

## Branchless Trace Recording and Conformance Checking
To comply with the project's **Radon Law ($CC=1$)**, the core logic completely avoids data-dependent jumps, `if`/`match` statements, and variable-bound loops. It accomplishes this through mathematical abstraction and comparison networks.

### Branchless Slot Assignment (`process_event_srbcg`)
Finding and allocating slots for incoming `run_id`s is done using a comparison network that compiles down to branchless conditional selection instructions (e.g., `CSEL`/`CMOV`):
1. **Fixed-Bound Search:** It iterates over all 64 slots in a statically fixed loop `0..64`, ensuring the compiler can generate unrolled straight-line assembly.
2. **Masked Comparison:** It generates a match mask for each slot mathematically: `is_match = (run_ids[i] == incoming_rid) as usize`.
3. **Arithmetic Selection:** The target slot index is updated without branches: `match_idx = (is_match * i) + ((1 - is_match) * match_idx)`.
4. **State Transition:** Determining whether a slot was found or can be allocated relies on arithmetic propagation (e.g., `found = (match_idx < 64) as usize`) to update counts and write to slots using inverted masks.

### Conformance Validation (`validate_against_tape`)
The validation function evaluates rules by comparing bitmasks and aggregating errors arithmetically:
- **Run Limit Check:** Inspects the `overflow_mask` produced by SRBCG.
- **Duplicate Firing:** Tracks multiple executions using bitwise logic to isolate repeated bits (`has_fired_mask = 0u64.wrapping_sub(((accumulated[s] & bit) != 0) as u64)`) instead of branching checks.
- **Seal Mismatch & Predecessor Constraints:** Relies purely on bitwise comparison. A violation is detected if `missing = pred_mask & !op_trace` is non-zero, ensuring order constraints are strictly enforced using bit masks compiled into the `PowlTape`.
