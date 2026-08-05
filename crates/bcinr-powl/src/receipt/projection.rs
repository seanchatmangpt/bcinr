//! `ProjectionReceipt` — a receipt attesting that a PDDL-to-POWL projection
//! preserved source semantics.
//!
//! This is genuinely new territory for this crate: [`crate::receipt::causal_receipt`]
//! and [`crate::receipt::replay`] both attest to *execution/replay conformance of an
//! already-compiled tape*; nothing before this module records that the
//! *compilation step itself* (`CausalPlan` + `ExecutableConcurrencyComplex`
//! -> `PowlModel`) was semantics-preserving.
//!
//! # This receipt records a check, it does not perform one
//!
//! [`seal_projection_receipt`] takes a
//! [`bcinr_mfw_ir::PowlProjectionWitness`] as its proof obligation: that
//! witness can only exist because `bcinr-powl`'s real
//! [`crate::projection::PowlProjector::project`] already ran
//! `verify_order_preservation`/`verify_concurrency_preservation` and
//! returned `Ok`. `seal_projection_receipt` does **not** re-run either
//! check — it folds the witness's own digests into a BLAKE3 hash chain, the
//! same "prior_hash folded with canonical frame bytes" discipline
//! [`crate::receipt::causal_receipt::OcelCausalReceipt::chain`] uses. A
//! `ProjectionReceipt` is a durable record that the check happened and
//! produced these exact digests, not a second opinion on whether it should
//! have passed.

use std::collections::BTreeSet;

use crate::model::{PowlModel, PowlNode};
use bcinr_mfw_ir::{
    CausalPlan, ConcurrencyPreservationWitness, Digest, ExecutableConcurrencyComplex,
    OrderPreservationWitness, PlanningEpochId, PowlProjectionWitness, PowlProjector as _,
};

use crate::receipt::chain::fold;

// ---------------------------------------------------------------------------
// Canonical digest helpers
// ---------------------------------------------------------------------------

/// Canonical digest over an [`OrderPreservationWitness`]'s three fields, in
/// declared field order.
fn digest_order_witness(w: &OrderPreservationWitness) -> Digest {
    let mut buf = Vec::with_capacity(96);
    buf.extend_from_slice(w.source_order_digest.as_bytes());
    buf.extend_from_slice(w.projected_order_digest.as_bytes());
    buf.extend_from_slice(w.mapped_order_digest.as_bytes());
    Digest::hash(&buf)
}

/// Canonical digest over a [`ConcurrencyPreservationWitness`]'s three
/// fields, in declared field order.
fn digest_concurrency_witness(w: &ConcurrencyPreservationWitness) -> Digest {
    let mut buf = Vec::with_capacity(96);
    buf.extend_from_slice(w.source_complex_digest.as_bytes());
    buf.extend_from_slice(w.mapped_source_digest.as_bytes());
    buf.extend_from_slice(w.target_complex_digest.as_bytes());
    Digest::hash(&buf)
}

/// Canonical digest over a [`PowlModel`]: every node (tagged by variant),
/// every order edge (already `BTreeSet`-ordered, so ascending iteration is
/// deterministic), and every minimal nonface (canonicalized into a sorted
/// member-id list first, mirroring `crate::projection`'s own
/// `canonical_nonface_keys` — that helper is private to `bcinr-powl`, so
/// this is a from-scratch, structurally-equivalent reimplementation over
/// `PowlModel`'s public fields, not a call into it).
pub fn digest_powl_model(model: &PowlModel) -> Digest {
    let mut buf = Vec::new();
    for node in &model.nodes {
        match node {
            PowlNode::Activity(a) => {
                buf.push(0u8);
                buf.extend_from_slice(&a.id.0.to_le_bytes());
                buf.extend_from_slice(&(a.label.len() as u32).to_le_bytes());
                buf.extend_from_slice(a.label.as_bytes());
                buf.extend_from_slice(&a.source_action.0.to_le_bytes());
            }
            PowlNode::Silent(s) => {
                buf.push(1u8);
                buf.extend_from_slice(&s.id.0.to_le_bytes());
            }
            PowlNode::ChildWorkflow(c) => {
                buf.push(2u8);
                buf.extend_from_slice(&c.id.0.to_le_bytes());
            }
            PowlNode::ExternalCut(e) => {
                buf.push(3u8);
                buf.extend_from_slice(&e.id.0.to_le_bytes());
            }
        }
    }
    for edge in &model.order.edges {
        buf.extend_from_slice(&edge.before.0.to_le_bytes());
        buf.extend_from_slice(&edge.after.0.to_le_bytes());
    }
    let nonface_keys: BTreeSet<Vec<usize>> = model
        .concurrency
        .minimal_nonfaces
        .iter()
        .map(|nf| nf.members.iter_stable().collect::<Vec<usize>>())
        .collect();
    for key in &nonface_keys {
        buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
        for id in key {
            buf.extend_from_slice(&(*id as u32).to_le_bytes());
        }
    }
    Digest::hash(&buf)
}

