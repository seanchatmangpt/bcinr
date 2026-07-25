# Rule 26: State Isolation and Write Ownership in BCINR

According to **Rule 26** of the BCINR Deterministic Substrate Constitution, the repository mandates strict physical and logical separation of concerns among agents to ensure rigorous state isolation and prevent self-certification. 

## Exclusive Write Ownership Policy

Agent work must use **exclusive write ownership**. Shared-file concurrent editing is strictly prohibited. The ownership boundaries are mapped exactly to each persona's role:

| Domain | Exclusive Writer | Responsibility |
| :--- | :--- | :--- |
| **Contracts and proofs** | `@hoare_oracle` | Preconditions, invariants, independent reference semantics |
| **Scanners and structural gates** | `@turing_machine` | Source/object-code audit plans, CI/CD, Git metadata enforcement |
| **Mutants and hostile fixtures** | `@armstrong_fault` | Hostile testing, typed refusal expectations |
| **Authoritative implementation** | `@von_neumann_bypass`| Branchless bounded production code |

Other agents are allowed to review files outside their domain, but they **may not edit** them. Generated files may only be written by the admitted generator.

## How the Policy is Enforced

The project ensures that `@von_neumann_bypass` cannot secretly edit mathematical contracts (which belong to `@hoare_oracle`) and `@hoare_oracle` cannot write production implementations through several strict mechanisms enforced by the `@turing_machine` gatekeeper:

### 1. CI/CD Gatekeepers (`bcinr-cheat-scanner`)
Pre-merge hooks in the CI/CD pipeline physically enforce the rules by scanning Git metadata:
* **Diff-to-Role Mapping:** The scanner verifies that the files modified by a commit strictly align with the domain mapped to the author's declared role (e.g., rejecting any commit where `@von_neumann_bypass` modifies proof files).
* **Lineage Check:** The gatekeeper ensures no commit retroactively edits a file owned by another agent without a valid, preceding ownership transfer recorded in the branch's topology.

### 2. Strict Git Commits and Merge Strategies
* **Domain-Isolated Commits:** A single Git commit must never cross ownership boundaries. Updating both a structural proof and its hot-path implementation in the same commit is prohibited.
* **The Zero-Conflict Law:** A Git merge conflict on any tracked file is considered physical proof that illegal concurrent editing occurred. Only orthogonal merges (disjoint diff sets) or strict fast-forward (`--ff-only`) merges are permitted.
* **Commit Attribution:** Every commit must explicitly declare the authorized persona executing the change via Git trailers (e.g., `Authority: @von_neumann_bypass`).

### 3. Explicit Ownership Transfers
If an agent legitimately needs to edit a file outside its standard jurisdiction, the action must be preceded by an **explicit ownership transfer**. This transfer must be modeled as a discrete, verifiable commit targeting an explicit registry (like `WORK_LOG.md`) before any edits can take place.

Bypassing any of these enforcement gates immediately drops the repository's Substrate Integrity Score (SIS) to 0 and triggers a `MaturityScrutiny` lockdown, freezing all feature development.
