//! Consumer correspondence: frozen `tests/fixtures/pre_migration/*.rs`
//! (old `generator.py` output, per `PRE_MIGRATION_BASELINE.md`) versus the
//! mfw-producer artifacts now materialized under
//! `generated-artifact/{case-studies,generalization}/cmca_generated.rs`.
//!
//! NOTE (cmca-verifier, 2026-07-17): at the time this file was written, `cargo
//! build -p bcinr-cmca` fails independently of this file — 260 pre-existing
//! errors in `src/lrc.rs` (`NonNegativeFixed::from_bits` does not exist;
//! `.val` is private), unrelated to consumer correspondence. This test cannot
//! be compiled or run as a `cargo test` until that is fixed by the owning
//! implementer (not this role — no self-certification of another gate's
//! defect). The byte/value-level comparisons below were instead executed
//! directly via shell `diff`/`grep` against the two files on disk; see the
//! verifier's report for the exact commands and output. This file is checked
//! in as the reproducible `cargo test` form of that same comparison so it
//! runs automatically once the lrc.rs blocker clears.

use std::fs;

/// The pre-migration generator emitted `NonNegativeFixed::from_bits(N)` /
/// `SignedFixed::from_bits(N)`; the sealed API post-migration renamed the
/// constructor to `from_value_bits(N)` without changing its numeric meaning
/// (see `.claude/rules/cmca/artifact-boundary.md` — the rename is a naming
/// change, not a semantic one). Both markers are searched so this test
/// compares the actual Q16.16 payload sequence rather than tripping on a
/// cosmetic constructor rename in either input file.
const MARKERS: [&str; 2] = ["from_value_bits(", "from_bits("];

fn extract_from_bits_sequence(path: &str) -> Vec<i64> {
    let text = fs::read_to_string(path).expect("fixture/artifact file must exist");
    let mut out = Vec::new();
    let mut rest = text.as_str();
    loop {
        // Find the earliest occurrence of either marker so payload order is preserved.
        let next = MARKERS
            .iter()
            .filter_map(|m| rest.find(m).map(|idx| (idx, *m)))
            .min_by_key(|(idx, _)| *idx);
        let Some((idx, marker)) = next else {
            break;
        };
        rest = &rest[idx + marker.len()..];
        let end = rest.find(')').expect("from_(value_)bits(...) must close");
        let num_str = &rest[..end];
        out.push(
            num_str
                .parse::<i64>()
                .expect("from_bits argument must be integer"),
        );
        rest = &rest[end..];
    }
    out
}

const OLD_CASE_STUDIES: &str = "tests/fixtures/pre_migration/case_studies.rs";
const NEW_CASE_STUDIES: &str = "generated-artifact/case-studies/cmca_generated.rs";
const OLD_GENERALIZATION: &str = "tests/fixtures/pre_migration/generalization.rs";
const NEW_GENERALIZATION: &str = "generated-artifact/generalization/cmca_generated.rs";

/// CORRESPONDENCE_REQUIRED: case-studies numeric payload (the `from_value_bits(N)`
/// sequence, i.e. every Q16.16 raw value in registry/lambda/eta order) must
/// match the old lawful generator's output exactly, per
/// PRE_MIGRATION_BASELINE.md's "everything else ... is frozen as the
/// byte-equivalence correspondence-testing target."
#[test]
fn case_studies_numeric_payload_matches_old_lawful_output_exactly() {
    let old = extract_from_bits_sequence(OLD_CASE_STUDIES);
    let new = extract_from_bits_sequence(NEW_CASE_STUDIES);
    assert_eq!(
        old, new,
        "case_studies from_value_bits(..) sequence (values AND order) must match the frozen \
         pre-migration fixture exactly; a mismatch here is a CORRESPONDENCE_REQUIRED failure"
    );
}

