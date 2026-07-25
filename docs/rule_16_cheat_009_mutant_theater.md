Based on the `AGENTS.md` BCINR Deterministic Substrate Constitution, **CHEAT-009 (Mutant theater)** addresses the use of ineffective or superficial mutants in adversarial testing. 

Under the rules established by `@armstrong_fault` (Master of Failure Law), every authoritative implementation must be tested with at least three independent, syntactically plausible mutants that alter a meaningful law. The specified types of mutants are considered "theater" for the following reasons:

1. **Mutants that cannot compile**:
   The primary goal of a mutant is to verify that the test suite can detect structural or mathematical logic flaws at runtime. If a mutant does not compile, it is caught by the compiler, completely bypassing the test suite. Mutants must be "syntactically plausible" so they actually test the suite's ability to catch semantic or logical violations.

2. **Mutants that are trivially different**:
   A valid mutant must alter a "meaningful law" (e.g., sign inversion, dropped factor, incorrect mask, truncation of a bounded table, bypassed refusal). Trivially different mutants do not meaningfully challenge the mathematical contracts or invariants, and thus killing them provides no real confidence in the code's robustness.

3. **Mutants detected only by `assert_ne!`**:
   This is explicitly prohibited under the **Typed-refusal requirement**. Simply verifying that a mutant's output is different from the baseline (`assert_ne!(baseline, mutant)`) proves only that a change occurred, not that the system correctly understood and rejected the invalid state. The constitution mandates that the test must prove the corrupted implementation either:
   - Triggers a specific, bounded typed refusal (e.g., `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient))`).
   - Is caught by the independent oracle identifying the exact violated postcondition when it produces a wrong accepted value.
