# ggen-legacy Reconstitution of bcinr-cmca Code Generation — Review

**Headline finding:** The patch's actual generation defect was reproduced by running it for
real — `ggen sync run` executes and renders `generated.rs`, but that output diverges from the
tracked file (`generated_bytes differ: legacy=12150 bytes, current=8019 bytes`) and would fail
`rustc` outright, because `allocator/mod.rs` imports `FACTOR_DOWNSTREAM_CONSEQUENCE` at three
call sites and the patch's SPARQL query never produces it. This is not currently sound to merge.

## 1. What's real

**Established earlier this session (patch provenance):** the pending bcinr-cmca reconstitution
patch's own files, SHA-256 hashes, and receipt are internally consistent — real files, real
hashes matching its own claims.

**Confirmed this round (ggen-legacy tooling and worked example):**

- `tools/v26.8.1/equivalence_runner.py` in `~/ggen-legacy` is real, runnable, stdlib-only
  infrastructure — confirmed live via `--help`, zero imports from ggen-legacy's own source
  tree, dispatch entirely by `observable_surfaces` named in a manifest. It is commit-independent:
  usable regardless of which ggen-legacy commit is checked out.
- The pinned kernel SHA `0f39227c102e0ac7519f0f27561356227a518653` is real and referenced
  extensively across `foundry/bootstrap.yaml`, `appliance/manifest.json`,
  `governance/source-ledger.md`, and multiple verifier scripts that hardcode it as the expected
  `ggen` coordinate.
- `migrations/ggen-v26.8.1/equivalence-report.json` is a real, applied precedent: 6/7 components
  byte-identical (`PRESERVED`) with matching BLAKE3 digests, 1 `ARCHIVED`, all four negative
  controls (byte drift, missing file, path escape, stale coordinate) correctly refused. Its
  `claim_boundary` is explicit: **"migration integrity and bounded composition only"** — a
  copy-provenance proof, not a behavioral-equivalence proof.
- `docs/v26.8.1/90-legacy/93-capability-equivalence-matrix.md` documents ggen-legacy's own
  two-tier equivalence standard: archaeological disposition (commit-evidenced, what happened to
  a capability) is necessary but insufficient; standing/equivalence promotion requires both an
  external `equivalenceVerifier` and a `negativeFalsifier` to actually run and pass. All 15
  mined capabilities sit at `ggen:UNKNOWN` standing by design — archaeology alone never promotes
  standing in this repo's own methodology.

## 2. What's fabricated

**Confirmed this round:** the commit `49c3a1eddf3d90560b9471573b6455dc240fe752`, which the
pending bcinr-cmca patch claims pins its reconstitution to a specific ggen-legacy state, is not
reachable and does not exist as a git object in the `~/ggen-legacy` checkout used for this
review:

```
$ git cat-file -e 49c3a1eddf3d90560b9471573b6455dc240fe752
NOT AN OBJECT IN THIS REPO
$ git merge-base --is-ancestor 49c3a1eddf3d90560b9471573b6455dc240fe752 HEAD
fatal: Not a valid commit name 49c3a1eddf3d90560b9471573b6455dc240fe752
```

`git log --all | grep` for that SHA also returns zero hits across the whole ref history. This is
a wrong or invented SHA relative to that repo — it does not pin anything checkable there. It is
distinct from, and should not be conflated with, the real pinned kernel SHA
`0f39227c102e0ac7519f0f27561356227a518653` used throughout ggen-legacy's own bootstrap/provenance
files (see §1).

This mismatch does not block using ggen-legacy's tooling (the equivalence runner and the
manifest schema are commit-independent per §1), but it does mean the patch's stated provenance
claim about which ggen-legacy state it reconstitutes from cannot currently be verified, and the
patch author should be asked to supply the correct SHA or drop the claim.

## 3. What's stale — the patch-vs-current diff

**Architecture:** the patch's `generator.py` (233 lines vs. current 397) contains zero RDF/TTL
parsing code. It is a pure orchestrator: copies ontology + fixed query/template assets into a
temp project, shells out to a pinned `ggen` binary (`ggen sync run`), reads back `generated.rs`,
canonicalizes with `rustfmt`, compares/writes/emits. TTL interpretation and Rust rendering are
both delegated to the external `ggen` kernel via SPARQL (`consequence-mass.rq`) and a Tera
template (`consequence_mass.rs.tera`) — matching the patch's own manifest claim.

