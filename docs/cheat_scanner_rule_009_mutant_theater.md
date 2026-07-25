Here is the documentation detailing how `CHEAT-009` (Mutant Theater) is enforced within the `bcinr-cheat-scanner`, based on the codebase (`tools/bcinr-cheat-scanner/src/main.rs`):

# Documentation for CHEAT-009 (Mutant Theater) Enforcement

`CHEAT-009` enforcement is implemented in `tools/bcinr-cheat-scanner/src/main.rs`. It ensures that tests labeled as "mutants" verify an exact typed refusal or specific mathematical divergence, rather than merely verifying that the output has changed in some arbitrary way.

## Detection Logic

The rule analyzes text in test files (files inside test/bench directories where the source contains the string `"mutant"`). If the scanner finds an `assert_ne!` macro call, it requires the test to *also* contain evidence of a rigorous, typed verification mechanism.

Specifically, it looks for at least one of the following exact string matches to prove the test is checking for a domain/refusal rejection or a specific postcondition failure:

**Pre-Sealed-API Refusal Shapes:**
- `Err(StabilityRefusal::`
- `Err(ObservatoryFlag::`

**Sealed API Opaque Outcome Accessors:**
- `.is_refused()`
- `.numeric_faults()`
- `.faults()`
- `.refusals()`

**Numeric Divergences (Non-Refusals):**
- `// Named law:` (a comment binding the assertion to the specific mathematical invariant or postcondition violated).

If a mutant test includes `assert_ne!` but fails to include *at least one* of these typed refusal or divergence markers, the scanner triggers an error and blocks the merge with the following message:

`"CHEAT[CHEAT-009]: <path> — mutant theater: test uses weak assert_ne instead of asserting a typed refusal"`

## Rule Metadata and Remediation

According to the `CheatRule` struct definition:
- **Detection Contract:** "Detects mutants that are trivial or not verified by assertions of typed refusals."
- **Remediation Code:** "Strengthen assertions in counterfactual tests to verify exact typed failure codes."

By mandating that mutant tests match against designated fault accessors or explicity named mathematical laws, the scanner ensures developers cannot satisfy mutation testing requirements with trivial, low-effort assertions like `assert_ne!(baseline, mutant)`. This forces all counterfactual testing to adhere strictly to the substrate's requirement for bounded, typed refusals.
