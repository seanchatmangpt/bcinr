# Mandatory Decomposition and Independence in BCINR

The BCINR project enforces a rigid, adversarial verification model defined primarily by **Rule 5 (Mandatory decomposition protocol)** and **Rule 27 (No self-certification)**. Because BCINR acts as a deterministic, civilizational-scale substrate, classical "best effort" engineering practices and developer consensus are discarded in favor of mathematical proof, structural enforcement, and cryptographic-level rigor. 

Central to this methodology is the principle that **independence is mandatory**.

## The Check-and-Balance System (Rule 5)

Every nontrivial task in BCINR is immediately shattered into four distinct, non-overlapping workstreams. This forms a structural check-and-balance system where each role has entirely different—and deliberately conflicting—incentives:

1. **`@hoare_oracle` (Mathematical Law):** Owns the axiomatic proof, pre/postconditions, and independent reference semantics. They define the absolute mathematical truth of *what* the implementation must do, irrespective of how hard it is to write.
2. **`@von_neumann_bypass` (Implementation):** Owns the branchless bounded code. They must write the actual implementation using bit-parallel mechanics and fixed-width state transitions, constrained by the Oracle's laws.
3. **`@turing_machine` (Structural Enforcement):** Owns the source and object-code audit plan. They act as the merciless structural gatekeeper, ensuring the implementer didn't sneak in branches, allocations, or loop backedges to satisfy the Oracle.
4. **`@armstrong_fault` (Hostile Verification):** Owns the adversarial mutants and negative-domain testing. They are tasked with actively trying to break the Implementation and proving that if a mathematical law is altered, a typed refusal or oracle mismatch is definitively triggered.

By isolating these roles, BCINR guarantees that the definition of correctness, the construction of the logic, the auditing of the structure, and the adversarial testing are never compromised by a single agent's convenience.

## Why Self-Certifying Equivalence is Banned (Rule 27)

Rule 27 explicitly forbids self-certification. The implementation agent (`@von_neumann_bypass`) cannot be the final approver for mathematical correctness, branchlessness, oracle independence, mutant adequacy, or object-code compliance. 

The constitution outlines specific tautological traps that this ban avoids:
* **The "Circular Oracle" Trap:** An implementer writing their own oracle might subconsciously design the reference specification to match the quirks, bugs, or limitations of their implementation. 
* **The "Tautological Mutant" Trap:** A mutation agent cannot derive expected results from the implementation under attack. If they did, they would simply be testing that corrupted code behaves like corrupted code, rather than verifying it violates an independent mathematical law.
* **The "Silent Repair" Trap:** A structural auditor (`@turing_machine`) cannot fix code and approve it. The auditor must remain an objective observer; modifying the code taints their independence.

## Mechanical Artifacts over Consensus

Ultimately, independence ensures that correctness is derived from mathematical collision rather than human or agent agreement. As Rule 27 states: 

> *"Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim."*

In BCINR, trust is placed exclusively in mechanical artifacts. Independence forces these artifacts (Hoare proofs, disassembly evidence, mutant ledgers) to be generated from different vantage points. When all four independent perspectives converge perfectly on a zero-allocation, CC=1, fully tested piece of code, it establishes true repository standing.
