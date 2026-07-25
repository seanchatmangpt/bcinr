# BCINR Git Workflow: Implementing Rule 26 (State Isolation and Write Ownership)

This document dictates how Rule 26 of the BCINR Deterministic Substrate Constitution translates into version control mechanics. The overarching principle is that the prohibition of shared-file concurrent editing must be mathematically guaranteed by the Git commit structure and merge strategies.

## 1. Constitutional Premise

Rule 26 mandates:
*   **Exclusive Write Ownership:** Files belong strictly to designated agents (e.g., `@hoare_oracle` for proofs, `@von_neumann_bypass` for implementation).
*   **No Concurrent Editing:** Shared-file concurrent editing is strictly prohibited.
*   **Explicit Transfer:** Ownership transfer must be recorded in the work log before editing rights change.
*   **Generator Exclusivity:** Generated files may only be written by the admitted generator.

In version control terms, these laws supersede standard agile branching models. The Git history itself is part of the substrate's verification evidence and must be structurally flawless.

## 2. Commit Structure and Atomic Separation

To guarantee exclusive write ownership, commits must strictly adhere to the following rules:

### 2.1. Domain-Isolated Commits
A single Git commit **must never** cross ownership boundaries.
*   **Prohibited:** A single commit that updates both a structural proof (`src/proofs.rs`) and its underlying implementation (`src/hot_path.rs`).
*   **Required:** Separate commits, each attributed to the correct domain owner.

### 2.2. Explicit Role Attribution
Every commit must explicitly declare the authorized persona executing the change. This should be enforced via Git trailers or structured commit headers.
```text
Author: <Agent ID>
Authority: @von_neumann_bypass
Jurisdiction: src/arithmetic/
```

### 2.3. Generator Immutability
Commits containing generated code must be produced exclusively by the automation tooling. If a human or non-generator agent commits a manual tweak to a generated file, the repository immediately loses standing (SIS = 0).

## 3. Merge Strategies and The Zero-Conflict Law

Standard Git workflows often tolerate concurrent editing by relying on semantic merges (conflict resolution). Under Rule 26, **a Git merge conflict on any tracked file is physical proof that illegal concurrent editing occurred.** 

### 3.1. Prohibition of Conflict Resolution
If a feature branch produces a file-level merge conflict, the branch violates the constitution.
*   **Prohibited:** Manual or automated resolution of Git merge conflicts within a file.
*   **Required:** The conflicting branch must be quarantined. Work must be structurally partitioned or serialized to prevent the overlap from occurring in the first place.

### 3.2. Admitted Merge Strategies
Only two merge strategies are lawful within the BCINR repository:
1.  **Orthogonal Merges (Disjoint Diff Sets):** A standard merge commit is permitted *only if* the set of files modified by Branch A has an intersection of `Ø` (zero) with the files modified by Branch B.
2.  **Strict Fast-Forward (`--ff-only`):** For serialized, token-passing workflows on the same file, changes must be strictly appended sequentially via fast-forwarding, ensuring one strict timeline of ownership.

## 4. The Work Log and Ownership Transfer

Rule 26 states that agents may edit a file outside their default domain *only if* an explicit ownership transfer is recorded.

*   **The Git Translation:** Ownership transfers must be modeled as discrete, verifiable commits targeting an explicit registry (e.g., `WORK_LOG.md` or a centralized `.toml` registry).
*   **Pre-requisite Validation:** Before any agent commits a change to a file outside its standard jurisdiction, there must be an unbroken Git ancestry proving they currently hold the write lock for that path.
*   **Example Commit Message:**
    ```text
    TRANSFER: @von_neumann_bypass yields `src/hot_path.rs` to `@turing_machine`
    Reason: Structural audit repairs required.
    ```

## 5. Verification Gatekeepers (CI/CD Enforcement)

The `@turing_machine` jurisdiction extends to Git metadata enforcement. The `bcinr-cheat-scanner` or repository CI gates MUST implement pre-merge hooks that verify:
1.  **Diff-to-Role Mapping:** The output of `git diff --name-only` for a PR perfectly aligns with the domain mapped to the PR's author.
2.  **Conflict-Free History:** The merge strategy is `--ff-only` or mathematically orthogonal.
3.  **Lineage Check:** No commit retroactively edits a file owned by another agent without a valid transfer commit preceding it in the branch's topology.

> [!WARNING]
> Any violation of these Git structures is a constitutional violation. Bypassing CI to force a merge on a shared file instantly forces `MaturityScrutiny` and drops the Substrate Integrity Score to 0.
