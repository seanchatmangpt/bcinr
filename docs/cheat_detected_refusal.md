# The `CheatDetected` Typed Refusal in BCINR

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the `CheatDetected` typed refusal is a critical enforcement mechanism designed to uphold the strict structural laws of the codebase. It acts as an absolute barrier against attempts to bypass verification gates.

## 1. The Anti-Cheat Manifesto (`CHEAT-001` through `CHEAT-010`)
The constitution identifies ten specific violations that compromise the integrity of the authoritative code:

*   **CHEAT-001 (Self-canceling operations)**: Adding useless operations (e.g., `a.wrapping_add(b) ^ a`) just to create apparent complexity.
*   **CHEAT-002 (Circular oracle)**: Copying production implementations to serve as independent references.
*   **CHEAT-003 (Magic constants)**: Using unexplained literals (e.g., `0xDEADBEEF`) that control production behavior.
*   **CHEAT-004 (Artificial file inflation)**: Padding files with dead code, generated boilerplate, or repeated comments to hit artifact expectations.
*   **CHEAT-005 (Boilerplate verification claims)**: Asserting verification in comments without linked proof or receipt artifacts.
*   **CHEAT-006 (Scanner evasion)**: Hiding prohibited operations using macros, string formatting, token splitting, or private wrappers.
*   **CHEAT-007 (Dead-path compliance)**: Providing structurally lawful code that is never actually executed on the hot path.
*   **CHEAT-008 (Benchmark theater)**: Benchmarking stubs, dead paths, constant-folded paths, or reduced problems not matching production constraints.
*   **CHEAT-009 (Mutant theater)**: Creating non-viable or trivial mutants that only fail via generic assertions (e.g., `assert_ne!`) rather than testing strict contract refusals.
*   **CHEAT-010 (Gate-jurisdiction theater)**: Running a passing scanner but intentionally excluding the relevant target, feature set, generated output, or specific crate.

## 2. Detection by the `bcinr-cheat-scanner`
During the gate jurisdiction process (initiated via `cargo make scan-cheats`), the `bcinr-cheat-scanner` analyzes the full syntax tree, including private functions, macro expansions, generated Rust code, and test references. 

When any of the ten violations are detected, the scanner outputs a finding using the exact format `CHEAT[rule-id]`, along with the specific file and span (e.g., `CHEAT[CHEAT-006]: prohibited operator hidden in macro expansion`).

## 3. Aggregation into `CheatDetected` and Pipeline Blocking
The integration of these findings during the gate jurisdiction process directly blocks the pipeline:

1.  **Strict Jurisdiction Audits**: The scanner is required to prove its jurisdiction included all changed files across the full feature matrix. A passing command with an incomplete jurisdiction is considered invalid evidence.
2.  **Zero Warnings, Absolute Blocking**: There are no warning-level cheat violations. *Every* single finding automatically blocks the merge. Baseline suppressions are explicitly banned without a separately admitted waiver artifact.
3.  **Typed Refusals Requirement**: In the context of the authoritative runtime and hostile mutation protocol, any detected cheat attempt must map directly to the `CheatDetected` bounded typed refusal code. The runtime is strictly prohibited from panicking, silently correcting the input, or falling back to a simpler algorithm. 
4.  **SIS Collapse**: The presence of severe violations (like scanner evasion, fabricated verification evidence, or gate-jurisdiction omission) triggers an absolute failure. The project's Substrate Integrity Score (SIS) is instantly set to `0`, bypassing any weighted averages.
5.  **Maturity Scrutiny Protocol**: Dropping the SIS to 0 forces the repository into a `MaturityScrutiny` lockdown. Feature development is completely frozen, the affected code is quarantined, and the pipeline remains locked until the structural defect is repaired, dependent artifacts are regenerated, and all gates successfully pass again.
