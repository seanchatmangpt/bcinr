### Rule 26 (Write Ownership) Enforcement in `bcinr`

According to the repository's internal documentation (`docs/write_ownership_isolation_rule26.md` and `docs/write_ownership_and_git_workflow.md`), **Rule 26 is NOT enforced using standard `.github/CODEOWNERS`** (the file does not exist in the repository). 

Instead, the exclusive write ownership model—which strictly partitions domains among `@hoare_oracle`, `@turing_machine`, `@armstrong_fault`, and `@von_neumann_bypass`—is structurally guaranteed via a combination of Git workflow rules, commit metadata, and CI/CD gatekeeping under the jurisdiction of the `@turing_machine` agent.

#### 1. Commit Structure & Role Attribution
Every single commit must definitively prove which authorized persona is executing the change:
* **Domain-Isolated Commits:** A single commit must never cross ownership boundaries (e.g., updating an axiomatic proof and its hot-path implementation in the same commit is forbidden).
* **Git Trailers:** Every commit must explicitly declare the authorized persona via trailers (e.g., `Authority: @von_neumann_bypass` and `Author: <Agent ID>`).
* **Explicit Ownership Transfers:** If an agent must edit a file outside its standard jurisdiction, the transfer must be recorded as a discrete commit targeting an explicit registry (like `WORK_LOG.md`), proving an unbroken Git ancestry of write-lock transfer.

#### 2. The Zero-Conflict Law & Merge Strategies
Standard agile branching models that rely on semantic conflict resolution are banned. In this substrate, **a Git merge conflict on any tracked file is physical proof that illegal concurrent editing occurred.**
Only two merge strategies are lawful:
* **Orthogonal Merges:** Permitted only if the modified files in Branch A and Branch B have an intersection of `Ø` (zero overlapping files, i.e., disjoint diff sets).
* **Strict Fast-Forward (`--ff-only`):** For serialized workflows on the same file, changes must be strictly appended sequentially.

#### 3. CI/CD Gatekeepers (`bcinr-cheat-scanner`)
Pre-merge hooks in the CI/CD pipeline physically enforce the rules by checking Git metadata. The gatekeepers verify:
* **Diff-to-Role Mapping:** The output of `git diff --name-only` for a Pull Request strictly aligns with the domain mapped to the PR author's declared role.
* **Lineage Check:** Ensures no commit retroactively edits a file owned by another agent without a valid transfer commit preceding it in the branch's topology.
* **Generator Immutability:** Commits containing generated code must be produced exclusively by admitted automation tooling.

Any violation of these structures—such as bypassing CI to force a merge on a shared file—is treated as a constitutional failure, instantly dropping the Substrate Integrity Score (SIS) to 0 and forcing a `MaturityScrutiny` lockdown.
