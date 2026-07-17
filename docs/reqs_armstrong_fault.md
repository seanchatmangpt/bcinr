# Adversarial Testing Requirements (v26.7.15)
## Author: @armstrong_fault
## Target: bcinr, praxis, mfact

### 1. Counterfactual Mutants (3 Per File)

As per the Failure Law, any test suite that cannot find a bug in a broken implementation is itself a failure. The following mutants must be syntactically plausible but structurally broken. The "hostile" test suite must detect and fail on every single one.

#### 1.1 Numeric Fluents (`crates/bcinr-pddl/src/concurrency.rs`)
- **Mutant 1 (The "Always Pairwise" Mutant):** Silently drops capacity > 2 constraints, returning a purely pairwise approximation instead of `Unsupported` when exact higher-order nonfaces cannot be constructed.
- **Mutant 2 (The "False Positive Nonface" Mutant):** Incorrectly flags a 3-way nonface even when cumulative numeric-fluent consumption is strictly less than the total capacity.
- **Mutant 3 (The "Constant Capacity" Mutant):** Ignores dynamic capacity updates and assumes initial capacity is static, failing to detect conflicts when available resources shrink during execution.

#### 1.2 V2 Scheduler Bridge (`crates/bcinr-powl/src/scheduler.rs`)
- **Mutant 1 (The "Guard Bypass" Mutant):** Consults the V2 Tape correctly but entirely skips the `ConcurrencyGuardTable.admits(&fired)` check, firing all ready ops blindly and generating an invalid state transition.
- **Mutant 2 (The "Single-Step Reversion" Mutant):** Restricts the V2 scheduler to fire a maximum of one operation per tick (`fired.count_ones() <= 1`), passing all sequential logic tests while completely neutralizing structural concurrency.
- **Mutant 3 (The "Index Skew" Mutant):** Misaligns the `PowlNodeId` lookup by an offset of 1 during the translation from V2 bitmasks to tape execution, causing incorrect operations to execute despite valid bitmask logic.

#### 1.3 Opaque Receipts (`crates/bcinr-powl-receipt/src/receipt.rs`)
- **Mutant 1 (The "Self-Fulfilling Prophecy" Mutant):** Modifies `verify_execution_receipt` to dynamically redefine `ready = fired`, trivially passing the `fired ⊆ ready` invariant every time and allowing malicious execution paths.
- **Mutant 2 (The "Legacy Opaque Hash" Mutant):** Uses the explicit `ready: EventSet` field but continues to verify against the old opaque `scheduler_decision_digest`, allowing a mismatch between the provided set and the actual hash.
- **Mutant 3 (The "Subset Inversion" Mutant):** Inverts the safety check to `ready.is_subset_of(&fired)`, meaning it only passes if the scheduler fired *everything* that was ready, which artificially outlaws valid partial-scheduling strategies.

---

### 2. Hostile Negative Fixtures

These negative fixtures must be implemented exactly as specified in the Ordered Residual Patch Plan to demonstrate that the test suite is capable of detecting structurally broken paths.

1. **`link3a_real_pddl_pipeline_cannot_produce_a_three_way_nonface_from_pairwise_independent_actions`**
   - **Goal:** Prove that when feeding pure classical STRIPS (lacking numeric fluents) into the PDDL pipeline, it cannot organically synthesize a capacity-3 nonface. A genuine exception or `Unsupported` error must be generated, rejecting the pairwise approximation.

2. **`link2_precedes_is_the_full_input_vector_order_even_though_every_pair_is_independent`**
   - **Goal:** Expose the artifact in `PddlCausalAnalyzer`. Feed $N$ completely independent actions. Ensure the test fails if the resulting `precedes` graph has an edge count $> 0$. It must correctly capture empty precedence for independent operations.

3. **`link6_real_scheduler_never_fires_the_triple_when_the_ready_set_is_the_triple`**
   - **Goal:** Show that without the V2 Scheduler bridge, passing a legitimate V2 Tape into the execution loop fails to concurrently fire the valid triple `{A, B, C}`. The test must demand the execution of the wide `CompiledPowlV2`.

4. **`link7_execution_receipt_fired_pair_differs_from_the_genuinely_ready_triple`**
   - **Goal:** Attack the opaque receipt mechanism. Construct a state where `ready = {A, B, C}` and `fired = {A, B}`. The prover artificially injects an unready operation into `fired` (e.g., `{A, B, D}`). The stateless verifier MUST fail the verification when enforcing `fired ⊆ ready`.

5. **`strict_predicate_fails_on_a_perfect_trace_due_to_mocked_dimensions`**
   - **Goal:** Provide a flawless execution trace to `PowlReplayVerifier`. The verifier must fail the final conformance predicate if `generalization` or `simplicity` are statically mocked to Q16.16 zero, proving that it enforces derivation of real state-space metrics.
