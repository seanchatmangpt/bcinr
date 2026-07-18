# Generalization Sequence-Order Correspondence — Root-Cause Decision

**v26.7.17 CMCA reconciliation, WORKSTREAM B.** Scope: resolve the failing
`generalization_numeric_payload_matches_old_lawful_output_exactly` correspondence test.

## Symptom

`cargo test -p bcinr-cmca --test consumer_correspondence
generalization_numeric_payload_matches_old_lawful_output_exactly -- --nocapture` failed:
the `from_value_bits(..)` payload sequence's **value set** matched the frozen
pre-migration fixture (`tests/fixtures/pre_migration/generalization.rs`) exactly, but the
**order** did not.

## Root cause

Compared the measure-head row order and lens column order in both files:

| Source | Measure-head order | Lens order |
|---|---|---|
| Old fixture (`tests/fixtures/pre_migration/generalization.rs`) | Cache, GeneralizationProof, Retrieval, Scheduling, Search | Coverage, Exploitation, GeneralizationProof, Proportional, Rare |
| New artifact (`generated-artifact/generalization/cmca_generated.rs`) | Cache, Search, Retrieval, Scheduling, GeneralizationProof | Exploitation, Proportional, Coverage, Rare, GeneralizationProof |
| `generalization.ttl` admitted `cmca:measureIndex`/`cmca:lensIndex` | Cache=0, Search=1, Retrieval=2, Scheduling=3, GeneralizationProof=4 | Exploitation=0, Proportional=1, Coverage=2, Rare=3, GeneralizationProof=4 |

The old fixture's order is **alphabetical by entity name** in both cases — it does not
match the admitted index order. The new artifact's order **does** match the admitted
index order exactly.

Read `tools/cmca-generator/generator.py` (the mfw producer): measure-heads and lenses are
each explicitly sorted by their admitted index before array emission —

```python
sorted_mh = sorted(measure_heads, key=lambda m: (mh_index[m], m))
...
sorted_lenses = sorted(lenses, key=lambda l: (lens_index[l], l))
```

(lines ~844/860, generator.py) — not by dict/file/parse order. This was verified
independently, not just read: `tools/cmca-generator/tests/test_ordering_law.py` (added
this pass) shows (a) shuffling the input Turtle statement order produces a byte-identical
generated payload (`test_shuffle_determinism`, 3 seeds), and (b) swapping two measures'
explicit `measureIndex` values swaps their corresponding array row positions
(`test_index_change_moves_array_position`). Both passed:

```
$ python3 tools/cmca-generator/tests/test_ordering_law.py
PASS  shuffle_determinism (seeds 1,2,3 byte-identical to unshuffled baseline, excluding the expected RDF_INPUT_DIGEST change)
PASS  index_change_moves_array_position (Cache<->Search measureIndex swap swaps LAMBDA rows)

All ordering-law tests passed.
```

`generalization.ttl` explicitly admits every `cmca:measureIndex`/`cmca:lensIndex` (added
in an earlier reconciliation round; confirmed present, injective, contiguous 0..4 for
both index spaces — see the file's own header note).
`docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` states array order is preserved and semantic
("Array order is preserved, not sorted... arrays inside digested content... carry semantic
meaning in their generation order") and contains no order-irrelevance clause for this
axis — so per the default law stated in the task ("mechanical array position follows the
explicit admitted semantic index, not incidental parse order"), the admitted-index order
is the correct target, and the old fixture's alphabetical order was never a valid
byte-equivalence target for this axis — it was an artifact of the legacy generator's
dict/sort-by-name behavior, not an admitted semantic law.

## Which side was wrong

**The old (frozen pre-migration) fixture's claimed target, not the generator or the
consumer.** The mfw generator (`generator.py`) already correctly orders by admitted
index; no generator fix was needed or made. The bcinr-cmca consumer
(`tests/consumer_correspondence.rs`) does not itself reorder anything — it only extracts
and compares `from_value_bits(..)` sequences byte-for-byte, so there is no consumer-side
reordering defect either.

## What changed

1. `tests/fixtures/PRE_MIGRATION_BASELINE.md` — added item 4 documenting that
   measure-head/lens array ordering is retracted from the byte-equivalence
   correspondence-testing target (it was itself the incidental/defective order); object
   (semantic-object) ordering is unaffected and remains a byte-equivalence target.
2. `tests/consumer_correspondence.rs` —
   `generalization_numeric_payload_matches_old_lawful_output_exactly` now asserts (a)
   value-set (sorted/multiset) equality against the old fixture (no value regression —
   this is still a real, order-INsensitive correspondence check, not weakened away), and
   (b) a separate, positive, order-sensitive check that the new artifact's measure-head
   row order actually matches the admitted-index order derived from
   `generalization.ttl` (Cache, Search, Retrieval, Scheduling, GeneralizationProof) — not
   merely "differs from the old fixture." Order was not dropped from the test; it was
   re-targeted at the correct law.
3. `tools/cmca-generator/tests/test_ordering_law.py` (new, mfw repo) —
   `test_shuffle_determinism` and `test_index_change_moves_array_position`, per task
   step 5. `duplicate_index`/`out_of_range_index`/`noncontiguous_index`/missing-index
   refusal tests already existed in
   `tools/cmca-generator/tests/test_negative_fixtures.py` (confirmed, not duplicated).
4. No changes to `generator.py` or to any consumer src/ code — both were already correct
   for this axis; no artifact regeneration was needed (the artifact already reflects
   admitted-index order).

## Test result after fix

```
$ cargo test -p bcinr-cmca --test consumer_correspondence
running 3 tests
test defective_paths_not_exercised_by_current_fixtures_is_a_nonclaim ... ok
test case_studies_numeric_payload_matches_old_lawful_output_exactly ... ok
test generalization_numeric_payload_matches_old_lawful_output_exactly ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Nonclaims

This does not assert the old fixture is wrong on any other axis (values, object order,
scaffolding) — those were separately diffed and found identical/unaffected, and remain
byte-equivalence targets as before. It does not assert the mfw generator has no other
defects; only that this specific ordering axis was checked and found correct against the
admitted-index law.

## See Also

- `tests/fixtures/PRE_MIGRATION_BASELINE.md` — the fixture-freeze document this
  supplements (item 4)
- `tests/consumer_correspondence.rs` — the updated correspondence test
- `/Users/sac/mfw/tools/cmca-generator/tests/test_ordering_law.py` — generator-side
  ordering-law evidence
- `/Users/sac/mfw/mfw-ontology/cmca/generalization.ttl` — the admitted
  `cmca:measureIndex`/`cmca:lensIndex` source of truth
- `/Users/sac/bcinr/docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` — array-order-is-semantic
  clause
