# Rule 5: Mandatory Decomposition Protocol

In the `bcinr` deterministic substrate constitution, every nontrivial implementation task must be immediately decomposed into four strictly independent workstreams. 

## The 4 Independent Workstreams

| Workstream             | Owner                 | Output                            |
| ---------------------- | --------------------- | --------------------------------- |
| **Mathematical law**       | `@hoare_oracle`       | Contracts and proof obligations   |
| **Structural enforcement** | `@turing_machine`     | Source and object-code audit plan |
| **Hostile verification**   | `@armstrong_fault`    | Mutants and refusal expectations  |
| **Implementation**         | `@von_neumann_bypass` | Branchless bounded code           |

## The Ban on Self-Certification and Overlapping Ownership

Independence across these workstreams is **mandatory**. Self-certification and overlapping ownership are explicitly prohibited to prevent circular logic, bias, and the weakening of verification standards. The rules state:

1. **No self-certifying oracles:** An implementation agent (`@von_neumann_bypass`) may not author its own final oracle and self-certify mathematical equivalence. 
2. **No self-approving repairs:** A structural auditor (`@turing_machine`) may not silently repair implementation code and then approve its own repair.
3. **No circular derivations in testing:** A mutation agent (`@armstrong_fault`) may not derive expected results (or refusals) from the implementation currently under attack.

By enforcing these strict ownership boundaries, the protocol ensures that rules are not weakened for implementation convenience, mathematical contracts are verified objectively, and hostile verification actually enforces the contract boundaries rather than merely echoing implementation behavior.
