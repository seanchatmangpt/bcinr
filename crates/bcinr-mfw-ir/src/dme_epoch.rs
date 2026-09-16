//! Recursive DME epoch closure for Multifractal Workflow.
//!
//! This module turns receipted residual obligations into the next bounded epoch
//! only when a machine-checkable descent witness exists. Receipt feedback is
//! observational; it cannot mint authority or silently change ontology identity.

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
}

fn unique(values: &[Digest]) -> bool {
    for (index, left) in values.iter().enumerate() {
        if values[index + 1..].iter().any(|right| left == right) {
            return false;
        }
    }
    true
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
    pub fn root(ontology_digest: Digest, residual_obligations: Vec<Digest>) -> Result<Self, EpochRefusal> {
        if !unique(&residual_obligations) {
            return Err(EpochRefusal::DuplicateResidual);
        }
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
/// requires the same ontology identity, a strictly smaller residual set, unique
/// residual identities, and an available DescentMeter step.
pub fn advance_epoch(
    parent: &DmeEpoch,
    feedback: &ReceiptFeedback,
    meter: &mut DescentMeter,
) -> Result<EpochAdvance, EpochRefusal> {
    if feedback.observed_ontology_digest != parent.ontology_digest {
        return Err(EpochRefusal::OntologyIdentityChanged);
    }
    if !unique(&feedback.residual_obligations) {
        return Err(EpochRefusal::DuplicateResidual);
    }

    if feedback.residual_obligations.is_empty() {
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

    if feedback.residual_obligations.len() >= parent.residual_obligations.len() {
        return Err(EpochRefusal::NonDescendingResidual);
    }

    let depth = meter.descend().map_err(|_| EpochRefusal::DescentBoundHit)?;
    let epoch_id = epoch_digest(
        Some(parent.epoch_id),
        parent.ontology_digest,
        &feedback.residual_obligations,
        depth,
        Some(feedback.receipt_digest),
    );

    Ok(EpochAdvance::Successor(DmeEpoch {
        epoch_id,
        parent_epoch_id: Some(parent.epoch_id),
        ontology_digest: parent.ontology_digest,
        residual_obligations: feedback.residual_obligations.clone(),
        depth,
        authority: EpochAuthority::None,
    }))
}
