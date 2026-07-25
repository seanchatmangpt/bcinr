# The "Exclusive Write Ownership" Law (Rule 26)

In the `bcinr` (BranchlessCInRust) deterministic substrate, Rule 26 strictly enforces **State Isolation and Write Ownership**. It mandates that agent work must use exclusive write ownership mapped to specific domains:

| Domain | Exclusive Writer |
| :--- | :--- |
| Contracts and proofs | `@hoare_oracle` |
| Scanners and structural gates | `@turing_machine` |
| Mutants and hostile fixtures | `@armstrong_fault` |
| Authoritative implementation | `@von_neumann_bypass` |

Shared-file concurrent editing across these domains is strictly prohibited. The constitution enforces this rule for several critical architectural and philosophical reasons:

## 1. Mandatory Independence and Separation of Concerns (Rule 5)
Every nontrivial task in `bcinr` must be decomposed into four independent workstreams. Shared editing fundamentally compromises this isolation. The constitution dictates:
- An implementation agent (`@von_neumann_bypass`) cannot author its own mathematical oracle.
- A structural auditor (`@turing_machine`) cannot silently repair implementation code and then self-approve the repair.
- A mutation agent (`@armstrong_fault`) cannot derive its expected failure states from the implementation it is supposed to attack.

By forcing separate files/domains with exclusive write locks, the system guarantees that the implementation is strictly downstream of the mathematical law, and that testing/auditing is performed completely independently.

## 2. Prevention of Self-Certification (Rule 27)
`bcinr` operates on the principle that *“Agent agreement is not evidence.”* The implementation agent must never be the final approver for mathematical correctness, branchlessness, mutant adequacy, or object-code compliance. If shared-file editing were permitted, agents could inadvertently or deliberately self-certify by aligning the implementation, tests, and oracles into a unified, self-fulfilling loop. Exclusive write ownership ensures each approval is backed by a mechanical artifact from a distinctly separate role.

## 3. Combating Circular Oracles and Cheating (Rule 16)
The Anti-Cheat Manifesto specifically bans the "Circular oracle" (CHEAT-002)—a reference implementation that is merely copied from or heavily influenced by the production code. By physically isolating the work of `@hoare_oracle` (proofs/reference semantics) from `@von_neumann_bypass` (branchless production implementation), the constitution forces the oracle to remain structurally and logically distinct (e.g., using an abstract state machine or a SAT/SMT bit-vector solver instead of a line-by-line translation). 

## 4. Chain of Custody and Enforced Handoffs
Rule 26 explicitly states: *"Other agents may review but may not edit without an explicit ownership transfer recorded in the work log."*
This creates a verifiable chain of custody. If a mutant requires an implementation change, the mutation agent cannot "just fix it" while they are testing. They must formally fail the gate (producing a `MUTATION_GATE_FAILED` state), forcing the feature development to freeze and transferring the task back to the implementation agent for a formal, mathematically documented repair. 

## Summary
The prohibition of shared-file concurrent editing is not merely a concurrency or version-control safeguard; it is the physical enforcement of the project's **Constitutional Precedence**. It ensures that rich mathematical semantics remain strictly upstream, while fixed deterministic mechanics remain strictly downstream, with an impenetrable, mechanically verified wall between them.
