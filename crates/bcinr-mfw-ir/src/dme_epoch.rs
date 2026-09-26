//! Recursive DME epoch closure for Multifractal Workflow.
//!
//! This module turns receipted residual obligations into the next bounded epoch
//! only when a machine-checkable descent witness exists. Receipt feedback is
//! observational; it cannot mint authority or silently change ontology identity.
//!
//! # Canonical identity and descent (hardening, v26.9.26)
//!
//! * Residual obligations are a *set*: they are stored and hashed in canonical
//!   (sorted) order, so transport reordering of the same obligations cannot mint a
//!   second epoch identity.
//! * Descent is set descent, not only count descent: a successor's residuals must be
//!   a strict subset of the parent's. Receipt feedback therefore cannot substitute
//!   fresh obligations for discharged ones (`ResidualNotInParent`).
//! * The caller's [`DescentMeter`] must be positioned at the parent's depth; a stale
//!   or foreign meter cannot re-mint a lower depth for a deeper epoch
//!   (`StaleDescentMeter`).
//!
//! # Sealed identity and hard ceiling (hardening, v26.9.26 round 2)
//!
//! * [`DmeEpoch`] fields are private: epochs are manufactured only by
//!   [`DmeEpoch::root`], [`DmeEpoch::root_with_budget`] and [`advance_epoch`], and
//!   [`advance_epoch`] recomputes the parent's identity from its contents
//!   (`ParentIdentityMismatch`), so a parent whose residual set was widened, or
//!   whose residuals carry duplicates, cannot be descended from.
//! * The descent budget is bound into the epoch identity and capped by the hard
//!   ceiling [`MAX_DME_EPOCH_DESCENT_BUDGET`]. A caller cannot widen it: a meter
//!   whose budget exceeds the epoch's bound budget is refused
//!   (`ForeignDescentBudget`), and a root budget above the ceiling is refused with a
//!   typed [`BoundHit`] (`DescentBudgetAboveCeiling`).
//! * Exhaustion carries its witness: `DescentBoundHit(BoundHit)` preserves the
//!   meter's limit and observed depth instead of discarding them.

use crate::{BoundHit, BoundKind, DescentMeter, Digest};

/// Hard ceiling on the recursive descent budget of any DME epoch chain.
pub const MAX_DME_EPOCH_DESCENT_BUDGET: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpochAuthority {
    None,
}

/// A sealed DME epoch. Constructible only through [`DmeEpoch::root`],
/// [`DmeEpoch::root_with_budget`] and [`advance_epoch`]:
///
/// ```compile_fail
/// use bcinr_mfw_ir::{Digest, DmeEpoch, EpochAuthority};
/// let forged = DmeEpoch {
///     epoch_id: Digest::ZERO,
///     parent_epoch_id: None,
///     ontology_digest: Digest::ZERO,
///     residual_obligations: vec![],
///     depth: 0,
///     descent_budget: 1_000_000,
///     source_receipt: None,
///     authority: EpochAuthority::None,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DmeEpoch {
    epoch_id: Digest,
    parent_epoch_id: Option<Digest>,
    ontology_digest: Digest,
    residual_obligations: Vec<Digest>,
    depth: usize,
    descent_budget: usize,
    source_receipt: Option<Digest>,
    authority: EpochAuthority,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptFeedback {
    pub receipt_digest: Digest,
    pub observed_ontology_digest: Digest,
    pub residual_obligations: Vec<Digest>,
    pub authority: EpochAuthority,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EpochAdvance {
    Closed {
        parent_epoch_id: Digest,
        receipt_digest: Digest,
        closure_digest: Digest,
        authority: EpochAuthority,
    },
    Successor(DmeEpoch),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EpochRefusal {
    OntologyIdentityChanged,
    NonDescendingResidual,
    /// The descent meter is exhausted; carries the meter's bound witness.
    DescentBoundHit(BoundHit),
    DuplicateResidual,
    /// Feedback names an obligation the parent epoch does not carry.
    ResidualNotInParent,
    /// The descent meter is not positioned at the parent epoch's depth.
    StaleDescentMeter,
    /// The meter's budget exceeds the descent budget bound into the epoch identity:
    /// a caller cannot widen the depth ceiling.
    ForeignDescentBudget,
    /// A root budget above [`MAX_DME_EPOCH_DESCENT_BUDGET`]; carries the witness.
    DescentBudgetAboveCeiling(BoundHit),
    /// The parent epoch's identity does not match its own contents.
    ParentIdentityMismatch,
}

/// Canonicalize a residual set: sorted; refuses duplicates.
fn canonical_residuals(values: &[Digest]) -> Result<Vec<Digest>, EpochRefusal> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(EpochRefusal::DuplicateResidual);
    }
    Ok(sorted)
}