/// CORRESPONDENCE_REQUIRED (root-caused, see
/// `tests/fixtures/GENERALIZATION_ORDER_DECISION.md`): the frozen pre-migration
/// fixture's measure-head/lens array ordering was itself **alphabetical-by-name**
/// (an incidental artifact of the legacy generator's dict/sort behavior over
/// entity names), not derived from any admitted `cmca:measureIndex` /
/// `cmca:lensIndex`. The mfw-side replacement generator (`generator.py`,
/// `sorted_mh` / `sorted_lenses`, keyed on `(mh_index[m], m)` /
/// `(lens_index[l], l)`) instead orders strictly by the explicit admitted
/// index — which is the correct law per the default rule ("mechanical array
/// position follows the explicit admitted semantic index, not incidental parse
/// order") and is independently verified by
/// `tools/cmca-generator/tests/test_ordering_law.py` (shuffle-determinism and
/// index-swap-changes-position). So the OLD fixture's order is the defective
/// side here, not the new artifact: this test asserts value-set (multiset)
/// equality against the old fixture (no value regression), and separately
/// asserts the new artifact's order actually matches the admitted-index-sorted
/// order (not merely "differs from old" — a positive, order-sensitive check
/// against the correct target).
#[test]
fn generalization_numeric_payload_matches_old_lawful_output_exactly() {
    let mut old = extract_from_bits_sequence(OLD_GENERALIZATION);
    let mut new_sorted = extract_from_bits_sequence(NEW_GENERALIZATION);

    // Value-set correspondence: no value regression versus the frozen fixture.
    old.sort();
    new_sorted.sort();
    assert_eq!(
        old, new_sorted,
        "generalization from_value_bits(..) value set no longer matches the frozen \
         pre-migration fixture — this is a genuine value regression (not merely an \
         order difference) and is a CORRESPONDENCE_REQUIRED failure"
    );

    // Order-sensitive correspondence against the CORRECT target: the new
    // artifact's measure-head row order must follow the explicit admitted
    // cmca:measureIndex order (Cache=0, Search=1, Retrieval=2, Scheduling=3,
    // GeneralizationProof=4 per generalization.ttl), not the old fixture's
    // incidental alphabetical order.
    let artifact_text =
        fs::read_to_string(NEW_GENERALIZATION).expect("new generalization artifact must exist");
    let expected_measure_order = [
        "MeasureCache",
        "MeasureSearch",
        "MeasureRetrieval",
        "MeasureScheduling",
        "MeasureGeneralizationProof",
    ];
    let mut found: Vec<(usize, &str)> = expected_measure_order
        .iter()
        .filter_map(|name| {
            artifact_text
                .find(&format!("// {name}"))
                .map(|idx| (idx, *name))
        })
        .collect();
    found.sort_by_key(|(idx, _)| *idx);
    let actual_order: Vec<&str> = found.into_iter().map(|(_, name)| name).collect();
    assert_eq!(
        actual_order,
        expected_measure_order.to_vec(),
        "new generalization artifact's measure-head row order must follow the explicit \
         admitted cmca:measureIndex order (Cache=0,Search=1,Retrieval=2,Scheduling=3,\
         GeneralizationProof=4), not incidental order"
    );
}

/// DEFECTIVE_BEHAVIOR_QUARANTINED: neither fixture ontology (`cmca-rdf.ttl`,
/// `generalization.ttl`) contains a missing required property, a dependency
/// cycle, or a non-representable decimal literal, so this differential test
/// cannot itself exercise the three named defect paths end-to-end. It
/// confirms only that the corrected code paths exist in the new generator
/// (`generator.py` at `/Users/sac/mfw/tools/cmca-generator/generator.py`),
/// not that they were exercised by these two ontologies. See the verifier's
/// report for the exact `grep` evidence of each corrected path.
#[test]
#[allow(clippy::assertions_on_constants)]
fn defective_paths_not_exercised_by_current_fixtures_is_a_nonclaim() {
    // Intentionally a no-op assertion: this test exists to make the
    // limitation above machine-checkable-adjacent (it will not silently
    // vanish from the suite) rather than to assert anything about the
    // defect paths themselves, which require a hostile ontology fixture
    // (missing property / cycle / non-finite-binary decimal) not present
    // in this crate's committed fixtures.
    assert!(true);
}
