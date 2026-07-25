Here is the requested documentation on `mutant_14`:

# Analysis of M14 (State Drift After Rejection)

**Location in Codebase**:
While `mutant_14` is conceptually part of the adversarial test matrix, it does **not** appear as a numbered feature flag inside `crates/bcinr-cmca/tests/hostile_mutants.rs` (which only houses mutants 1 through 11). 

Instead, it is formally documented in `docs/test_hostile_mutant_14_state_drift.md` and its verification is implemented globally in `crates/bcinr-cmca/tests/case_studies.rs` within the `test_rejection_invariance` test harness.

### Mathematical Law Broken
`mutant_14` simulates a violation of **Rule 10: No mutation before complete admission**.

In the deterministic branchless architecture of `bcinr`, persistent state must never be mutated speculatively. The required lawful state transaction must strictly follow this shape:
1. Load current immutable state
2. Compute fixed-size candidate state
3. Verify all predicates to derive an admission mask
4. Apply fieldwise masked commit

The final committed state must mathematically be computed as `select(mask, candidate_state, current_state)`. When an operation is rejected (i.e., the admission mask evaluates to `0`), the committed state must remain mathematically and structurally bit-for-bit unchanged. M14 introduces a defect where a branchless abstraction incorrectly leaks a speculative mutation into persistent memory despite a transaction rejection.

### Expected Outcome / Refusal
The framework guarantees the detection of `mutant_14` through the `test_rejection_invariance` harness:

1. **Typed Refusal:** An authoritative function (e.g., `allocate()`) is intentionally fed invalid parameters (like an invalid proof or digest), causing it to correctly return a bounded, typed refusal (`res.is_refused()`).
2. **Invariance Assertion:** The test captures a strict snapshot of the state prior to execution (`weights_before`, `last_switch_t_before`, etc.) and executes a field-by-field equality check `assert_eq!(post_state, pre_state)` after the refusal is produced.
3. **Detection Output:** If the M14 mutation (state drift) is injected, the state will have changed, causing the test's equality assertion to fail. This exposes the speculative mutation leak and triggers the exact error:
   `"CHEAT-021: REJECTION_STATE_DRIFT - [variable] modified on rejection!"`

*Note: The Anti-Cheat scanner inherently enforces that `test_rejection_invariance` is never removed. If it is missing, the build is instantly blocked with a `CHEAT[CHEAT-021]` violation.*