const EPOCH_DOMAIN: &[u8] = b"bcinr-mfw-ir/dme-epoch/v2";

fn epoch_digest(
    parent: Option<Digest>,
    ontology: Digest,
    residuals: &[Digest],
    depth: usize,
    budget: usize,
    receipt: Option<Digest>,
) -> Digest {
    let mut bytes = Vec::with_capacity(EPOCH_DOMAIN.len() + 32 * (3 + residuals.len()) + 24);
    bytes.extend_from_slice(EPOCH_DOMAIN);
    bytes.extend_from_slice(parent.unwrap_or(Digest::ZERO).as_bytes());
    bytes.extend_from_slice(ontology.as_bytes());
    bytes.extend_from_slice(&(depth as u64).to_le_bytes());
    bytes.extend_from_slice(&(budget as u64).to_le_bytes());
    bytes.extend_from_slice(&(residuals.len() as u64).to_le_bytes());
    for residual in residuals {
        bytes.extend_from_slice(residual.as_bytes());
    }
    bytes.extend_from_slice(receipt.unwrap_or(Digest::ZERO).as_bytes());
    Digest::hash(&bytes)
}

impl DmeEpoch {
    /// A root epoch with the ceiling budget [`MAX_DME_EPOCH_DESCENT_BUDGET`].
    pub fn root(
        ontology_digest: Digest,
        residual_obligations: Vec<Digest>,
    ) -> Result<Self, EpochRefusal> {
        Self::root_with_budget(
            ontology_digest,
            residual_obligations,
            MAX_DME_EPOCH_DESCENT_BUDGET,
        )
    }

    /// A root epoch whose chain may descend at most `descent_budget` levels. The
    /// budget is bound into the identity and may not exceed the hard ceiling.
    pub fn root_with_budget(
        ontology_digest: Digest,
        residual_obligations: Vec<Digest>,
        descent_budget: usize,
    ) -> Result<Self, EpochRefusal> {
        if descent_budget > MAX_DME_EPOCH_DESCENT_BUDGET {
            return Err(EpochRefusal::DescentBudgetAboveCeiling(BoundHit {
                kind: BoundKind::RecursiveDescent,
                limit: MAX_DME_EPOCH_DESCENT_BUDGET as u64,
                observed: descent_budget as u64,
            }));
        }
        let residual_obligations = canonical_residuals(&residual_obligations)?;
        let epoch_id = epoch_digest(
            None,
            ontology_digest,
            &residual_obligations,
            0,
            descent_budget,
            None,
        );
        Ok(Self {
            epoch_id,
            parent_epoch_id: None,
            ontology_digest,
            residual_obligations,
            depth: 0,
            descent_budget,
            source_receipt: None,
            authority: EpochAuthority::None,
        })
    }

