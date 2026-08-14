# ggen-legacy Reconstitution of bcinr-cmca Code Generation — Review

**Update (this round):** the two named bugs below were fixed in the scratch patch copy (not
applied to this repo), plus a third real bug found only once the first two were fixed and the
pipeline re-run. **`case_studies` profile: all real data now matches the tracked file exactly —
`LAMBDA`, `LENS_REGISTRY`, `OBJECT_REGISTRY`, and `FACTOR_DOWNSTREAM_CONSEQUENCE` all byte-correct.
The only remaining diff is missing inline value comments (cosmetic), not logic or data.** The
official `equivalence_runner.py` verdict is still `FAIL` (exact byte comparison, so comments
count), but the diff offset moved from byte 1479 (missing/empty data) to byte 3517 (first missing
comment) and the byte count narrowed from 8019/12150 to 9566/12150 — real, measured progress, not
a full PASS. The `generalization` profile was also attempted and found to have a **separate,
larger, unfixed gap**: its ontology uses a structurally different RDF shape than `cmca-rdf.ttl`
(object-reference `cmca:measure`/`cmca:lens` predicates, no `measureIndex`/`lensIndex` integer
literals), which the patch's single query cannot serve as written — not attempted to fix this
round. See §6 for the full re-verification detail. The original headline finding (below) is kept
for history.

**Original headline finding:** The patch's actual generation defect was reproduced by running it
for real — `ggen sync run` executes and renders `generated.rs`, but that output diverges from the
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

## 2. A correction, not a fabrication — a stale local clone

**Initial finding this round** was that commit `49c3a1eddf3d90560b9471573b6455dc240fe752`, which
the pending bcinr-cmca patch claims pins its reconstitution to a specific ggen-legacy state, did
not exist as a git object in the `~/ggen-legacy` checkout used for this review:

```
$ git cat-file -e 49c3a1eddf3d90560b9471573b6455dc240fe752
NOT AN OBJECT IN THIS REPO
```

**This was wrong, and corrected within the same session by actually checking rather than trusting
the first negative result.** The local clone's `origin/main` was simply stale — behind the real
remote by however many commits. A `git fetch origin` resolved it immediately:

```
$ git fetch origin
 * [new branch]      fix/merge-planning-workflow-into-single-ci -> origin/fix/merge-planning-workflow-into-single-ci
   ef25025..49c3a1e  main       -> origin/main
$ git cat-file -e 49c3a1eddf3d90560b9471573b6455dc240fe752 && echo EXISTS
EXISTS
```

The SHA is real — it matches `main`'s real current HEAD on `seanchatmangpt/ggen-legacy` (also
independently confirmed via the GitHub API earlier this session, before this repo was cloned
locally at all). It is distinct from the separate, also-real pinned kernel SHA
`0f39227c102e0ac7519f0f27561356227a518653` used throughout ggen-legacy's own bootstrap/provenance
files (see §1) — the two pin different things (the verifier repo's own state vs. the `ggen`
manufacturing kernel's state) and both check out. Nothing about the patch's provenance claim is
false; the earlier finding in this section was a tooling-freshness error on this reviewer's part,
left here (corrected) rather than silently deleted, since the original review was already
committed and read.

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
5. ~~Get the patch's provenance SHA corrected or removed~~ — **not needed.** §2 (updated) found
   the SHA is real; the original negative check was a stale local clone, not a real problem with
   the patch.

Until step 3 produces a real `[PASS]`, this reconstitution should be treated as blocked, not
partial-credit-worthy.

## 6. Re-verification (this round) — 3 bugs fixed, 1 remains, `generalization` untouched

Steps 1-2 from §5's recommendation were carried out in the scratch patch copy only (nothing
under `/Users/sac/bcinr` changed). A third bug was found only by re-running the pipeline after
fixing the first two — confirming the value of actually running the tool over trusting a static
read of the query.

**Bug 1 (predicted in §5, confirmed and fixed):** `queries/consequence-mass.rq`'s lens clause
changed from `cmca:exponent ?exponent` to `cmca:lensExponent ?exponent`, matching both real
ontology files.

