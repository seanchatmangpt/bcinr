// ILLUSTRATIVE — pilot pseudocode-as-Rust, generated from ConstitutionIR claim
// `cmca.numeric.fault-join-semilattice` (see claim.yaml in this directory).
//
// This file does NOT compile against the real bcinr-cmca crate. It does not import the real
// `fixed.rs`/`allocator.rs` types (whose actual public API is, per the currently-running
// background verification of the real crate, itself in flux — see the real
// crates/bcinr-cmca/MUTANT_KILL_MATRIX.md for the live from_bits/val API-break finding, read
// here only for cross-reference, not touched by this pilot). Placeholder types below stand in
// for whatever the real crate's fault-bearing step/fault-set types turn out to be.
//
// A real generator, given claim.yaml's `falsifier_family` and `evidence_classes_required:
// [property_test]`, would need to bind these placeholders to real crate types and a real
// property-test harness (proptest, per this workspace's existing dev-dependency) — that
// binding step is exactly the "ad-hoc decision" flagged in SUMMARY.md.

use proptest::prelude::*;

/// Placeholder stand-in for the real crate's fault-tag type. The real invariant does not
/// specify a representation (bitset vs. enum-set vs. Vec<Tag>) — this pilot picks a small
/// closed enum arbitrarily, for illustration only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FaultTag {
    Overflow,
    DomainRefusal,
    RangeExceeded,
}

/// Placeholder stand-in for the real crate's fault-set type (`NumericFaultSet`, per the real
/// ledger's G2 entry, which notes this type is a "missing ... type decision", not yet
/// implemented in the real crate as of this writing).
type FaultSet = std::collections::BTreeSet<FaultTag>;

/// Placeholder stand-in for a fault-bearing computation "step": something that, given no
/// input beyond its own construction, deterministically raises a fixed fault set. The real
/// steps in `fixed.rs`/`allocator.rs` are value-bearing (value, fault) pairs, not bare
/// fault-raising thunks — this pilot's simplification for illustration purposes.
struct Step {
    faults: FaultSet,
}

impl Step {
    fn raising(tags: &[FaultTag]) -> Self {
        Step { faults: tags.iter().copied().collect() }
    }

    /// Placeholder stand-in for the real crate's sequential-composition operator. THIS is
    /// the function under test — a real generator would bind this to the actual composition
    /// function found in `allocator.rs`/`fixed.rs`, not this local reimplementation.
    fn compose(&self, other: &Step) -> FaultSet {
        // Illustrative CORRECT implementation (join-semilattice union), so this file's own
        // proptest below passes against itself. A real generated test would call into the
        // production function, not this local reimplementation, and would fail if the
        // production function used first-wins or last-wins instead.
        self.faults.union(&other.faults).copied().collect()
    }
}

/// Independent oracle: computed directly from the invariant statement in claim.yaml
/// (`faults(a ; b) = faults(a) union faults(b)`), not derived by reading `Step::compose`'s
/// own logic back out — satisfying the "independent, non-production-derived oracle"
/// requirement in numeric-hot-path.md's Required Evidence Class section.
fn oracle_union(a: &FaultSet, b: &FaultSet) -> FaultSet {
    a.union(b).copied().collect()
}

fn arb_fault_tag() -> impl Strategy<Value = FaultTag> {
    prop_oneof![
        Just(FaultTag::Overflow),
        Just(FaultTag::DomainRefusal),
        Just(FaultTag::RangeExceeded),
    ]
}

fn arb_fault_set() -> impl Strategy<Value = FaultSet> {
    prop::collection::vec(arb_fault_tag(), 0..=3).prop_map(|v| v.into_iter().collect())
}

proptest! {
    /// Direct encoding of claim.yaml's `falsifier_family[0]`: construct two independently
    /// fault-bearing steps and check the composed result equals the independent oracle's
    /// union, over the admitted domain (arbitrary fault-tag combinations), not a fixed
    /// example set.
    #[test]
    fn fault_composition_is_union_over_arbitrary_faults(
        faults_a in arb_fault_set(),
        faults_b in arb_fault_set(),
    ) {
        let step_a = Step { faults: faults_a.clone() };
        let step_b = Step { faults: faults_b.clone() };

        let composed = step_a.compose(&step_b);
        let expected = oracle_union(&faults_a, &faults_b);

        prop_assert_eq!(composed, expected);
    }

    /// The specific falsifier from claim.yaml: two DISJOINT, non-overlapping single-fault
    /// steps must retain BOTH faults after composition, not collapse to first-only or
    /// last-only.
    #[test]
    fn disjoint_single_faults_both_survive_composition(
        tag_a in arb_fault_tag(),
        tag_b in arb_fault_tag(),
    ) {
        prop_assume!(tag_a != tag_b);

        let step_a = Step::raising(&[tag_a]);
        let step_b = Step::raising(&[tag_b]);

        let composed = step_a.compose(&step_b);

        prop_assert!(composed.contains(&tag_a), "first-fault must not be dropped (last-wins collapse)");
        prop_assert!(composed.contains(&tag_b), "second-fault must not be dropped (first-wins collapse)");
        prop_assert_eq!(composed.len(), 2, "composed set must be exactly the two-element union, no more, no less");
    }

    /// Zero-element check from claim.yaml's postcondition: the empty fault set is the
    /// identity element of composition.
    #[test]
    fn empty_fault_set_is_composition_identity(faults in arb_fault_set()) {
        let step = Step { faults: faults.clone() };
        let empty = Step { faults: FaultSet::new() };

        prop_assert_eq!(step.compose(&empty), faults.clone());
        prop_assert_eq!(empty.compose(&step), faults);
    }
}
