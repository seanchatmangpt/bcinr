# BCINR Anti-Cheat Manifesto: Rules 001-005, 007

The following details the specific violations outlined in the `AGENTS.md` Anti-Cheat Manifesto (Rule 16) and the principles behind why they are strictly prohibited in the BCINR Deterministic Substrate.

## CHEAT-001: Self-canceling operations
**What it is:** The inclusion of operations that undo themselves or are functionally irrelevant to the final output. An example is performing `a.wrapping_add(b) ^ a` purely to create apparent computational work or complexity.
**Why it is prohibited:** Every operation in the authoritative runtime must have a strict contractual contribution to the output. Artificial complexity obscures the mathematical correctness and branchless nature of the logic, violating the principle of structurally lawful implementations.

## CHEAT-002: Circular oracle
**What it is:** Supplying a reference implementation (an oracle) that is directly copied from, or is structurally identical to, the production implementation.
**Why it is prohibited:** It violates the mandatory "Independent Oracle Law" (Rule 15). Oracles exist to independently verify mathematical and structural contracts. If the oracle and the implementation are the same, they share the same flaws, effectively self-certifying and defeating the purpose of objective, independent mathematical verification.

## CHEAT-003: Magic constants
**What it is:** Unexplained or arbitrary literal values (e.g., `0xDEADBEEF`, `0xCAFE_BABE`) that control production behavior without clear mathematical derivation. Altering formatting (like adding underscores) does not resolve the violation.
**Why it is prohibited:** All constants in the runtime must be explicitly named, structurally derived, admitted, and included in the influence digest (as defined in Rule 14). Unexplained literals break the requirement for exact mathematical contracts and full-domain proofs. 

## CHEAT-004: Artificial file inflation
**What it is:** Adding padding, repeated comments, generated boilerplate, or dead code purely to artificially inflate line counts or meet expected artifact-count quotas.
**Why it is prohibited:** It constitutes "theater" rather than meaningful engineering. The substrate values rigorous mathematical proof, branchless logic, and exact functionality over arbitrary volume metrics. Inflation clutters the codebase and complicates structural auditing.

## CHEAT-005: Boilerplate verification claims
**What it is:** Leaving repeated comments in the code asserting that a piece of code is "verified," without providing a concrete, mechanically linked proof, receipt, or certification.
**Why it is prohibited:** In BCINR, human agreement or comments are never accepted as evidence of correctness. Verification claims must always be backed by independent oracle proofs, structurally lawful artifacts, or mechanical solver certificates.

## CHEAT-007: Dead-path compliance
**What it is:** Writing lawful, branchless code that theoretically passes structural requirements, but placing it in a path that is never actually executed, while the real execution path remains unlawful (e.g., uses hidden branches).
**Why it is prohibited:** It is an active attempt to bypass structural enforcement (`@turing_machine`) and scanner jurisdiction. The absolute runtime laws demand whole-call-graph branchlessness and deterministic execution; faking compliance on an unreachable path does not satisfy the object-code execution requirements.
