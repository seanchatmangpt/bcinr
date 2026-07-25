# Rule 26 & 27: State Isolation and No Self-Certification

Based on the `AGENTS.md` constitution for the BCINR Deterministic Substrate, Rule 26 and Rule 27 establish strict guidelines for state isolation, write ownership, and the separation of implementation and verification to guarantee the integrity of the substrate.

## Rule 26: State Isolation and Write Ownership

To enforce strict separation of concerns and prevent unintended modifications across distinct areas of the project, agent work must use exclusive write ownership. 

### Exclusive Write Ownership Domains
- **`@hoare_oracle`**: Exclusive writer for **contracts and proofs**.
- **`@turing_machine`**: Exclusive writer for **scanners and structural gates**.
- **`@armstrong_fault`**: Exclusive writer for **mutants and hostile fixtures**.
- **`@von_neumann_bypass`**: Exclusive writer for the **authoritative implementation**.

### Restrictions on Editing and Collaboration
- **Review vs. Edit**: While other agents may review code outside their domain, they **may not edit** it unless there is an explicit ownership transfer recorded in the work log.
- **No Shared-File Concurrent Editing**: Shared-file concurrent editing is strictly prohibited. This ensures that modifications to any given file remain completely under the authority of its designated owner, avoiding conflicted states and maintaining a clear chain of custody.
- **Generated Code**: Generated files may only be written by the admitted generator, ensuring deterministic and reproducible outputs.

## Rule 27: No Self-Certification

Rule 27 mandates that the agent writing the implementation cannot be the one to verify and approve it. This guarantees that implementation biases, assumptions, or errors are independently subjected to rigorous scrutiny.

### Prohibited Final Approvals for Implementation Agents
The implementation agent (`@von_neumann_bypass`) **may not** be the final approver for:
- Mathematical correctness
- Branchlessness
- Oracle independence
- Mutant adequacy
- Object-code compliance
- Standing

### The Mechanics of Verification
- **Independent Validation**: Approvals must come from a separate, designated role whose sole focus is on verifying that specific dimension of the code (e.g., `@hoare_oracle` for mathematical correctness or `@turing_machine` for branchlessness).
- **Mechanical Evidence Required**: An agent's verbal or written agreement does not count as evidence. Every approval must be backed by a **mechanical artifact** (such as a proof, scan result, or failed mutant log). As the constitution states: "Five agents repeating the same claim is still one unsupported claim."
