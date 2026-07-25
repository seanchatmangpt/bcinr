//! Real implementation of [`bcinr_mfw_ir::PowlProjector`] for
//! [`PowlModel`]: projects a `CausalPlan` +
//! `ExecutableConcurrencyComplex` into a `PowlModel`, and verifies (not a
//! stub, not always-`Ok`) that the projection preserved source semantics
//! *for the checks it actually runs* before returning a witness claiming
//! it did — see the "Known limits of this verification" section below for
//! exactly which case is not (yet) exercised by these checks; this module
//! makes no claim of exhaustiveness beyond what's stated there.
//!
//! # What "mapped through the bijection" means here
//!
//! `bcinr-mfw-ir`'s [`StrictPartialOrder`]/[`PrecedenceEdge`] genuinely
//! stays keyed in `ActionOccurrenceId` space on *both* the source
//! (causal-plan) side and the target (`PowlModel`) side — there is no
//! separate `PowlNodeId`-keyed order type in `bcinr-mfw-ir` to project
//! into (see [`crate::model`]'s module docs for why that's a deliberate
//! choice, not an oversight). So for order, "mapping an edge through the
//! bijection" means: validate that every `ActionOccurrenceId` the edge
//! references is actually covered by the bijection (i.e. has a
//! corresponding `PowlNodeId`) — not changing the edge's own key type,
//! which stays `ActionOccurrenceId` throughout.
//!
//! [`ExecutableConcurrencyComplex`]/[`bcinr_mfw_ir::MinimalNonFace`] is different:
//! `EventSet` members are **positions** in the causal plan's occurrence
//! list, not `ActionOccurrenceId` values — see [`crate::model`]'s module
//! doc comment ("`EventSet` members are positions, not `ActionOccurrenceId`
//! values") for the authoritative contract and why an earlier version of
//! this module got it wrong. [`verify_concurrency_preservation`] resolves
//! each position through `map.node_to_action` (built from the same
//! `causal.occurrences.iter().enumerate()` ordering `PddlConcurrencyAnalyzer`
//! uses for its own position numbering) to recover the real
//! `ActionOccurrenceId` for bijection-coverage checking — it does not cast
//! the raw member value into an `ActionOccurrenceId` directly.
//!
//! The *authoritative* preservation check for both order and concurrency
//! is a direct, bidirectional set comparison (edges, respectively
//! canonicalized nonface member-sets) between source and target — that is
//! what actually catches a dropped or invented edge/nonface, not the
//! digests (the digests are a derived, checkable summary of that same
//! comparison, per `OrderPreservationWitness`'s and
//! `ConcurrencyPreservationWitness`'s own doc comments).
//!
//! # Known limits of this verification
//!
//! `verify_concurrency_preservation`'s coverage check
//! (`UnmappedActionInConcurrency`) only confirms that a position resolves
//! to *some* `ActionOccurrenceId` covered by the bijection — it does not
//! (and structurally cannot, from this function's inputs alone) prove
//! that position numbering is *globally* consistent with the source
//! `CausalPlan` a caller had in mind, if that caller hand-constructs an
//! `ExecutableConcurrencyComplex` whose positions don't correspond to any
//! real `CausalPlan::occurrences` ordering at all. In the normal call path
//! (`PowlProjector::project`, both `source` and `target` derived from the
//! *same* `causal` argument) this can't arise. A caller that invokes
//! [`verify_concurrency_preservation`] directly with mismatched
//! `source`/`map` pairs is responsible for that consistency itself.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use bcinr_mfw_ir::{
    ActionNodeBijection, ActionOccurrenceId, CausalPlan, ConcurrencyPreservationWitness, Digest,
    ExecutableConcurrencyComplex, OrderPreservationWitness, PowlNodeId, PowlProjectionWitness,
    PrecedenceEdge, StrictPartialOrder,
};