**CMCA-118's fate:** CMCA-118 (literal-aware `#`-comment stripping) lives entirely in
`parse_ttl`/`strip_ttl_comment` in the current `generator.py`, which the patch deletes wholesale
along with the code `tests/generator_ttl_comment_stripping.rs` was written to exercise. Under the
patch's architecture, comment-stripping becomes `ggen`'s problem, not this crate's — so the fix
is moot for the patch's own `generator.py`. It is **not** moot for the repo: that test target
would fail-to-apply or fail-to-run against a `generator.py` that no longer parses TTL, and would
need to be deleted or rewritten to target `ggen`'s TTL handling instead. Whether `ggen` itself
correctly handles a `#` inside a quoted Turtle literal is unverified — `ggen` is an external
pinned binary, and this question was not part of the equivalence-runner check in §4.

**3862ebc5** (default output path / dead macro emission fix) is similarly superseded without a
surviving target: the patch's CLI (`--profile`/`--check`/`--write`/`--emit-dir`, hardcoded
`PROFILE_NAMES`) has a completely different interface from the current positional-argument CLI
that commit's fix addressed.

**Genuine bug in the patch as shipped, confirmed structurally before it was run:**
`queries/consequence-mass.rq`'s lens clause requires `cmca:exponent ?exponent`, but both ontology
files (`cmca-rdf.ttl`, `generalization.ttl` — unmodified by the patch) only ever assert
`cmca:lensExponent`. SPARQL basic graph pattern matching fails silently on an absent triple, so
every `?lens` row would drop from the result set, producing an empty `LENS_REGISTRY` and empty
`lambda` rows while the `Q` constant (counted separately via `COUNT(?lens)` over `a cmca:Lens`
triples, unaffected by the exponent predicate) stays nonzero — a real shape mismatch between the
declared `Q` and the actual `LensSpec` array length.

Applying the patch as-is therefore requires more than swapping `generator.py`: it requires either
fixing the predicate name in `consequence-mass.rq` or renaming the predicate across both ontology
files, contrary to the manifest's implication that only `generator.py` changes.

Object/measure/factor bindings, by contrast, were confirmed to line up correctly against
`codegen-compat.ttl`'s explicit `cmcag:rustConstName`/`cmcag:index` triples — the patch's
`N`/`F`/`K`/`Q` consts, `ETA`, `LAMBDA`, `OBJECT_REGISTRY`, `LENS_REGISTRY`, and the specific
const names (`OBJECT_ARTIFACT__A`, `MEASURE_CACHE`, `LENS_EXPLOITATION`,
`FACTOR_BUSINESS_VALUE`, etc.) all match what `allocator/mod.rs`,
`tests/single_lens_allocation.rs`, and `allocation_receipt.rs` currently import and use — under
the condition that the lens-exponent predicate bug above is fixed first.

## 4. What was actually attempted, and its real result

All five steps below were run for real against the actual patch assets and the actual `ggen`
binary — no simulation, no forced pass, no rounding a partial result up to "it works."

**Step 1.** `ggen` binary already built at `/Users/sac/ggen/target/debug/ggen` — confirmed
executable, `ggen sync run --help` works.

**Step 2.** The patch's assets were not applied in `/Users/sac/bcinr` (this repo is untouched);
they were located in a prior session's scratch checkout at
`/private/tmp/.../scratchpad/cmca-reconstitution/bcinr-cmca-ggen-legacy/crates/bcinr-cmca/`.
Its `reconstitution/ggen.toml.template` matched ggen's real `[[generation.rules]]` contract with
no adaptation needed. The patch's `ontology/` directory shipped only `codegen-compat.ttl`; the
unmodified `cmca-rdf.ttl`/`generalization.ttl` had to be copied in from this repo for
`generator.py --profile case_studies` to find its ontology input.

**Step 3.** The patch's own orchestrator was run for real:
`GGEN_BIN=/Users/sac/ggen/target/debug/ggen python3 generator.py --emit-dir ... --profile
case_studies`. `ggen sync run` executed in 269.936ms, exit 0, and wrote `generated.rs`
(8019 bytes, canonical, after real `rustfmt`).

