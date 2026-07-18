# Pre-Migration Baseline — bcinr-cmca RDF Generator

Frozen fixtures captured before relocating CMCA RDF admission+generation to `/Users/sac/mfw`
(migration step 1). This document is a point-in-time capture, not a standing rule — see
`.claude/rules/cmca/rdf-generation.md` and `.claude/rules/cmca/artifact-boundary.md` for the
timeless invariants this capture is evaluated against.

**Captured:** 2026-07-17
**Generator:** `crates/bcinr-cmca/generator.py` (working tree at capture time, see digest below)

## Commands run

```bash
cd /Users/sac/bcinr/crates/bcinr-cmca

# cmca-rdf.ttl -> case_studies.rs
python3 generator.py ontology/cmca-rdf.ttl /tmp/case_studies_run1.rs
python3 generator.py ontology/cmca-rdf.ttl /tmp/case_studies_run2.rs
diff /tmp/case_studies_run1.rs /tmp/case_studies_run2.rs
diff /tmp/case_studies_run1.rs src/generated/case_studies.rs

# generalization.ttl -> generalization.rs
python3 generator.py ontology/generalization.ttl /tmp/generalization_run1.rs
python3 generator.py ontology/generalization.ttl /tmp/generalization_run2.rs
diff /tmp/generalization_run1.rs /tmp/generalization_run2.rs
diff /tmp/generalization_run1.rs src/generated/generalization.rs
```

`generator.py`'s argv handling (`main()`, near the bottom of the file) accepts
`sys.argv[1]` as the input TTL path and `sys.argv[2]` as the output path when
`len(sys.argv) >= 3`; both ontologies were run with explicit argv per this path — no
separate default-path behavior was required for `generalization.ttl` in this generator
version (the two-positional-arg form works uniformly for both ontologies).

## Byte-comparison results

| Comparison | Result |
|---|---|
| `case_studies` run1 vs run2 (`diff`, exit code) | PASS — identical (exit 0) |
| `case_studies` run1 vs committed `src/generated/case_studies.rs` | PASS — identical (exit 0) |
| `generalization` run1 vs run2 (`diff`, exit code) | PASS — identical (exit 0) |
| `generalization` run1 vs committed `src/generated/generalization.rs` | PASS — identical (exit 0) |

All four comparisons passed byte-for-byte. REPORTED: observed directly via the `diff`
invocations above in this session; not independently re-verified by another agent.

## SHA-256 digests

```
5f3b550a116aab25628f9e8272ac1348f21d19114eb2cbca11c7c4c9176ebf69  ontology/cmca-rdf.ttl
448195add75142b933b24d29950c22d4c2c92f82f23caf4f95368e7770b44152  ontology/generalization.ttl
cb61ea09887e04455d2196aed50c027f18842f58db386e6b6d519988285a9b45  generator.py
7a9329f8eb52c12cff5babccfd52f9ab3c13603183713b44e3c36c34bdd470d4  src/generated/case_studies.rs
8e68cd9dac84240f9f44d304f7c5bc9b3d9a6d09b58f8d99e568ffeb8075e7a0  src/generated/generalization.rs
dc17bc42a80d3c2970157f4750c679322c069d3fd94c59fb6028f5239d1d01d2  src/generated/stability_profile.rs
995a40ae20faa523116b79a0f70821c4350dcc26ae8ce57e5ed2fa21c631ced6  src/generated/mod.rs
```

Digests computed with `shasum -a 256 <file>` from `/Users/sac/bcinr/crates/bcinr-cmca`.

## Fixture copies

Verbatim copies (via `cp`, not regenerated) of the current generated files, taken at the
same time as the digests above, live at:

```
tests/fixtures/pre_migration/case_studies.rs
tests/fixtures/pre_migration/generalization.rs
tests/fixtures/pre_migration/stability_profile.rs
tests/fixtures/pre_migration/mod.rs
```

These are the byte-for-byte targets for post-migration correspondence testing against the
mfw-produced artifact, subject to the known-defect carve-outs below.

## Known lawful vs defective behavior

This generator run reproduces the current `generator.py`'s actual behavior, which per
`.claude/rules/cmca/rdf-generation.md` Invariants 2, 3, and 5 includes behaviors that are
themselves defects relative to those invariants. Freezing byte-identical output is
necessary for regression detection, but the byte content is not uniformly a correctness
target — the three behaviors below are frozen as **evidence of known defects**, not as
targets to preserve:

1. **Zero-default fallback for missing properties** — `main()` uses
   `props.get(f_full, 0.0)` (and `business_values[obj] = props.get('cmca:businessValue', 0.0)`)
   to fill in a factor value when a property is absent from the ontology, indistinguishable
   in the generated output from an explicit `0.0`. This violates rule
   `cmca/rdf-generation.md` Invariant 2 ("Missing is not zero"). **Frozen as:** evidence of
   the defect only, not a byte-equivalence target. **Correct replacement:** a missing
   required property must produce a typed refusal identifying the property and object,
   never a silently substituted `0.0`.