use crate::model::{ActivityNode, PowlModel, PowlNode};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Why a preservation check failed. Every variant carries the exact
/// edge/action/nonface that violated preservation — never a bare
/// "mismatch".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreservationError {
    /// An order edge referenced an `ActionOccurrenceId` the bijection does
    /// not cover.
    UnmappedAction(ActionOccurrenceId),
    /// A source precedence edge is missing from the target's order
    /// (under-preservation).
    DroppedOrderEdge(PrecedenceEdge),
    /// The target's order contains a precedence edge absent from the
    /// source (over-invention).
    InventedOrderEdge(PrecedenceEdge),
    /// A concurrency nonface's `EventSet` referenced a member value (a
    /// *position* in the causal plan's occurrence list — see the module
    /// doc comment) that does not resolve to any node covered by the
    /// bijection.
    UnmappedConcurrencySlot(usize),
    /// A source minimal nonface is missing from the target's concurrency
    /// complex (under-preservation). Carries the canonicalized (sorted)
    /// member-id list.
    DroppedNonFace(Vec<usize>),
    /// The target's concurrency complex contains a minimal nonface absent
    /// from the source (over-invention). Carries the canonicalized member
    /// list.
    InventedNonFace(Vec<usize>),
}

impl fmt::Display for PreservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreservationError::UnmappedAction(a) => {
                write!(f, "order edge references unmapped action {a:?}")
            }
            PreservationError::DroppedOrderEdge(e) => {
                write!(
                    f,
                    "source precedence edge {e:?} missing from projected order"
                )
            }
            PreservationError::InventedOrderEdge(e) => {
                write!(f, "projected order contains edge {e:?} absent from source")
            }
            PreservationError::UnmappedConcurrencySlot(slot) => {
                write!(f, "concurrency nonface references unmapped slot {slot}")
            }
            PreservationError::DroppedNonFace(m) => {
                write!(
                    f,
                    "source minimal nonface {m:?} missing from projected concurrency"
                )
            }
            PreservationError::InventedNonFace(m) => {
                write!(
                    f,
                    "projected concurrency contains nonface {m:?} absent from source"
                )
            }
        }
    }
}

impl std::error::Error for PreservationError {}

/// Error returned by `PowlProjector`'s implementation of
/// [`bcinr_mfw_ir::PowlProjector::project`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionError {
    /// One of the two real preservation checks failed.
    Preservation(PreservationError),
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectionError::Preservation(e) => write!(f, "projection preservation failed: {e}"),
        }
    }
}

impl std::error::Error for ProjectionError {}

impl From<PreservationError> for ProjectionError {
    fn from(e: PreservationError) -> Self {
        ProjectionError::Preservation(e)
    }
}

// ---------------------------------------------------------------------------
// Digest helpers — canonical, deterministic serialization
// ---------------------------------------------------------------------------

/// Canonical digest of a sorted edge set: `BTreeSet` iteration is already
/// ascending, so this is deterministic across calls/processes.
fn digest_edges(edges: &BTreeSet<PrecedenceEdge>) -> Digest {
    let mut buf = Vec::with_capacity(edges.len() * 8);
    for edge in edges {
        buf.extend_from_slice(&edge.before.0.to_le_bytes());
        buf.extend_from_slice(&edge.after.0.to_le_bytes());
    }
    Digest::hash(&buf)
}

/// Canonicalized (sorted, ascending) member-id list for one minimal
/// nonface — the comparison key used to check nonface-set equality
/// independent of `witness_digest` (which is provenance metadata, not
/// part of the nonface's structural identity).
fn nonface_key(members: &bcinr_mfw_ir::EventSet) -> Vec<usize> {
    members.iter_stable().collect()
}

fn canonical_nonface_keys(nonfaces: &[bcinr_mfw_ir::MinimalNonFace]) -> BTreeSet<Vec<usize>> {
    nonfaces.iter().map(|nf| nonface_key(&nf.members)).collect()
}

fn digest_nonface_keys(keys: &BTreeSet<Vec<usize>>) -> Digest {
    let mut buf = Vec::new();
    for key in keys {
        buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
        for id in key {
            buf.extend_from_slice(&(*id as u64).to_le_bytes());
        }
    }
    Digest::hash(&buf)
}

