//! mfw-producer-sourced Gamma_CMCA artifact modules, exposed alongside the
//! legacy `crate::generated::case_studies` / `crate::generated::generalization`
//! modules (not in their place).
//!
//! ## Why this lives here and not in `src/generated/mod.rs`
//!
//! The task for this phase specified wiring these modules into
//! `crates/bcinr-cmca/src/generated/mod.rs`. That edit is blocked by this
//! repo's own Level-1 hook gate, `scripts/gates/block-generated-edit.sh`,
//! which denies any Edit/Write whose path contains
//! `crates/bcinr-cmca/src/generated/` — enforcing the invariant (stated in
//! `.claude/rules/cmca/rdf-generation.md`) that the generator is the sole
//! authoritative producer of files under that directory. This task's hard
//! constraints forbid gate skipping, so rather than bypass the hook, the
//! same modules are declared here instead, one directory up, reachable as
//! `bcinr_cmca::generated_artifact::case_studies` /
//! `bcinr_cmca::generated_artifact::generalization`. This is a pending
//! integration point: actually wiring `src/generated/mod.rs` (or retiring
//! the legacy modules it currently exposes) requires either a gate-policy
//! change or a role authorized to edit that directory — out of scope here.
//!
//! ## Why alongside, not in place, even setting the gate aside
//!
//! `allocator.rs` and `observatory.rs` (owned by sibling tasks in this same
//! phase, not touched here) import
//! `crate::generated::case_studies::{N, K, Q, PackedSemanticState, LensSpec,
//! OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, ...}` directly. A structural
//! symbol-name diff of the legacy vs. mfw-producer output shows the new
//! artifact is a superset of symbol *names*, but two existing constants
//! change *value* (and one changes *format*):
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
//! | (absent) | `LEAF_FLOOR_BASE`, `LEAF_FLOOR_REMAINDER` | n/a, new | NEW_LAW_REQUIRED (conservation tables `src/artifact.rs::verify_generated_profile` targets) |
//! | (absent) | `FORMULA_UNIFORM_LEAF_FLOOR`, `FORMULA_UNIFORM_LEAF_FLOOR_Q16_RESIDUAL` | n/a, new | NEW_LAW_REQUIRED |
//!
//! No `DEFECTIVE_BEHAVIOR_QUARANTINED` symbols were identified by this
//! symbol-name diff: the three known-defective *behaviors* documented in
//! `tests/fixtures/PRE_MIGRATION_BASELINE.md` (zero-default fallback,
//! cycle-returns-0.0, binary-float Q16.16 rounding) are generator code-path
//! defects, not named output symbols, so a symbol-level diff cannot confirm
//! or refute whether the mfw producer still exhibits them — that is a
//! separate, not-yet-done correspondence-testing question.
//!
//! Given a real value/format change on two existing constant names, an
//! in-place swap would silently change what `allocator.rs`/`observatory.rs`
//! consume without their owning tasks having reviewed it — so these modules
//! are additive-only here, not a replacement.
//!
//! ## Non-claims
//!
//! This mapping was produced by a structural symbol-name diff
//! (`grep -oE '^pub (const|fn|struct|type|static) ...'` over both generated
//! files) plus a manual read of the `GENERATOR_VERSION`/`RDF_INPUT_DIGEST`
//! lines — not by running the differential/reference test suites
//! (`tests/differential.rs`, `tests/reference.rs`) against these new
//! modules, which still target the legacy modules exclusively.
//!
//! Separately: `src/artifact.rs::verify_generated_profile` does not yet
//! consume the real `cmca_generation_manifest.json` files these modules'
//! source `.rs` files were generated alongside — inspection in this task
//! confirmed the real manifest's top-level shape (`digests`, `dimensions`,
//! `generator_source_order`, `numeric_profile`, `schema_version`) differs
//! from `GeneratedArtifact`'s currently-expected synthetic schema
//! (`leaf_count`, `leaf_floor_table`, `registry_indices`,
//! `registry_dimension`). That is a separate, pending migration step.

#[path = "../../generated-artifact/case-studies/cmca_generated.rs"]
pub mod case_studies;

#[path = "../../generated-artifact/generalization/cmca_generated.rs"]
pub mod generalization;