    pub fn epoch_id(&self) -> Digest {
        self.epoch_id
    }
    pub fn parent_epoch_id(&self) -> Option<Digest> {
        self.parent_epoch_id
    }
    pub fn ontology_digest(&self) -> Digest {
        self.ontology_digest
    }
    /// Residual obligations in canonical (sorted) order.
    pub fn residual_obligations(&self) -> &[Digest] {
        &self.residual_obligations
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    /// Descent budget bound into this epoch chain's identity.
    pub fn descent_budget(&self) -> usize {
        self.descent_budget
    }
    /// Receipt whose feedback manufactured this epoch (`None` for a root).
    pub fn source_receipt(&self) -> Option<Digest> {
        self.source_receipt
    }
    pub fn authority(&self) -> EpochAuthority {
        self.authority
    }

    /// Recompute this epoch's identity from its contents.
    fn identity_holds(&self) -> bool {
        let canonical = matches!(
            canonical_residuals(&self.residual_obligations),
            Ok(ref sorted) if *sorted == self.residual_obligations
        );
        canonical
            && self.descent_budget <= MAX_DME_EPOCH_DESCENT_BUDGET
            && self.depth <= self.descent_budget
            && epoch_digest(
                self.parent_epoch_id,
                self.ontology_digest,
                &self.residual_obligations,
                self.depth,
                self.descent_budget,
                self.source_receipt,
            ) == self.epoch_id
    }
}

/// Advance one MFW epoch from typed receipt feedback.
///
/// A closed residual set emits a closure witness and no successor. A successor
/// requires the same ontology identity, a residual set that is a strict subset of
/// the parent's, unique residual identities, a meter positioned at the parent's
/// depth, and an available DescentMeter step.
pub fn advance_epoch(
    parent: &DmeEpoch,
    feedback: &ReceiptFeedback,
    meter: &mut DescentMeter,
) -> Result<EpochAdvance, EpochRefusal> {
    if !parent.identity_holds() {
        return Err(EpochRefusal::ParentIdentityMismatch);
    }
    if feedback.observed_ontology_digest != parent.ontology_digest {
        return Err(EpochRefusal::OntologyIdentityChanged);
    }
    let residuals = canonical_residuals(&feedback.residual_obligations)?;

    if residuals.is_empty() {
        let closure_digest = epoch_digest(
            Some(parent.epoch_id),
            parent.ontology_digest,
            &[],
            parent.depth,
            parent.descent_budget,
            Some(feedback.receipt_digest),
        );
        return Ok(EpochAdvance::Closed {
            parent_epoch_id: parent.epoch_id,
            receipt_digest: feedback.receipt_digest,
            closure_digest,
            authority: EpochAuthority::None,
        });
    }

    if residuals.len() >= parent.residual_obligations.len() {
        return Err(EpochRefusal::NonDescendingResidual);
    }
    // Set descent: O(m log n) membership over the parent's canonical (sorted,
    // duplicate-free, identity-verified) residual set.
    if residuals
        .iter()
        .any(|residual| parent.residual_obligations.binary_search(residual).is_err())
    {
        return Err(EpochRefusal::ResidualNotInParent);
    }
    if meter.depth != parent.depth {
        return Err(EpochRefusal::StaleDescentMeter);
    }
    if meter.budget > parent.descent_budget {
        return Err(EpochRefusal::ForeignDescentBudget);
    }

    let depth = meter.descend().map_err(EpochRefusal::DescentBoundHit)?;
    let epoch_id = epoch_digest(
        Some(parent.epoch_id),
        parent.ontology_digest,
        &residuals,
        depth,
        parent.descent_budget,
        Some(feedback.receipt_digest),
    );

    Ok(EpochAdvance::Successor(DmeEpoch {
        epoch_id,
        parent_epoch_id: Some(parent.epoch_id),
        ontology_digest: parent.ontology_digest,
        residual_obligations: residuals,
        depth,
        descent_budget: parent.descent_budget,
        source_receipt: Some(feedback.receipt_digest),
        authority: EpochAuthority::None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(value: &str) -> Digest {
        Digest::hash(value.as_bytes())
    }

    fn feedback(parent: &DmeEpoch, residuals: Vec<Digest>) -> ReceiptFeedback {
        ReceiptFeedback {
            receipt_digest: d("r"),
            observed_ontology_digest: parent.ontology_digest,
            residual_obligations: residuals,
            authority: EpochAuthority::None,
        }
    }

    /// Audit probe P3: widening a parent's residual set while leaving its id
    /// unchanged let feedback "descend" into fresh obligations.
    #[test]
    fn forged_parent_residuals_are_refused_by_identity_recomputation() {
        let mut parent = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
        let mut widened = vec![d("x1"), d("x2"), d("x3")];
        widened.sort_unstable();
        parent.residual_obligations = widened;
        let mut meter = DescentMeter::new(8);
        assert_eq!(
            advance_epoch(&parent, &feedback(&parent, vec![d("x1")]), &mut meter),
            Err(EpochRefusal::ParentIdentityMismatch)
        );
        assert_eq!(meter.depth, 0);
    }

    /// Court probes A1/A2: a parent carrying duplicate residuals `[a, a, b]` admitted
    /// the non-descending successor `{a, b}`.
    #[test]
    fn duplicate_parent_residuals_are_refused_even_with_a_recomputed_id() {
        let mut parent = DmeEpoch::root(d("o"), vec![d("a"), d("b")]).unwrap();
        let mut dup = vec![d("a"), d("a"), d("b")];
        dup.sort_unstable();
        parent.residual_obligations = dup;
        parent.epoch_id = epoch_digest(
            None,
            parent.ontology_digest,
            &parent.residual_obligations,
            0,
            parent.descent_budget,
            None,
        );
        let mut meter = DescentMeter::new(8);
        assert_eq!(
            advance_epoch(
                &parent,
                &feedback(&parent, vec![d("a"), d("b")]),
                &mut meter
            ),
            Err(EpochRefusal::ParentIdentityMismatch)
        );
    }

    #[test]
    fn forged_budget_or_depth_is_refused() {
        let root = DmeEpoch::root_with_budget(d("o"), vec![d("a"), d("b")], 2).unwrap();
        let mut widened = root.clone();
        widened.descent_budget = 1_000_000;
        let mut deep = root.clone();
        deep.depth = 3;
        for forged in [widened, deep] {
            assert_eq!(
                advance_epoch(
                    &forged,
                    &feedback(&forged, vec![d("a")]),
                    &mut DescentMeter::new(2)
                ),
                Err(EpochRefusal::ParentIdentityMismatch)
            );
        }
    }
}
