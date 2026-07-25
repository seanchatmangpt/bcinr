# Non-Self Certification Architecture in BCINR

The Non-Self Certification architecture in BCINR guarantees that mathematical proofs, authoritative implementations, structural checks, and hostile verifications remain completely isolated and mechanically verifiable. This eliminates trust, confirmation bias, and "agent consensus" by replacing them with rigid mathematical and structural proofs.

## 1. Segregation of Duties (Rule 5: Mandatory Decomposition Protocol)
Every nontrivial implementation task is immediately shattered into four distinct, non-overlapping workstreams. This forms a check-and-balance system with deliberately conflicting incentives:
* **`@hoare_oracle` (Mathematical Law):** Owns the axiomatic proofs, Hoare contracts, pre/postconditions, and the independent reference semantics.
* **`@von_neumann_bypass` (Implementation):** Owns the authoritative branchless, bounded, zero-allocation code.
* **`@turing_machine` (Structural Enforcement):** Owns the structural auditing (CC=1 enforcement, object code verification) and acts as the gatekeeper.
* **`@armstrong_fault` (Hostile Verification):** Owns adversarial mutants and ensures corrupted code strictly triggers a typed refusal or oracle mismatch.

## 2. State Isolation and Exclusive Write Ownership (Rule 26)
Write ownership is practically enforced by strict state isolation. Shared-file concurrent editing is strictly prohibited. 
* `@hoare_oracle` is the **exclusive writer** for contracts and proofs.
* `@von_neumann_bypass` is the **exclusive writer** for authoritative implementations.
* `@turing_machine` is the **exclusive writer** for scanners and structural gates.
* `@armstrong_fault` is the **exclusive writer** for mutants and hostile fixtures.

Other agents may review but **cannot edit** files outside their domain without an explicit ownership transfer recorded in the work log. This physical isolation prevents an implementer from subtly altering a mathematical proof to accommodate a flawed implementation, prevents "circular oracles", and prohibits an auditor from silently fixing code and self-approving their own repair.

## 3. Structural Enforcement of No Self-Certification (Rule 27 & ConstitutionIR)
Rule 27 declares that "Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim." The implementation agent (`@von_neumann_bypass`) can never act as the final approver for mathematical correctness, branchlessness, mutant adequacy, or standing. Every approval must come from a separate role and be backed by a generated mechanical artifact.

Beyond prose discipline, this rule is modeled and enforced systematically at the schema level via **ConstitutionIR** (the machine-readable intermediate representation for BCINR claims):
* Every formalized claim schema requires an `owner` (the role implementing/proposing satisfaction) and a `verifier` (the role checking satisfaction).
* The ConstitutionIR schema introduces an absolute constraint: `verifier != owner`. 
* Because JSON schema cannot natively express cross-field inequality natively, this `verifier != owner` rule is rigidly enforced via a second-pass structural validator (or via `$data` extensions like `ajv`). 
* Any artifact or claim document where the verifier is the same as the owner is rejected by the schema—meaning a self-certified claim is treated as fundamentally invalid IR and will block compilation.

Through mandatory decomposition, exclusive domain write ownership, and schema-level validation of distinct roles, the BCINR architecture ensures that definitions of correctness, implemented logic, and adversarial testing collide as objectively verified artifacts rather than assumptions.
