# CMCA-114: #[doc(hidden)] gives zero compile-time enforcement — the "certified" authority chain is a reachable, unconditional rubber-stamp

**Type:** Bug / Security-relevant Design Gap
**Priority:** High (autofde-lab integration is imminent and would reach this exact path)

## Summary

CMCA-102's Branch B implementation (`#[doc(hidden)]` on the 7 authority-chain
types) suppresses their appearance in generated rustdoc but changes **nothing**
about their actual Rust visibility, constructibility, or usability — this is
documented, unambiguous language semantics, confirmed by direct inspection.
Every one of the `admit_*` constructors is an unconditional rubber stamp
(e.g. `admit_control_state(digest: u64) -> Self { Self { digest } }` accepts
any `u64` and calls it "admitted," with zero verification). A downstream
crate — including autofde-lab, importing `bcinr_cmca::allocator::*` exactly as
this crate's own 14 integration-test files already do — can construct and use
the entire "certified"-sounding chain today with zero compiler warning, zero
error, zero lint. `#[doc(hidden)]` addresses the *documentation-visibility*
symptom CMCA-102 was scoped to, not the *premature/unverified use* problem its
own Summary/Context text names as the actual concern.

## Context

Found by adversarial review of CMCA-102's implementation, specifically by
checking what `#[doc(hidden)]` actually enforces versus what the ticket's own
premise (types "withheld... pending formal Hoare-logic verification") implies
is needed.

- `crates/bcinr-cmca/src/allocator/mod.rs:784,808,832,856,880,904,931` — all 7
  types are `pub struct` inside `pub mod allocator` (`lib.rs:96`), no
  `pub(crate)`, no `#[deprecated]`, no feature gate.
- Constructors are all zero-verification: `admit_learning` (:797),
  `admit_selection_only` (:821), `admit_control_state` (:845),
  `admit_certificate` (:869), `admit_envelope` (:893), `admit_outcome` (:917)
  each unconditionally return `Self { ... }`. Only `admit_adaptive_update`
  (:954) does real numeric validation — and even that accepts pre-fabricated
  tokens from the zero-check constructors above.
- Reachability confirmed as already-exercised, not hypothetical: 14
  integration test files in `crates/bcinr-cmca/tests/` already import these
  types via the exact external-crate path (`bcinr_cmca::allocator::{...}`)
  autofde-lab would use.
- CMCA-102's own acceptance criteria offered `pub(crate)` as the primary
  option, with `#[doc(hidden)] pub` only "if external-crate-internal
  reachability is required" — the implementer picked the weaker option
  citing the existing test suite's reliance on the external-crate import
  path, a real constraint, but one that could have been resolved by
  refactoring those tests to use in-module (`super::`) access instead of
  requiring `pub`.
- The cheapest available middle ground — `#[deprecated(note = "...")]`, which
  keeps items usable (doesn't break the test suite) but emits a compiler
  warning at every call site, including a downstream consumer's — was not
  used and was available within Branch B's own stated options.

## Acceptance Criteria

- [ ] Add `#[deprecated(note = "CMCA-102/CMCA-114: authority chain pending
      Hoare-logic verification, do not use in production code")]` (or
      equivalent) to all 7 types and their constructors, so any caller —
      including autofde-lab — gets a real, unavoidable compile-time signal,
      not just an absence from generated docs.
- [ ] Re-evaluate whether the crate's own `tests/*.rs` suite can be adjusted
      to use `pub(crate)` instead of the external-crate import path (e.g. by
      moving the relevant tests inside `allocator/mod.rs`'s own
      `#[cfg(test)]` module, or via a `#[cfg(test)] pub use` re-export
      scoped to test builds only) — if feasible, this is strictly stronger
      than `#[doc(hidden)] + #[deprecated]` and should be preferred.
- [ ] If `pub(crate)` is genuinely infeasible without a larger test-suite
      refactor: document that decision explicitly in this ticket's
      resolution (don't silently re-close as "already handled by CMCA-102").
- [ ] Verify a downstream consumer (write a throwaway external test crate, or
      at minimum confirm via `cargo doc`/compiler output) that the
      `#[deprecated]` warning actually fires when `bcinr_cmca::allocator::CertifiedLearning`
      is imported and used from outside the crate.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`

## Related

- CMCA-102 (this ticket closes the real gap CMCA-102's Branch B left open)
