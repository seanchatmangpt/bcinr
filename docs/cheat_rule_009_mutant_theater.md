Based on the `AGENTS.md` constitution, **CHEAT-009 (Mutant theater)** refers to the creation of invalid or ineffective mutants that simulate rigorous testing without actually verifying the system's mathematical and structural laws.

### What Constitutes an Invalid Mutant
According to the **CHEAT-009** rule, an invalid (theatrical) mutant is one that:
1. **Cannot compile**: It breaks the build rather than testing the logic.
2. **Is trivially different**: It doesn't alter a meaningful, load-bearing law.
3. **Is detected only by `assert_ne!`**: It merely checks if the output changed, rather than verifying the contract.

*(Additionally, the Master of Failure Law (`@armstrong_fault`) specifies that plausible mutants must alter meaningful laws, such as sign inversion, dropped factors, index skews, or bypassed refusals).*

### Why Mutants Detected Only by `assert_ne!` are Theatrical
Using a simple `assert_ne!(baseline, mutant)` is explicitly prohibited under the **Typed-refusal requirement** because it is mathematically insufficient. It merely proves that the output is different, but not *why* it is wrong or that the system's defenses actually caught the violation.

To be valid, a mutant test must either:
- Prove that the corrupted implementation triggers a specific, bounded **typed refusal** (e.g., `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient))`).
- Have an **independent oracle** identify the exact violated postcondition if a wrong value is accepted instead of triggering a refusal.

In the deterministic substrate of BCINR, a test must prove that a corrupted implementation violates a specific mathematical contract. Mutants must demonstrate that the system's laws actively enforce correctness, rather than simply proving that changing the input changes the output.
