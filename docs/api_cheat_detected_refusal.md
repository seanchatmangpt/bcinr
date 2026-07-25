I have used `grep_search` to look for `CheatDetected` within both `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/`.

### Findings

`CheatDetected` does **not** currently appear in the source code of either `crates/bcinr-cmca/src/` or `crates/bcinr-api/src/`.

Based on the project's documentation (specifically `AGENTS.md`, `docs/cheat_detected_refusal.md`, and `docs/api_audit_cheat_refusal.md`), `CheatDetected` is currently a **purely conceptual CI gate and constitutional requirement** that has not yet been fully implemented as an actual runtime typed refusal enum/code in these crates.

### Concept and Intended Implementation

While absent from the Rust source code, the documentation heavily defines how `CheatDetected` is intended to function both at build-time and runtime:

1. **Build-Time CI Gate (`bcinr-cheat-scanner`)**
   It is fundamentally defined by the "Anti-Cheat Manifesto" (Rule 16 in `AGENTS.md`), which catalogs violations `CHEAT-001` through `CHEAT-010`. During the CI gate process (`cargo make scan-cheats`), the `bcinr-cheat-scanner` analyzes the full syntax tree (including private functions, macro expansions, generated code, and test references) to find structural violations such as:
   - Self-canceling operations (CHEAT-001)
   - Circular oracles (CHEAT-002)
   - Magic constants (CHEAT-003)
   - Scanner evasion tactics (CHEAT-006)
   - Gate-jurisdiction theater (CHEAT-010)

   If any of these are found, the scanner outputs a finding (e.g., `CHEAT[CHEAT-006]`). This immediately blocks the merge, drops the Substrate Integrity Score (SIS) to `0`, and forces the repository into a `MaturityScrutiny` lockdown.

2. **Intended Branchless Runtime Enforcement**
   Rule 18 ("Typed Refusals") of `AGENTS.md` mandates `CheatDetected` as a required typed refusal category. The documentation (`docs/api_audit_cheat_refusal.md`) states that at runtime, it is meant to act as a deterministic circuit breaker. If a hostile mutation or unverified execution attempts a cheat, the authoritative runtime must cleanly abort the operation by deterministically returning the `CheatDetected` refusal. 

   To do this branchlessly (as per Rule 9 and Rule 10), it would be implemented via full-width masks. A "cheat detected" predicate would evaluate to a mask, and the runtime would use masked state selection to refuse the state mutation, preserving the exact bit-for-bit previous state, rather than using branching control flow (like `if cheat { return Err(CheatDetected) }`), panics, or fallbacks.
