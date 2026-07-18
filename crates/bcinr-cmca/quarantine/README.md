# quarantine/ — bcinr-cmca legacy RDF machinery

Last Updated: 2026-07-17

## What this is

This directory holds the legacy in-tree CMCA RDF admission/generation machinery,
quarantined (moved, not deleted) now that the producer role has moved to the `mfw`
repository and the artifact-consumption scaffold exists in this crate
(`src/artifact.rs`, `generated-artifact/`).

- `legacy-generator/generator.py` — the original in-tree generator
  (`crates/bcinr-cmca/generator.py`, `GENERATOR_VERSION = "v1.1.0"`), moved verbatim.
- `legacy-ontology/` — the original in-tree ontology sources
  (`crates/bcinr-cmca/ontology/cmca-rdf.ttl`, `generalization.ttl`), moved verbatim.

Both were moved with `git mv`, preserving history and byte content. Nothing in this
directory was edited as part of the quarantine.

## Why

Per `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` and
`.claude/rules/cmca/artifact-boundary.md`, CMCA RDF admission and code generation is
now owned by the producer at `/Users/sac/mfw/tools/cmca-generator` (generator_version
`v2.0.0-mfw`, reproduced independently — see
`crates/bcinr-cmca/generated-artifact/PRODUCER_REPRODUCTION.md`). bcinr-cmca's role is
now consumer-only: it verifies already-generated artifact bytes
(`src/artifact.rs::verify_generated_profile`) and never runs RDF/SHACL/Python
generation logic itself. `.claude/rules/cmca/rdf-generation.md` also documents three
known invariant violations in the legacy generator (zero-default fallback, silent
cycle-to-zero, binary-float Q16.16 rounding) that this quarantine does not fix — it is
superseded, not repaired.

## Status

- **Superseded by:** `/Users/sac/mfw/tools/cmca-generator` (the mfw producer).
- **Kept for:** historical reference and as the source for the frozen
  byte-equivalence fixtures at `tests/fixtures/pre_migration/` (see
  `tests/fixtures/PRE_MIGRATION_BASELINE.md`), which were captured from this exact
  `generator.py` + `ontology/` pair before the move.
- **Not reachable from production build:** nothing under `Cargo.toml`, `src/`
  (outside the `#[cfg(test)]`-gated `src/artifact.rs`), or `Makefile.toml`'s
  `verify-generated` task now references `quarantine/legacy-generator/generator.py`
  or `quarantine/legacy-ontology/`. `crates/bcinr-cmca/Cargo.toml` has no Python/RDF
  dependency and never did (the crate is Rust-only; `generator.py` was invoked
  out-of-band by `make`/CI shell scripts, never `cargo build`).

## Non-claims

This quarantine does not certify that `src/artifact.rs::verify_generated_profile`
already validates the *real* mfw-emitted manifest shape — as documented in
`src/artifact.rs`'s own module docs, that function currently checks a synthetic
hand-crafted JSON schema mirroring the contract's described fields
(`leaf_count`, `leaf_floor_table`, `registry_indices`, `registry_dimension`), and the
actual `generated-artifact/*/cmca_generation_manifest.json` files produced by the mfw
producer use a different shape (`digests`, `dimensions`, `generator_source_order`,
`numeric_profile`, `schema_version` — confirmed by inspection in this task). Wiring
`verify_generated_profile` to the real manifest shape is a separate, not-yet-done
migration step.

## See Also

- `crates/bcinr-cmca/src/artifact.rs` — the new consumer-side verification module
- `crates/bcinr-cmca/generated-artifact/PRODUCER_REPRODUCTION.md` — reproduction of
  the mfw producer prior to hand-off
- `crates/bcinr-cmca/tests/fixtures/PRE_MIGRATION_BASELINE.md` — the frozen
  byte-equivalence baseline captured from this quarantined generator/ontology pair
- `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` — the Gamma_CMCA producer/consumer
  contract (v1.1)
- `.claude/rules/cmca/artifact-boundary.md`, `.claude/rules/cmca/rdf-generation.md`
