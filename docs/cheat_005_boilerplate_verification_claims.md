# CHEAT-005: Boilerplate Verification Claims

The **Anti-Cheat Manifesto** (Rule 16) in BCINR strictly prohibits `CHEAT-005` (Boilerplate verification claims). This rule ensures that the mathematical standing of the deterministic substrate is not diluted by empty, repeated text comments pretending to be proofs.

## How the Scanner Enforces CHEAT-005

At the static analysis layer, `bcinr-cheat-scanner` (the structural auditor tool) implements a text-based scan designed to catch mock verification claims.

- **Detection Mechanism**: The scanner reads the source code line-by-line and counts specific, repetitive assertion strings. It specifically searches for lines containing both `"Hoare-logic Verification Line"` and `"Branchless path is the unique solution to the state constraints of"`.
- **Threshold**: If it detects 5 or more of these identical boilerplate comments in a single file, it flags a `CHEAT-005` violation (`"mock Hoare-logic verification claims detected"`) and fails the build.
- **Remediation**: The scanner instructs the author to *"Provide real axiomatic proofs or remove the mock comments."*

## How the Architecture Verifies True Proofs

Beyond just scanning for boilerplate padding, the broader BCINR architecture (defined in the `AGENTS.md` constitution) mandates structural verification to prove that a claim is mathematically backed:

1. **Independent Oracle Law (Rule 15)**: A true verification claim must be backed by an independent mathematical specification (the "oracle"). This cannot be a copy of the production code. It must take the form of a direct mathematical formula, an abstract state machine, a symbolic proof, an arbitrary-precision implementation, a SAT/SMT bit-vector model, or an exhaustive reduced-domain enumerator.
2. **Full-Domain Requirement (Rule 4)**: The `@hoare_oracle` agent must produce a Hoare contract for every primitive. This contract must be proven over the full domain using formal proofs, solver certificates, or exhaustive bounded theorem artifacts—not random testing or textual assertions.
3. **Hostile Mutants (Rule 19)**: Verification claims are tested mechanically by `@armstrong_fault`. A proof is only considered valid if injecting a syntactically plausible mutant causes the test suite to fail with a **typed refusal code** or an exact oracle mismatch explicitly tied to the violated postcondition, rather than a generic `assert_ne!`.
4. **Mechanical Artifacts**: The constitution requires all approvals to be backed by mechanical artifacts (proof obligations, receipt digests, and disassembly audits) rather than agent agreement or text comments (Rule 16. CHEAT-005).

By combining the syntactic rejection of repetitive boilerplate text (via `CHEAT-005`) with strict, execution-based oracle and mutation gates, the substrate guarantees that "verified" means mechanically proven via artifacts, not just asserted in comments.
