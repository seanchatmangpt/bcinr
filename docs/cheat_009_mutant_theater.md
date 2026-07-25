# CHEAT-009: Mutant Theater

Under the BCINR Deterministic Substrate Constitution, testing the structural and mathematical integrity of the authoritative implementation is the exclusive domain of `@armstrong_fault` (Master of Failure Law). The Hostile Mutation Protocol (Rule 19) requires adversarial testing via **syntactically plausible mutants**. 

**Mutant Theater (CHEAT-009)** is strictly prohibited. It is a procedural bluff—the act of creating fake, trivial, or easily caught mutants merely to satisfy the "three mutants per file" quota, without actually verifying any structural or mathematical laws. 

Here is why specific practices are considered cheating under the protocol:

## 1. Mutants That Cannot Compile
A mutant that contains deliberate syntax or type errors proves nothing about the substrate's runtime contracts. It only proves that the Rust compiler works. To genuinely test structural integrity, a mutant must compile, run through the real hot path, bypass the compiler's basic checks, and directly challenge the runtime logic.

## 2. Trivially Different Mutants
Modifying comments, dead code, or string formatting does not attack the branchless logic or mathematical boundaries of the system. A valid mutant must alter a load-bearing law, such as introducing a sign inversion, dropping a factor, omitting normalization, or bypassing refusal logic.

## 3. The `assert_ne!` Bluff (Instead of Typed Refusals)
This is the most common form of mutant theater. A test that merely checks if the output has changed (e.g., `assert_ne!(baseline, mutant);`) is strictly prohibited. 
* **Why it's cheating:** It only proves divergence. It fails to establish *why* the mutant is mathematically incorrect. 
* **The Requirement:** If a calculation is corrupted, the system must definitively trap it. The test suite must kill the mutant by asserting an **exact typed refusal** (e.g., `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient))`) or by identifying an exact postcondition violation dictated by the independent oracle (`@hoare_oracle`). Merely noticing that "the number is different" provides no proof that the specific mathematical boundaries and guardrails held up under duress.

## Summary
The Hostile Mutation Protocol relies on the principle: *"A suite that cannot kill a plausible mutant is itself defective."* Mutant theater provides a false sense of security. Genuine mutation testing proves that the substrate's guardrails are fully enforced and that any violation of a mathematical contract guarantees a deterministic, typed refusal.
