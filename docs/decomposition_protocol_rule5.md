# Mandatory Decomposition Protocol (Rule 5)

In the BCINR Deterministic Substrate Constitution, **Rule 5 (Mandatory decomposition protocol)** mandates that every nontrivial implementation task must be immediately decomposed into four strictly independent workstreams. This segregation of duties ensures that code is structurally sound, mathematically proven, and free from confirmation bias or self-certifying logic.

## The Four Independent Workstreams

Each workstream is exclusively owned by a specialized agent role, establishing clear boundaries:

1. **Mathematical Law (`@hoare_oracle`)**
   - **Role:** Axiomatic proof lead and specification owner.
   - **Outputs:** Contracts and proof obligations.
   - **Duties:** Defines preconditions, postconditions, invariants, and independent reference semantics. They author the Hoare contracts and verify the domain using full-domain mathematical proof or solver certification.

2. **Structural Enforcement (`@turing_machine`)**
   - **Role:** Structural auditor and merge gatekeeper.
   - **Outputs:** Source and object-code audit plan.
   - **Duties:** Enforces the absolute runtime laws (`CC=1`, zero allocations). They audit both source and disassembled object-code to ensure no panic paths, branches, or runtime loop backedges exist.

3. **Hostile Verification (`@armstrong_fault`)**
   - **Role:** Adversarial test architect and mutation owner.
   - **Outputs:** Mutants and refusal expectations.
   - **Duties:** Designs counterfactual mutants and hostile fixtures to test negative domains. They verify that corrupted implementations trigger explicit typed refusals rather than just unequal results.

4. **Implementation (`@von_neumann_bypass`)**
   - **Role:** Authoritative implementation owner.
   - **Outputs:** Branchless bounded code.
   - **Duties:** Implements branchless arithmetic logic, bit-parallel masks, and fixed lookup tables. They transform sequential semantic decisions into straight-line, fixed-width code.

## The Ban on Self-Certification

To preserve the absolute integrity of the substrate, self-certification is explicitly banned (Rules 5 and 27):
- **No Self-Certifying Oracles:** An implementer (`@von_neumann_bypass`) cannot write their own tests or self-certify equivalence. Oracles must be structurally and logically distinct.
- **No Silent Repairs:** A structural auditor (`@turing_machine`) cannot silently repair code and approve their own fix. If an auditor edits the code, they lose their independence and become the implementer.
- **No Derivative Expectations:** The mutation agent (`@armstrong_fault`) cannot derive expected results from the implementation currently under attack.

---

## Repository Workflow Enforcement

The segregation of duties is mathematically guaranteed and mechanically enforced in the Git repository via strict version control structures, CI gates, and specialized tooling.

### 1. Commit Structure and Git Trailers
Every commit must be domain-isolated and explicitly declare the authorized persona performing the change. A single commit **must never cross ownership boundaries** (e.g., updating both a structural proof and its underlying implementation).

This is strictly enforced via Git trailers or structured commit headers:
```text
Author: <Agent ID>
Authority: @von_neumann_bypass
Jurisdiction: src/arithmetic/
```

### 2. State Isolation and Write Ownership Transfer
Files belong strictly to designated agents. Agents may only edit a file outside their default domain if an explicit **ownership transfer** is recorded as a verifiable transfer commit in the Git ancestry:
```text
TRANSFER: @von_neumann_bypass yields `src/hot_path.rs` to `@turing_machine`
Reason: Structural audit repairs required.
```
Shared-file concurrent editing is physically prohibited by these constraints.

### 3. The Zero-Conflict Law
In the BCINR repository, a Git merge conflict is considered physical proof that illegal concurrent editing occurred. 
- Manual or automated resolution of merge conflicts is **prohibited**. 
- Only two merge strategies are lawful:
  1. **Strict Fast-Forward (`--ff-only`)**: Ensures one sequential timeline of ownership.
  2. **Orthogonal Merges**: A merge commit is only permitted if the set of files modified has an intersection of `Ø` (disjoint diff sets).

### 4. CI Gates and Verification Gatekeepers
The `@turing_machine` jurisdiction extends to Git metadata enforcement through repository CI gates (pre-merge hooks):
- **Diff-to-Role Mapping:** The output of `git diff --name-only` must perfectly align with the domain mapped to the PR's author.
- **Lineage Check:** CI verifies that no commit retroactively edits a file owned by another agent without a valid transfer commit preceding it in the branch topology.
- **Automated Validation Scripts:** The CI relies on tasks executed via `cargo make` (e.g., `cargo make scan-cheats` via `bcinr-cheat-scanner`, and `cargo make contract-gate` via `bcinr-contract-gate`) to validate branchless contract compliance, enforce structural complexity constraints, and detect cheat anti-patterns.
