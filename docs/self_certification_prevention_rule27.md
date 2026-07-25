# Structural Enforcement of Rule 27 (No Self-Certification) in BCINR

Rule 27 explicitly forbids the implementation agent (e.g., `@von_neumann_bypass`) from self-certifying their work on critical dimensions like mathematical correctness, mutant adequacy, and branchlessness. The rule dictates that agent agreement is not evidence; instead, each approval must come from an independent role and be backed by a mechanical artifact. 

Based on a review of the repository (including `AGENTS.md`, `.claude/rules/30-authority-separation.md`, `docs/write_ownership_and_git_workflow.md`, and `.claude/skills/mutant-kill-protocol/SKILL.md`), the project enforces this structurally through the following mechanisms:

## 1. Explicit Role Attribution and Mechanical Evidence
Approvals cannot be given merely via comments, text, or verbal agent agreement ("Five agents repeating the same claim is still one unsupported claim"). They must be logged as mechanical evidence.
* As stated in `30-authority-separation.md`, an approval must be formalized as an **audit or mutant-kill record** (e.g., `MUTANT_KILL_MATRIX.md`, disassembly logs, or object-code audit reports).
* These evidence records must be explicitly attributed to a constitutional role **distinct from the implementer** (e.g., `@turing_machine` for branchlessness, `@hoare_oracle` for mathematical correctness, or `@armstrong_fault` for hostile verification), with that distinction named explicitly in the record.

## 2. CI Verification and Git Workflow Constraints
The separation of roles is structurally baked into the version control and CI/CD pipelines to prevent self-certification bypasses (governed by the interaction of Rule 26 and 27):
* **Diff-to-Role Mapping:** The repository's CI gates (e.g., `bcinr-cheat-scanner` pre-merge hooks) verify that the output of `git diff --name-only` for a Pull Request strictly aligns with the domain mapped to the PR's author. The implementation owner literally cannot push a change to the hostile mutant files or the verification proofs without failing CI.
* **Git Trailers:** Commits must explicitly declare the authorized persona executing the change using structured commit headers or Git trailers (e.g., `Author: <Agent ID>`, `Authority: @von_neumann_bypass`, `Jurisdiction: src/...`). The CI uses these headers to validate file ownership.
* **Conflict-Free Merges:** Any overlapping modifications across domains immediately trigger a CI failure, forcing isolated responsibility. Lawful merges must be mathematically orthogonal or use strict `--ff-only`.

## 3. The Work Log / Ownership Transfer Gate
If an agent needs to step out of their domain, they cannot just open a PR. There must be an explicit ownership transfer recorded via a discrete, verifiable commit targeting an explicit registry (e.g., `WORK_LOG.md`). The CI checks for this unbroken Git ancestry before allowing edits outside an agent's default jurisdiction.

## Conclusion
BCINR structurally removes the ability to self-certify. It replaces trust with **domain-segregated mechanical artifacts**, enforced by **Git trailers, CI/CD domain-mapping checks**, and **strict orthogonal merge rules** that physically block an implementation agent from generating their own verification records.
