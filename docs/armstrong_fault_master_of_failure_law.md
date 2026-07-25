# The `@armstrong_fault` Role: Master of Failure Law

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`, Rule 4), the `@armstrong_fault` role acts as the **Adversarial Test Architect and Mutation Owner**. This role enforces the system's failure laws and ensures that mathematical contracts strictly hold up against corrupted logic and hostile inputs.

## What is Counterfactual Mutant Design?

Counterfactual mutant design is the practice of intentionally creating syntactically plausible, logically flawed variations of the authoritative implementation code (known as "mutants"). The goal is to verify that the system's independent oracles and failure mechanisms successfully detect exact violations of mathematical laws.

Every authoritative implementation file must feature at least three independent, syntactically plausible mutants. Each mutant must deliberately alter a meaningful law. Examples of acceptable mutations include:
- Sign inversion or dropped factors
- Incorrect masks or incorrect clamps
- Normalization omission or index skew
- Stale digest acceptance or bypassed refusals
- State mutation before complete admission
- Truncation of a bounded table or unsupported fallbacks

These counterfactual scenarios ensure that the implementation's invariants are truly load-bearing. 

## Exclusive Authority Over Hostile Fixtures and Negative-Domain Testing

To preserve the mandatory decomposition and independence protocol, `@armstrong_fault` holds **exclusive authority and write ownership** over:
- Counterfactual mutant design
- Hostile fixtures
- Negative-domain testing
- Refusal-path verification
- Test-suite adequacy judgments

Because the `bcinr` substrate enforces bounded, branchless, and allocation-free execution, testing must strictly go beyond the "happy path." `@armstrong_fault` architects **hostile fixtures** and conducts **negative-domain testing** to prove that the system rigorously rejects invalid, out-of-bounds, or malicious inputs. 

Instead of panicking, unwinding, or silently defaulting to a fallback outside admitted policies, invalid inputs must trigger bounded, **typed refusals** (e.g., `ContractViolation`, `UnsupportedDomain`, `ContractionMarginInsufficient`). The independence of this role guarantees that the implementation owner (`@von_neumann_bypass`) cannot self-certify the robustness of their own code against adversarial conditions.

## Why a Test Suite That Cannot Kill a Plausible Mutant is Defective

The BCINR Constitution mandates a strict standard:

> **"A suite that cannot kill a plausible mutant is itself defective."**

A test suite's purpose in this ecosystem is not merely to check if the baseline code produces the expected output; it must guarantee that the implementation is mathematically airtight and structurally lawful. 

If a plausible mutant is injected and the test suite passes anyway, it exposes a critical gap: the suite fails to enforce the system's axiomatic laws. Furthermore, detecting a mutant through a simple output mismatch (e.g., `assert_ne!(baseline, mutant)`) is explicitly prohibited. The suite must specifically catch the exact failure law being violated, either by identifying the broken postcondition via the independent oracle or by returning the correct typed refusal.

A surviving mutant immediately changes the project standing to `MUTATION_GATE_FAILED` and blocks all feature work. Therefore, an inability to kill a plausible mutant renders the test suite defective, as it fails to uphold the absolute runtime laws of the deterministic substrate.
