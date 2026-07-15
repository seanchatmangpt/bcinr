//! `PowlModel` — a dual order+concurrency representation of a POWL program.
//!
//! This is a genuinely new, richer model than [`crate::compiler::PowlAstNode`]:
//! `PowlAstNode` stays the existing hand-authored-workflow *input* language
//! (fed to [`crate::compiler::compile_powl`]); `PowlModel` is the new
//! *planner-output* representation, produced by a
//! [`bcinr_mfw_ir::PowlProjector`] (see [`crate::projection`]) from a
//! `CausalPlan` + `ExecutableConcurrencyComplex`. `PowlModel` can itself be
//! lowered into the existing v2 tape representation via
//! [`crate::compiler::compile_powl_v2`]. The two models are never conflated:
//! there is no `From<PowlAstNode> for PowlModel` or vice versa in this phase.
//!
//! # Node identity vs. action identity
//!
//! [`PowlModel::provenance`] maps a [`PowlNodeId`] to the
//! [`ActionOccurrenceId`] it was projected from. Not every node needs an
//! entry: only nodes that trace back to a real PDDL action occurrence do.
//! [`SilentNode`] intentionally carries no `source_action` field — a τ
//! (silent) node has no source action to reference — so it never
//! contributes an entry to `provenance`. This is a partial map by design,
//! not an oversight.
//!
//! # `EventSet` members are positions, not `ActionOccurrenceId` values
//!
//! [`PowlModel::concurrency`] is carried through from the source
//! `ExecutableConcurrencyComplex` **unchanged** (no field is rewritten),
//! but its `EventSet` members were never `ActionOccurrenceId` numeric
//! values in the first place — an earlier version of this doc comment
//! claimed they were, which was false and caused a real bug (see below).
//!
//! The actual, authoritative contract (matching this workspace's only real
//! `ConcurrencyAnalyzer` producer, `PddlConcurrencyAnalyzer::analyze` in
//! `crates/bcinr-pddl/src/concurrency.rs`, whose own doc comment is
//! explicit about this): an `EventSet` member value is the **position** of
//! an occurrence within the source `CausalPlan::occurrences` list — a
//! dense `0..occurrences.len()` index — never the occurrence's own
//! (caller-assigned, possibly sparse) `ActionOccurrenceId`. `bcinr-mfw-ir`
//! itself does not pin this down (neither `EventSet`'s nor
//! `MinimalNonFace`'s doc comments say which convention is authoritative);
//! this crate is now consistent with `bcinr-pddl`'s producer.
//!
//! [`crate::projection::PowlProjector::project`] builds each node's
//! `PowlNodeId` from that exact same position (`PowlNodeId(i)` for
//! `causal.occurrences[i]`), so for a `PowlModel` produced by the real
//! projector, an `EventSet` member numerically coincides with the
//! `PowlNodeId` of the node it refers to — `PowlNodeId` is not a separate
//! addressing space concurrency needs to be re-keyed into, it already *is*
//! the same numbering. [`crate::compiler::compile_powl_v2`] relies on
//! exactly this coincidence (after independently re-verifying node-id
//! density) rather than resolving through `provenance`/`ActionOccurrenceId`
//! for concurrency members — see that function's doc comment.
//!
//! Treating an `EventSet` member as a raw `ActionOccurrenceId` (as both
//! [`crate::projection::verify_concurrency_preservation`] and
//! [`crate::compiler::compile_powl_v2`] used to) only coincidentally works
//! when every occurrence's `ActionOccurrenceId` equals its position — true
//! of every hand-built fixture in this codebase, and true of
//! `MfwPlanner::occurrences_from_tape`'s real output only as long as no
//! tape op is filtered out. See
//! `crates/bcinr-pddl/tests/mfw_capacity2_fixture.rs`'s
//! `link4_adversarial_confirmed_bug_...` test for the reproduction that
//! caught this.

use std::collections::BTreeMap;

use bcinr_mfw_ir::{
    ActionOccurrenceId, ExecutableConcurrencyComplex, PowlNodeId, StrictPartialOrder,
};

/// A concrete, named activity node — the common case: one PowlNode per
/// source `ActionOccurrence`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityNode {
    pub id: PowlNodeId,
    pub label: String,
    pub source_action: ActionOccurrenceId,
}

/// A silent (τ) transition node. Carries no `source_action`: it does not
/// trace back to any real PDDL action occurrence, so it never appears as a
/// key in [`PowlModel::provenance`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SilentNode {
    pub id: PowlNodeId,
}

/// UNSUPPORTED (stub): a node representing an embedded child workflow.
///
/// This variant exists in the enum so callers can pattern-match on it and
/// see it is a recognised-but-not-yet-implemented shape — it is never
/// constructed by [`crate::projection::PowlProjector`] or consumed by
/// [`crate::compiler::compile_powl_v2`] in this phase. Any code path that
/// receives one returns a typed error
/// ([`crate::compiler::CompileErrorV2::UnsupportedNodeKind`]) rather than
/// silently ignoring it or treating it as a no-op. Real child-workflow
/// projection (sub-tape linking, cross-tape receipt chaining) is out of
/// this phase's time budget — this is MOCKED-as-a-typed-refusal, not a
/// working feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildWorkflowNode {
    pub id: PowlNodeId,
}

