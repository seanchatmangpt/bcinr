Here is the requested research on Rule 26 regarding exclusive write ownership from `AGENTS.md`:

### Rule 26: State Isolation and Write Ownership

Under Rule 26, agent work must use exclusive write ownership for specific domains. Shared-file concurrent editing is strictly prohibited.

**Domain Ownership:**
- **`@hoare_oracle`**: Exclusive writer for **contracts and proofs**
- **`@turing_machine`**: Exclusive writer for **scanners and structural gates**
- **`@armstrong_fault`**: Exclusive writer for **mutants and hostile fixtures**
- **`@von_neumann_bypass`**: Exclusive writer for **authoritative implementation**

*Note: Other agents may review these files but cannot edit them without an explicit ownership transfer recorded in the work log. Generated files may only be written by the admitted generator.*

### Why is this strictly enforced?

This strict separation of write ownership enforces **independence and prevents self-certification**, which are absolute constitutional requirements of the repository:

1. **Mandatory Independence (Rule 5):** The repository requires all nontrivial tasks to be decomposed into independent workstreams. 
   - An implementation agent (`@von_neumann_bypass`) cannot author its own final oracle and self-certify equivalence.
   - A structural auditor (`@turing_machine`) cannot silently repair implementation code and then approve its own repair.
   - A mutation agent (`@armstrong_fault`) cannot derive expected results from the implementation under attack.
2. **No Self-Certification (Rule 27):** The implementation agent may not be the final approver for mathematical correctness, branchlessness, oracle independence, or mutant adequacy. Each approval must come from a separate, independent role and be backed by a mechanical artifact.
3. **State Isolation:** It eliminates shared-file concurrent editing, ensuring that conflicting writes or silent overrides do not bypass structural gates or violate mathematical contracts.
