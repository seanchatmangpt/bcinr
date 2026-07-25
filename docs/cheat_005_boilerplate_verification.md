# CHEAT-005: Boilerplate Verification Claims in BCINR

According to the `bcinr` determinism substrate constitution (`AGENTS.md`), **CHEAT-005 (Boilerplate verification claims)** is an explicit anti-cheat violation. The constitution defines this cheat as:

> "Repeated comments asserting verification without a linked proof or receipt."

In `bcinr`, leaving comments like `// Verified branchless` or `// Mathematical contract satisfied` is fundamentally illegal if they are not directly linked to a mechanical artifact.

## Why is this an Anti-Cheat Violation?

The prohibition of boilerplate verification claims enforces BCINR's core philosophy: **"If a property cannot be stated precisely, it is not yet law."** Here is why repeated textual assertions without linked proof violate the substrate's laws:

### 1. Proof Artifacts Must Be Mechanical and Reproducible
`bcinr` mandates that qualitative human or agent assertions are legally meaningless. According to Rule 1, the repository *"does not accept implementations that merely appear correct in tests."* Every authoritative primitive requires reproducible, executable evidence, such as:
- A Hoare logic contract
- Object-code disassembly evidence
- Surviving hostile mutants
- A bit-vector solver certificate

A text comment cannot be executed, mutated, or audited by the `bcinr-cheat-scanner`. It bypasses the entire structural gate system.

### 2. The Ban on Self-Certification
Rule 27 explicitly forbids self-certification, stating:
> *"Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim."*

A boilerplate verification comment is the epitome of an unsupported claim. True verification requires the generation of mechanical artifacts signed by independent roles (e.g., `@hoare_oracle` for mathematical proof, `@turing_machine` for structural audits), not just a developer asserting that code works.

### 3. Substrate Integrity and "Verification Theater"
Under Rule 24 (Substrate Integrity Score), "fabricated verification evidence" is classified as an **absolute failure**. It instantly forces the Substrate Integrity Score (SIS) to `0` and triggers the `MaturityScrutiny` repository lockdown protocol. 

The `bcinr-cheat-scanner` actively hunts for CHEAT-005 to prevent "verification theater," ensuring that no unverified code can sneak into the deterministic hot path under the disguise of repetitive, empty assurances.
