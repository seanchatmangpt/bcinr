# Rule 23: Required Repository Gates

In the `bcinr` deterministic substrate, **Rule 23** establishes the absolute minimum mechanical verification threshold that any change must pass before being considered valid. The authoritative runtime's integrity is guaranteed entirely by an unbroken matrix of these gating checks. 

At a minimum, the repository requires executing the admitted equivalents of the following six gates:

### 1. `cargo make scan-cheats`
Executes the `bcinr-cheat-scanner` to parse the full syntax tree and inspect source code, macro expansions, and generated Rust for algorithmic anti-patterns. It enforces Rule 16's Anti-Cheat Manifesto, ensuring there is no scanner evasion, dead-path compliance, magic constants, or other prohibited shortcuts (CHEAT-001 through CHEAT-010). 

### 2. `cargo make contract-gate`
Validates branchless mathematical contract compliance. It acts as the Enforcer (`@turing_machine`), ensuring that every authoritative function strictly adheres to the Radon Law (Cyclomatic Complexity $CC=1$). Logic must be expressed as bitwise polynomials, without hidden branches, panic paths, or data-dependent loops.

### 3. `cargo make ci`
A comprehensive continuous integration pipeline that aggregates essential checks. This includes code formatting (`fmt`), strict linting with `clippy` (including scans for LLM-generated stubs), vulnerability and license checks (`audit` and `deny`), and running standard tests across all features and targets.

### 4. `cargo make test-mutants`
Implements the Hostile Mutation Protocol (Rule 19) managed by `@armstrong_fault`. It verifies that adversarial mutants injected at compile-time are successfully detected and killed by their dedicated, isolated oracle tests, proving that broken logic results in typed refusals rather than silent failures.

### 5. `cargo make audit-object-code`
Disassembles the release artifacts into a raw dump for object-code audit (Rule 20). Because source-level $CC=1$ is necessary but mathematically insufficient, this gate provides the reproducible assembly evidence required to classify per-symbol behavior, proving that the hot path contains no conditional jumps, loop backedges, or allocator calls.

### 6. `cargo make verify-generated`
Enforces the Generated-Code Law (Rule 21). It recomputes hashes (like BLAKE3) over committed payloads (e.g., `cmca_generated.rs`) and checks them against their declared manifests. This strictly verifies that the generated authoritative code is reproducible, matches the expected schema, and hasn't been hand-edited, doing so without invoking external generators or network dependencies.

---

## The Necessity of Proving Complete Jurisdiction

A core tenet of Rule 23 is that **"A green command with incomplete jurisdiction is not evidence."**

Proving complete jurisdiction means explicitly demonstrating that a gating task actually inspected the relevant changed files, in the correct feature configurations, and for the necessary compilation targets. This is strictly required for a gate report to be considered valid evidence for the following reasons:

1. **Preventing "CHEAT-010: Gate-Jurisdiction Theater"**
   Reporting a passing scanner that omitted the actual authoritative code or generated output is defined as compliance evasion (CHEAT-010). For instance, running a scanner without a necessary `--workspace` flag might yield a successful exit code, but silently skip newly modified crates or files. 

2. **Ensuring Holistic Enforcement**
   The deterministic guarantees (Radon Law $CC=1$, Zero-Allocation) rely on whole-call-graph branchlessness. If a private function, macro, or generated file falls outside a gate's jurisdiction, hidden branches or allocations could leak into the hot path undetected.

3. **Substrate Integrity Score (SIS) Protection**
   According to Rule 24, a "gate-jurisdiction omission" is an absolute failure condition. Regardless of mathematical proofs or test coverage, failing to prove complete jurisdiction instantly forces the Substrate Integrity Score to $SIS = 0$ and triggers a strict `MaturityScrutiny` lockdown (quarantine of affected code and a feature freeze).

To prove jurisdiction and avoid theater, Rule 23 mandates that every final evidence report must explicitly state:
- `command`
- `exit status`
- `files inspected`
- `features inspected`
- `targets inspected`
- `findings`
- `artifact digest`
