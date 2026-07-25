# Agent Ownership Boundaries and State Isolation in BCINR

Based on the repository's constitution and architectural documentation (`docs/exclusive_write_ownership_law.md` and `docs/write_ownership_and_git_workflow.md`), **Rule 5 (Mandatory decomposition protocol)** and **Rule 26 (State isolation and write ownership)** mandate a strict physical and logical separation of concerns among agents.

## Ownership Boundaries (Rule 26 & Rule 5)

Rule 5 dictates that any nontrivial task must be decomposed into four independent workstreams. To prevent agents from self-certifying (e.g., an implementation agent writing its own mathematical oracle, or an auditor fixing the code they are auditing), Rule 26 enforces **exclusive write ownership** mapped to specific domains:

| Domain | Exclusive Writer | Responsibility |
| :--- | :--- | :--- |
| **Contracts and proofs** | `@hoare_oracle` | Preconditions, invariants, Hoare triples, independent reference semantics |
| **Scanners and structural gates**| `@turing_machine` | Source/object-code audit plans, CI/CD, Git metadata enforcement |
| **Mutants and hostile fixtures** | `@armstrong_fault` | Hostile testing, typed refusal expectations |
| **Authoritative implementation** | `@von_neumann_bypass`| Branchless bounded production code |

Shared-file concurrent editing across these domains is strictly prohibited. Generated files may only be written by the admitted generator.

## Enforcement Mechanisms

Exclusive write ownership is **NOT** enforced via `.github/CODEOWNERS` (the file does not exist in the repository). Instead, it is enforced via a combination of strict Git workflow structures, manual logs, and CI/CD pre-merge hooks under the jurisdiction of `@turing_machine`.

### 1. Manual Logs and Explicit Ownership Transfer
If an agent needs to edit a file outside its standard jurisdiction, it must be preceded by an explicit ownership transfer.
* **Work Logs:** Ownership transfers must be modeled as discrete, verifiable commits targeting an explicit registry (e.g., a `WORK_LOG.md` or centralized `.toml` registry).
* **Commit Attribution:** Every commit must explicitly declare the authorized persona executing the change via Git trailers (e.g., `Authority: @von_neumann_bypass` and `Author: <Agent ID>`).

### 2. Strict Git Commits and Merge Strategies
* **Domain-Isolated Commits:** A single Git commit must never cross ownership boundaries (e.g., updating both a structural proof and its hot-path implementation in one commit is prohibited).
* **The Zero-Conflict Law:** A Git merge conflict on any tracked file is considered physical proof that illegal concurrent editing occurred. 
* Only **orthogonal merges** (disjoint diff sets) or **strict fast-forward (`--ff-only`)** merges are permitted.

### 3. CI/CD Gatekeepers (bcinr-cheat-scanner)
Pre-merge hooks in the CI/CD pipeline physically enforce the rules by checking Git metadata:
* **Diff-to-Role Mapping:** Verifies that the files modified by a PR strictly align with the domain mapped to the PR author's declared role.
* **Lineage Check:** Ensures no commit retroactively edits a file owned by another agent without a valid transfer commit preceding it in the branch's topology.

Bypassing these gates immediately drops the repository's Substrate Integrity Score (SIS) to 0 and triggers a `MaturityScrutiny` lockdown.
