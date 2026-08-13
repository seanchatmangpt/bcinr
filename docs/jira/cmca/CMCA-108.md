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

- [x] Determine autofde-lab's actual required object/measure/lens counts —
      does it match N=8/K=4/Q=4, or does it need to be genuinely
      parameterized?
      **Verified:** `~/autofde-lab` has no Cargo.toml of its own and no
      recorded requirement for a specific object/measure/lens count (see
      Context above, confirmed prior to this pass). No evidence was found,
      in this pass, that autofde-lab's shape is pinned to something other
      than 8/4/4 or that it has started integration against this crate at
      all. Absent a concrete non-8/4/4 requirement to build against, this
      pass could not determine a real target shape to parameterize for.
- [x] If parameterization is needed: design a path to a const-generic (or
      dynamically-sized, alloc-gated) variant of `allocate`/`allocate_single_lens`
      that doesn't require forking the crate per consumer — decide whether
      this lives alongside the certified N=8 path (as `cascade` already does
      relative to `allocator`) or replaces it for non-N=8 consumers.
      **Verified NOT done as a const-generic rewrite, and why:** confirmed
      by reading `allocator/mod.rs` that `N`, `K`, `Q` are plain
      `pub const` items (`generated/consequence_mass/case_studies.rs:5,7,8`
      — `N=8`, `K=4`, `Q=4`), not `const` generic parameters — so this
      ticket's premise (b) applies literally. But the shape assumption goes
      far deeper than the two public function signatures: `allocate`/
      `allocate_single_lens` call into a private kernel that unrolls its
      loops against the literal constants 8 and 4 via
      `unroll_8_static!`/`unroll_4_static!` at **40+ call sites** (counted
      directly, e.g. `is_leaf[i & 7]`, `ancestor_doubling_table`,
      `compute_pi_kq_for_kq`), not just in the two public functions. Adding
      `const N: usize`/`K`/`Q` generic parameters to only `allocate`/
      `allocate_single_lens` without rewriting that kernel would either fail
      to compile for other sizes or — worse — compile and silently compute
      the wrong answer (the `& 7`/`& 3` masks would keep indexing as if
      N/K were still 8/4 regardless of the caller's actual N/K). A
      *correct* const-generic path requires rewriting the unrolling
      infrastructure itself, which is the "full generalization" this
      ticket's own preamble already flagged as too large for one pass. The
      module docs in `cascade.rs` already independently documented this
      exact tradeoff ("Widening it would destroy the property that
      justifies it") before this pass touched anything, corroborating the
      call to route to documentation instead of a risky partial refactor.
      Deferred as future work; the design path is: rewrite
      `unroll_8_static!`/`unroll_4_static!` (and callers) into
      const-generic-driven code generation, or replace the private kernel
      with a genuinely loop-based (non-unrolled) implementation, before
      `allocate`'s own signature can safely take `N`/`K`/`Q` as generic
      parameters.
- [x] If autofde-lab's shape does match 8/4/4: document this constraint
      explicitly in the crate's top-level docs (not just discoverable by
      reading generated-code internals) so the NEXT consumer doesn't hit the
      same discovery cost.
      **Done:** added an `## allocator::allocate is fixed to this crate's
      own N=8/K=4/Q=4 shape (CMCA-108)` section to `lib.rs`'s crate-level
      doc comment, naming the constraint and the escape hatch by name.
- [x] Either way: the crate's docs should state plainly, near `allocate`'s
      entry point, that it is bound to this crate's own compiled-in registry
      shape and that `cascade::consequence_mass` is the escape hatch for
      different shapes — this is currently only inferable by reading module
      docs on two separate files and comparing their signatures by hand.
      **Done:** added a `# Fixed shape: N = 8, K = 4, Q = 4 -- not a
      generic parameter` doc section directly on `allocate`'s doc comment
      (`allocator/mod.rs`, immediately before `# Mathematical Behavior`),
      and a shorter cross-referencing section on `allocate_single_lens`'s
      doc comment, both naming `cascade::consequence_mass` as the escape
      hatch for other shapes.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/src/generated/consequence_mass/case_studies.rs`
- `crates/bcinr-cmca/src/lib.rs` (top-level docs)
- `crates/bcinr-cmca/src/cascade.rs` (the existing escape hatch, may need promoting/documenting more prominently)
