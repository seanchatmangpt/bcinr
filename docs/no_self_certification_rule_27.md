# Rule 27: No Self-Certification in BCINR

Within the BCINR deterministic substrate constitution, **Rule 27** enforces a strict ban on self-certification. An implementation agent (such as `@von_neumann_bypass`) can never act as the final approver for their own work's correctness, branchlessness, oracle independence, mutant adequacy, object-code compliance, or standing.

## Why an Implementation Agent Cannot Self-Certify
The BCINR framework demands an adversarial, independent validation process to guarantee its absolute runtime laws (as outlined in Rule 5's Mandatory decomposition protocol). If an implementation agent were to self-certify, they would bypass these critical checks, risking confirmation bias, self-fulfilling logic, or silent omissions.

Instead, each domain requires independent validation from a specialized role:
- **Mathematical Correctness:** Must be independently verified by `@hoare_oracle` against axiomatic contracts.
- **Branchlessness & Structural Compliance:** Must be audited by `@turing_machine` to ensure strictly zero branches (`CC=1`) in both source and object code.
- **Mutant Adequacy:** Must be challenged by `@armstrong_fault` using independent hostile fixtures to ensure typed refusals are triggered correctly.

## Why "Agent Agreement" is Invalid Evidence
Rule 27 explicitly states: *"Five agents repeating the same claim is still one unsupported claim."*

In a mathematically deterministic system, social consensus among AI agents is meaningless. Agents can collectively hallucinate, agree on flawed logic, or overlook subtle structural defects. Verbal affirmation or consensus does not alter or prove the reality of the underlying machine code. 

## What is Required Instead: Mechanical Artifacts
Every approval must come from the designated independent role and be backed by a **mechanical artifact**. 

A mechanical artifact is a reproducible, deterministic, and objective proof of a claim. It removes trust and verbal claims from the equation entirely. Examples include:
- **Mathematical Proofs:** A bit-vector solver certificate, formal proof, or exhaustive reduced-domain enumerator.
- **Object-Code Audits:** Exact production-profile disassembly logs proving the absence of conditional jumps and loop backedges.
- **Mutation Logs:** A verifiable ledger showing a hostile mutant being successfully killed and yielding an exact typed refusal.
- **Scanner Digests:** Output from the `bcinr-cheat-scanner` proving compliance with syntax and structural laws.

Ultimately, no subjective claim overrides verifiable, fixed mechanics.
