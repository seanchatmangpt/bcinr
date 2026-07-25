Based on the `AGENTS.md` file, here is the detailed breakdown of the role of `@armstrong_fault` and what "hostile fixtures" and "negative-domain testing" entail:

### Role of `@armstrong_fault` (Master of Failure Law)
The `@armstrong_fault` agent acts as the **Adversarial test architect and mutation owner**. This role is responsible for rigorously testing the system's resilience by designing counterfactual scenarios and ensuring that the deterministic runtime rightfully rejects invalid or corrupted states.

### Hostile Fixtures and Negative-Domain Testing
Under **Rule 4**, "hostile fixtures" and "negative-domain testing" are part of the exclusive authority of `@armstrong_fault`. They entail a systematic, adversarial approach to verification:

1. **Hostile Mutants Requirement:**
   - Every authoritative implementation file must have at least **three independent, syntactically plausible mutants**.
   - These mutants simulate adversarial or failure conditions by altering a meaningful law (e.g., sign inversion, dropping a factor, incorrect mask, bypassing a refusal, or omitting a normalization).

2. **Typed-Refusal Verification (Negative-Domain):**
   - The goal of negative-domain testing isn't just to see if the output is different (simple `assert_ne!(baseline, mutant)` is explicitly **prohibited**). 
   - Instead, the tests must mathematically prove that the corrupted implementation either:
     - Triggers a specific bounded **typed refusal** (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`).
     - Or, if it produces a wrong accepted value, the independent oracle must identify the exact violated postcondition.

3. **Hostile Mutation Protocol (Rule 19):**
   - The agent must identify load-bearing laws, produce mutants, inject them into the real build path, and run the normal suite.
   - The "kill evidence" (proving the mutant was successfully rejected or identified as a mismatch) must be recorded in a **mutant ledger**.

**The Governing Standard:** *"A suite that cannot kill a plausible mutant is itself defective."* If any hostile test (mutant) survives without being caught by a typed refusal or contract violation, the project's standing immediately changes to `MUTATION_GATE_FAILED` and blocks all feature work.
