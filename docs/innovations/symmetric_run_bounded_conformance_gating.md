# Innovation Proposal: Symmetric Run-Bounded Conformance Gating (SRBCG)

## 1. Executive Summary

This proposal introduces **Symmetric Run-Bounded Conformance Gating (SRBCG)**, a deterministic, zero-allocation, and constant-time conformance gating protocol designed to eliminate silent event skipping in Object-Centric Event Log (OCEL) conformance checking.

Currently, the conformance checker in `crates/bcinr-powl/src/ocel.rs` records event patterns under a fixed run capacity constraint (`MAX_RUNS = 64`). When log validation encounters more than 64 unique run IDs, it silently ignores all subsequent events for any 65th or later run. This silent skipping represents a major correctness and security vulnerability, as invalid execution sequences, duplicate fires, and predecessor violations in skipped runs will bypass validation, leading to false-positive `ConformanceResult::Conforms` reports.

SRBCG solves this by:
1. Extending the `ConformanceResult` enum with a typed `RunLimitExceeded` refusal variant.
2. Replacing the silent skipping behaviour with a deterministic, branchless gating check that monitors slot allocations.
3. Formulating a Radon Law-compliant ($CC=1$), loop-free (fully unrolled), and allocation-free slot tracking algorithm that propagates an overflow mask to guarantee constant-time execution and timing side-channel immunity.

---

## 2. Vulnerability & Limitation Analysis

### 2.1 The Silent Event Skipping Flaw
In `crates/bcinr-powl/src/ocel.rs`, the `validate_against_tape` function processes recorded events by mapping each `event.run_id` to a slot in a fixed-size table:

```rust
    // We need to visit every run_id. With no_std/no-heap we use a fixed-size
    // table of up to 64 run_ids seen in this log.
    const MAX_RUNS: usize = 64;
    let mut run_ids: [u64; MAX_RUNS] = [u64::MAX; MAX_RUNS];
    ...
```

To find or assign a slot, the code invokes the `slot_for!` macro:

```rust
    macro_rules! slot_for {
        ($rid:expr) => {{
            let mut found = MAX_RUNS;
            let mut i = 0;
            while i < run_count {
                if run_ids[i] == $rid {
                    found = i;
                    break;
                }
                i += 1;
            }
            if found == MAX_RUNS && run_count < MAX_RUNS {
                found = run_count;
                run_ids[run_count] = $rid;
                run_count += 1;
            }
            found
        }};
    }
```

When `slot_for!` is invoked in the event processing loops, it returns `MAX_RUNS` (64) if the capacity is exceeded. The code then silently skips processing the event:

```rust
        match event.activity {
            "op_fired" => {
                let s = slot_for!(event.run_id);
                if s == MAX_RUNS {
                    continue;
                } // too many runs; skip
                ...
            }
            "run_sealed" => {
                let s = slot_for!(event.run_id);
                if s == MAX_RUNS {
                    continue;
                }
                ...
            }
```

### 2.2 Violations of the BCINR Constitution
This design violates several foundational tenets of the BCINR Deterministic Substrate:
1. **Unbounded Silent Fallbacks (Section 18, Typed Refusals)**: The code fails to produce a typed refusal when inputs exceed operational limits. Instead of refusing invalid or over-capacity logs, it silently discards data.
2. **Timing Side-Channels and Data-Dependent Loops**: The lookup in `slot_for!` is performed using a branching loop (`while i < run_count`) containing early-exit breaks (`if run_ids[i] == $rid { break; }`). The execution latency depends directly on the order and value of incoming run IDs, creating potential timing side-channels.
3. **Speculative State and Incomplete Admission**: State transitions and predecessor checks are evaluated over a partial set of events, breaking the conservation law: a log containing critical violations could be certified as conforming simply because the violating events happened within run 65+.

---

## 3. Proposed Innovation: Symmetric Run-Bounded Conformance Gating

SRBCG restructures slot allocation and conformance verification into a branchless, timing-invariant pipeline that tracks capacity limits explicitly and refuses to certify logs that exceed maximum run capacities.

```mermaid
graph TD
    subgraph Current Vulnerable Flow
        A[Log Events] --> B[Assign Run Slot]
        B -->|Slot >= 64| C[Silently Ignore Event]
        C --> D[Continue validation]
        D -->|False Positive| E[ConformanceResult::Conforms]
    end

    subgraph Proposed SRBCG Flow
        F[Log Events] --> G[Branchless Slot Lookup]
        G -->|Overflow Flag Set| H[Accumulate Overflow Mask]
        H --> I[Evaluate Conformance Metrics]
        I -->|Overflow Mask Active| J[ConformanceResult::RunLimitExceeded]
    end

    style Proposed SRBCG Flow fill:#112233,stroke:#00aaff,stroke-width:2px;
```