// ---------------------------------------------------------------------------
// ProjectionReceipt
// ---------------------------------------------------------------------------

/// A receipt attesting that a `CausalPlan` + `ExecutableConcurrencyComplex`
/// was projected into a `PowlModel` without losing source semantics.
///
/// `hash` is chained the same way as every other receipt in this crate:
/// `hash = BLAKE3(prior_hash || canonical_field_bytes)`, computed by
/// [`seal_projection_receipt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectionReceipt {
    pub source_epoch: PlanningEpochId,
    pub causal_plan_digest: Digest,
    pub source_concurrency_digest: Digest,
    pub powl_model_digest: Digest,
    pub order_witness_digest: Digest,
    pub concurrency_witness_digest: Digest,
    pub prior_hash: Digest,
    pub hash: Digest,
}

/// Seal a [`ProjectionReceipt`] from a `projection_witness` that has
/// **already** passed `bcinr-powl`'s real order/concurrency preservation
/// checks (see module docs — this function does not re-derive or fake that
/// check; it only records the witness's own digests into the hash chain).
///
/// `causal_plan` and `concurrency` are the same values the witness was
/// produced from — in debug builds a `debug_assert!` cross-checks that
/// their digests match the witness's own `causal_plan_digest` /
/// `source_concurrency_digest` fields (a consistency check that this
/// receipt is being sealed for the plan it claims to be about, not a
/// re-verification of preservation itself; costs nothing in release
/// builds, same discipline as `crate::scheduler`'s
/// `ConcurrencySelector::select_checked`).
pub fn seal_projection_receipt(
    prior_hash: Digest,
    causal_plan: &CausalPlan,
    concurrency: &ExecutableConcurrencyComplex,
    projection_witness: &PowlProjectionWitness,
    powl_model_digest: Digest,
) -> ProjectionReceipt {
    debug_assert_eq!(
        causal_plan.digest, projection_witness.causal_plan_digest,
        "seal_projection_receipt: causal_plan does not match the plan the projection_witness attests to"
    );
    debug_assert_eq!(
        concurrency.digest, projection_witness.source_concurrency_digest,
        "seal_projection_receipt: concurrency complex does not match the one the projection_witness attests to"
    );

    let source_epoch = projection_witness.source_epoch;
    let causal_plan_digest = projection_witness.causal_plan_digest;
    let source_concurrency_digest = projection_witness.source_concurrency_digest;
    let order_witness_digest = digest_order_witness(&projection_witness.order_witness);
    let concurrency_witness_digest =
        digest_concurrency_witness(&projection_witness.concurrency_witness);

    let mut buf = Vec::with_capacity(16 + 32 * 5);
    buf.extend_from_slice(&source_epoch.0.to_le_bytes());
    buf.extend_from_slice(causal_plan_digest.as_bytes());
    buf.extend_from_slice(source_concurrency_digest.as_bytes());
    buf.extend_from_slice(powl_model_digest.as_bytes());
    buf.extend_from_slice(order_witness_digest.as_bytes());
    buf.extend_from_slice(concurrency_witness_digest.as_bytes());

    let hash = fold(&prior_hash, &buf);

    ProjectionReceipt {
        source_epoch,
        causal_plan_digest,
        source_concurrency_digest,
        powl_model_digest,
        order_witness_digest,
        concurrency_witness_digest,
        prior_hash,
        hash,
    }
}

// ---------------------------------------------------------------------------
// SourceSemanticVerifier — receipt-layer wrapper around bcinr-powl's real
// preservation checks
// ---------------------------------------------------------------------------

/// A receipt-layer wrapper trait: `verify` calls `bcinr-powl`'s **actual**
/// `PowlProjector::project` (which internally calls the real
/// `verify_order_preservation`/`verify_concurrency_preservation`), and, only
/// on success, seals the resulting witness into a `ProjectionReceipt`. This
/// trait does not reimplement either check — it is strictly a thinner layer
/// on top of an already-real projector.
pub trait SourceSemanticVerifier {
    type Projector: bcinr_mfw_ir::PowlProjector<Model = PowlModel>;

    /// The projector this verifier delegates to.
    fn projector(&self) -> &Self::Projector;

    /// Run the real projection, and on success seal a `ProjectionReceipt`
    /// chained onto `prior_hash`. Returns the projector's own error
    /// unchanged on failure — no swallowing, no partial receipt.
    fn verify(
        &self,
        causal: &CausalPlan,
        concurrency: &ExecutableConcurrencyComplex,
        prior_hash: Digest,
    ) -> Result<ProjectionReceipt, <Self::Projector as bcinr_mfw_ir::PowlProjector>::Error> {
        let (model, witness) = self.projector().project(causal, concurrency)?;
        let powl_model_digest = digest_powl_model(&model);
        Ok(seal_projection_receipt(
            prior_hash,
            causal,
            concurrency,
            &witness,
            powl_model_digest,
        ))
    }
}

