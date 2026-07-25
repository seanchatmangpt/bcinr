# CHEAT-021: Rejection State Drift

## Overview
In the `bcinr` branchless architecture, state transitions are applied via mask-based execution rather than conditional logic. A rejected transaction generates a canonical admission mask of `0`, meaning the `select` function should simply return the current state instead of the candidate state.

`CHEAT-021` (Rejection State Drift) is an anti-cheat rule that mathematically guarantees this invariant: **any transaction yielding a typed refusal must leave the persistent state perfectly bit-for-bit unchanged.**

## Anti-Cheat System Enforcement

The anti-cheat system enforces this guarantee through a two-pronged approach: static analysis via the cheat scanner, and an enforced runtime unit test harness.

### 1. Static Analysis (`bcinr-cheat-scanner`)
The `bcinr-cheat-scanner` tool (`tools/bcinr-cheat-scanner/src/main.rs`) strictly mandates the existence of a rejection invariance test using string-based text analysis.
- **Rule Enforcement**: When scanning test files, it looks for any live `case_studies.rs` file located directly under a crate's `tests/` directory (excluding inert nested fixtures).
- **Verification**: If it finds a `case_studies.rs` integration suite, it asserts that the file contains the exact string `"test_rejection_invariance"`. 
- **Violation Flag**: If the string is not present, the scanner immediately fails the build and flags a violation: 
  `"CHEAT[CHEAT-021]: <path> — rejection state drift: case studies missing test_rejection_invariance check"`.

### 2. Required Test Harness (`test_rejection_invariance`)
The mandated test itself (e.g., located in `crates/bcinr-cmca/tests/case_studies.rs`) is responsible for providing the executable proof that the state remains untouched during a refusal.

The test enforces the rule by:
1. **Capturing State**: Taking a snapshot of the complete persistent state structures before any action is taken (e.g., `weights`, `last_switch_t`, `prev_mode`).
2. **Triggering Refusal**: Invoking the authoritative function (like `allocate`) with parameters explicitly designed to fail the admission policy (such as an invalid digest, or out-of-bounds metrics).
3. **Asserting Refusal**: Confirming the function correctly returns a typed refusal (`res.is_refused()`).
4. **Asserting Invariance**: Performing a rigorous field-by-field equality check (`assert_eq!`) comparing the post-transaction state directly with the pre-transaction snapshot. 

If any bit has drifted during the rejected call, the test fails, echoing:
`"CHEAT-021: REJECTION_STATE_DRIFT - [variable] modified on rejection!"`.

This combination of the cheat scanner mandating the test's existence and the test verifying the mathematical boundaries ensures complete field-equality and prevents speculative mutations from leaking through branchless abstractions.
