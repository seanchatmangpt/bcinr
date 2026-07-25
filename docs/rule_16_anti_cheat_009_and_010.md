Here are the details from the **Anti-cheat manifesto (Rule 16)** in `AGENTS.md`, specifically regarding CHEAT-009 and CHEAT-010:

### CHEAT-009 — Mutant theater
**Definition:** Creating mutants that cannot compile, are trivially different, or are detected only by `assert_ne!`.

*Context (from Rule 4/19):* The manifesto prohibits this because every authoritative implementation file must have at least three syntactically plausible mutants that alter a meaningful law (like sign inversion, dropped factor, incorrect mask). A proper mutant test must prove that the corrupted implementation triggers a *typed refusal* (e.g., `assert_eq!(result, Err(...))`) or violates a specific mathematical contract, rather than just returning "something different" (which `assert_ne!` tests for). 

### CHEAT-010 — Gate-jurisdiction theater
**Definition:** Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target.

*Context (from Rule 23):* This emphasizes that running a command that succeeds (e.g., a green build or pass) is not valid evidence unless you can prove its jurisdiction encompasses the changed files. The audit must explicitly inspect all requested features, targets, and generated output, and reporting a "pass" on a limited scope to simulate full compliance is prohibited.