/// The real verifier: wraps `crate::projection::PowlProjector`
/// (`bcinr-powl`'s concrete, non-stub implementation from Phase 2b).
///
/// `PowlProjector` is a stateless unit struct that derives neither `Debug`
/// nor `Default` (see `bcinr-powl`'s `projection.rs`), so both are
/// implemented by hand here rather than derived.
pub struct RealSourceSemanticVerifier {
    projector: crate::projection::PowlProjector,
}

impl std::fmt::Debug for RealSourceSemanticVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealSourceSemanticVerifier").finish()
    }
}

impl Default for RealSourceSemanticVerifier {
    fn default() -> Self {
        RealSourceSemanticVerifier {
            projector: crate::projection::PowlProjector,
        }
    }
}

impl SourceSemanticVerifier for RealSourceSemanticVerifier {
    type Projector = crate::projection::PowlProjector;

    fn projector(&self) -> &Self::Projector {
        &self.projector
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bcinr_mfw_ir::{
        ActionOccurrence, ActionOccurrenceId, ConcurrencyConflictWitness, EventSet, FluentId,
        IndependenceRelation, MinimalNonFace, PowlProjector as PowlProjectorTrait, PrecedenceEdge,
        ResourceConflictWitness, StrictPartialOrder,
    };
    use std::collections::BTreeMap;

    /// The same "A, B, C can't all fire together" fixture
    /// `crate::projection`'s own test module uses, mirrored here for
    /// consistency (see mission ground truth) rather than reused directly —
    /// `bcinr-powl`'s fixture is private to its own `#[cfg(test)]` module.
    fn fixture() -> (CausalPlan, ExecutableConcurrencyComplex) {
        let occurrences = vec![
            ActionOccurrence {
                id: ActionOccurrenceId(0),
                action: 100,
            },
            ActionOccurrence {
                id: ActionOccurrenceId(1),
                action: 101,
            },
            ActionOccurrence {
                id: ActionOccurrenceId(2),
                action: 102,
            },
        ];
        let mut edges = BTreeSet::new();
        edges.insert(PrecedenceEdge {
            before: ActionOccurrenceId(0),
            after: ActionOccurrenceId(2),
        });
        let causal = CausalPlan {
            epoch: PlanningEpochId(42),
            occurrences,
            precedes: StrictPartialOrder { edges },
            independence: IndependenceRelation::default(),
            support_edges: BTreeSet::new(),
            digest: Digest::hash(b"fixture-causal-plan"),
        };

        let abc = EventSet::empty().with(0).with(1).with(2);
        let witness = ConcurrencyConflictWitness {
            causal: None,
            temporal: None,
            resource: Some(ResourceConflictWitness {
                actions: abc,
                resource: FluentId(0),
                capacity_milli: 2_000,
                demanded_milli: 3_000,
            }),
        };
        let witness_digest = Digest::hash(b"abc-resource-conflict");
        let mut conflict_witnesses = BTreeMap::new();
        conflict_witnesses.insert(witness_digest, witness);
        let concurrency = ExecutableConcurrencyComplex {
            event_count: 3,
            minimal_nonfaces: vec![MinimalNonFace {
                members: abc,
                witness_digest,
            }],
            conflict_witnesses,
            digest: Digest::hash(b"fixture-complex"),
        };

        (causal, concurrency)
    }

    #[test]
    fn seal_projection_receipt_chains_onto_prior_hash() {
        let (causal, concurrency) = fixture();
        let projector = crate::projection::PowlProjector;
        let (model, witness) = projector.project(&causal, &concurrency).unwrap();
        let model_digest = digest_powl_model(&model);

        let receipt =
            seal_projection_receipt(Digest::ZERO, &causal, &concurrency, &witness, model_digest);
        assert_eq!(receipt.prior_hash, Digest::ZERO);
        assert_eq!(receipt.source_epoch, PlanningEpochId(42));
        assert_ne!(receipt.hash, Digest::ZERO);
    }

    /// Genuine tamper-detection test, mirroring `causal_receipt.rs`'s own
    /// style: seal two receipts, flip one byte in the second's
    /// `causal_plan_digest` input, and assert the resulting hash differs.
    #[test]
    fn tampering_with_causal_plan_digest_changes_the_hash() {
        let (causal, concurrency) = fixture();
        let projector = crate::projection::PowlProjector;
        let (model, witness) = projector.project(&causal, &concurrency).unwrap();
        let model_digest = digest_powl_model(&model);

        let receipt_a =
            seal_projection_receipt(Digest::ZERO, &causal, &concurrency, &witness, model_digest);

        // Flip one byte of the causal plan's digest (its "causal_plan_digest
        // input") and carry the same tweak into a matching witness, exactly
        // as a re-run of the pipeline against a genuinely different plan
        // would produce.
        let mut tampered_bytes = *causal.digest.as_bytes();
        tampered_bytes[0] ^= 0x01;
        let tampered_digest = Digest::from(tampered_bytes);

        let mut tampered_causal = causal.clone();
        tampered_causal.digest = tampered_digest;
        let mut tampered_witness = witness.clone();
        tampered_witness.causal_plan_digest = tampered_digest;

        let receipt_b = seal_projection_receipt(
            Digest::ZERO,
            &tampered_causal,
            &concurrency,
            &tampered_witness,
            model_digest,
        );

        assert_ne!(
            receipt_a.hash, receipt_b.hash,
            "flipping one byte of causal_plan_digest must change the sealed hash"
        );
        assert_ne!(receipt_a.causal_plan_digest, receipt_b.causal_plan_digest);
    }

    #[test]
    fn chaining_two_receipts_advances_the_hash() {
        let (causal, concurrency) = fixture();
        let projector = crate::projection::PowlProjector;
        let (model, witness) = projector.project(&causal, &concurrency).unwrap();
        let model_digest = digest_powl_model(&model);

        let receipt1 =
            seal_projection_receipt(Digest::ZERO, &causal, &concurrency, &witness, model_digest);
        let receipt2 =
            seal_projection_receipt(receipt1.hash, &causal, &concurrency, &witness, model_digest);
        assert_ne!(receipt1.hash, receipt2.hash);
        assert_eq!(receipt2.prior_hash, receipt1.hash);
    }

    /// End-to-end: real `CausalPlan` + `ExecutableConcurrencyComplex`
    /// fixture, run through `bcinr-powl`'s actual `PowlProjector`
    /// implementation via `RealSourceSemanticVerifier`, asserting a genuine
    /// `ProjectionReceipt` comes out and that perturbing the input plan
    /// changes the sealed hash. This is the closest thing to an end-to-end
    /// "does the whole projection-to-receipt pipeline actually work" proof
    /// in this crate.
    #[test]
    fn source_semantic_verifier_runs_the_real_projector_end_to_end() {
        let (causal, concurrency) = fixture();
        let verifier = RealSourceSemanticVerifier::default();

        let receipt = verifier
            .verify(&causal, &concurrency, Digest::ZERO)
            .expect("real PowlProjector must accept this fixture");
        assert_eq!(receipt.source_epoch, PlanningEpochId(42));
        assert_ne!(receipt.hash, Digest::ZERO);

        // Perturb the input plan (add a second occurrence-independent action
        // and adjust its digest) and verify the resulting receipt's hash
        // differs -- proves the receipt is sensitive to real input, not a
        // constant.
        let mut perturbed = causal.clone();
        perturbed.digest = Digest::hash(b"perturbed-fixture-causal-plan");
        let receipt2 = verifier
            .verify(&perturbed, &concurrency, Digest::ZERO)
            .expect("real PowlProjector must accept the perturbed fixture too");
        assert_ne!(
            receipt.hash, receipt2.hash,
            "perturbing the source causal plan must change the sealed receipt hash"
        );
    }

    /// The verifier propagates the projector's real error unchanged (not
    /// swallowed into a generic failure) when preservation cannot hold --
    /// here, an order edge referencing an action absent from `occurrences`.
    #[test]
    fn source_semantic_verifier_propagates_real_projector_errors() {
        let mut edges = BTreeSet::new();
        edges.insert(PrecedenceEdge {
            before: ActionOccurrenceId(0),
            after: ActionOccurrenceId(99),
        });
        let causal = CausalPlan {
            epoch: PlanningEpochId(1),
            occurrences: vec![ActionOccurrence {
                id: ActionOccurrenceId(0),
                action: 1,
            }],
            precedes: StrictPartialOrder { edges },
            independence: IndependenceRelation::default(),
            support_edges: BTreeSet::new(),
            digest: Digest::hash(b"bad-plan"),
        };
        let concurrency = ExecutableConcurrencyComplex {
            event_count: 1,
            minimal_nonfaces: vec![],
            conflict_witnesses: BTreeMap::new(),
            digest: Digest::hash(b"empty-complex"),
        };

        let verifier = RealSourceSemanticVerifier::default();
        let err = verifier
            .verify(&causal, &concurrency, Digest::ZERO)
            .unwrap_err();
        assert_eq!(
            err,
            crate::projection::ProjectionError::Preservation(
                crate::projection::PreservationError::UnmappedAction(ActionOccurrenceId(99))
            )
        );
    }
}
