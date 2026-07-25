# CI Enforcement of MaturityScrutiny Protocol

The `MaturityScrutiny` protocol is structurally embedded in the repository's deterministic constitution (`AGENTS.md` and `docs/sis_maturity_protocol.md`) and is stringently enforced by the continuous integration (CI) pipeline whenever the Substrate Integrity Score (SIS) falls below 100.

## 1. Substrate Integrity Score (SIS) and Absolute Failures

The SIS mathematically reflects the completeness of axiomatic proofs, behavioral oracles, mutation hostility, and determinism. While the score is generally a weighted average of violations, the presence of any **Absolute Failure** immediately forces **`SIS = 0`**, blocking any merge and invoking the `MaturityScrutiny` protocol. Absolute failures include:
- Hidden authoritative branches (JCC, panics, loop backedges)
- Hot-path allocations
- Unwitnessed state mutations
- Surviving adversarial mutants
- Circular oracles, fabricated evidence, or scanner evasions

## 2. CI Pipeline Triggers and Enforcement

GitHub Actions (`.github/workflows/ci.yml`) enforces these constraints by executing the `cargo make ci` target defined in `Makefile.toml`. The pipeline strictly halts merges on defects via these specific gates:
- **`contract-gate`**: Validates the deterministic, branchless `CC=1` contract compliance of authoritative functions.
- **`scan-cheats`**: Uses the `bcinr-cheat-scanner` to detect scanner evasion, JCC loops, magic constants, or hidden allocations. 
- **`test-mutants`**: Validates the hostile mutant matrix by strictly verifying that every injected mutation (`mutant_1`, `mutant_2`, etc.) is killed by its independent oracle. Fails if a mutant survives.
- **`audit-object-code`**: Disassembles binary artifacts (e.g., `libbcinr_cmca`) to inspect the true release object code for conditional jumps, allocations, and panic paths.

Any non-zero exit code from these Makefile targets results in an unbypassable CI failure. 

## 3. The 9-Step MaturityScrutiny Protocol

When an absolute failure triggers `MaturityScrutiny`, developers/agents are expressly forbidden from working around the failure by moving the feature elsewhere. A strict 9-step remediation process applies:

1. **Freeze feature development:** All feature work is halted across the tree.
2. **Quarantine affected code:** Unverified or structurally non-compliant logic must be physically isolated.
3. **Identify all reachable authoritative symbols:** Trace the transitive call graph for the scope of the violation.
4. **Rerun proofs, scans, mutants, and disassembly:** Establish baseline reproducible evidence.
5. **Produce a root-cause report:** Document the breakdown in the axioms.
6. **Repair the structural defect:** Remediate the breach via branchless fixed-point mathematics and full-width bit masks.
7. **Regenerate all dependent artifacts:** Rebuild dependencies, manifests, and signatures.
8. **Rerun the complete gate matrix:** Execute `cargo make ci` from scratch to verify 0 remaining failures.
9. **Issue a new standing receipt:** Publish a fresh, signed receipt proving compliance (e.g., regenerating `PACKAGE_REALITY_RECEIPT.md`).

## 4. Feature Freezing and Code Quarantining

- **Feature Freeze:** The repository mandates exclusive domain ownership. A structural defect gives the `@turing_machine` (structural enforcer) exclusive write access to address the problem, suspending feature velocity in favor of deterministic soundness. The process requires a total freeze of other activities until `cargo make ci` passes.
- **Code Quarantining:** Code that breaks core runtime axioms is not merely `#cfg`-gated; it is moved into explicit quarantine boundaries. For example, in the `bcinr-cmca` crate, an RDF generator component that breached architectural boundary rules was relocated to `crates/bcinr-cmca/quarantine/legacy-generator/`. The CI tasks were then re-engineered to verify artifacts (like digests) strictly *without* invoking the quarantined code. Code in quarantine cannot pollute the hot path or be executed by the primary gates.
