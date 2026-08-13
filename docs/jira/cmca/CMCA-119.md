# CMCA-119: no runnable example, five overlapping entry points, no unified error trait — onboarding gap right as a real consumer arrives

**Type:** Design Gap / Onboarding
**Priority:** Medium (non-blocking individually, but compounds CMCA-108 for autofde-lab)

## Summary

Three related onboarding gaps found while reviewing the crate's API surface
as a downstream integrator (autofde-lab) would encounter it:

1. **No runnable example anywhere in the crate.** `examples/test_drive.rs`
   was deleted this session (superseded by assertion-only tests); there is no
   `examples/` directory at all today. The only doc-tested example
   (`lib.rs:29-79`) walks through `observatory::evaluate_calibration`, never
   touching `allocate`/`allocate_single_lens`/`escort_distribution`/
   `cascade::consequence_mass` — the actual allocation entry points a new
   integrator needs.
2. **Five overlapping entry points** (`allocate`, `allocate_in`,
   `allocate_single_lens`, `escort::escort_distribution`,
   `cascade::consequence_mass`) with no single "start here" guide — each is
   individually documented (module docs do explain *why* each variant
   exists), but a new integrator has to read and cross-reference several
   files to figure out which one to actually call.
3. **No unified error-handling trait.** At least 9 files each define their
   own `pub enum ...Refusal` (`StabilityRefusal`, `HierarchyRefusal`,
   `LensSelectionRefusal`, `AllocationRefusal`, `CertificationRefusal`, and
   others in `proposal.rs`, `observatory.rs`, `cascade.rs`,
   `reference_escort.rs`) — none implement `std::error::Error` or `Display`.
   A downstream consumer gets `Debug`-only enums and must write bespoke
   `match` arms per refusal type with no common `?`-propagatable error path.

## Context

Found by adversarial cross-cutting review of the crate's public API surface,
explicitly framed around the user's stated near-term goal (autofde-lab
integration).

## Acceptance Criteria

- [ ] Add at least one real, runnable example (in `examples/` or a
      prominent doc-test) demonstrating a realistic end-to-end flow: build a
      tree/registry, call the appropriate entry point (pick one and justify
      the choice in the example's own comments), interpret the result, and
      (optionally) seal a receipt via `allocation_receipt`.
- [ ] Add a short "which entry point do I want" section to `lib.rs`'s
      top-level module docs, pointing to the relevant module docs for detail
      rather than duplicating them — this is a navigation aid, not new
      prose.
- [ ] Evaluate whether the crate's various `...Refusal` enums can reasonably
      implement `std::error::Error`/`Display` (likely via a shared derive
      macro or a hand-written impl per enum) — if genuinely infeasible given
      the crate's `no_std` stance in some configurations, document why and
      what the recommended downstream handling pattern is instead.

## Files likely touched

- `crates/bcinr-cmca/examples/` (new)
- `crates/bcinr-cmca/src/lib.rs`
- Every file defining a `...Refusal` enum, if the `Error`/`Display` fix is taken on

## Related

- CMCA-108 (the N=8/K=4/Q=4 lock-in finding from the same review pass — a
  new example/guide should make this constraint visible up front, not
  something a new integrator discovers by trial and error)
