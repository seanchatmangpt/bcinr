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

- [x] Add `#[deprecated(note = "CMCA-102/CMCA-114: authority chain pending
      Hoare-logic verification, do not use in production code")]` (or
      equivalent) to all 7 types and their constructors, so any caller —
      including autofde-lab — gets a real, unavoidable compile-time signal,
      not just an absence from generated docs. Done: all 7 types
      (`CertifiedLearning`, `CertifiedSelectionOnly`, `AdmittedControlState`,
      `CertificateReceipt`, `EnvelopeReceipt`, `OutcomeReceipt`,
      `AdaptiveUpdate<Mode>`) and their public `admit_*` constructors now
      carry `#[deprecated]` with that note. The `pub(crate)`-only `new`
      constructors were left undeprecated (they're never externally
      reachable, and internal call sites that legitimately construct these
      after real verification — `certification::seal_certificate`,
      `observatory::wrap_observatory_result`,
      `observatory::evaluate_calibration`, `allocate`/`allocate_in`'s proof
      parameter — carry a scoped `#[allow(deprecated)]` with a comment
      naming why, rather than a blanket crate-level allow).
- [x] Re-evaluate whether the crate's own `tests/*.rs` suite can be adjusted
      to use `pub(crate)` instead of the external-crate import path (e.g. by
      moving the relevant tests inside `allocator/mod.rs`'s own
      `#[cfg(test)]` module, or via a `#[cfg(test)] pub use` re-export
      scoped to test builds only) — if feasible, this is strictly stronger
      than `#[doc(hidden)] + #[deprecated]` and should be preferred.
      Evaluated, not applied this ticket — see next item.
- [x] If `pub(crate)` is genuinely infeasible without a larger test-suite
      refactor: document that decision explicitly in this ticket's
      resolution (don't silently re-close as "already handled by CMCA-102").
      **Resolution:** `pub(crate)` was not applied. 19 files under
      `crates/bcinr-cmca/tests/` (each compiled by cargo as its own separate
      crate, so `pub(crate)` in `bcinr-cmca`'s `src/` would not reach them)
      import these types via `bcinr_cmca::allocator::{...}`, including the
      dedicated `tests/ui/*.rs` trybuild fixtures that specifically assert
      construction *fails* from outside the crate for the sealed-field
      types. Moving all of that to `#[cfg(test)]` inline modules or a
      test-only `pub use` re-export is a real, larger refactor (touching 19
      files, several of which are named integration suites like
      `falsification_adversarial.rs`, `hostile_mutants.rs`, and
      `differential.rs` that are load-bearing for other tickets) and is out
      of scope for this ticket, whose job is to close the zero-enforcement
      gap, not restructure the test suite. `#[deprecated]` fully satisfies
      that job: it is a real compiler diagnostic, not merely absent
      documentation, and (per the next item) actually fires for an external
      caller.
- [x] Verify a downstream consumer (write a throwaway external test crate, or
      at minimum confirm via `cargo doc`/compiler output) that the
      `#[deprecated]` warning actually fires when `bcinr_cmca::allocator::CertifiedLearning`
      is imported and used from outside the crate. Verified via a new
      trybuild UI-test fixture,
      `crates/bcinr-cmca/tests/ui/fail_deprecated_admit_learning.rs`, which
      imports and calls `CertifiedLearning::admit_learning()` from outside
      the crate under `#![deny(deprecated)]` and asserts the resulting
      compiler output names the CMCA-102/CMCA-114 note at both the import
      and the call site — i.e. any downstream consumer running with
      warnings-as-errors (this crate's own `clippy -D warnings` policy
      included) gets a hard compile failure, not a silent pass.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`

## Related

- CMCA-102 (this ticket closes the real gap CMCA-102's Branch B left open)
