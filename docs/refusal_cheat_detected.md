# The `CheatDetected` Typed Refusal in BCINR

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`), `CheatDetected` is a critical enforcement mechanism and bounded typed refusal category. It acts as an absolute barrier against attempts to bypass verification gates or violate the codebase's strict structural laws.

Currently, it exists primarily as a conceptual CI gate and constitutional requirement; it has not yet been fully implemented as a runtime Rust enum variant in crates like `bcinr-cmca` or `bcinr-api`.

## 1. How it Triggers at Build-Time (CI Gate)
During the gate jurisdiction process (initiated via `cargo make scan-cheats`), the `bcinr-cheat-scanner` analyzes the full syntax tree—including private functions, macro expansions, generated Rust code, and test references. It triggers when it finds any of the ten specific violations defined in the **Anti-Cheat Manifesto**:

*   **CHEAT-001 (Self-canceling operations)**: Adding useless operations (e.g., `a.wrapping_add(b) ^ a`) to create apparent complexity.
*   **CHEAT-002 (Circular oracle)**: Copying production implementations to serve as independent references.
*   **CHEAT-003 (Magic constants)**: Using unexplained literals (e.g., `0xDEADBEEF`) that control production behavior.
*   **CHEAT-004 (Artificial file inflation)**: Padding files with dead code, generated boilerplate, or repeated comments to hit artifact expectations.
*   **CHEAT-005 (Boilerplate verification claims)**: Asserting verification in comments without linked proof or receipt artifacts.
*   **CHEAT-006 (Scanner evasion)**: Hiding prohibited operations using macros, string formatting, token splitting, or private wrappers.
*   **CHEAT-007 (Dead-path compliance)**: Providing structurally lawful code that is never actually executed on the hot path.
*   **CHEAT-008 (Benchmark theater)**: Benchmarking stubs, dead paths, constant-folded paths, or reduced problems not matching production constraints.
*   **CHEAT-009 (Mutant theater)**: Creating non-viable or trivial mutants that only fail via generic assertions rather than testing strict contract refusals.
*   **CHEAT-010 (Gate-jurisdiction theater)**: Running a passing scanner but intentionally excluding the relevant target, feature set, generated output, or specific crate.

**Consequences of a Trigger:**
When a violation is detected, the scanner outputs a finding using the exact format `CHEAT[rule-id]` (e.g., `CHEAT[CHEAT-006]`). This immediately:
- Blocks the merge (there are no warning-level cheat violations).
- Drops the project's Substrate Integrity Score (SIS) to `0` instantly, bypassing any weighted averages.
- Forces the repository into a `MaturityScrutiny` lockdown, freezing feature development until the defect is repaired and all gates pass again.

## 2. How it Triggers at Runtime (Intended Enforcement)
Rule 18 ("Typed Refusals") mandates `CheatDetected` as a required typed refusal category. If a hostile mutation or unverified execution attempts a cheat, the authoritative runtime must cleanly abort the operation by deterministically returning the `CheatDetected` refusal.

To adhere to the codebase's strict branchless and allocation-free laws (Rule 9 and Rule 10), it would be implemented via **full-width masks**:
- A "cheat detected" predicate evaluates to a mask.
- The runtime uses masked state selection to refuse the state mutation.
- The state is preserved bit-for-bit exactly as it was, rather than using branching control flow (like `if cheat { return Err(...) }`), panics, or fallbacks.
