# Mandatory Decomposition Protocol (Rule 5)

According to **Rule 5** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), every nontrivial implementation task must be immediately decomposed into four independent workstreams. 

## The Four Workstreams and Owners

| Workstream | Owner | Output |
| :--- | :--- | :--- |
| **Mathematical law** | `@hoare_oracle` | Contracts and proof obligations |
| **Structural enforcement** | `@turing_machine` | Source and object-code audit plan |
| **Hostile verification** | `@armstrong_fault` | Mutants and refusal expectations |
| **Implementation** | `@von_neumann_bypass` | Branchless bounded code |

## Prohibition on Self-Certification

It is strictly prohibited for agents to self-certify equivalence across these workstreams because **independence is mandatory**. The protocol establishes strict separations of concern to prevent circular validation and to guarantee robust enforcement of the core architectural laws:

- **No circular oracles:** An implementation agent (`@von_neumann_bypass`) may not author its own final oracle and self-certify equivalence. This ensures the implementation is validated against an independent mathematical truth, rather than validating against its own logic.
- **No self-approved repairs:** A structural auditor (`@turing_machine`) may not silently repair implementation code and then approve its own repair. This preserves the auditor's objective distance from the codebase being analyzed.
- **No tautological testing:** A mutation agent (`@armstrong_fault`) may not derive expected results from the implementation under attack. This ensures that hostile verification is an independent check on mathematical and structural correctness rather than an affirmation of the system's current behavior.

Furthermore, **Rule 27 (No self-certification)** explicitly expands on this by stating that an implementation agent cannot be the final approver for mathematical correctness, branchlessness, oracle independence, mutant adequacy, object-code compliance, or standing. Every approval must come from a distinct role and be backed by a mechanical artifact, enforcing the principle that mere agent agreement does not constitute evidence.