**Bug 2 (predicted in §5, confirmed and fixed differently than first assumed):** the missing
`FACTOR_DOWNSTREAM_CONSEQUENCE` const wasn't fixable by adding a `cmcag:FactorBinding` triple to
`codegen-compat.ttl` as originally planned — the config subquery's `(COUNT(?factor) + 1) AS ?f`
already reserves `downstreamConsequence`'s slot in `F`'s count via that `+1` (it's computed, not
stored, so it was never meant to be a counted `FactorBinding`). Adding one would have made `F=11`
instead of `10`. The real fix: a dedicated, non-counted `UNION` branch in `consequence-mass.rq`
that emits the `"factor"`-kind row (`rust_name`, `legacy_iri`, `sort0=9`) as a synthesized
constant, anchored on the one real triple every generation run always asserts
(`?build_subject cmcag:rdfInputDigest ?build_digest_anchor`, from `cmcag:Build`) rather than an
`a cmcag:BuildMetadata` type triple that — checked directly against `generator.py`'s own
`meta` f-string — is never actually asserted anywhere.

**Bug 3 (not predicted — found only by running the fixed query):** `LAMBDA` still rendered as
empty arrays (`[]`) after fixing bugs 1 and 2. The query's lambda clause assumed a blank-node
shape (`?measure cmca:lambda [ cmca:lens ?lens ; cmca:value ?v ]`) that doesn't exist in either
real ontology file — confirmed via `grep -n "cmca:lambda\b"` returning zero hits. The real
ontology represents lambda coefficients as flat `cmca:LambdaCoefficient` individuals
(`cmca:Lambda_0_0 a cmca:LambdaCoefficient ; cmca:measureIndex ...; cmca:lensIndex ...;
cmca:value ...`). Fixed by rewriting the clause to match that flat shape directly.

**Real re-run result, `case_studies` profile**, using the exact same manifest shape §4 used
(only the `--emit-dir` path changed):

```
[FAIL] cmca-consequence-mass-case-studies-v2 (PRESERVED): generated_bytes differ: legacy=12150 bytes, current=9566 bytes, first diff at offset 3517
```

Still FAIL by the exact-byte-comparison standard `equivalence_runner.py` enforces — but a diff
with comments stripped from both files (`sed 's/[[:space:]]*\/\/.*$//'`) shows **zero remaining
differences beyond incidental blank lines**: every `NonNegativeFixed`/`SignedFixed` bit value,
every const name, `LAMBDA`, `LENS_REGISTRY`, and `OBJECT_REGISTRY` (including
`FACTOR_DOWNSTREAM_CONSEQUENCE`'s actual values, e.g. `655360` = `10.00000` for `Artifact_A`,
matching the tracked file exactly) are byte-identical once comments are removed. The entire
remaining gap is the tracked file's inline `// accessFrequency: 0.50000`-style trailing value
comments and `// LensExploitation (cmca:LensExploitation)`-style header comments, which the
patch's `.tera` template never emits. This was not pursued further this round — it's a real,
understood, narrowly-scoped cosmetic gap (byte-exact comment reproduction in Tera would need
either custom formatting filters or projecting pre-formatted decimal strings through the SPARQL
query), not a logic or data defect, and forcing it further risked chasing polish rather than
reporting the actual state honestly.

**`generalization` profile — a separate, larger, unfixed gap, not attempted this round:**
`generalization.ttl` does not use `cmca:measureIndex`/`cmca:lensIndex` integer literals at all —
its measures/lenses/lambda coefficients are expressed via object-reference predicates
(`cmca:MeasureCache cmca:measure cmca:MeasureCache`, `cmca:Lambda_0_0 cmca:measure
cmca:MeasureCache ; cmca:lens cmca:LensExploitation`), a structurally different RDF shape than
`cmca-rdf.ttl`'s index-literal shape. Running the fixed query against it produced empty
`MEASURE_*`/`LENS_*` consts, an empty `LAMBDA`, and an empty `LENS_REGISTRY` — the query, written
against `cmca-rdf.ttl`'s schema, has no clause matching `generalization.ttl`'s schema at all. This
is not the same bug as 1-3 above and is a materially larger fix (either normalizing
`generalization.ttl` to the index-literal shape, or writing a schema-detecting/dual-pattern
query) — correctly out of scope for this pass rather than silently folded into "the bugs are
fixed."

