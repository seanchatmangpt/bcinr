### Analysis of `ObjectCodeAuditFailed` and `CheatDetected` Refusals

These typed refusal codes do not currently appear in the source code of the main crate or the cheat scanner tool. They were only found in the `AGENTS.md` constitution and the project's documentation files (e.g., `docs/cheat_detected_refusal.md`, `docs/object_code_audit_failed_refusal.md`).

#### Enforcement: Runtime vs. Build-Time
Based on the documentation, these typed refusals operate as **both** strict build-time gate failures and runtime circuit breakers:

1. **Build-Time Gate Failures (Pipeline Blocking)**
   - **`CheatDetected`**: Triggered by `bcinr-cheat-scanner` during `cargo make scan-cheats`. Any structural violation (like scanner evasion or magic constants) automatically blocks the merge without any warnings.
   - **`ObjectCodeAuditFailed`**: Triggered during `cargo make audit-object-code` if the production-profile disassembly reveals any hidden conditional jumps, loop backedges, or panics.
   - For both, failing the static gate instantly drops the Substrate Integrity Score (SIS) to `0` and forces the repository into a `MaturityScrutiny` lockdown, completely freezing feature development until the defect is repaired and artifacts regenerated.

2. **Runtime Enforcement (Deterministic Circuit Breakers)**
   - **`ObjectCodeAuditFailed`**: Acts as a physical security perimeter at runtime. If the compiled binary lacks a valid `@turing_machine` assembly verification manifest or certificate, the operation is fundamentally rejected. The admission mask deterministically evaluates to `0` (Rule 10), physically preventing unverified logic from mutating the persistent state. Additionally, without an accepted certificate, the `ReceiptSound` law (Rule 11) fails, completely freezing the adaptive learning mode.
   - **`CheatDetected`**: Must be explicitly mapped to a bounded typed refusal code by the runtime. If a hostile mutation or unverified execution attempts a cheat, the runtime is strictly prohibited from panicking, silently correcting the input, or falling back to a simpler algorithm—it must deterministically return the `CheatDetected` refusal to cleanly abort the operation.
