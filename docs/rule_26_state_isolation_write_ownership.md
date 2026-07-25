### Exclusive Writers per Domain

Under the BCINR constitution, agent work must strictly use exclusive write ownership. The domains and their designated exclusive writers are mapped as follows:

| Domain | Exclusive Writer |
| :--- | :--- |
| **Contracts and proofs** | `@hoare_oracle` |
| **Scanners and structural gates** | `@turing_machine` |
| **Mutants and hostile fixtures** | `@armstrong_fault` |
| **Authoritative implementation** | `@von_neumann_bypass` |

Other agents are permitted to review these domains, but they **may not edit** them without an explicit ownership transfer that is recorded in the work log. Additionally, generated files may be written only by the admitted generator.

### Why Shared-File Concurrent Editing is Prohibited

Shared-file concurrent editing is prohibited in order to enforce **strict independence and prevent self-certification**. 

According to the constitution's principles (especially those detailed in Rules 5 and 27):
- An implementation agent (`@von_neumann_bypass`) cannot author its own oracle and self-certify equivalence.
- A structural auditor (`@turing_machine`) cannot silently repair implementation code and then approve its own repair.
- A mutation agent (`@armstrong_fault`) cannot derive expected results from the implementation under attack.

By restricting file modification to exclusive writers and prohibiting shared-file concurrent editing, the project ensures that each step of the verification matrix (mathematical law, structural enforcement, hostile verification, and implementation) remains completely independent. No single agent can silently bypass the rigid rules of the substrate constitution.
