# Rule 26: State Isolation and Write Ownership in BCINR

According to Rule 26 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), write ownership is strictly partitioned among the four distinct agent roles to enforce absolute independence and rigorously separate concerns.

## 1. Exclusive Write Ownership

Agent work must use **exclusive write ownership** distributed across specific domains:

| Domain | Exclusive Writer |
| :--- | :--- |
| **Contracts and proofs** | `@hoare_oracle` |
| **Scanners and structural gates** | `@turing_machine` |
| **Mutants and hostile fixtures** | `@armstrong_fault` |
| **Authoritative implementation** | `@von_neumann_bypass` |

**Why this is required:**
Exclusive ownership enforces the mandatory independence outlined in Rule 5 (Mandatory Decomposition Protocol) and Rule 27 (No Self-Certification). By restricting write access, the system ensures that:
- The implementation agent (`@von_neumann_bypass`) cannot author its own final oracle, write a circular test, or self-certify its implementation.
- The structural auditor (`@turing_machine`) cannot silently repair implementation code and then approve its own repair. 
- The adversarial tester (`@armstrong_fault`) cannot derive expected results from the implementation under attack.

Other agents are permitted to *review* files within another's domain, but they **may not edit** them without an explicit ownership transfer recorded in the work log.

## 2. Prohibition of Shared-File Concurrent Editing

Rule 26 explicitly states: **"Shared-file concurrent editing is prohibited."** 

**Why this is prohibited:**
Shared editing breaks down the rigid separation of powers necessary for the substrate's integrity. If multiple agents can concurrently modify the same file, the boundaries between implementation, auditing, mathematical proof, and hostile verification blur. Prohibiting concurrent edits ensures:
- A clear, isolated audit trail for every change.
- Zero risk of cross-contamination where one agent accidentally (or intentionally) weakens a higher-order rule to satisfy a lower-order objective.
- Explicit handoffs and formal communication between roles, preventing silent workarounds.

## 3. Rules Around Generated Files

Rule 26 dictates: **"Generated files may be written only by the admitted generator."** 

**Why this is enforced:**
As stated in Rule 21 (Generated-code law), generated code must be perfectly reproducible. Any hand-editing by an agent or human destroys the guarantee that the generated output is a deterministic function of its source. If generated files exhibit unexplained drift from what the admitted generator produces, they instantly invalidate the standing of the repository. Therefore, generated code can only be generated mechanically and must never be directly manipulated.
