Based on the `AGENTS.md` constitution in the BCINR repository, the interaction between **Rule 5 (Mandatory decomposition protocol)** and **Rule 27 (No self-certification)** strictly prevents the implementation owner (`@von_neumann_bypass`) from self-certifying the adequacy of mutants created by `@armstrong_fault`. 

Here is a breakdown of why this is required:

### 1. Structural Independence (Rule 5)
Rule 5 requires every nontrivial implementation task to be decomposed into four entirely independent workstreams. 
* `@von_neumann_bypass` is strictly responsible for writing the **branchless bounded code** (the authoritative implementation).
* `@armstrong_fault` is strictly responsible for **hostile verification**, which includes designing counterfactual mutants and defining typed refusal expectations.

Rule 5 explicitly states that **"Independence is mandatory"** and "No mutation agent may derive expected results from the implementation under attack." This separation ensures the implementation and its adversarial tests are developed in a vacuum relative to each other.

### 2. Prohibition of Conflict of Interest (Rule 27)
Rule 27 explicitly forbids the implementation agent from acting as the final approver for a specific list of criteria, which includes **mutant adequacy**. It states: *"Each approval must come from a different role and be backed by a mechanical artifact."*

### Why the Implementation Owner Cannot Self-Certify Mutants:
The synthesis of these two rules exists to prevent specific failures in the verification process:
* **Preventing "Mutant Theater" (CHEAT-009):** If `@von_neumann_bypass` were allowed to certify `@armstrong_fault`'s mutants, they could consciously or subconsciously approve weak, syntactically trivial mutants that their implementation can easily defeat. This would create the illusion of rigorous testing without actually challenging the code.
* **Avoiding Circular Verification:** An implementation owner verifying their own attacks is likely to base the adequacy of the mutant on the implementation's current behavior, rather than on the independent mathematical contracts defined by `@hoare_oracle`. Adversarial tests must verify the *law*, not just the *current code*.
* **Enforcing Adversarial Integrity:** The BCINR substrate relies on a true adversarial relationship between the code and its tests. `@armstrong_fault` acts as a hostile attacker. Allowing the defender (`@von_neumann_bypass`) to set the rules of the attack or judge its adequacy fundamentally destroys the adversarial model required to achieve PhD-level verification and a Substrate Integrity Score (SIS) of 100.