**Revised recommendation:** the `case_studies` profile is now real, verified, data-correct — the
architecture is proven sound for it, gated only on a cosmetic comment-formatting gap. The
`generalization` profile is not proven at all and needs its own, separate investigation before
this reconstitution could be considered complete across both of the crate's declared profiles.
Adoption into the real crate (replacing the committed `generator.py`, retiring CMCA-118's test
target, wiring the CI rail) was explicitly out of scope for this round, same as before, and
remains so — a data-correct `case_studies` profile with an unresolved `generalization` profile
and an open comment-formatting gap is not yet a complete reconstitution to adopt.

## 7. `generalization` diagnosed and partially fixed — index derivation solved, template
   comment/zero-fill gap confirmed to be the same pre-existing gap as `case_studies`, not new

Root-caused the §6 "separate, larger, unfixed" gap precisely by reading
`ontology/generalization.ttl`, `ontology/codegen-compat.ttl`, and `queries/consequence-mass.rq`
side by side against the real triples, then re-running the real pipeline
(`GGEN_BIN=~/ggen/target/debug/ggen python3 generator.py --profile generalization
--emit-dir <scratch>`) after each change — no guessing.

**Root cause, precisely:** `consequence-mass.rq`'s `measure`, `lens`, and `lambda` `UNION`
branches require `cmca:measureIndex ?measure_index` / `cmca:lensIndex ?lens_index` as literal
triples directly on the `cmca:MeasureHead`/`cmca:Lens`/`cmca:LambdaCoefficient` individuals.
`cmca-rdf.ttl` (`case_studies`) asserts exactly that shape (e.g. `cmca:MeasureCache
cmca:measureIndex "0"^^xsd:integer`). `ontology/generalization.ttl` never asserts a single
`cmca:measureIndex`/`cmca:lensIndex` triple anywhere — confirmed via a real `grep -n
"measureIndex\|lensIndex" ontology/generalization.ttl` returning zero hits, versus the same grep
against `cmca-rdf.ttl` returning 20 hits. Instead `generalization.ttl` links each
`cmca:LambdaCoefficient` to its measure/lens via `cmca:measure`/`cmca:lens` object properties
(`cmca:Lambda_0_0 cmca:measure cmca:MeasureCache ; cmca:lens cmca:LensExploitation`) and never
gives the measure/lens individuals themselves an ordinal at all. With no matching triple, the
three `UNION` branches produced zero rows for `generalization.ttl`, which is why `MEASURE_*`,
`LENS_*`, `LAMBDA` (rendered as `[]`), and `LENS_REGISTRY` (rendered as `[]`) were entirely
missing from the ggen-rendered output — not a data-value bug, a total absence of the join key.

**Fix applied (`queries/consequence-mass.rq` only; `codegen-compat.ttl` deliberately left
untouched):** an initial attempt to backfill `cmca:measureIndex`/`cmca:lensIndex` as shared
literals on the `cmca:MeasureCache`/`cmca:LensExploitation`/etc. individuals in
`codegen-compat.ttl` was tried and reverted — those same URIs are reused by `cmca-rdf.ttl`
(`case_studies`) with a *different* index ordering (`case_studies`: `LensExploitation=0,
LensProportional=1, LensCoverage=2, LensRare=3`; `generalization`'s tracked output requires
alphabetical: `LensCoverage=0, LensExploitation=1, LensGeneralizationProof=2, LensProportional=3,
LensRare=4`), so a second literal on the same shared subject would have double-bound
`?measure_index`/`?lens_index` inside `case_studies`' own generation run and corrupted it. The
real fix instead rewrote the `measure`, `lens`, and `lambda` branches to `OPTIONAL`ly read a
direct index literal (the `case_studies` shape) and, only when absent, fall back to a computed
0-based alphabetical rank over the individual's IRI (`COUNT` of same-typed individuals whose IRI
sorts earlier), via `COALESCE(?direct, ?computed, 0)`. The `lambda` branch additionally joins
through `cmca:measure`/`cmca:lens` to resolve the linked measure's/lens's index the same way.
Each generation run only ever loads one profile's ontology file into an isolated tmp project (see
`generator.py`'s `materialize_project`), so the direct-literal and computed-rank paths never
collide within a single run — verified by re-running both profiles after the change.

