Based on the `AGENTS.md` file, here are the details regarding the "Hostile verification" workstream and the role of `@armstrong_fault`:

### Hostile Verification Workstream (Rule 5)
Under the **Mandatory decomposition protocol**, every nontrivial implementation task must be decomposed into four independent workstreams. 
* **Owner:** `@armstrong_fault`
* **Exact Output:** `mutants and refusal expectations`
* **Independence Requirement:** The mutation agent (`@armstrong_fault`) must not derive expected results from the implementation under attack.

### The Role of `@armstrong_fault` — Master of Failure Law (Rule 4)
`@armstrong_fault` is the **Adversarial test architect and mutation owner**.

**Exclusive Authority:**
* Counterfactual mutant design
* Hostile fixtures
* Negative-domain testing
* Refusal-path verification
* Test-suite adequacy judgments

**Specific Responsibilities and Requirements:**
1. **Minimum Mutant Requirement:** They must ensure every authoritative implementation file has at least three independent, syntactically plausible mutants. These mutants must alter a meaningful law (e.g., sign inversion, dropped factor, incorrect mask, bypassed refusal).
2. **Typed-Refusal Requirement:** They must ensure that tests do more than just assert inequality (`assert_ne!(baseline, mutant)` is prohibited). Tests must prove that a corrupted implementation either triggers a specific bounded typed refusal or violates a specific postcondition identifiable by the independent oracle.
3. **Enforcing the Standard:** They operate under the guiding standard: *“A suite that cannot kill a plausible mutant is itself defective.”*
