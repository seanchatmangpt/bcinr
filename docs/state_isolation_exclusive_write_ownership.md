# State Isolation and Write Ownership in BCINR

Under Rule 26 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the project mandates strict state isolation and exclusive write ownership for all agent work. This is to guarantee mathematical rigor, structural branchlessness, and robust mutation testing by ensuring absolute independence among the distinct roles performing the work.

## Domain-to-Writer Mapping

The constitution strictly divides the repository into four distinct workstreams and maps them to specialized agents who hold exclusive write privileges over those specific areas:

| Domain | Exclusive Writer | Constitutional Role |
| :--- | :--- | :--- |
| **Contracts and Proofs** | `@hoare_oracle` | Axiomatic proof lead and specification owner. Holds exclusive authority over preconditions, postconditions, invariants, algebraic laws, and independent reference semantics. |
| **Scanners and Structural Gates** | `@turing_machine` | Structural auditor and merge gatekeeper. Responsible for cyclomatic complexity (CC=1) enforcement, object-code audits, and verifying the authoritative instruction shape. |
| **Mutants and Hostile Fixtures** | `@armstrong_fault` | Adversarial test architect and mutation owner. Crafts plausible counterfactual mutants, negative-domain tests, and verifies typed refusals to ensure no mutant survives. |
| **Authoritative Implementation** | `@von_neumann_bypass` | Architect of arithmetic logic. Owns the branchless arithmetic design, SWAR construction, mask-based state selection, and fixed-width state transitions. |

## Enforcement of Ownership Transfer

The constitution relies on these exclusive ownership boundaries to prevent self-certification (as outlined in Rule 27, which states an implementer cannot be the final approver for mathematical correctness or structural audits). The mechanics of enforcing state isolation and ownership transfer are as follows:

1. **Strict Review-Only Default:** Agents are permitted to review files in any domain, but they are constitutionally barred from editing domains they do not own. The constitution explicitly forbids "shared-file concurrent editing."
2. **Explicit Ledger in the Work Log:** The constitution requires that "an explicit ownership transfer [is] recorded in the work log" before an agent can assume editing rights for another's domain. This acts as an auditable ledger. Without this formal, recorded hand-off, any cross-domain edit is a constitutional violation.
3. **Preservation of Independence:** Rule 5 mandates that "independence is mandatory" across these four workstreams. By forcing ownership transfers to be explicitly logged, the system prevents an implementation agent from silently taking over the structural audit or mathematical proofs. The log proves who held the pen, ensuring that subsequent independent verification gates are enforced rather than bypassed by a single agent acting unilaterally.
4. **Generator Isolation:** The principle of exclusive write ownership extends to automated systems as well. Rule 26 strictly specifies that "Generated files may be written only by the admitted generator," preventing human agents from manually patching or drifting generated files without going through the authorized generator pipeline.