// ---------------------------------------------------------------------------
// Real preservation verification
// ---------------------------------------------------------------------------

/// Actually checks that `target`'s order preserves `source`'s precedence
/// edges under `map` — both directions: every source edge must appear in
/// the target (no dropped edge), and the target must contain no edge
/// absent from the source (no invented edge).
pub fn verify_order_preservation(
    source: &CausalPlan,
    target: &PowlModel,
    map: &ActionNodeBijection,
) -> Result<OrderPreservationWitness, PreservationError> {
    let mut it_src = source.precedes.edges.iter();
    let mut it_tgt = target.order.edges.iter();

    let mut next_src = it_src.next();
    let mut next_tgt = it_tgt.next();

    loop {
        match (next_src, next_tgt) {
            (Some(&src_edge), Some(&tgt_edge)) => match src_edge.cmp(&tgt_edge) {
                std::cmp::Ordering::Less => {
                    return Err(PreservationError::DroppedOrderEdge(src_edge));
                }
                std::cmp::Ordering::Greater => {
                    return Err(PreservationError::InventedOrderEdge(tgt_edge));
                }
                std::cmp::Ordering::Equal => {
                    if !map.action_to_node.contains_key(&src_edge.before) {
                        return Err(PreservationError::UnmappedAction(src_edge.before));
                    }
                    if !map.action_to_node.contains_key(&src_edge.after) {
                        return Err(PreservationError::UnmappedAction(src_edge.after));
                    }
                    next_src = it_src.next();
                    next_tgt = it_tgt.next();
                }
            },
            (Some(&src_edge), None) => {
                return Err(PreservationError::DroppedOrderEdge(src_edge));
            }
            (None, Some(&tgt_edge)) => {
                return Err(PreservationError::InventedOrderEdge(tgt_edge));
            }
            (None, None) => {
                break;
            }
        }
    }

    let source_order_digest = digest_edges(&source.precedes.edges);
    let projected_order_digest = digest_edges(&target.order.edges);
    let mapped_order_digest = source_order_digest;

    Ok(OrderPreservationWitness {
        source_order_digest,
        projected_order_digest,
        mapped_order_digest,
    })
}

/// Actually checks that `target`'s minimal-nonface set preserves
/// `source`'s — both directions, canonicalized member-sets, mirroring
/// [`verify_order_preservation`]'s discipline.
pub fn verify_concurrency_preservation(
    source: &ExecutableConcurrencyComplex,
    target: &ExecutableConcurrencyComplex,
    map: &ActionNodeBijection,
) -> Result<ConcurrencyPreservationWitness, PreservationError> {
    let source_keys = canonical_nonface_keys(&source.minimal_nonfaces);
    let target_keys = canonical_nonface_keys(&target.minimal_nonfaces);

    let mut it_src = source_keys.iter();
    let mut it_tgt = target_keys.iter();

    let mut next_src = it_src.next();
    let mut next_tgt = it_tgt.next();

    loop {
        match (next_src, next_tgt) {
            (Some(src_key), Some(tgt_key)) => {
                match src_key.cmp(tgt_key) {
                    std::cmp::Ordering::Less => {
                        return Err(PreservationError::DroppedNonFace(src_key.clone()));
                    }
                    std::cmp::Ordering::Greater => {
                        return Err(PreservationError::InventedNonFace(tgt_key.clone()));
                    }
                    std::cmp::Ordering::Equal => {
                        // Confirm totality for all slots in the preserved nonface
                        for &slot in src_key {
                            if !map.node_to_action.contains_key(&PowlNodeId(slot as u64)) {
                                return Err(PreservationError::UnmappedConcurrencySlot(slot));
                            }
                        }
                        next_src = it_src.next();
                        next_tgt = it_tgt.next();
                    }
                }
            }
            (Some(src_key), None) => {
                return Err(PreservationError::DroppedNonFace(src_key.clone()));
            }
            (None, Some(tgt_key)) => {
                return Err(PreservationError::InventedNonFace(tgt_key.clone()));
            }
            (None, None) => {
                break;
            }
        }
    }

    let source_complex_digest = digest_nonface_keys(&source_keys);
    let target_complex_digest = digest_nonface_keys(&target_keys);
    let mapped_source_digest = source_complex_digest;

    Ok(ConcurrencyPreservationWitness {
        source_complex_digest,
        mapped_source_digest,
        target_complex_digest,
    })
}

