### Definition
`ObjectCodeAuditFailed` is a bounded **typed refusal code** defined by the repository's strict constitution (Rule 18 in `AGENTS.md`). It acts as a physical security perimeter bridging static verification and runtime execution. 

Interestingly, it does **not** currently exist as a concrete Rust enum variant in the primary crates (e.g., `bcinr-cmca` or `bcinr-api`). Instead, it is heavily documented as a **CI gate concept and constitutional rule** enforced at the machine-code level.

### How it Triggers
It conceptually triggers under **Rule 20 (Object-code audit)** when the `@turing_machine` (Enforcer of Determinism) analyzes the final production-profile disassembly. It fails if the compiler (such as LLVM) has implicitly inserted any of the following into the authoritative hot path, despite the Rust source code appearing branchless (`CC=1`):
* Conditional jumps
* Loop backedges
* Panic paths
* Allocators

### Substrate Lockdown & Consequences
When an `ObjectCodeAuditFailed` refusal is triggered, the substrate mathematically prevents the execution and locks down the repository through compounding constitutional mechanisms:

1. **Mask-Based Admission Refusal (Rule 10)**: The unverified logic fails admission. The admission mask evaluates deterministically to `0`. The transaction `next_state = select(admitted_mask, candidate_state, current_state)` rigidly selects `current_state`, bit-for-bit rejecting the unverified logic without branches.
2. **Failure of ReceiptSound Law (Rule 11)**: Without a valid assembly verification manifest, the system cannot verify an `AcceptedCertificate`. The required condition `AdmittedControlState ∧ AcceptedCertificate` fails, branchlessly freezing adaptive learning modes.
3. **Substrate Integrity Score Drop (Rule 24)**: Any failed object-code audit is an absolute failure, plunging the Substrate Integrity Score (SIS) to `0` regardless of other weighted averages.
4. **MaturityScrutiny Quarantine (Rule 25)**: An `SIS = 0` locks the repository, forcing feature development to freeze. Code is quarantined until a root-cause report is produced, structural defects are repaired, and all artifacts are regenerated to issue a new standing receipt.
