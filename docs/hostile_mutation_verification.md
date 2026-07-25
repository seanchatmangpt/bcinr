# Hostile Mutation Testing Verification (`@armstrong_fault`)

Under the `bcinr` deterministic substrate constitution, **`@armstrong_fault`** acts as the "Master of Failure Law." This role owns adversarial test architecture, counterfactual mutant design, negative-domain testing, refusal-path verification, and test-suite adequacy. 

A core philosophy governs this domain: *"A suite that cannot kill a plausible mutant is itself defective."*

## The Hostile Mutation Protocol
For every implementation file, `@armstrong_fault` enforces the following protocol:
1. **Identify** at least three load-bearing laws (e.g., sign inversion, dropped factor, incorrect mask, bypassed refusal, incorrect clamp).
2. **Produce** one syntactically plausible but structurally broken mutant per law.
3. **Inject** the mutant through the real build path.
4. **Run** the normal test suite.
5. **Verify** the expected typed refusal or oracle mismatch.
6. **Record** the kill evidence in a mutant ledger (recording ID, file, law, exact mutation, expected vs. actual detection, test name, receipt digest, and standing).

If a mutant survives, the project's standing immediately changes to `MUTATION_GATE_FAILED`, quarantining the code and blocking all feature work.

## How is a Mutant Definitively "Killed"?
A mutant is **not** considered killed merely because a test fails or because the output differs from a baseline. A mutant is definitively killed only when the test proves that the corrupted implementation:
1. Triggers a precise, bounded **Typed Refusal**, OR
2. Violates a specific postcondition identified by the independent oracle (coordinated with `@hoare_oracle`), when it produces a wrong accepted value rather than a refusal.

## Prohibited Arbitrary Assertions vs. Required Typed Refusals

### ❌ Prohibited: `assert_ne!` (Mutant Theater)
Using arbitrary assertions like `assert_ne!(baseline, mutant);` is explicitly **prohibited**. 
- It falls under **CHEAT-009: Mutant Theater**.
- Just proving that a mutant produces a "different" output does not verify that the system correctly caught the underlying invariant violation. It provides no proof that the system safely halted or accurately diagnosed the mathematical failure.

### ✅ Required: Typed Refusals
All rejected authoritative operations must produce a bounded **Typed Refusal** code (e.g., `ContractViolation`, `NumericRangeExceeded`, `ContractionMarginInsufficient`, `DigestMismatch`). 
- A valid kill must look like: 
  `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient));`
- **Why this is required:** It proves that the deterministic runtime correctly recognized the exact structural law being broken and refused it safely without panicking, falling back to a simpler algorithm, silently clamping outside admitted policy, or returning a plausible default.

This strict enforcement ensures that every line of logic in `bcinr` is a bounded mathematical contract with real execution teeth.
