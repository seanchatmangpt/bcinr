# State Isolation and Write Ownership (Rule 26)

According to Rule 26 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), agent work must adhere to strict state isolation through exclusive write ownership.

## Partitioning of Exclusive Write Ownership

Write ownership is strictly partitioned among the four constitutional roles, mapping specific domains to their exclusive writers:

| Domain | Exclusive Writer |
| :--- | :--- |
| Contracts and proofs | `@hoare_oracle` |
| Scanners and structural gates | `@turing_machine` |
| Mutants and hostile fixtures | `@armstrong_fault` |
| Authoritative implementation | `@von_neumann_bypass` |

## Prohibition of Shared-File Concurrent Editing

Shared-file concurrent editing is **strictly prohibited**. This prohibition exists to guarantee the integrity of the mandatory decomposition protocol (Rule 5) and to prevent self-certification (Rule 27). By enforcing exclusive write access, the system ensures that no agent can silently alter code outside its jurisdiction (for example, the structural auditor cannot silently repair implementation code and approve its own repair, and the implementation agent cannot alter the oracle to match their code). Independence is mandatory, and shared editing would compromise the rigid structural boundaries between these domains.

*Note: As an extension of this isolation, generated files may only be written by the admitted generator.*

## Formal Recording of Ownership Transfers

While other agents are permitted to *review* files outside their domain, they may not edit them under any circumstances unless an **explicit ownership transfer is recorded in the work log**. This ensures there is a permanent, auditable trail anytime write authority temporarily shifts between constitutional roles.