/// UNSUPPORTED (stub): a node representing an externally-cut (opaque,
/// outside-this-tape) region. Same status as [`ChildWorkflowNode`]: present
/// in the enum, never constructed or compiled in this phase, any consumer
/// must return a typed error rather than pretend to handle it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalCutNode {
    pub id: PowlNodeId,
}

/// A node in a [`PowlModel`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowlNode {
    Activity(ActivityNode),
    Silent(SilentNode),
    /// UNSUPPORTED stub — see [`ChildWorkflowNode`].
    ChildWorkflow(ChildWorkflowNode),
    /// UNSUPPORTED stub — see [`ExternalCutNode`].
    ExternalCut(ExternalCutNode),
}

impl PowlNode {
    /// The node's identity, regardless of variant.
    pub fn id(&self) -> PowlNodeId {
        match self {
            PowlNode::Activity(a) => a.id,
            PowlNode::Silent(s) => s.id,
            PowlNode::ChildWorkflow(c) => c.id,
            PowlNode::ExternalCut(e) => e.id,
        }
    }

    /// `true` for the two stub variants ([`ChildWorkflow`][PowlNode::ChildWorkflow],
    /// [`ExternalCut`][PowlNode::ExternalCut]) that no compiler/projector in
    /// this phase actually implements.
    pub fn is_unsupported_stub(&self) -> bool {
        matches!(self, PowlNode::ChildWorkflow(_) | PowlNode::ExternalCut(_))
    }
}

/// A dual order+concurrency representation of a POWL program: the new
/// planner-output IR (see module docs for how this differs from
/// [`crate::compiler::PowlAstNode`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowlModel {
    pub nodes: Vec<PowlNode>,
    /// Precedence order over the model's nodes, expressed in
    /// `ActionOccurrenceId` space (see module docs).
    pub order: StrictPartialOrder,
    /// Executable-concurrency complex, carried through from the source
    /// unchanged (`ActionOccurrenceId`-keyed — see module docs).
    pub concurrency: ExecutableConcurrencyComplex,
    /// Partial map: only nodes that trace back to a real action occurrence
    /// have an entry (see module docs — `SilentNode` never does).
    pub provenance: BTreeMap<PowlNodeId, ActionOccurrenceId>,
}

impl PowlModel {
    /// An empty model: no nodes, empty order, empty (zero-nonface)
    /// concurrency complex, empty provenance.
    pub fn empty() -> Self {
        PowlModel {
            nodes: Vec::new(),
            order: StrictPartialOrder::default(),
            concurrency: ExecutableConcurrencyComplex {
                event_count: 0,
                minimal_nonfaces: Vec::new(),
                conflict_witnesses: BTreeMap::new(),
                digest: bcinr_mfw_ir::Digest::ZERO,
            },
            provenance: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powl_node_id_dispatches_across_variants() {
        let a = PowlNode::Activity(ActivityNode {
            id: PowlNodeId(0),
            label: "a".into(),
            source_action: ActionOccurrenceId(0),
        });
        let s = PowlNode::Silent(SilentNode { id: PowlNodeId(1) });
        let c = PowlNode::ChildWorkflow(ChildWorkflowNode { id: PowlNodeId(2) });
        let e = PowlNode::ExternalCut(ExternalCutNode { id: PowlNodeId(3) });

        assert_eq!(a.id(), PowlNodeId(0));
        assert_eq!(s.id(), PowlNodeId(1));
        assert_eq!(c.id(), PowlNodeId(2));
        assert_eq!(e.id(), PowlNodeId(3));

        assert!(!a.is_unsupported_stub());
        assert!(!s.is_unsupported_stub());
        assert!(c.is_unsupported_stub());
        assert!(e.is_unsupported_stub());
    }

    #[test]
    fn silent_node_never_needs_a_provenance_entry() {
        // SilentNode carries no source_action field at all -- this test
        // documents that fact structurally: constructing one requires only
        // `id`, nothing else.
        let s = SilentNode { id: PowlNodeId(7) };
        let node = PowlNode::Silent(s);
        let model = PowlModel {
            nodes: vec![node],
            order: StrictPartialOrder::default(),
            concurrency: PowlModel::empty().concurrency,
            provenance: BTreeMap::new(), // deliberately empty: no Activity nodes
        };
        assert_eq!(model.nodes.len(), 1);
        assert!(model.provenance.is_empty());
    }

    #[test]
    fn empty_model_has_no_nodes_and_admits_everything() {
        let model = PowlModel::empty();
        assert!(model.nodes.is_empty());
        assert!(model.provenance.is_empty());
        assert!(model.concurrency.minimal_nonfaces.is_empty());
    }
}
