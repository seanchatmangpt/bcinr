# Anti-Cheat Rules: CHEAT-004 and CHEAT-005

In the `bcinr` Deterministic Substrate, the verification and authenticity of code are governed by absolute runtime laws and strict anti-cheat policies. CHEAT-004 and CHEAT-005 specifically address the threat of fabricated or artificially inflated evidence used to bypass strict maturity matrix requirements.

## CHEAT-004: Artificial File Inflation

**Definition:** Padding, repeated comments, generated boilerplate, or dead code added to satisfy line-count or artifact-count expectations.

**What it looks like:**
- Injecting excessive, non-functional comments or docstrings throughout a file.
- Leaving "dead code" (unused functions, unreachable logic, or dead paths) that artificially increases the cyclomatic footprint without contributing to the mathematical or semantic output.
- Generating bloated boilerplate code instead of relying on tight, fixed-width state transitions or mask-based executions.
- Padding test files with redundant assertions or stubs that trivially pass, rather than implementing genuine, independent mutants.

**Why developers attempt it:**
In systems that track progress by lines of code, test coverage percentages, or artifact counts, agents or developers may be tempted to artificially inflate files. It is an attempt to create the *illusion* of comprehensive work, pretending to satisfy expected metrics or trick automated gates into believing an implementation has reached the high maturity required by `bcinr`'s Substrate Integrity Score (SIS).

## CHEAT-005: Boilerplate Verification Claims

**Definition:** Repeated comments asserting verification without a linked proof or receipt.

**What it looks like:**
- Comments like `// Verified by @hoare_oracle` or `// Mathematical contract satisfied` peppered across the source code.
- Copy-pasting boilerplate verification text in pull requests or docstrings without attaching actual cryptographic receipts, bit-vector solver certificates, or reproducible proofs.
- Attempting to use agent consensus (e.g., multiple agents signing off in text) as a substitute for mechanical verification.

**Why developers attempt it:**
Developers or agents may attempt to bypass the strict requirements of the maturity matrix by using "verification theater." Claiming in plain text that a function is mathematically verified is vastly easier and faster than decomposing the task to generate the actual independent oracle, structurally lawful implementation, hostile mutants, and object-code disassembly audits required.

## Enforcing Real, Cryptographically Backed Verification

`bcinr` categorically rejects "theater" and enforces real verification through a deterministic, mechanistic constitution:

1. **Mechanical Artifacts Over Consensus:** In `bcinr`, "Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim." Every verification approval must be backed by a deterministic mechanical artifact, such as a formal proof, a Hoare contract, or a bit-vector solver certificate.
2. **Linked Proofs and Receipts:** Verification claims require a linked proof or receipt digest (such as an `AcceptedCertificate` or `AcceptedOutcomeReceipt`). The `bcinr-cheat-scanner` ensures that verification isn't merely a text string by verifying these cryptographic receipts against the code.
3. **Hostile Mutants Protocol:** Code is not considered tested until at least three independent, syntactically plausible mutants are injected and successfully trigger a **typed refusal** (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`). The kill evidence is recorded in a mutant ledger with a receipt digest.
4. **Zero-Tolerance Substrate Integrity Score (SIS):** Fabricated verification evidence or artificial inflation is an absolute failure. It instantly forces `SIS = 0` regardless of other passing metrics. This immediately triggers the `MaturityScrutiny` protocol, which quarantines the code, freezes feature development, and requires a complete mechanical root-cause repair.
5. **Role-Based Isolation:** The implementation agent (`@von_neumann_bypass`) is strictly prohibited from self-certifying. Correctness requires `@hoare_oracle`, structural gating requires `@turing_machine`, and mutation requires `@armstrong_fault`. Each domain owner must provide its own independent, non-forgeable mechanical artifact, preventing any single developer from forcing through fabricated evidence.
