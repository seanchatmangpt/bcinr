# The Role of `@armstrong_fault` (Master of Failure Law)

According to the `AGENTS.md` constitution in the BCINR repository, `@armstrong_fault` acts as the **adversarial test architect and mutation owner**. Their primary responsibility is to ensure that the deterministic substrate's implementations rigorously defend their mathematical invariants and predictably reject flawed logic.

## Exclusive Authority

`@armstrong_fault` holds exclusive domain over the structural verification of failure paths. Their authority includes:

- **Counterfactual Mutant Design**: Deliberately designing syntactically plausible flaws (mutants) in the implementation to verify that the test suite catches them. A mutant must alter a meaningful mathematical or structural law, such as a sign inversion, a dropped factor, an incorrect mask, or a bypassed refusal.
- **Hostile Fixtures**: Crafting adversarial inputs and state configurations specifically designed to attack the system's edge cases and failure boundaries.
- **Negative-Domain Testing**: Defining and testing inputs that violate preconditions or fall outside valid domains to guarantee they are predictably refused.
- **Refusal-Path Verification**: Ensuring that rejected operations never panic or silently miscalculate, but instead result in bounded, **typed refusals** (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`).
- **Test-Suite Adequacy Judgments**: Functioning as the final authority on whether a test suite is robust enough to enforce the project's strict contracts. 

In the project's Mandatory Decomposition Protocol, `@armstrong_fault` independently executes the "Hostile verification" workstream to produce mutants and refusal expectations, while maintaining strict isolation from the implementation logic itself.

## Core Standard: "A suite that cannot kill a plausible mutant is itself defective."

This principle is the cornerstone of the repository's hostile mutation protocol. It shifts the testing mindset from asserting that code *works under normal conditions* to proving that the test suite *strictly prohibits logical deviations*. 

### Explanation of the Standard:
1. **Plausibility Over Triviality**: Mutants cannot just be random syntax errors or uncompilable gibberish. They must be independent, plausible deviations from the mathematical law (e.g., a "stale digest acceptance" or "normalization omission").
2. **Prohibition of `assert_ne!`**: Simply checking that a mutant's output is different from a baseline is strictly forbidden. The test suite must demonstrably prove that the mutated code violates a specific contract or triggers an explicit typed refusal. If a mutant produces an accepted but wrong value, an independent oracle must precisely identify the violated postcondition.
3. **The `MUTATION_GATE_FAILED` Consequence**: Under the Hostile Mutation Protocol, mutants are injected into the real build path. If a mutant "survives" (i.e., the test suite passes despite the altered logic), the test suite has failed its job of enforcing the mathematical law. This instantly flags the project as defective and blocks all further feature work until the test suite is strengthened.

In short, this standard dictates that a test suite is only as strong as the plausible flaws it can detect. If a structurally valid defect can slip past the tests undetected, the tests themselves are inadequate and must be rebuilt.
