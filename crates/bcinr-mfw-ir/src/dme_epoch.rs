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

use crate::{DescentMeter, Digest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpochAuthority {
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DmeEpoch {
    pub epoch_id: Digest,
    pub parent_epoch_id: Option<Digest>,
    pub ontology_digest: Digest,
    pub residual_obligations: Vec<Digest>,
    pub depth: usize,
    pub authority: EpochAuthority,
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
    DescentBoundHit,
    DuplicateResidual,
    /// Feedback names an obligation the parent epoch does not carry.
    ResidualNotInParent,
    /// The descent meter is not positioned at the parent epoch's depth.
    StaleDescentMeter,
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

fn epoch_digest(
    parent: Option<Digest>,
    ontology: Digest,
    residuals: &[Digest],
    depth: usize,
    receipt: Option<Digest>,
) -> Digest {
    let mut bytes = Vec::with_capacity(32 * (3 + residuals.len()) + 16);
    bytes.extend_from_slice(parent.unwrap_or(Digest::ZERO).as_bytes());
    bytes.extend_from_slice(ontology.as_bytes());
    bytes.extend_from_slice(&(depth as u64).to_le_bytes());
    bytes.extend_from_slice(&(residuals.len() as u64).to_le_bytes());
    for residual in residuals {
        bytes.extend_from_slice(residual.as_bytes());
    }
    bytes.extend_from_slice(receipt.unwrap_or(Digest::ZERO).as_bytes());
    Digest::hash(&bytes)
}

impl DmeEpoch {
    pub fn root(
        ontology_digest: Digest,
        residual_obligations: Vec<Digest>,
    ) -> Result<Self, EpochRefusal> {
        let residual_obligations = canonical_residuals(&residual_obligations)?;
        let epoch_id = epoch_digest(None, ontology_digest, &residual_obligations, 0, None);
        Ok(Self {
            epoch_id,
            parent_epoch_id: None,
            ontology_digest,
            residual_obligations,
            depth: 0,
            authority: EpochAuthority::None,
        })
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
    // Set descent: O((n + m) log n) membership over a sorted view of the parent
    // (parent fields are public, so its order is not trusted).
    let mut parent_set = parent.residual_obligations.clone();
    parent_set.sort_unstable();
    if residuals
        .iter()
        .any(|residual| parent_set.binary_search(residual).is_err())
    {
        return Err(EpochRefusal::ResidualNotInParent);
    }
    if meter.depth != parent.depth {
        return Err(EpochRefusal::StaleDescentMeter);
    }

    let depth = meter.descend().map_err(|_| EpochRefusal::DescentBoundHit)?;
    let epoch_id = epoch_digest(
        Some(parent.epoch_id),
        parent.ontology_digest,
        &residuals,
        depth,
        Some(feedback.receipt_digest),
    );

    Ok(EpochAdvance::Successor(DmeEpoch {
        epoch_id,
        parent_epoch_id: Some(parent.epoch_id),
        ontology_digest: parent.ontology_digest,
        residual_obligations: residuals,
        depth,
        authority: EpochAuthority::None,
    }))
}