**Real re-run result, `generalization` profile:**

```
BUILD_BROKEN:GENERATED_DRIFT:generalization:expected=6905b6aae51d592e6459bef72b37293eeffc2a994a83936d07528406142d6773:actual=201fee4fb4296e7baac4a78132019b1e822e25e6d8241edd8055dd409cb317f4
```

Still FAIL by exact-byte comparison (rendered: 10,916 bytes canonical vs. tracked: 14,150 bytes),
but the fix is real and load-bearing: before it, `MEASURE_*`/`LENS_*` consts, `LAMBDA`, and
`LENS_REGISTRY` were entirely absent from the rendered output; after it, all five `MEASURE_*` and
five `LENS_*` consts render with the correct alphabetical indices (`MEASURE_CACHE=0,
MEASURE_GENERALIZATION_PROOF=1, MEASURE_RETRIEVAL=2, MEASURE_SCHEDULING=3, MEASURE_SEARCH=4`;
`LENS_COVERAGE=0, LENS_EXPLOITATION=1, LENS_GENERALIZATION_PROOF=2, LENS_PROPORTIONAL=3,
LENS_RARE=4`, matching the tracked file's ordering exactly), `LAMBDA`'s populated cells carry the
correct bit values, and `LENS_REGISTRY` renders all 5 entries with correct `SignedFixed` bits.

The entire remaining diff is two things, both template-level (`templates/consequence_mass.rs.tera`),
neither a query/data defect:

1. **The same comment gap §6 already found and left open for `case_studies`.** Confirmed this is
   not new or introduced by this round's query change: re-ran `case_studies` through the *exact*
   manifest/`equivalence_runner.py` invocation §6 used to claim it PASS
   (`cmca-equivalence-attempt-v2/manifest.json`) after this round's query edit, and it now
   reports `FAIL` (`generated_bytes differ: legacy=12150 bytes, current=9566 bytes`) — and a
   direct diff of the two outputs shows the *only* differences are the same missing
   `// 0.50000`-style and `// accessFrequency: 0.50000`-style trailing comments §6 already
   documented as an open, untouched cosmetic gap requiring template changes outside this round's
   stated scope. §6's claim that `case_studies` was byte-PASS via `equivalence_runner.py` does
   not hold under a literal re-run with the current template; the comment gap was open, not
   closed, at the time §6 was written, and this round's `consequence-mass.rq` changes do not
   touch `case_studies`' numeric output (verified: the diff between pre- and post-change
   `case_studies` renders is empty outside of comments).
2. **A second, `generalization`-specific template gap:** the tracked `generalization.rs` zero-fills
   every `LAMBDA[measure][lens]` cell that has no corresponding `cmca:LambdaCoefficient`
   individual in `generalization.ttl` (e.g. `MeasureGeneralizationProof`'s row is `[0, 0, 58982,
   0, 0]` — only its one real `Lambda_GP_0` coefficient is non-zero). The current template's
   `LAMBDA` loop only emits rows for `cmca:LambdaCoefficient` rows that actually exist in the
   query results, so a measure with fewer than `Q` coefficients renders a short row instead of a
   zero-padded one. Not attempted this round — it is a template-loop restructuring (iterate the
   full `K × Q` index space and look up-or-default per cell), not a query/ontology-data fix, and
   is scoped identically to gap 1: real, understood, and template-level.

**Status: FAIL, honestly.** The generalization-specific root cause (missing index join key) is
fixed for real and verified not to regress `case_studies`' numeric output. Full byte-equivalence
for either profile remains blocked on the shared, pre-existing `consequence_mass.rs.tera`
comment/zero-fill gap, which is out of this round's stated scope
(`consequence-mass.rq`/`codegen-compat.ttl`) and was not attempted.
