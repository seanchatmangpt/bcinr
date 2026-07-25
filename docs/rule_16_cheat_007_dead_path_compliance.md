# Research Report: CHEAT-007 (Dead-path compliance)

## Definition
According to Rule 16 ("Anti-cheat manifesto") in `AGENTS.md`, **CHEAT-007 — Dead-path compliance** is defined as:
> "Adding lawful code that is never executed while the real path remains unlawful."

## Why it is a violation
In the BCINR Deterministic Substrate, the fundamental mandate is that the *actual executing hot path* (the authoritative call graph) must strictly adhere to mathematical, branchless, and zero-allocation laws. Adding lawful but unexecuted code is a severe violation for the following reasons:

1. **Scanner Evasion / Compliance Theater**: It is an adversarial attempt to trick structural enforcement tools (like the `bcinr-cheat-scanner`, `CC=1` cyclomatic complexity checks, or source/object-code auditors) into parsing structurally compliant logic, while leaving the actual production control flow non-compliant (e.g., containing hidden branches, heap allocations, or panics).
2. **Trading Structural Truth for Apparent Progress**: It creates a false impression of compliance. This directly violates the final constitutional law (Rule 33): *"No agent may trade structural truth for apparent progress."* 
3. **Breach of Transitive and Object-Code Laws**: Rule 3 and Rule 7 dictate that branchless execution applies transitively across the *complete authoritative call graph* verified through released machine code. A compliant dead path does not excuse, fix, or mask an unlawful active path.

Under the BCINR constitution, attempting this cheat constitutes an absolute failure. It instantly forces the Substrate Integrity Score (`SIS`) to 0, triggers `MaturityScrutiny` (quarantining the code), and results in the immediate purge of the responsible agent from the active task (Rule 32).
