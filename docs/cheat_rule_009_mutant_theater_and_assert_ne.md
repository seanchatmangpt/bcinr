# CHEAT-009: Mutant Theater and the Prohibition of `assert_ne!`

## The Core Mandate

Under the BCINR Deterministic Substrate Constitution, **Rule 4 (`@armstrong_fault` — Master of Failure Law)** strictly prohibits testing hostile mutants using simple inequality checks:

```rust
// PROHIBITED (Mutant Theater)
assert_ne!(baseline, mutant);
```

Instead, the Constitution mandates a **Typed-refusal requirement**:
```rust
// REQUIRED
assert_eq!(
    result,
    Err(StabilityRefusal::ContractionMarginInsufficient)
);
```

When a mutant is evaluated, relying solely on `assert_ne!` triggers **CHEAT-009 — Mutant theater**, which is considered an evasion of rigorous testing and immediately invalidates project standing.

## Why is `assert_ne!` considered "Mutant Theater"?

1. **Difference is Not Detection:** `assert_ne!` only proves that a mutation caused *some* change in the output. It does not prove that the system's guardrails actively detected the mathematical or logical corruption. It is "theater" because it provides the illusion of test coverage without actually proving structural safety or failure handling.
2. **Evasion of Contractual Obligations:** A test suite relying on inequality cannot distinguish between a benign variation, a catastrophic silent failure, and a safely bounded refusal. As Rule 4 states: *"A suite that cannot kill a plausible mutant is itself defective."* Killing a mutant requires actively trapping it in a verified failure mode, not merely observing that it looks different than the baseline.
3. **Violation of Oracle Independence:** If a mutant manages to produce an accepted (but incorrect) value rather than a typed refusal, the independent mathematical oracle (`@hoare_oracle`) must be able to pinpoint the *exact* violated postcondition. `assert_ne!` abdicates this responsibility, operating as a lazy assertion rather than an axiomatic proof.

## Why Must the Test Prove a Specific Typed Refusal?

1. **Proof of Bounded Execution (Rule 18):** Every primitive in BCINR must operate within a fixed mathematical envelope. When a mutant alters a load-bearing law (e.g., dropping a factor, altering a mask, or bypassing a clamp), the system must deterministically route to a bounded typed refusal (e.g., `ContractViolation`, `DigestMismatch`, `ContractionMarginInsufficient`). Asserting the *exact* refusal code proves that the branchless logic successfully isolated the specific structural defect.
2. **Defense of Persistent State (Rule 10):** A typed refusal guarantees that persistent state remains bit-for-bit unchanged during a rejected operation. If a test only checks `assert_ne!`, it cannot guarantee that speculative or partial mutation didn't leak into the system prior to the operation's completion.
3. **Traceable Hostile Mutation Protocol (Rule 19):** Every mutant must have a ledger entry documenting the `expected detection` and `actual detection`. A typed refusal provides the exact, deterministic identifier required for this ledger, ensuring that adversarial tests rigorouosly map back to the Hoare contract constraints.

## Summary

In the BCINR substrate, **logic is expressed as arithmetic** and **failure is bounded by typed refusals**. `assert_ne!` is mathematically lazy—it observes a symptom without diagnosing the invariant breach. True verification requires the test suite to formally demonstrate *how* the substrate defended its mathematical boundaries against the hostile mutant.
