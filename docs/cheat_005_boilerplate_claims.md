# CHEAT-005: Boilerplate Verification Claims

The **Anti-Cheat Manifesto** (Rule 16) in the BCINR constitution strictly prohibits `CHEAT-005` (Boilerplate verification claims). This rule ensures that the mathematical standing of the deterministic substrate is not diluted by empty, repeated text comments pretending to be proofs.

## Why Boilerplate Claims Are Banned

In BCINR's rigorous environment, human assertions—such as repeated comments claiming verification without a linked proof or receipt (e.g., repeatedly writing `"Hoare-logic Verification Line"` or `"Branchless path is the unique solution to the state constraints of"`)—are viewed as an attempt to artificially inflate verification claims. 

The constitution dictates that human or agent assertions are insufficient. As Rule 27 states: *"Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim."* If a property cannot be stated precisely and verified mechanically, it is not recognized as valid law. Repeated comments are just text padding meant to simulate rigorous work without actually providing it.

## Enforcing Mechanical Proofs over Human Claims

The prohibition of `CHEAT-005` forces developers and agents to rely on mechanical proofs rather than text assertions. This is enforced through several integrated systems:

1. **Static Detection (`bcinr-cheat-scanner`)**: The structural auditor tool statically scans source code line-by-line for repetitive verification boilerplate. If it detects a threshold (e.g., 5 or more) of identical mock assertions, it immediately flags a `CHEAT-005` violation, fails the build, and instructs the author to *"Provide real axiomatic proofs or remove the mock comments."*
2. **Mandatory Artifacts**: True verification claims must always be backed by mechanical artifacts (proof obligations, receipt digests, or disassembly audits) rather than mere text comments.
3. **Independent Oracle Law (Rule 15)**: A claim must be mathematically backed by an independent "oracle." This must be a structurally distinct mathematical model (like a SAT/SMT bit-vector model, a symbolic proof, or a direct mathematical formula), not just an assertion in the code.
4. **Hostile Mutation Testing (Rule 19)**: Verification is proven mechanically by injecting plausible mutants. A proof is only considered valid if a mutant causes a failure with a specific **typed refusal code** or an exact oracle mismatch, proving that the verification is active and executing, rather than static text.

By combining the strict syntactic rejection of boilerplate text (`CHEAT-005`) with execution-based oracle and mutation gates, BCINR ensures that "verified" exclusively means mechanically proven through executable artifacts.
