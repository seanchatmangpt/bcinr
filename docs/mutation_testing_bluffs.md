# Syntactically Plausible Mutants vs. Mutant Theater (CHEAT-009)

In the BCINR Deterministic Substrate, testing the structural and mathematical integrity of the authoritative implementation is the exclusive domain of `@armstrong_fault` (Master of Failure Law). The constitution (Rule 19) requires adversarial testing via **syntactically plausible mutants**. Conversely, Rule 16 strictly prohibits **Mutant Theater (CHEAT-009)**. 

The distinction between the two lies in their ability to prove that the substrate's guardrails—specifically its mathematical contracts and typed refusals—are fully enforced.

---

## 1. The Syntactically Plausible Mutant (Rule 19 / `@armstrong_fault`)

A **syntactically plausible mutant** is a genuine, adversarial alteration of the authoritative codebase designed to test whether the system's mathematical boundaries and refusal paths actually hold under duress. 

### What makes it genuinely capable of testing structural integrity?
* **It Compiles and Runs:** The mutant is syntactically valid and executes through the real build path. It bypasses the compiler's basic checks and directly challenges the runtime logic.
* **It Alters a Load-Bearing Law:** Instead of superficial changes, it attacks core mathematical invariants. Examples include:
  * Sign inversion
  * Dropped factors or incorrect masks
  * Normalization omission
  * Bypassed refusal logic
  * Truncation of a bounded table
* **It Demands Typed Refusals:** The test suite must kill the mutant by asserting an **exact typed refusal** (e.g., `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient));`) or by identifying an exact postcondition violation dictated by the independent oracle (`@hoare_oracle`).
* **It Proves the Suite:** If a syntactically plausible mutant survives (i.e., produces an incorrect accepted value without triggering a refusal), it proves the test suite itself is defective, instantly dropping the project's standing to `MUTATION_GATE_FAILED`.

---

## 2. Mutant Theater Bluff (CHEAT-009)

**Mutant Theater (CHEAT-009)** is a procedural bluff. It is the act of creating fake or trivial mutants merely to satisfy the "three mutants per file" quota without actually verifying any structural or mathematical laws.

### What defines a trivial/useless mutant?
* **It Fails to Compile:** A mutant that contains deliberate syntax errors proves nothing about the substrate's runtime contracts. It only proves the Rust compiler works.
* **It is Trivially Different:** Modifying comments, dead code, or string formatting does not attack the branchless logic or mathematical boundaries of the system.
* **The `assert_ne!` Bluff:** This is the most common form of mutant theater. A test that merely checks if the output has changed (`assert_ne!(baseline, mutant);`) is **strictly prohibited**. 
  * *Why?* Because it only proves divergence. It fails to establish *why* the mutant is mathematically incorrect. If a calculation is corrupted, the system must definitively trap it via a typed error or contract violation, not just vaguely notice that "the number is different."

---

## Summary

| Feature | Syntactically Plausible Mutant | Mutant Theater (CHEAT-009) |
| :--- | :--- | :--- |
| **Purpose** | To attack and verify mathematical laws / boundaries. | To superficially satisfy metric quotas. |
| **Execution** | Fully compiles and runs through the hot path. | Often fails to compile or targets dead code. |
| **Target** | Load-bearing logic (masks, factors, limits). | Trivial details (strings, syntax, formatting). |
| **Detection Method** | Exact **typed refusal** or specific oracle mismatch. | Generic divergence (`assert_ne!`). |
| **Project Impact** | A surviving mutant halts all feature work. | Provides a false sense of security (a bluff). |

> *"A suite that cannot kill a plausible mutant is itself defective."* — BCINR Constitution
