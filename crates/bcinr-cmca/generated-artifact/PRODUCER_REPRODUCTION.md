# PRODUCER_REPRODUCTION.md

Last Updated: 2026-07-17

## Purpose

Independent reproduction of the MFW CMCA producer (`/Users/sac/mfw/tools/cmca-generator/generator.py`,
`generator_version: v2.0.0-mfw`) prior to BCINR consuming its output, per task instructions.
This is a consumer-side reproduction check, not a claim of correctness of the generator's
semantics — only of exit status, determinism, and digest agreement across independent runs.

## Note on filenames

The generator's actual output filenames are `cmca_generated.rs`, `cmca_generation_manifest.json`,
and `cmca_generation_receipt.json` (not the bare `manifest.json` / `receipt.json` named in the
task prompt). These are the real files produced; no renaming was done during hand-off.

## Commands run

Working directory: `/Users/sac/mfw/tools/cmca-generator/`

```
python3 generator.py /Users/sac/mfw/mfw-ontology/cmca/cmca-rdf.ttl <out_dir_A>
python3 generator.py /Users/sac/mfw/mfw-ontology/cmca/cmca-rdf.ttl <out_dir_B>
python3 generator.py /Users/sac/mfw/mfw-ontology/cmca/generalization.ttl <out_dir_A>
python3 generator.py /Users/sac/mfw/mfw-ontology/cmca/generalization.ttl <out_dir_B>
```

All four invocations used fresh, previously-nonexistent temp output directories under
`/private/tmp/claude-501/-Users-sac-bcinr/7304b26a-5021-4a1e-a164-a649277d1670/scratchpad/repro/`.

## Exit codes

| Ontology | Run | Exit code | Refusal |
|---|---|---|---|
| cmca-rdf.ttl (case-studies) | A | 0 | none |
| cmca-rdf.ttl (case-studies) | B | 0 | none |
| generalization.ttl | A | 0 | none |
| generalization.ttl | B | 0 | none |

`result` field in every `cmca_generation_receipt.json` was `"success"`.

## Six digests per manifest

### case-studies (cmca-rdf.ttl), N=8 K=4 Q=4 F=10

| Digest | Value |
|---|---|
| rdf_digest | blake3:d41228c35e898bea45cc43d3204dc5bf03003a970b473eea9953bdde5ad4c896 |
| admission_digest | blake3:494df59a72ac5f0a2f0e02bf5ffb4e68df998a169c00c29344cd64f7b28413e3 |
| generator_digest | blake3:941277d9848d80c9a0e2f4e470d7eb0139e434c2c731601e340831a66ab50b64 |
| numeric_profile_digest | blake3:a321b76a4c8143036dbcda620fd24d439aeb4febfea9165c1d14de73c32715fc |
| formula_registry_digest | blake3:fc96ee0c6239925cb960b12ce8e58ff9f019d720d3791ea3b90702fb323d20b3 |
| generated_payload_digest | blake3:308b5a92b83a91355150ebde5541215a9996f6d124dcd136c19266995a06e4ce |

Identical across run A and run B (verified by diffing the `digests` object).

### generalization (generalization.ttl), N=9 K=5 Q=5 F=10

| Digest | Value |
|---|---|
| rdf_digest | blake3:36d791cd8a6089994dd9610dae370d935d1c915a8c62923f69a4a717e1601809 |
| admission_digest | blake3:24f67f2b87768e69ac5565ffe8ba09fe832ae25c9a2ac5a7a15cfeb88f2a5719 |
| generator_digest | blake3:941277d9848d80c9a0e2f4e470d7eb0139e434c2c731601e340831a66ab50b64 |
| numeric_profile_digest | blake3:a321b76a4c8143036dbcda620fd24d439aeb4febfea9165c1d14de73c32715fc |
| formula_registry_digest | blake3:fc96ee0c6239925cb960b12ce8e58ff9f019d720d3791ea3b90702fb323d20b3 |
| generated_payload_digest | blake3:6d443ce9c27072901a54fea3b1de160be002fb650ce3c30d6a95d44f4b4f23af |

Identical across run A and run B (verified by diffing the `digests` object).

Note: `generator_digest`, `numeric_profile_digest`, and `formula_registry_digest` are identical
between the two ontologies, as expected — they are properties of the generator/toolchain and
formula registry, not of the input RDF. `rdf_digest`, `admission_digest`, and
`generated_payload_digest` differ between ontologies, as expected since the two input files and
their derived payloads differ.

## Determinism check

Two independent invocations per ontology, into differently-named output directories, then a
follow-up pair into identically-named directory basenames to eliminate a confound (the receipt's
embedded `command` array records the literal output-dir argument, so differently-named dirs
produce cosmetically different `receipt.json` / `receipt_digest` even when all real outputs are
identical).

- `cmca_generated.rs`: SHA-256 identical across both case-studies runs and across both
  generalization runs (`shasum -a 256`), confirmed byte-for-byte via `diff`.
- `cmca_generation_manifest.json`: byte-for-byte identical via `diff` for both ontologies
  (including the full `digests` object).
- `cmca_generation_receipt.json`: byte-for-byte identical via `diff` for both ontologies **once
  the output directory basename was held constant** between run A and run B (`.../runA/out` vs
  `.../runB/out`, both leaf name `out`). With differing leaf directory names (`cs1` vs `cs2`),
  the `command` field and downstream `receipt_digest` differ — this is directory-path leakage
  into the receipt's provenance field, not generator nondeterminism, and does not affect
  `cmca_generated.rs` or `cmca_generation_manifest.json`, which were identical regardless of
  output-dir naming.

Conclusion: **generator output is reproducible byte-for-byte** given the same input ontology and
output directory name. All six digests agree across independent runs for both ontologies.

## Artifact hand-off

Copied (three files each, no generator/Python/RDF tooling):

- `/Users/sac/bcinr/crates/bcinr-cmca/generated-artifact/case-studies/cmca_generated.rs`
- `/Users/sac/bcinr/crates/bcinr-cmca/generated-artifact/case-studies/cmca_generation_manifest.json`
- `/Users/sac/bcinr/crates/bcinr-cmca/generated-artifact/case-studies/cmca_generation_receipt.json`
- `/Users/sac/bcinr/crates/bcinr-cmca/generated-artifact/generalization/cmca_generated.rs`
- `/Users/sac/bcinr/crates/bcinr-cmca/generated-artifact/generalization/cmca_generation_manifest.json`
- `/Users/sac/bcinr/crates/bcinr-cmca/generated-artifact/generalization/cmca_generation_receipt.json`

These are freshly generated in this reproduction session (run A of each ontology), not copied
from any prior mfw working-directory output.

## Scope and limits (not claimed here)

- This report does not audit whether the generator's semantics correctly encode the RDF/SHACL
  source ontologies — that is a slow-rail correctness question outside this reproduction check.
- This report does not run any BCINR-side gate (cheat-scan, mutant-kill, object-code audit)
  against the copied artifact — that is the consuming role's responsibility per AGENTS.md §4/§27
  (no self-certification across roles).
- `generator_version` was recorded as `v2.0.0-mfw`; no independent check was made that this
  matches an expected/pinned version elsewhere in BCINR docs.