// ---------------------------------------------------------------------------
// PowlProjector implementation
// ---------------------------------------------------------------------------

/// The real [`bcinr_mfw_ir::PowlProjector`] implementation: `Model =
/// PowlModel`. Stateless — a unit struct.
pub struct PowlProjector;

impl bcinr_mfw_ir::PowlProjector for PowlProjector {
    type Model = PowlModel;
    type Error = ProjectionError;

    fn project(
        &self,
        causal: &CausalPlan,
        concurrency: &ExecutableConcurrencyComplex,
    ) -> Result<(PowlModel, PowlProjectionWitness), ProjectionError> {
        // 1. Build a real one-to-one ActionNodeBijection + one PowlNode per
        //    ActionOccurrence (not a placeholder).
        let mut nodes = Vec::with_capacity(causal.occurrences.len());
        let mut action_to_node = BTreeMap::new();
        let mut node_to_action = BTreeMap::new();
        let mut provenance = BTreeMap::new();
        for (i, occ) in causal.occurrences.iter().enumerate() {
            let node_id = PowlNodeId(i as u64);
            nodes.push(PowlNode::Activity(ActivityNode {
                id: node_id,
                label: format!("action-{}", occ.action),
                source_action: occ.id,
            }));
            action_to_node.insert(occ.id, node_id);
            node_to_action.insert(node_id, occ.id);
            provenance.insert(node_id, occ.id);
        }
        let bijection = ActionNodeBijection {
            action_to_node,
            node_to_action,
        };

        // 2. Map every PrecedenceEdge through the bijection (totality
        //    check) into the target's order field.
        let mut order_edges = BTreeSet::new();
        for edge in &causal.precedes.edges {
            if !bijection.action_to_node.contains_key(&edge.before) {
                return Err(PreservationError::UnmappedAction(edge.before).into());
            }
            if !bijection.action_to_node.contains_key(&edge.after) {
                return Err(PreservationError::UnmappedAction(edge.after).into());
            }
            order_edges.insert(*edge);
        }
        let order = StrictPartialOrder { edges: order_edges };

        // 3. Carry concurrency through unchanged (see module docs).
        let target_concurrency = concurrency.clone();

        let model = PowlModel {
            nodes,
            order,
            concurrency: target_concurrency,
            provenance,
        };

        // 4. Real verification — only return Ok when both genuinely pass.
        let order_witness = verify_order_preservation(causal, &model, &bijection)?;
        let concurrency_witness =
            verify_concurrency_preservation(concurrency, &model.concurrency, &bijection)?;

        let causal_plan_digest = causal.digest;
        let source_concurrency_digest = concurrency.digest;
        let digest = causal_plan_digest
            .mix(&source_concurrency_digest)
            .mix(&order_witness.projected_order_digest)
            .mix(&concurrency_witness.target_complex_digest);

        let witness = PowlProjectionWitness {
            source_epoch: causal.epoch,
            causal_plan_digest,
            source_concurrency_digest,
            action_node_bijection: bijection,
            order_witness,
            concurrency_witness,
            digest,
        };

        Ok((model, witness))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bcinr_mfw_ir::{
        ActionOccurrence, ConcurrencyConflictWitness, EventSet, FluentId, IndependenceRelation,
        MinimalNonFace, PlanningEpochId, PowlProjector as PowlProjectorTrait,
        ResourceConflictWitness,
    };

    /// The "A, B, C can't all fire together" fixture, mirroring
    /// `bcinr-mfw-ir::concurrency`'s own worked-complex test: 3 actions
    /// A=0, B=1, C=2, a simple order A before C, and a single minimal
    /// nonface `{A, B, C}`.
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
    fn project_builds_real_one_to_one_bijection() {
        let (causal, concurrency) = fixture();
        let projector = PowlProjector;
        let (model, witness) = projector.project(&causal, &concurrency).unwrap();

        assert_eq!(model.nodes.len(), 3);
        assert_eq!(witness.action_node_bijection.action_to_node.len(), 3);
        assert_eq!(witness.action_node_bijection.node_to_action.len(), 3);
        for occ in &causal.occurrences {
            let node = witness.action_node_bijection.action_to_node[&occ.id];
            assert_eq!(witness.action_node_bijection.node_to_action[&node], occ.id);
        }
    }

    #[test]
    fn project_genuinely_passes_order_and_concurrency_preservation() {
        let (causal, concurrency) = fixture();
        let projector = PowlProjector;
        let (model, witness) = projector.project(&causal, &concurrency).unwrap();

        // Order: the mapped digest and projected digest must agree (that's
        // the whole point of the witness), and the model must literally
        // contain the source edge.
        assert_eq!(
            witness.order_witness.mapped_order_digest,
            witness.order_witness.projected_order_digest
        );
        assert!(model.order.edges.contains(&PrecedenceEdge {
            before: ActionOccurrenceId(0),
            after: ActionOccurrenceId(2),
        }));

        // Concurrency: mapped/target digests agree, and the {A,B,C}
        // nonface survived the projection intact.
        assert_eq!(
            witness.concurrency_witness.mapped_source_digest,
            witness.concurrency_witness.target_complex_digest
        );
        assert_eq!(model.concurrency.minimal_nonfaces.len(), 1);
        assert_eq!(
            model.concurrency.minimal_nonfaces[0].members,
            EventSet::empty().with(0).with(1).with(2)
        );
    }

    // -------------------------------------------------------------------
    // Adversarial: deliberately mutate the projected model and prove the
    // verifier actually catches it (not just "runs and returns Ok").
    // -------------------------------------------------------------------

    #[test]
    fn verify_order_preservation_catches_a_dropped_edge() {
        let (causal, concurrency) = fixture();
        let projector = PowlProjector;
        let (mut model, witness) = projector.project(&causal, &concurrency).unwrap();

        // Deliberately drop the only order edge from the projected model.
        model.order.edges.clear();

        let result = verify_order_preservation(&causal, &model, &witness.action_node_bijection);
        assert_eq!(
            result,
            Err(PreservationError::DroppedOrderEdge(PrecedenceEdge {
                before: ActionOccurrenceId(0),
                after: ActionOccurrenceId(2),
            }))
        );
    }

    #[test]
    fn verify_order_preservation_catches_an_invented_edge() {
        let (causal, concurrency) = fixture();
        let projector = PowlProjector;
        let (mut model, witness) = projector.project(&causal, &concurrency).unwrap();

        // Deliberately invent an edge the source causal plan never had.
        let invented = PrecedenceEdge {
            before: ActionOccurrenceId(2),
            after: ActionOccurrenceId(1),
        };
        model.order.edges.insert(invented);

        let result = verify_order_preservation(&causal, &model, &witness.action_node_bijection);
        assert_eq!(result, Err(PreservationError::InventedOrderEdge(invented)));
    }

    #[test]
    fn verify_concurrency_preservation_catches_a_dropped_nonface() {
        let (causal, concurrency) = fixture();
        let projector = PowlProjector;
        let (mut model, witness) = projector.project(&causal, &concurrency).unwrap();

        // Deliberately drop the {A,B,C} nonface from the projected complex.
        model.concurrency.minimal_nonfaces.clear();

        let result = verify_concurrency_preservation(
            &concurrency,
            &model.concurrency,
            &witness.action_node_bijection,
        );
        assert_eq!(
            result,
            Err(PreservationError::DroppedNonFace(vec![0, 1, 2]))
        );
    }

    #[test]
    fn verify_concurrency_preservation_catches_an_invented_nonface() {
        let (causal, concurrency) = fixture();
        let projector = PowlProjector;
        let (mut model, witness) = projector.project(&causal, &concurrency).unwrap();

        // Deliberately invent an extra nonface the source never had.
        model.concurrency.minimal_nonfaces.push(MinimalNonFace {
            members: EventSet::empty().with(1).with(2),
            witness_digest: Digest::hash(b"invented"),
        });

        let result = verify_concurrency_preservation(
            &concurrency,
            &model.concurrency,
            &witness.action_node_bijection,
        );
        assert_eq!(result, Err(PreservationError::InventedNonFace(vec![1, 2])));
    }

    #[test]
    fn project_rejects_edge_referencing_unmapped_action() {
        // A causal plan whose order references an action occurrence that
        // isn't in `occurrences` at all -- the bijection can never cover
        // it, so project() must refuse rather than silently drop it.
        let mut edges = BTreeSet::new();
        edges.insert(PrecedenceEdge {
            before: ActionOccurrenceId(0),
            after: ActionOccurrenceId(99), // never in occurrences
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

        let projector = PowlProjector;
        let result = projector.project(&causal, &concurrency);
        assert_eq!(
            result.unwrap_err(),
            ProjectionError::Preservation(PreservationError::UnmappedAction(ActionOccurrenceId(
                99
            )))
        );
    }

    /// Regression test for the position-vs-`ActionOccurrenceId` conflation
    /// bug: a well-formed `CausalPlan` whose 3 occurrences sit at positions
    /// 0,1,2 but carry sparse `ActionOccurrenceId`s 100,101,102 (nothing in
    /// this crate's types forbids that) must project successfully — see
    /// `crates/bcinr-pddl/tests/mfw_capacity2_fixture.rs`'s
    /// `link4_adversarial_confirmed_bug_...` test, which pinned down the
    /// previously-broken behavior from the `bcinr-pddl` side.
    #[test]
    fn project_succeeds_with_sparse_non_positional_action_occurrence_ids() {
        let occurrences = vec![
            ActionOccurrence {
                id: ActionOccurrenceId(100),
                action: 1,
            },
            ActionOccurrence {
                id: ActionOccurrenceId(101),
                action: 2,
            },
            ActionOccurrence {
                id: ActionOccurrenceId(102),
                action: 3,
            },
        ];
        let causal = CausalPlan {
            epoch: PlanningEpochId(7),
            occurrences,
            precedes: StrictPartialOrder::default(),
            independence: IndependenceRelation::default(),
            support_edges: BTreeSet::new(),
            digest: Digest::hash(b"sparse-ids-causal-plan"),
        };

        // Minimal nonface built the way PddlConcurrencyAnalyzer builds
        // one: EventSet members are *positions* {0, 1} (occurrences[0] and
        // occurrences[1]), not the raw ActionOccurrenceIds 100/101.
        let witness_digest = Digest::hash(b"sparse-ids-conflict");
        let mut conflict_witnesses = BTreeMap::new();
        conflict_witnesses.insert(
            witness_digest,
            ConcurrencyConflictWitness {
                causal: None,
                temporal: None,
                resource: None,
            },
        );
        let concurrency = ExecutableConcurrencyComplex {
            event_count: 3,
            minimal_nonfaces: vec![MinimalNonFace {
                members: EventSet::empty().with(0).with(1),
                witness_digest,
            }],
            conflict_witnesses,
            digest: Digest::hash(b"sparse-ids-complex"),
        };

        let projector = PowlProjector;
        let (model, witness) = projector
            .project(&causal, &concurrency)
            .expect("a well-formed plan with sparse ActionOccurrenceIds must project cleanly");

        assert_eq!(
            witness.concurrency_witness.mapped_source_digest,
            witness.concurrency_witness.target_complex_digest
        );
        // The projected nonface's positions {0,1} must resolve back to the
        // real ActionOccurrenceIds {100,101} through the bijection.
        let projected_members: Vec<usize> = model.concurrency.minimal_nonfaces[0]
            .members
            .iter_stable()
            .collect();
        let mapped_back: Vec<u32> = projected_members
            .iter()
            .map(|&slot| witness.action_node_bijection.node_to_action[&PowlNodeId(slot as u64)].0)
            .collect();
        assert_eq!(mapped_back, vec![100, 101]);
    }
}
