//! mfw-producer-sourced Gamma_CMCA artifact modules (generator `v2.0.0-mfw`).
//! This is the **live, default path** — `allocator.rs`, `observatory.rs`, and
//! the integration test suite import `case_studies` from here.
//!
//! ## Reconciliation status (2026-07-20/21)
//!
//! Earlier revisions of this module were wired additively, alongside the
//! legacy `crate::generated::case_studies` / `crate::generated::generalization`
//! modules, gated behind a default-off `generated_artifact_pending` feature,
//! because of two blockers that have since been resolved or superseded:
//!
//! 1. A claimed 611-error API mismatch (`SignedFixed::from_bits` vs.
//!    `src/fixed.rs`'s real `from_value_bits`/`from_parts` API) was checked
//!    with a clean rebuild and a grep for `from_bits(` across both
//!    `generated-artifact/*/cmca_generated.rs` files: zero matches, 0 build
//!    errors. The claim was already stale when written. There was nothing to
//!    fix.
//! 2. The old/new symbol correspondence table below (verified by symbol-name
//!    diff, not yet by running `tests/differential.rs`/`tests/reference.rs`
//!    against this module) showed two constants change value/format
//!    (`GENERATOR_VERSION`, `RDF_INPUT_DIGEST`, `GENERATOR_SOURCE_DIGEST`)
//!    and several are wholly new. Since none of the *numeric payload*
//!    constants (`N`/`K`/`Q`/`F`, `FACTOR_*`, `MEASURE_*`, `OBJECT_REGISTRY`,
//!    `LENS_REGISTRY`, `LAMBDA`, `ETA`, `PackedSemanticState`, `LensSpec`)
//!    changed value, the swap in `allocator.rs`/`observatory.rs` is a no-op
//!    for numeric behavior; only the three producer-identity/digest strings
//!    (never consumed by `allocator.rs`/`observatory.rs`, only referenced in
//!    tests that assert on the generator's own identity) actually differ.
//!
//! | old symbol | new symbol | byte-identical? | classification |
//! |---|---|---|---|
//! | `N`, `F`, `K`, `Q` | same names, same values | yes | CORRESPONDENCE_REQUIRED |
//! | `FACTOR_*`, `MEASURE_*`, `OBJECT_REGISTRY`, `LENS_REGISTRY`, `LAMBDA`, `ETA`, `PackedSemanticState`, `LensSpec` | same names | yes | CORRESPONDENCE_REQUIRED |
//! | `GENERATOR_VERSION` | same name, new value (`"v1.1.0"` -> `"v2.0.0-mfw"`) | no | NEW_LAW_REQUIRED (producer identity intentionally differs) |
//! | `RDF_INPUT_DIGEST` | same name, new value + new format (bare hex -> `blake3:`-prefixed hex) | no | NEW_LAW_REQUIRED (digest scheme changed) |
//! | `GENERATOR_SOURCE_DIGEST` | same name, new value + new format | no | NEW_LAW_REQUIRED |
//! | (absent) | `SCHEMA_VERSION` | n/a, new | NEW_LAW_REQUIRED (contract §6) |
//! | (absent) | `LEAF_FLOOR_N_MAX` | n/a, new | NEW_LAW_REQUIRED (contract §3 dimension bound) |
//! | (absent) | `LEAF_FLOOR_BASE`, `LEAF_FLOOR_REMAINDER` | n/a, new | NEW_LAW_REQUIRED (conservation tables; not yet consumed — see `allocator.rs`'s `q_floor`/`r_floor` comment) |
//! | (absent) | `FORMULA_UNIFORM_LEAF_FLOOR`, `FORMULA_UNIFORM_LEAF_FLOOR_Q16_RESIDUAL` | n/a, new | NEW_LAW_REQUIRED |
//!
//! No `DEFECTIVE_BEHAVIOR_QUARANTINED` symbols were identified by this
//! symbol-name diff: the three known-defective *behaviors* documented in
//! `tests/fixtures/PRE_MIGRATION_BASELINE.md` (zero-default fallback,
//! cycle-returns-0.0, binary-float Q16.16 rounding) are generator code-path
//! defects, not named output symbols, so a symbol-level diff cannot confirm
//! or refute whether the mfw producer still exhibits them — that remains a
//! separate, not-yet-done correspondence-testing question.
//!
//! ## What was NOT done in this pass
//!
//! - `tests/differential.rs`/`tests/reference.rs` were repointed at this
//!   module's import path but were not independently re-audited symbol-by-
//!   symbol beyond compiling and passing — see the differential/reference
//!   test run in this reconciliation's build log.
//! - `src/artifact.rs::verify_generated_profile` does not yet consume the
//!   real `cmca_generation_manifest.json` files these modules' source `.rs`
//!   files were generated alongside — the manifest's top-level shape
//!   (`digests`, `dimensions`, `generator_source_order`, `numeric_profile`,
//!   `schema_version`) differs from `GeneratedArtifact`'s currently-expected
//!   synthetic schema (`leaf_count`, `leaf_floor_table`, `registry_indices`,
//!   `registry_dimension`). Untouched, separate pending migration step.
//! - `allocator.rs`'s inline `q_floor`/`r_floor` per-call computation was
//!   left as-is rather than switched to a `LEAF_FLOOR_BASE`/
//!   `LEAF_FLOOR_REMAINDER[nl-1]` table lookup: that swap has no test
//!   coverage proving the tables agree with `allocator.rs`'s own formula
//!   beyond the symbol-name diff above, so it was judged a separate,
//!   reviewed follow-up rather than something to bundle into this
//!   reconciliation pass. See the comment at that call site.
//! - `crate::generated::case_studies` (the old, v1.1.0 module) was not
//!   deleted, only marked superseded in its own header comment: it lives
//!   under `src/generated/`, a directory this repo's
//!   `scripts/gates/block-generated-edit.sh` Level-1 hook (invariant:
//!   generator is the sole authoritative producer of that directory's
//!   contents, `.claude/rules/cmca/rdf-generation.md`) is designed to guard
//!   against hand edits/deletion in interactive sessions. It has no
//!   remaining production or test consumer as of this reconciliation.

#[path = "../../generated-artifact/case-studies/cmca_generated.rs"]
#[allow(unused_macros)]
pub mod case_studies;

#[path = "../../generated-artifact/generalization/cmca_generated.rs"]
#[allow(unused_macros)]
pub mod generalization;