### 3.1 Gated Refusal Variant
We introduce `ConformanceResult::RunLimitExceeded` as a first-class variant in `ConformanceResult`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ConformanceResult {
    Conforms,
    Violation {
        run_id: u64,
        op_idx: u32,
        missing_pred_mask: u64,
    },
    DuplicateFire {
        run_id: u64,
        op_idx: u32,
    },
    SealMismatch {
        run_id: u64,
        declared: u64,
        accumulated: u64,
    },
    EmptyLog,
    /// Refusal: The log contains more unique run IDs than the fixed limits
    /// of the deterministic validator.
    RunLimitExceeded,
}
```

### 3.2 Radon Law ($CC=1$) Slot Gating
To track run ID slot assignments branchlessly, we replace the looping `slot_for!` lookup with a parallel comparator network. Since `MAX_RUNS` is 64, we can represent slot availability and match status using bitwise logic.

For every event processing step, we:
1. Compare the incoming `run_id` with all 64 stored `run_ids` in parallel.
2. Determine if a match exists.
3. If no match exists and the active slot count is less than 64, write the new `run_id` to the next available slot.
4. If no match exists and the slot count is already 64, set an `overflow_mask` to `0xFFFFFFFFFFFFFFFF`.

The branchless implementation of this lookup and state transition is detailed below:

```rust
/// Symmetric Run-Bounded Conformance Gating (SRBCG) Slot Tracker.
///
/// Ensures CC=1 execution while checking run capacities. No heap allocations
/// are performed, and no data-dependent jumps are generated.
#[inline(always)]
pub fn process_event_srbcg(
    run_ids: &mut [u64; 64],
    run_count: &mut usize,
    incoming_rid: u64,
    overflow_mask: &mut u64,
) -> usize {
    let mut match_idx = 64usize;
    let current_count = *run_count;

    // Unrolled comparison across all 64 slots.
    // Compiles to branchless conditional selections (CSEL/CMOV).
    for i in 0..64 {
        let is_match = (run_ids[i] == incoming_rid) as usize;
        // If a match is found, match_idx becomes the slot index.
        // Otherwise, it remains unchanged.
        match_idx = (is_match * i) + ((1 - is_match) * match_idx);
    }

    // Determine if we need to allocate a new slot.
    let found = (match_idx < 64) as usize;
    let can_allocate = (current_count < 64) as usize;

    // Actions based on state:
    // Case 1: Found existing slot -> use match_idx, no count change, no overflow.
    // Case 2: Not found & can allocate -> use current_count, increment count, no overflow.
    // Case 3: Not found & cannot allocate -> use 64, no count change, set overflow.
    
    let allocate_idx = current_count;
    let target_idx = (found * match_idx) 
        + ((1 - found) * (can_allocate * allocate_idx + (1 - can_allocate) * 64));

    // Update count: increment if not found and can allocate.
    *run_count = current_count + ((1 - found) * can_allocate);

    // Update run_ids: write incoming_rid to target_idx if we allocated a new slot.
    let should_write = (1 - found) * can_allocate;
    for i in 0..64 {
        let mask = 0u64.wrapping_sub((should_write & (i == target_idx) as usize) as u64);
        run_ids[i] = (incoming_rid & mask) | (run_ids[i] & !mask);
    }

    // Accumulate overflow mask if not found and cannot allocate.
    let has_overflowed = (1 - found) * (1 - can_allocate);
    *overflow_mask |= 0u64.wrapping_sub(has_overflowed as u64);

    target_idx
}
```

At the end of log processing, the `overflow_mask` is evaluated. If it is non-zero, the validator immediately bypasses standard results and returns `ConformanceResult::RunLimitExceeded`.

---

## 4. Mathematical and Logical Contract

Under `@hoare_oracle` jurisdiction, the SRBCG contract satisfies the following specifications:

$$\{P(L, T)\} \quad \text{validate\_against\_tape\_srbcg}(L, T) \quad \{Q(L, T, R)\}$$

### 4.1 Preconditions $P(L, T)$
- **Well-Formed Log**: $L$ is a valid `OcelLog` with events $E = L.\text{events}()$ stored in a contiguous array.
- **Valid Tape**: $T$ is a compiled `PowlTape` containing operational rules and predecessor masks.

### 4.2 Postconditions $Q(L, T, R)$
Let $U(L)$ be the set of unique `run_id` values present in the events of log $L$:
$$U(L) = \{ e.\text{run\_id} \mid e \in L.\text{events}() \}$$

- **Refusal on Capacity Exhaustion**:
  $$|U(L)| > 64 \implies R = \text{ConformanceResult::RunLimitExceeded}$$
- **Exact Conformance Gating**:
  If $|U(L)| \le 64$, the validator evaluates conformance for every unique run. The output $R$ is:
  - `ConformanceResult::EmptyLog` if $|E| = 0$.
  - `ConformanceResult::DuplicateFire` if $\exists e_1, e_2 \in E$ such that $e_1.\text{run\_id} = e_2.\text{run\_id} \land e_1.\text{op\_idx} = e_2.\text{op\_idx} \land e_1 \ne e_2$.
  - `ConformanceResult::SealMismatch` if $\exists r \in U(L)$ such that the declared trace in `run_sealed` does not match the accumulated fired operations.
  - `ConformanceResult::Violation` if predecessor ordering rules on $T$ are breached.
  - `ConformanceResult::Conforms` if and only if all events in all runs strictly satisfy $T$.
- **Timing Invariance**:
  For any two logs $L_1, L_2$ with equal event counts $|L_1.\text{events}()| = |L_2.\text{events}()|$, the execution time $T_x$ satisfies:
  $$T_x(L_1) = T_x(L_2) \pm \epsilon$$
- **Zero Allocations**:
  No heap allocations are performed ($\text{Heap Allocations} = 0$).

---

## 5. Verification Strategy

To verify the correctness, safety, and performance constraints of SRBCG, we implement a multi-layered verification strategy.

### 5.1 Independent Reference Oracle
We define an independent oracle in `tests/reference.rs` that computes the exact set of unique run IDs using standard collections (like `BTreeSet`) and checks for limit violations:

```rust
fn oracle_validate_against_tape(log: &OcelLog, tape: &PowlTape) -> ConformanceResult {
    let mut run_set = std::collections::BTreeSet::new();
    for e in log.events() {
        run_set.insert(e.run_id);
    }
    
    if run_set.len() > 64 {
        return ConformanceResult::RunLimitExceeded;
    }
    
    // Fall back to standard validation logic for conformance...
    reference_conformance_check(log, tape)
}
```

A differential testing block will execute 50,000 runs to compare SRBCG to the oracle, specifically checking:
1. **Saturation Boundaries**: Logs containing exactly $63$, $64$, $65$, and $100$ unique run IDs.
2. **Vulnerability Isolation**: Injecting a predecessor violation *only* in run 65. The oracle and SRBCG must both return `RunLimitExceeded`, whereas the legacy implementation would falsely return `Conforms`.
3. **Empty and Single-Run Edge Cases**: Validating behavior at the minimum boundary limits.

### 5.2 Hostile Mutants
Under `@armstrong_fault` Master of Failure Law, we inject four mutants to test validation sensitivity:

1. **Mutant 1 (Omission of Overflow Propagation)**:
   ```rust
   // Mutant code: Skip updating the overflow mask
   let has_overflowed = (1 - found) * (1 - can_allocate);
   // *overflow_mask |= 0u64.wrapping_sub(has_overflowed as u64); // Commented out
   ```
   *Expectation*: Validation will skip events on run 65+ but return `Conforms` instead of `RunLimitExceeded`. The test suite must catch this by asserting that a log with 65 runs returns `RunLimitExceeded`.

2. **Mutant 2 (Off-by-One Capacity Gating)**:
   ```rust
   // Mutant code: Restrict capacity to 63 slots
   let can_allocate = (current_count < 63) as usize;
   ```
   *Expectation*: A log with 64 runs triggers `RunLimitExceeded`. The test suite must assert that logs with exactly 64 runs conform successfully.

3. **Mutant 3 (Duplicate Key Overwrite)**:
   ```rust
   // Mutant code: Write even if the key is already found
   let should_write = 1usize;
   ```
   *Expectation*: When an existing run ID is seen, its slot position is rewritten, potentially corrupting other stored run IDs. The test suite must catch this through subsequent trace mismatches or duplicate fire checks.

### 5.3 Object-Code Disassembly Audit Plan
The `@turing_machine` role will perform a disassembly audit of the compiled release code to confirm:
1. **Zero Loop Backedges**: The comparison block for the 64 slots must be completely unrolled into a flat sequence of instructions.
2. **Zero Conditional Jumps**: The `process_event_srbcg` helper must contain no branch instructions (e.g., `je`, `jne`, `cbz`). It must rely entirely on register-based logic and conditional selects (`csel`/`cmov`).
3. **Zero Heap Linkages**: Disassembly must verify that `__rust_alloc` is not referenced in the call path.
