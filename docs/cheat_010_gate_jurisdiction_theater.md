# CHEAT-010: Gate-Jurisdiction Theater

## Definition
Under the Anti-Cheat Manifesto (**Rule 16** in `AGENTS.md`), **CHEAT-010 (Gate-jurisdiction theater)** is defined as:
> "Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target."

This constitutes a form of compliance evasion where a tool, scanner, or audit is run and reports a "pass," but its execution scope intentionally or accidentally excludes the actual authoritative code or environments that require verification. 

For instance, this includes:
- Running `cargo make scan-cheats` or `audit-object-code` but omitting newly modified authoritative source files from the scanner's path.
- Excluding generated code or macro expansions from the inspection.
- Running tests only on the default feature set while ignoring other supported features.
- Testing only on one target architecture when multiple are supported.

## Required Proof of Jurisdiction
As mandated by **Rule 23 (Required repository gates)**: 
> "A green command with incomplete jurisdiction is not evidence."

Before any gate result is accepted, it must be proven that the task's jurisdiction included the changed files. The report must explicitly state the command, exit status, files inspected, features inspected, targets inspected, findings, and artifact digest.

## Why Gate-Jurisdiction Omission Sets SIS to 0
In `bcinr`, the deterministic runtime is guaranteed entirely by mechanical, unbroken verification. The Substrate Integrity Score (SIS) is a maturity matrix metric. However, according to **Rule 24 (Substrate Integrity Score)**, certain acts are considered "absolute failures regardless of score." 

**Gate-jurisdiction omission** is listed as one of these absolute failures. 

When an absolute failure occurs:
1. It forces **`SIS = 0`** instantly. No weighted average of other passing tests can conceal this constitutional violation.
2. It triggers the **`MaturityScrutiny` protocol** (Rule 25), which mandates:
   - A complete freeze on feature development.
   - Quarantine of the affected code.
   - A root-cause report and repair of the structural defect.
   - Rerunning the complete gate matrix and issuing a new standing receipt.

Gate-jurisdiction theater is heavily penalized because it undermines `@turing_machine` (the Enforcer of Determinism). If a file or target is silently excluded from the gate's jurisdiction, unverified branching ($CC > 1$), allocations, or unsupported math could slip into the hot path undetected, completely breaking the fundamental deterministic guarantees of the substrate.
