//! Candidate-only machine-experience compile-back contract.
//!
//! Successful execution evidence may justify a reusable capability candidate, but
//! neither execution success nor model output is itself semantic standing or authority.
//! Promotion therefore requires independent admission and verification receipts.

use crate::Digest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExperienceAuthority {
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceEvidence {
    pub semantic_subject: Digest,
    pub execution_receipt: Digest,
    pub execution_succeeded: bool,
    pub ontology_digest: Digest,
    pub manufacturer_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromotionCandidate {
    pub candidate_id: Digest,
    pub semantic_subject: Digest,
    pub ontology_digest: Digest,
    pub manufacturer_digest: Digest,
    pub source_execution_receipt: Digest,
    pub authority: ExperienceAuthority,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedCapability {
    pub capability_id: Digest,
    pub promotion_candidate_id: Digest,
    pub admission_receipt: Digest,
    pub verification_receipt: Digest,
    pub ontology_digest: Digest,
    pub manufacturer_digest: Digest,
    pub authority: ExperienceAuthority,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExperienceRefusal {
    ExecutionNotSuccessful,
    MissingAdmissionReceipt,
    MissingVerificationReceipt,
    OntologyIdentityChanged,
    ManufacturerIdentityChanged,
}

fn combine(parts: &[Digest]) -> Digest {
    let mut bytes = Vec::with_capacity(parts.len() * 32);
    for part in parts {
        bytes.extend_from_slice(part.as_bytes());
    }
    Digest::hash(&bytes)
}

/// Construct a candidate reusable pattern from successful receipted execution.
///
/// The result is intentionally authority-free and is not qualified for routing yet.
pub fn candidate_from_execution(
    evidence: &ExperienceEvidence,
) -> Result<PromotionCandidate, ExperienceRefusal> {
    if !evidence.execution_succeeded {
        return Err(ExperienceRefusal::ExecutionNotSuccessful);
    }
    let candidate_id = combine(&[
        evidence.semantic_subject,
        evidence.execution_receipt,
        evidence.ontology_digest,
        evidence.manufacturer_digest,
    ]);
    Ok(PromotionCandidate {
        candidate_id,
        semantic_subject: evidence.semantic_subject,
        ontology_digest: evidence.ontology_digest,
        manufacturer_digest: evidence.manufacturer_digest,
        source_execution_receipt: evidence.execution_receipt,
        authority: ExperienceAuthority::None,
    })
}

/// Qualify a promotion candidate only after independent admission and verification.
///
/// This function never grants runtime consequence authority. It manufactures only a
/// deterministic capability identity suitable for a downstream KNOWN-route registry.
pub fn qualify_candidate(
    candidate: &PromotionCandidate,
    admitted_ontology_digest: Digest,
    admitted_manufacturer_digest: Digest,
    admission_receipt: Option<Digest>,
    verification_receipt: Option<Digest>,
) -> Result<QualifiedCapability, ExperienceRefusal> {
    if candidate.ontology_digest != admitted_ontology_digest {
        return Err(ExperienceRefusal::OntologyIdentityChanged);
    }
    if candidate.manufacturer_digest != admitted_manufacturer_digest {
        return Err(ExperienceRefusal::ManufacturerIdentityChanged);
    }
    let admission_receipt = admission_receipt.ok_or(ExperienceRefusal::MissingAdmissionReceipt)?;
    let verification_receipt = verification_receipt.ok_or(ExperienceRefusal::MissingVerificationReceipt)?;
    let capability_id = combine(&[
        candidate.candidate_id,
        admission_receipt,
        verification_receipt,
        candidate.ontology_digest,
        candidate.manufacturer_digest,
    ]);
    Ok(QualifiedCapability {
        capability_id,
        promotion_candidate_id: candidate.candidate_id,
        admission_receipt,
        verification_receipt,
        ontology_digest: candidate.ontology_digest,
        manufacturer_digest: candidate.manufacturer_digest,
        authority: ExperienceAuthority::None,
    })
}
