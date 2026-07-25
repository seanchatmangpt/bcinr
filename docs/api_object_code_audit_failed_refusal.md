I have searched for `ObjectCodeAuditFailed` in `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/`, and here are my findings.

### Search Results
I found **zero occurrences** of `ObjectCodeAuditFailed` in the Rust source code for both `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/` (even with a case-insensitive search). 

However, I found it extensively documented in `AGENTS.md` and the `docs/` directory (`docs/object_code_audit_failed_refusal.md` and `docs/rule_18_object_code_audit_failed_refusal.md`).

### Conclusion
`ObjectCodeAuditFailed` is currently purely a **CI gate concept and constitutional rule** described in `AGENTS.md`, rather than a concrete, implemented Rust enum variant in those crates.

### How it operates conceptually (from the architecture docs)
According to the repository's constitutional docs, `ObjectCodeAuditFailed` represents a structural compliance failure at the machine-code level (Rule 20). It occurs if the `@turing_machine` role detects that the compiler (e.g., LLVM) implicitly inserted conditional jumps, allocators, panics, or loop backedges into the release-profile disassembly.

If triggered or evaluated at runtime, the documentation states it rejects the operation **branchlessly** using structural checks:
1. **Mask-Based State Selection (Rule 10):** The admission mask evaluates to `0`. The transition logic `next_state = select(admitted_mask, candidate_state, current_state)` mechanically selects `current_state`, bit-for-bit rejecting the unverified logic without using any `if` statements.
2. **Failure of the ReceiptSound Law (Rule 11):** Without a successful audit manifest, the system cannot verify an `AcceptedCertificate`. The condition `AdmittedControlState ∧ AcceptedCertificate` evaluates to false/0, which branchlessly freezes the adaptive learning state.

In practice, this is strictly enforced by the CI pipeline (`cargo make audit-object-code`), where any failure acts as an absolute violation (Substrate Integrity Score drops to 0) and triggers a `MaturityScrutiny` lockdown (Rule 25).