**Step 4.** A real one-case manifest was built and run through `equivalence_runner.py` per its
own `generated_bytes`/`$EQV_OUT` contract (legacy_adapter = `cp` of the tracked file,
current_adapter = re-run `generator.py` + `cp` its output). Real result:

```
[FAIL] cmca-consequence-mass-case-studies (PRESERVED): generated_bytes differ: legacy=12150 bytes, current=8019 bytes, first diff at offset 1479
```

**Step 5.** A byte diff against the tracked file confirmed two concrete defects — one predicted
in §3, one newly discovered only by actually running the pipeline:

1. **Predicted and confirmed:** the `cmca:exponent`/`cmca:lensExponent` mismatch causes
   `LENS_REGISTRY`-related consts (`LENS_EXPLOITATION`, etc.) and all `lambda` rows to silently
   drop out — `LAMBDA` rows render as empty `[]` arrays instead of 4-element arrays.
2. **New, found only by running it:** `FACTOR_DOWNSTREAM_CONSEQUENCE`/`_IRI` is entirely absent
   from the rendered output — the query has no clause producing it — and `allocator/mod.rs`
   imports that exact const at lines 352, 1931, and 2668. The ggen-rendered file would fail
   `rustc` with an unresolved import, independent of the lens bug.

No blocker (missing tool, unrunnable pipeline, environment gap) was hit. The finding is a real,
reproducible generation defect in the patch's `consequence-mass.rq`, not an inability to test it.

Scratch artifacts (not committed anywhere, `/Users/sac/bcinr` untouched):
- `.../cmca-equivalence-attempt/emitted/case_studies.rs` — real ggen output
- `.../cmca-equivalence-attempt/receipt.json` — generator.py's own receipt
  (`standing: ALIVE`, `equivalent_to_tracked: false`)
- `.../cmca-equivalence-attempt/manifest.json`, `.../eqv-report.json` — equivalence_runner.py's
  real manifest and FAIL report

## 5. Recommendation

**Not currently sound to merge.** The patch's core architectural bet — delegate TTL parsing and
Rust rendering entirely to the pinned `ggen` kernel via SPARQL + Tera — is a real, exercised
design that runs end to end without simulation. But as shipped it produces output that (a) does
not byte-match the tracked generated code and (b) would fail to compile against
`allocator/mod.rs`'s real imports. Applying by "trust the manifest, ggen renders it correctly" is
falsified by this session's own run.

Whether it's worth pursuing: yes, conditionally. The remaining gap is narrow and named, not
open-ended — this is a query-authoring bug, not an architecture-soundness question about
delegating to `ggen`.

**Concrete next step, in order:**

1. Fix `queries/consequence-mass.rq`'s lens clause: change `cmca:exponent ?exponent` to
   `cmca:lensExponent ?exponent` (do not rename the ontology predicate — the ontology files are
   shared, unmodified, real files used elsewhere in this repo).
2. Add a clause to `consequence-mass.rq` (or a template default) that produces
   `FACTOR_DOWNSTREAM_CONSEQUENCE`/`_IRI`, matching whatever triple pattern in `cmca-rdf.ttl`/
   `generalization.ttl` currently backs that const in the hand-written or previously-generated
   `case_studies.rs`.
3. Re-run this session's exact reproduction (`GGEN_BIN=... python3 generator.py --emit-dir ...
   --profile case_studies`, then `equivalence_runner.py` against the same one-case manifest) and
   require a `[PASS]`, i.e. `generated_bytes` byte-identical to the tracked file, before treating
   the predicate/const-coverage bugs as closed.
4. Only after that PASS: decide the fate of `tests/generator_ttl_comment_stripping.rs`
   (CMCA-118) — either delete it as moot under the new architecture, or rewrite it to assert on
   `ggen`'s TTL comment-handling behavior directly, so CMCA-118's underlying guarantee is not
   silently dropped.
5. Get the patch's provenance SHA (`49c3a1eddf3d90560b9471573b6455dc240fe752`) corrected or
   removed — it does not exist in `~/ggen-legacy` and currently makes an unverifiable claim about
   what ggen-legacy state the patch reconstitutes from.

Until step 3 produces a real `[PASS]`, this reconstitution should be treated as blocked, not
partial-credit-worthy.
