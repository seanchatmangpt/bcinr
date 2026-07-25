# Negative Domain Testing Protocols (`@armstrong_fault`)

The negative domain testing protocols in the `bcinr` project are strictly governed by `@armstrong_fault`, the adversarial test architect and Master of Failure Law. The primary mandate of this protocol is rooted in the philosophy that a test suite incapable of catching a plausible bug in a broken implementation is itself defective. 

## 1. Design of Hostile Counterfactual Mutants

For every authoritative implementation file, `@armstrong_fault` mandates the creation of at least **three independent, syntactically plausible mutants**. Each mutant is designed to be structurally broken while remaining compilable. They must intentionally alter a meaningful, load-bearing mathematical law or branchless constraint.

Mutant designs focus on corrupting core logic mechanics:
*   **Invalid Masks & Bit-skips:** Supplying incorrect bitmasks, skipping bit-parallel validation steps, or bypassing `ConcurrencyGuardTable` admission checks.
*   **Arithmetic Corruptions:** Sign inversions, dropped factors, failure to apply mathematical normalization, or utilizing constant static values instead of dynamic capacities.
*   **State & Boundary Violations:** Index skews, truncation of bounded lookup tables, incorrect clamping outside of admitted bounds, or speculatively mutating persistent state *before* complete admission has been validated.
*   **Security & Refusal Bypasses:** Accepting a stale digest/certificate, circumventing refusal checks, or falling back to a simpler, unsupported algorithm instead of generating an exception.

*Examples of Hostile Mutants (e.g., V2 Scheduler Bridge):*
*   **The "Guard Bypass" Mutant:** Skips the `admits(&fired)` concurrency check, firing operations blindly and generating an invalid state transition.
*   **The "Single-Step Reversion" Mutant:** Artificially restricts the scheduler to firing a maximum of one operation per tick, which neutralizes structural concurrency while passing all sequential logic tests.

## 2. Asserting with `TypedRefusals` (The Typed-Refusal Requirement)

A cornerstone of `@armstrong_fault`'s protocol is how test failures are asserted to ensure rigorous evaluation.

*   **No Lazy Assertions:** The test suite is strictly prohibited from relying on generic inequality checks (e.g., `assert_ne!(baseline, mutant)`) as proof that a mutant was killed.
*   **No Panics or Unwinding:** Because the deterministic hot path must adhere strictly to zero-allocation and a cyclomatic complexity of one (`CC=1`), no hostile input or mutation may trigger a panic, an unwind, or bounds-check failure.
*   **Bounded Typed Refusals:** The test must explicitly prove that the corrupted implementation either violates a specific Hoare contract or correctly triggers a **bounded typed refusal** code.

```rust
// PROHIBITED: Lazy inequality checking
assert_ne!(baseline, mutant);

// REQUIRED: Explicit TypedRefusal assertion
assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient));
```

Other required bounded typed refusals include `ContractViolation`, `UnsupportedDomain`, `DigestMismatch`, `LearningFrozen`, `ReceiptRejected`, and `BranchlessContractFailed`. If a mutant produces a mathematically wrong accepted value instead of a direct refusal, the independent oracle (`@hoare_oracle`) must identify the exact violated postcondition (e.g., verifying that a specific bitflag like `ObservatoryFlag::NumericallyUncertain` was correctly raised).

## 3. The Hostile Mutation Execution Protocol

The process of evaluating these mutants follows a rigorous step-by-step protocol:
1. Identify at least three load-bearing laws per file.
2. Produce one mutant per law.
3. Inject the mutant through the real build path.
4. Run the standard test suite.
5. Verify the expected `TypedRefusal` or exact oracle mismatch.
6. Record the kill evidence in the mutant ledger (`MUTANT_KILL_MATRIX.md`), including mutant id, source file, changed law, expected/actual detection, test name, receipt digest, and standing.

**Zero-Tolerance Standing:** A single surviving mutant means the test suite has failed. This immediately sets the project standing to `MUTATION_GATE_FAILED` and blocks all further feature work until the defect is resolved.
