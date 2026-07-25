# Rule 5: Mandatory Decomposition Protocol

According to `AGENTS.md` in the `bcinr` project, every nontrivial implementation task must be decomposed immediately into four independent workstreams. This protocol ensures mathematical and structural rigor through strict separation of concerns.

## The Four Independent Workstreams

| Workstream | Owner | Required Output |
| :--- | :--- | :--- |
| **Mathematical law** | `@hoare_oracle` | contracts and proof obligations |
| **Structural enforcement** | `@turing_machine` | source and object-code audit plan |
| **Hostile verification** | `@armstrong_fault` | mutants and refusal expectations |
| **Implementation** | `@von_neumann_bypass` | branchless bounded code |

## Strict Independence and No Self-Certification

Independence between these workstreams is strictly mandatory to prevent conflicts of interest and preserve the deterministic integrity of the substrate. Specifically:

- **No self-certification**: No implementation agent may author its own final oracle and self-certify equivalence.
- **No silent repairs**: No structural auditor may silently repair implementation code and then approve its own repair.
- **No derivative expectations**: No mutation agent may derive expected results from the implementation under attack.

By enforcing this strict independence, the repository guarantees that no single agent can bypass the rigorous mathematical and structural gates required for authoritative code admission.
