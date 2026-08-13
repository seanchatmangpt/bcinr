# CMCA-108: allocate()/allocate_single_lens() are hard-locked to compile-time N=8/K=4/Q=4 — likely blocks autofde-lab integration

**Type:** Bug / Design Gap
**Priority:** Critical (blocking for the stated near-term goal: autofde-lab production integration)

## Summary

`allocate`, `allocate_in`, and `allocate_single_lens` all take fixed-size arrays
keyed to this crate's own compile-time constants (`N=8`, `K=4`, `Q=4`, from
`crates/bcinr-cmca/src/generated/consequence_mass/case_studies.rs:5,7,8`), not
generic over `const N: usize`. If autofde-lab's real data shape needs a
different object count, measure count, or lens count, the certified allocation
path (`allocate`/`allocate_single_lens`) cannot be called at all without
forking or regenerating this crate against a different ontology. Only
`cascade::consequence_mass` (arbitrary tree shape, no fixed N) escapes this
constraint.

## Context

Found by an adversarial cross-cutting review of the crate's public API surface
as a downstream integrator (autofde-lab) would encounter it, immediately after
this session's CMCA-101 through 107 hardening work landed on `main`.

`allocator/mod.rs:2012-2029` (`allocate`) and `:2119-2135`
(`allocate_single_lens`) both take `&[PackedSemanticState; N]`, `&[LensSpec; Q]`,
`&[[NonNegativeFixed; Q]; K]` — `PackedSemanticState`/`LensSpec` are themselves
generated types from this crate's own ontology pipeline
(`lib.rs:96-98`'s doc comment on `generated_profile`). A caller CAN construct
their own `[PackedSemanticState; 8]` with different factor *values* (the
struct fields aren't opaque), but cannot change the *shape* (8 objects, 4
measures, 4 lenses) without this crate itself being regenerated against a
different `ontology/*.ttl`.

Confirmed: autofde-lab (`~/autofde-lab`) is a separate git repo, a Python
project (`pyproject.toml`, no project-owned `Cargo.toml` — only vendored
third-party fixtures under `vendor/gyms/` reference Cargo at all). Integrating
this crate is a net-new Rust/Cargo build surface for that project, not a
drop-in dependency bump — and whether the certified path is even usable
depends entirely on whether autofde-lab's real registry shape happens to
match 8/4/4.

This must be resolved or explicitly acknowledged (with a real decision on
which entry point autofde-lab will actually use) **before** integration work
starts, not discovered mid-integration.

## Acceptance Criteria

- [ ] Determine autofde-lab's actual required object/measure/lens counts —
      does it match N=8/K=4/Q=4, or does it need to be genuinely
      parameterized?
- [ ] If parameterization is needed: design a path to a const-generic (or
      dynamically-sized, alloc-gated) variant of `allocate`/`allocate_single_lens`
      that doesn't require forking the crate per consumer — decide whether
      this lives alongside the certified N=8 path (as `cascade` already does
      relative to `allocator`) or replaces it for non-N=8 consumers.
- [ ] If autofde-lab's shape does match 8/4/4: document this constraint
      explicitly in the crate's top-level docs (not just discoverable by
      reading generated-code internals) so the NEXT consumer doesn't hit the
      same discovery cost.
- [ ] Either way: the crate's docs should state plainly, near `allocate`'s
      entry point, that it is bound to this crate's own compiled-in registry
      shape and that `cascade::consequence_mass` is the escape hatch for
      different shapes — this is currently only inferable by reading module
      docs on two separate files and comparing their signatures by hand.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/src/generated/consequence_mass/case_studies.rs`
- `crates/bcinr-cmca/src/lib.rs` (top-level docs)
- `crates/bcinr-cmca/src/cascade.rs` (the existing escape hatch, may need promoting/documenting more prominently)
