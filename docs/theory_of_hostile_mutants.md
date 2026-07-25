# Theory and Practice of Hostile Mutants in BCINR

## Introduction
Based on the BCINR Deterministic Substrate Constitution (`AGENTS.md`), hostile mutation testing is a rigorous, adversarial verification practice governed by Rule 19 and overseen by the `@armstrong_fault` agent (Master of Failure Law). The core philosophy is that a test suite is only as strong as the plausible mutants it can successfully "kill". A suite that cannot kill a plausible mutant is considered structurally defective.

## Counterfactual Testing
Counterfactual testing in the context of BCINR involves deliberately injecting mathematically and syntactically plausible defects (mutants) into the authoritative implementation to verify whether the test suite and validation mechanisms (oracles) can detect the corruption. Instead of merely asserting that the happy path works, counterfactual testing asks "what happens if a specific load-bearing law is broken?" The corrupted implementation must either catch the error through a typed refusal or be caught by an independent mathematical oracle. The goal is to prove that the system's defenses and verification gates actively trap regressions, rather than passively passing tests.

## Meaningful Law Alterations
A meaningful law alteration is a deliberate corruption that targets a core mathematical, structural, or logical invariant of the system. According to the constitution, every authoritative implementation file must include at least three such independent, syntactically plausible mutants. Examples of altering a meaningful law include:

*   **Sign inversion**: Flipping a positive operation to negative or reversing mathematical logic.
*   **Dropped factor**: Omitting a multiplier or crucial term in a mathematical calculation.
*   **Incorrect mask**: Using a bitmask that selects the wrong state or fails to properly isolate intended bits.
*   **Normalization omission**: Failing to normalize a vector or value where the contract requires it.
*   **Index skew**: Off-by-one errors or selecting the wrong element in a bounded lookup table.
*   **Stale digest acceptance**: Bypassing cryptographic or state-digest freshness checks.
*   **State mutation before admission**: Violating the strict transaction sequence by speculatively altering state before full validation.
*   **Truncation of a bounded table**: Arbitrarily shortening fixed-size lookup tables.
*   **Bypassed refusal**: Removing early-exit or failure paths that should reject invalid inputs or states.
*   **Incorrect clamp**: Applying wrong boundaries or failing to saturate arithmetic correctly.
*   **Unsupported fallback**: Using a simpler but uncertified algorithmic path.

## Superiority of Typed-Refusals over `assert_ne!`
The constitution strictly prohibits the use of simple inequality assertions (like `assert_ne!(baseline, mutant);`) when evaluating mutants. 

Checking for exact typed-refusals is superior for several reasons:
1.  **Semantic Precision**: `assert_ne!` only proves that the output changed; it doesn't prove that the system safely handled the failure. A typed refusal (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`) proves the runtime mathematically and semantically identified the exact invariant that was violated.
2.  **Safety Guarantees**: A mutant might produce a different output that still appears valid to a naive caller. Requiring a typed refusal ensures that invalid states are actively trapped and rejected at the system boundary, preventing silent data corruption.
3.  **Dual-Layered Verification**: If a mutant yields a successfully accepted but incorrect value (rather than triggering a refusal), the independent oracle must explicitly identify the exact violated postcondition. This enforces a rigorous verification system where either the runtime cleanly refuses, or the mathematical oracle objects.

## The Hostile Mutation Protocol (Rule 19)
For every implementation file, the following protocol must be strictly executed:
1.  **Identify** at least three load-bearing laws.
2.  **Produce** one mutant per law.
3.  **Inject** the mutant through the real build path.
4.  **Run** the normal test suite.
5.  **Verify** the expected typed refusal or independent oracle mismatch.
6.  **Record** the kill evidence in a mutant ledger (tracking mutant id, source file, changed law, exact mutation, expected detection, actual detection, test name, receipt digest, and standing).

A surviving mutant immediately changes the project standing to `MUTATION_GATE_FAILED` and blocks all feature work until the defect is resolved.
