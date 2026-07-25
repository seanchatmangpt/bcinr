# Analysis of Anti-Cheat Rules: CHEAT-004 and CHEAT-005

In accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the authoritative runtime is governed by strict mathematical contracts, rigorous auditing, and reproducible evidence. Rule 16 establishes the Anti-Cheat Manifesto to prevent the illusion of rigor from masking structural deficiencies. 

Two critical violations in this manifesto are **CHEAT-004 (Artificial file inflation)** and **CHEAT-005 (Boilerplate verification claims)**.

## CHEAT-004: Artificial File Inflation

**Definition:** Padding, repeated comments, generated boilerplate, or dead code added to satisfy line-count or artifact-count expectations.

### What it Looks Like
Artificial file inflation manifests as code that artificially bulks up the repository without adding semantic value or satisfying mathematical obligations. Examples include:
- Meaningless blank lines or overly verbose comment blocks that repeat the same information.
- Dead functions or unreachable code blocks ("dead-path compliance") that are never linked into the authoritative runtime.
- Expansive macro-generated boilerplate intended purely to hit artifact size metrics.

### Subversion of Project Integrity
The BCINR substrate requires a branchless, fixed-width, bounded execution model (Rule 3). Artificial inflation directly subverts the structural auditing process managed by `@turing_machine`. It acts as camouflage, making object-code scans and complexity analyses (`CC=1` verification) significantly harder by introducing noise. It wastes the evaluator's capacity and games project metrics, prioritizing visual size over the strictly required mathematical contracts. By attempting to spoof the maturity of the codebase, it bypasses the constitutional mandate that every line of code must strictly contribute to a deterministic output.

## CHEAT-005: Boilerplate Verification Claims

**Definition:** Repeated comments asserting verification without a linked proof or receipt.

### What it Looks Like
This violation occurs when developers or automated agents litter the codebase with unsubstantiated assertions of correctness. Examples include:
- Inline comments such as `// VERIFIED`, `// Proof complete`, `// CC=1 checked`, or `// Branchless` floating above functions.
- Documentation blocks claiming that the Substrate Integrity Score (SIS) is 100 or that a Hoare contract has been satisfied, but failing to link to the corresponding artifact digest, bit-vector solver certificate, or mutation test receipt.

### The False Sense of Security
Boilerplate verification claims are dangerous because they create a facade of rigorous testing. They trick reviewers, agents, and automated tools into assuming that a piece of code has already survived hostile mutation by `@armstrong_fault` or object-code auditing by `@turing_machine`. 

As Rule 27 strictly dictates: *"Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim."* A string of text asserting verification is subjective and meaningless within a system that requires hard, deterministic proofs. 

### Why Claims Must Link to Artifacts/Receipts
The fundamental law of the BCINR Constitution is **reproducible evidence**. For a primitive to be admitted into the authoritative runtime, it must possess independent oracles, hostile mutants, and object-code verification. 

A verification claim is only valid if it provides a cryptographic link (receipt/artifact) demonstrating that:
1. The exact structural bytecode was analyzed and produced no branches.
2. The specific input domain successfully fulfilled the mathematical contract (`@hoare_oracle`).
3. The implementation survived counterfactual hostile mutants via typed refusals.

Without a linked artifact, a claim cannot be mechanically verified. If the verification isn't backed by a reproducible receipt, it is treated as fabricated evidence, forcing the `SIS` score to 0 and triggering a `MaturityScrutiny` lockdown (Rule 24).
