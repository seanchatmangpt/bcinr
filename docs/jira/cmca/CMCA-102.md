# CMCA-102: bcinr-cmca: authority chain (CertifiedLearning et al.) held unexported pending Hoare-logic verification

**Type:** Tech Debt / Verification
**Priority:** Low
**Status:** **Done (Branch B)** — Branch A (the actual Hoare-logic proof) is out of
reach for an automated session and remains open work. Branch B implemented: all 7
types stay `pub` (a grep of real usage found this crate's own integration-test
suite already reaches them via `bcinr_cmca::allocator::*` as an external crate, so
`pub(crate)` would break the suite) but each now carries `#[doc(hidden)]` plus a
doc comment naming CMCA-102 and the real blocker. Verified via `cargo doc`: none
of the 7 types appear in generated public rustdoc. Two `trybuild` `.stderr`
fixtures needed updating (the hidden attribute changes how rustc prints the
type's path in diagnostics) — updated via `TRYBUILD=overwrite`, compile-fail
assertions themselves unchanged.

## Summary

crates/bcinr-cmca/src/lib.rs (~lines 123-129) carries an INTEGRATION NOTE (v26.7.24) fencing off a full recovery authority chain -- CertifiedLearning, CertifiedSelectionOnly, AdmittedControlState, CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, AdaptiveUpdate -- all real, substantial types already implemented in allocator.rs, deliberately withheld from the crate's public API. The stated reason is not "not ready to design" but a named, unfinished formal-verification dependency: Hoare-logic proof of the authority chain's dependency closure, which also gates a planned StabilityRefusal extension (CMCA_LEARNING_FROZEN and related typed refusals). Until that proof lands, the chain is real code with no reachable caller inside or outside the crate.

## Context

No correctness risk today: nothing external can reach these types, so leaving the fence in place cannot break any caller. The risk is entirely one of legibility over time -- a `pub use` boundary silently drawn around ~7 types with a comment as the only signal reads, to a future maintainer or auditor, as unfinished/abandoned work rather than as a deliberate, tracked gate. The blocking dependency is the Hoare-logic verification effort itself (proving the authority chain's dependency closure so CertifiedLearning/CertifiedSelectionOnly/AdmittedControlState and the receipt types can be composed and exposed safely, per this project's PhD Gates discipline in phd_gates.md and the precedent of the 3 audited unsafe blocks in SAFETY.md) -- not a trivial "add pub use" fix. This ticket does not attempt that proof; it tracks the decision point and gives the fence a permanent home either way.

## Acceptance Criteria

- [ ] Branch A (verification completes): Hoare-logic proof of dependency closure for the authority chain (CertifiedLearning, CertifiedSelectionOnly, AdmittedControlState, CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, AdaptiveUpdate) is written up per this project's PhD Gates / SAFETY.md conventions and linked from the crate docs.
- [ ] Branch A: the chain is exported via `pub use` from bcinr-cmca::lib.rs, with `///` doc examples on each newly public type per CLAUDE.md's public-API documentation standard.
- [ ] Branch A: StabilityRefusal gains the planned typed refusal variants (CMCA_LEARNING_FROZEN and siblings named in the current INTEGRATION NOTE), with unit tests exercising each new variant.
- [ ] Branch A: the INTEGRATION NOTE comment is removed and replaced with a doc comment on the export site referencing the completed verification artifact.
- [ ] Branch B (defer explicitly): the currently-implicit fence (no `pub use`) is replaced with explicit `pub(crate)` visibility (or `#[doc(hidden)] pub` if external-crate-internal reachability is required) on each of the 7 types in allocator.rs.
- [ ] Branch B: a doc comment at the visibility boundary states the gate is intentional, names this ticket (or its permanent tracking ID) as the reference, and names the Hoare-logic verification as the unblocking condition -- so a future reader sees a designed gate, not an oversight.
- [ ] Branch B: `cargo doc` output for bcinr-cmca is inspected to confirm the gated types no longer appear as ambiguously-absent public API (i.e., either genuinely invisible via pub(crate), or visibly marked hidden-by-design if doc(hidden) is used).
- [ ] Either branch: `make check && make test && make clippy && make fmt` pass with no new warnings introduced by the visibility change.
- [ ] This ticket is closed only when one branch's criteria are fully met; partial completion (e.g., proof started but not landed) keeps the ticket open with Branch A criteria as the tracked remainder.

## Files likely touched

- `/Users/sac/bcinr/crates/bcinr-cmca/src/lib.rs`
- `/Users/sac/bcinr/crates/bcinr-cmca/src/allocator.rs`
- `/Users/sac/bcinr/crates/bcinr-cmca/phd_gates.md`
- `/Users/sac/bcinr/crates/bcinr-logic/src/SAFETY.md`