2. **Cycle-returns-0.0 behavior** — `get_consequence_mass()` detects a revisited object
   (`if obj in path: return 0.0`) and silently contributes `0.0` for the cyclic node instead
   of halting generation. This violates rule `cmca/rdf-generation.md` Invariant 3
   ("Dependency cycles refuse, never contribute a default"). **Frozen as:** evidence of the
   defect only, not a byte-equivalence target. **Correct replacement:** a detected cycle in
   the consequence/derivation graph must produce a typed refusal identifying the cycle
   (e.g. the object sequence forming it), never a partial/zero contribution allowing
   generation to proceed.

3. **Binary-float (not Decimal) Q16.16 rounding** — `to_q16_16()` computes
   `int(round(val * 65536))` over Python `float` (IEEE-754 binary double) values throughout
   the pipeline (business values, factors, eta, lens exponents), rather than performing
   exact decimal arithmetic under a declared rounding-mode profile. This violates rule
   `cmca/rdf-generation.md` Invariant 5 ("Fixed-point numeric conversion is exact decimal
   arithmetic, not binary float"). **Frozen as:** evidence of the defect only, not a
   byte-equivalence target. **Correct replacement:** conversion from the ontology's decimal
   literal to Q16.16 must route through Python's `Decimal` (or an equivalent exact-decimal
   path) under a declared, digested precision/rounding-mode profile, with no intermediate
   binary-float representation of the literal.

Everything else in the frozen output (registry structure, macro-unroll scaffolding, and all
values not touched by the three defects above) is frozen as the **byte-equivalence
correspondence-testing target**: the mfw-side replacement generator must reproduce it exactly
for inputs that do not exercise the three

4. **Measure-head/lens array ordering (this file's `generalization.rs` fixture only) —
   NOT a byte-equivalence target; itself defective/noncanonical.** Added in the v26.7.17
   WORKSTREAM B reconciliation pass (see
   `tests/fixtures/GENERALIZATION_ORDER_DECISION.md` for the full root-cause finding).
   The measure-head row order in this fixture (`generalization.rs`: MeasureCache,
   MeasureGeneralizationProof, MeasureRetrieval, MeasureScheduling, MeasureSearch) and the
   lens order (LensCoverage, LensExploitation, LensGeneralizationProof, LensProportional,
   LensRare) are both **alphabetical-by-entity-name** — an incidental artifact of the
   legacy `generator.py`'s dict/sort behavior over subject names, never derived from the
   explicit `cmca:measureIndex` / `cmca:lensIndex` admitted in `generalization.ttl`
   (MeasureCache=0, MeasureSearch=1, MeasureRetrieval=2, MeasureScheduling=3,
   MeasureGeneralizationProof=4; LensExploitation=0, LensProportional=1, LensCoverage=2,
   LensRare=3, LensGeneralizationProof=4). The mfw-side replacement generator
   (`tools/cmca-generator/generator.py`, `sorted_mh` / `sorted_lenses`, keyed on
   `(mh_index[m], m)` / `(lens_index[l], l)`) instead sorts strictly by the explicit
   admitted index, per the default rule that mechanical array position follows the
   explicit admitted semantic index, not incidental parse order. **Frozen as:** evidence
   that the *old* order was itself never an admitted semantic law, not a
   byte-equivalence target. **Correct replacement (already the mfw generator's actual
   behavior, verified, no fix needed there):** measure-head/lens array position is keyed
   on the explicit admitted index. The originally-worded item above ("object/measure-head/
   lens ordering ... frozen as the byte-equivalence correspondence-testing target") is
   retracted for the measure-head/lens axis specifically; object (semantic-object)
   ordering is unaffected (both generators sort objects by name, and no explicit object
   index exists in this ontology) and remains a byte-equivalence target as originally
   stated.
defective code paths above.

## Nonclaims

This document does not assert that the three listed defects are the only ones present in
`generator.py`; it lists only the behaviors the migration ticket specifically named. It
does not claim Invariant 1 (structured refusal vs. disableable assertion), Invariant 4
(index injectivity/bounds/contiguity), Invariant 6 (reproducibility beyond the two runs
captured here), or Invariant 7 (manifest completeness) have been separately audited in this
capture — those remain open questions for the release ledger, not this fixture-freeze
document.

## See Also

- `.claude/rules/cmca/rdf-generation.md` — the invariants this capture is evaluated against
- `.claude/rules/cmca/artifact-boundary.md` — the producer/consumer artifact contract this
  migration is building toward
- `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` — current release status (REPORTED facts)
