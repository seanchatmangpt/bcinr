# Analysis of M14 (State Drift After Rejection)

**Location**: 
While `mutant_14` does not appear as a numbered feature flag inside `tests/hostile_mutants.rs` (which houses mutants 1 through 11), it is explicitly tracked in the `bcinr` architecture as **"M14 state drift after rejection"** (as defined in `ORIGINAL_REQUEST.md`). Its verification is enforced globally via `crates/bcinr-cmca/tests/case_studies.rs` within the `test_rejection_invariance` harness, which is rigorously mandated by the static analysis rule **CHEAT-021**.

### Mathematical Law Broken
M14 simulates a break of **Rule 10: No mutation before complete admission**.

In the `bcinr` branchless architecture, persistent state must never be mutated speculatively. The required lawful transaction shape is:
```text
current immutable state
→ fixed-size candidate state
→ verify all predicates
→ derive admission mask
→ fieldwise masked commit
```
The final state must mathematically be computed as `select(mask, candidate_state, current_state)`. When an operation is rejected (the admission mask evaluates to `0`), the committed state must remain perfectly bit-for-bit unchanged. M14 represents a defect where branchless abstractions leak speculative mutation into persistent memory despite a rejection.

### Expected Outcome / Refusal
1. **Typed Refusal:** The authoritative function (e.g., `allocate()`) is intentionally fed invalid parameters so that it correctly returns a bounded, typed refusal code (caught via `res.is_refused()`).
2. **Invariance Assertion:** The `test_rejection_invariance` harness takes a complete snapshot of the state prior to the transaction (`weights_before`, `last_switch_t_before`, etc.). Following the forced refusal, it executes a strict field-by-field equality check (`assert_eq!(post_state, pre_state)`).
3. **Detection Output:** If M14 (state drift) is present, the field-equality assertion fails, exposing the leak and echoing:
   `"CHEAT-021: REJECTION_STATE_DRIFT - [variable] modified on rejection!"`

*Note: The Anti-Cheat scanner (`tools/bcinr-cheat-scanner/src/main.rs`) strictly enforces that this harness is never removed. If `test_rejection_invariance` is missing from `case_studies.rs`, the build is instantly blocked with a `CHEAT[CHEAT-021]` violation.*
