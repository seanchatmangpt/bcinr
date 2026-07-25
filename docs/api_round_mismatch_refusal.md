### Findings on `RefusalSet::ROUND_MISMATCH`

I have investigated `RefusalSet::ROUND_MISMATCH` within `crates/bcinr-cmca/src/` (specifically in `allocator.rs`, `certification.rs`, `proposal.rs`, and the `REFUSAL_REALIZATION_REPORT.md`).

#### What structural condition it represents:
It represents the condition where a caller-supplied round identity does not match the round a chain artifact was produced for. Specifically, it applies when a certificate is sealed against a superseded round, or a proposal was made for a different round than what the caller supplied.

#### Exactly how this bitmask is set branchlessly:
**It is NOT set branchlessly, nor is it set at all within the authoritative hot path (`allocate()`).** 

As explicitly documented in `allocator.rs` and the `REFUSAL_REALIZATION_REPORT.md`:
* **Disposition**: `OWNED_BY_DIFFERENT_COMPONENT`
* No code path in `allocate()` constructs this bit. It appears only in its own `pub const` declaration and in `primary_reason()`'s read-only pattern match.
* The mismatch condition is instead realized upstream by two other modules using typed return types (`Result::Err`), rather than through branchless bitmasks:
  1. `proposal::ProposalRefusal::RoundIdentityMismatch` (when `proposal.round_identity != expected_round_identity`)
  2. `certification::CertificationRefusal::RoundIdentityMismatch` (when verifying the sealed bindings in `seal_certificate`)

Since these upstream modules handle the refusal before reaching the `CC=1` strictly branchless `allocate()` hot-path, they utilize explicit branching (early returns) and typed enums. The `ROUND_MISMATCH` bitmask in `allocator::RefusalSet` exists for vocabulary alignment (via `primary_reason()`) but is never actually constructed by the branchless allocator.
