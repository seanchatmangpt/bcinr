Based on my review of `AGENTS.md`, here are the details regarding Rule 26 ("State isolation and write ownership") and its prohibition of shared-file concurrent editing.

### Why Shared-File Concurrent Editing is Prohibited
Shared-file concurrent editing is prohibited to enforce strict **state isolation** and **mandatory independence** among agents. The BCINR deterministic substrate constitution dictates that an agent cannot verify or certify its own work (e.g., an implementation agent cannot author its own oracle, and a structural auditor cannot repair code and approve its own repair, as established in Section 5). Allowing multiple agents to edit the same file concurrently would violate these strict boundaries of authority and introduce the risk of self-certification or compromised independence.

### The Required Protocol
Instead of shared concurrent editing, agents must follow an **exclusive write ownership** protocol:

1. **Exclusive Writers per Domain**: Each specific domain is assigned a single, exclusive writer:
   - **Contracts and proofs**: `@hoare_oracle`
   - **Scanners and structural gates**: `@turing_machine`
   - **Mutants and hostile fixtures**: `@armstrong_fault`
   - **Authoritative implementation**: `@von_neumann_bypass`

2. **Read-Only Reviews**: Other agents are allowed to review files outside their domain, but they **may not edit** them.

3. **Explicit Ownership Transfers**: If another agent needs to edit a file outside its domain, an explicit ownership transfer must first be recorded in the work log.

4. **Generated Code**: Generated files may only be written by the formally admitted generator.
