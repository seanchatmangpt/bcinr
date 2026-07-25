### Structural Enforcement of Rule 27 (No Self-Certification) in BCINR

Rule 27 of the BCINR Constitution explicitly forbids the implementation agent (e.g., `@von_neumann_bypass`) from self-certifying their work on critical dimensions like mathematical correctness, mutant adequacy, object-code compliance, and branchlessness.

The core principle is clear: **"Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim."** Each approval must come from an independent role and be backed by a mechanical artifact.

The project enforces this mandate structurally through the following mechanisms:

#### 1. Mechanical Artifacts over Human Agreement
Approvals cannot be given merely via comments, textual reviews, or verbal consensus. They must take the form of concrete, reproducible mechanical evidence:
* An approval must be formalized as a specific artifact, such as an **audit record**, a **mutant-kill matrix**, disassembly logs, or object-code audit reports.
* These evidence records must be explicitly attributed to a constitutional role **distinct from the implementer** (e.g., `@turing_machine` for branchlessness, `@hoare_oracle` for mathematical correctness, or `@armstrong_fault` for hostile verification), with that role named explicitly in the record.

#### 2. CI Verification and Git Workflow Constraints
The separation of roles is structurally baked into the version control and CI/CD pipelines. This ensures that the implementation owner physically cannot bypass the rule:
* **Diff-to-Role Mapping:** The repository's CI gates (e.g., `bcinr-cheat-scanner`) verify that the modified files in a pull request strictly align with the domain mapped to the PR's author. The implementation owner literally cannot push a change to the hostile mutant files or the verification proofs without failing CI.
* **Git Trailers & Headers:** Commits must explicitly declare the authorized persona executing the change using structured commit headers (e.g., `Authority: @von_neumann_bypass`, `Jurisdiction: src/...`). CI uses these headers to strictly validate file ownership.
* **Conflict-Free Merges:** Any overlapping modifications across distinct domains immediately trigger a CI failure, forcing isolated responsibility. Lawful merges must be mathematically orthogonal.

#### 3. Strict State Isolation and Anti-Circular Verification
Rule 5 (Mandatory Decomposition) requires every nontrivial task to be split across strictly separated workstreams:
* `@von_neumann_bypass` writes the code.
* `@armstrong_fault` writes the hostile mutants.

If `@von_neumann_bypass` were allowed to certify `@armstrong_fault`'s mutants, they could approve weak mutants that their implementation can easily defeat (referred to as "Mutant Theater" or CHEAT-009). This structural separation prevents circular verification where tests are derived from the implementation rather than the absolute mathematical laws.

#### 4. The Work Log and Ownership Transfers
If an agent needs to step out of their assigned domain, they cannot simply open a PR. There must be an explicit ownership transfer recorded via a discrete, verifiable commit targeting an explicit registry (e.g., a `WORK_LOG.md`). CI checks for this unbroken Git ancestry before allowing edits outside an agent's default jurisdiction.

#### Summary
BCINR completely removes the ability to self-certify by replacing trust with **domain-segregated mechanical artifacts**, enforced by **CI/CD domain-mapping checks**, **Git trailers**, and **strict orthogonal merge rules** that physically block an implementation agent from generating or approving their own verification records.
