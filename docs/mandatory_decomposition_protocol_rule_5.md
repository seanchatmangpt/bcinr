# Mandatory Decomposition Protocol (Rule 5)

Rule 5 of the BCINR Deterministic Substrate Constitution establishes the **Mandatory decomposition protocol**. This rule dictates that every nontrivial implementation task must be immediately divided into four strictly independent workstreams. Each workstream is governed by a distinct agent persona (or "Transcendent Construct"), ensuring a rigid separation of concerns that spans mathematical specification, structural auditing, adversarial testing, and branchless implementation.

## The Four Independent Workstreams

| Workstream | Owner | Role & Output |
| :--- | :--- | :--- |
| **Mathematical law** | `@hoare_oracle` | Acts as the axiomatic proof lead and specification owner. Responsible for delivering mathematical contracts (preconditions, postconditions, invariants), proof obligations, and independent reference semantics (the oracle). |
| **Structural enforcement** | `@turing_machine` | Acts as the structural auditor and merge gatekeeper. Responsible for delivering source and object-code audit plans, enforcing the absolute `CC=1` (cyclomatic complexity) law, and ensuring no hidden branches or allocations exist. |
| **Hostile verification** | `@armstrong_fault` | Acts as the adversarial test architect. Responsible for designing counterfactual mutants, hostile fixtures, and refusal expectations to ensure the implementation cannot survive corruption without triggering a typed refusal. |
| **Implementation** | `@von_neumann_bypass` | Acts as the authoritative implementation owner. Responsible for delivering the branchless, bounded, and allocation-free code that precisely fulfills the mathematical law. |

## The Strict Prohibition of Self-Certification

A core tenet of Rule 5—further reinforced by Rule 15 (Independent oracle law) and Rule 27 (No self-certification)—is the absolute prohibition of self-certification. The constitution explicitly mandates that **independence is mandatory**. 

### Why Self-Certification is Forbidden
1. **Prevention of Circular Logic (The Oracle Independence Law):** An implementation agent (`@von_neumann_bypass`) is explicitly forbidden from authoring its own final oracle and self-certifying equivalence. If the same entity writes both the implementation and the mathematical reference, confirmation bias, shared flawed assumptions, or "line-by-line translation" can infect both. The oracle must be structurally and logically distinct to serve as a true mathematical anchor.
2. **Integrity of the Audit Trail:** A structural auditor (`@turing_machine`) cannot silently repair implementation code and then approve its own repair. This ensures that the implementation is objectively evaluated exactly as submitted. Repairs must be deliberately processed through the proper implementation channels to maintain a transparent, verifiable history.
3. **Objective Adversarial Testing:** A mutation agent (`@armstrong_fault`) must not derive its expected results from the implementation under attack. If the adversarial tests baseline their expectations on the implementation's current behavior rather than the independent mathematical specification, the tests will falsely pass a fundamentally flawed implementation.
4. **Mechanical Evidence Over Consensus:** The constitution states, "Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim." Every component of maturity (mathematical correctness, branchlessness, mutant adequacy) must be approved by a different role and backed by a rigid, mechanical artifact rather than subjective human or agent consensus.

By enforcing this strict division of labor and forbidding self-certification, the BCINR constitution guarantees that every primitive is rigorously cross-examined mathematically, structurally, and adversarially before it is considered for the deterministic substrate.
