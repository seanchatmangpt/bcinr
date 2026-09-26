//! Candidate-only machine-experience compile-back contract.
//!
//! Successful execution evidence may justify a reusable capability candidate, but
//! neither execution success nor model output is itself semantic standing or authority.
//! Promotion therefore requires independent admission and verification receipts.
//!
//! # Sealing (hardening, v26.9.26 round 2)
//!
//! * [`PromotionCandidate`] has private fields: the only constructor is
//!   [`candidate_from_execution`], which refuses unsuccessful execution. A caller can
//!   no longer hand-build a candidate and skip the execution-success law.
//! * [`qualify_candidate`] recomputes the candidate identity from the candidate's
//!   own fields and refuses a mismatch (`CandidateIdentityMismatch`), so a candidate
//!   whose semantic subject (or any other field) changed cannot keep its identity.
//! * The capability identity binds the semantic subject directly, and
//!   [`QualifiedCapability`] carries it: two capabilities for different subjects can
//!   never share an identity.
//! * The all-zero digest is refused as an admission or verification receipt
//!   (`NullReceipt`): it is the placeholder value, not evidence.

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

/// A sealed promotion candidate. Constructible only by [`candidate_from_execution`].
///
/// Hand-building a candidate (and so skipping the execution-success law) does not
/// compile outside this crate:
///
/// ```compile_fail
/// use bcinr_mfw_ir::{Digest, ExperienceAuthority, PromotionCandidate};
/// let forged = PromotionCandidate {
///     candidate_id: Digest::hash(b"anything"),
///     semantic_subject: Digest::hash(b"subject"),
///     ontology_digest: Digest::hash(b"ontology"),
///     manufacturer_digest: Digest::hash(b"manufacturer"),
///     source_execution_receipt: Digest::hash(b"failed-exec"),
///     authority: ExperienceAuthority::None,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromotionCandidate {
    candidate_id: Digest,
    semantic_subject: Digest,
    ontology_digest: Digest,
    manufacturer_digest: Digest,
    source_execution_receipt: Digest,
    authority: ExperienceAuthority,
}

impl PromotionCandidate {
    pub fn candidate_id(&self) -> Digest {
        self.candidate_id
    }
    pub fn semantic_subject(&self) -> Digest {
        self.semantic_subject
    }
    pub fn ontology_digest(&self) -> Digest {
        self.ontology_digest
    }
    pub fn manufacturer_digest(&self) -> Digest {
        self.manufacturer_digest
    }
    pub fn source_execution_receipt(&self) -> Digest {
        self.source_execution_receipt
    }
    pub fn authority(&self) -> ExperienceAuthority {
        self.authority
    }
}

/// A qualified capability. Constructible only by [`qualify_candidate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedCapability {
    capability_id: Digest,
    promotion_candidate_id: Digest,
    semantic_subject: Digest,
    admission_receipt: Digest,
    verification_receipt: Digest,
    ontology_digest: Digest,
    manufacturer_digest: Digest,
    authority: ExperienceAuthority,
}

impl QualifiedCapability {
    pub fn capability_id(&self) -> Digest {
        self.capability_id
    }
    pub fn promotion_candidate_id(&self) -> Digest {
        self.promotion_candidate_id
    }
    pub fn semantic_subject(&self) -> Digest {
        self.semantic_subject
    }
    pub fn admission_receipt(&self) -> Digest {
        self.admission_receipt
    }
    pub fn verification_receipt(&self) -> Digest {
        self.verification_receipt
    }
    pub fn ontology_digest(&self) -> Digest {
        self.ontology_digest
    }
    pub fn manufacturer_digest(&self) -> Digest {
        self.manufacturer_digest
    }
    pub fn authority(&self) -> ExperienceAuthority {
        self.authority
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExperienceRefusal {
    ExecutionNotSuccessful,
    MissingAdmissionReceipt,
    MissingVerificationReceipt,
    OntologyIdentityChanged,
    ManufacturerIdentityChanged,
    /// Admission and verification receipts are the same receipt, or one of them is
    /// the source execution receipt: the evidence is not independent.
    NonIndependentReceipts,
    /// The candidate's identity does not match the identity recomputed from its
    /// own fields.
    CandidateIdentityMismatch,
    /// An admission or verification receipt is the all-zero placeholder digest.
    NullReceipt,
}

fn candidate_identity(
    semantic_subject: Digest,
    execution_receipt: Digest,
    ontology_digest: Digest,
    manufacturer_digest: Digest,
) -> Digest {
    combine(
        b"bcinr-mfw-ir/experience/candidate/v1",
        &[
            semantic_subject,
            execution_receipt,
            ontology_digest,
            manufacturer_digest,
        ],
    )
}

fn combine(domain: &[u8], parts: &[Digest]) -> Digest {
    let mut bytes = Vec::with_capacity(domain.len() + parts.len() * 32);
    bytes.extend_from_slice(domain);
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
    let candidate_id = candidate_identity(
        evidence.semantic_subject,
        evidence.execution_receipt,
        evidence.ontology_digest,
        evidence.manufacturer_digest,
    );
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
    let recomputed = candidate_identity(
        candidate.semantic_subject,
        candidate.source_execution_receipt,
        candidate.ontology_digest,
        candidate.manufacturer_digest,
    );
    if recomputed != candidate.candidate_id {
        return Err(ExperienceRefusal::CandidateIdentityMismatch);
    }
    if candidate.ontology_digest != admitted_ontology_digest {
        return Err(ExperienceRefusal::OntologyIdentityChanged);
    }
    if candidate.manufacturer_digest != admitted_manufacturer_digest {
        return Err(ExperienceRefusal::ManufacturerIdentityChanged);
    }
    let admission_receipt = admission_receipt.ok_or(ExperienceRefusal::MissingAdmissionReceipt)?;
    let verification_receipt =
        verification_receipt.ok_or(ExperienceRefusal::MissingVerificationReceipt)?;
    if admission_receipt == Digest::ZERO || verification_receipt == Digest::ZERO {
        return Err(ExperienceRefusal::NullReceipt);
    }
    if admission_receipt == verification_receipt
        || admission_receipt == candidate.source_execution_receipt
        || verification_receipt == candidate.source_execution_receipt
    {
        return Err(ExperienceRefusal::NonIndependentReceipts);
    }
    let capability_id = combine(
        b"bcinr-mfw-ir/experience/capability/v2",
        &[
            candidate.candidate_id,
            candidate.semantic_subject,
            admission_receipt,
            verification_receipt,
            candidate.ontology_digest,
            candidate.manufacturer_digest,
        ],
    );
    Ok(QualifiedCapability {
        capability_id,
        promotion_candidate_id: candidate.candidate_id,
        semantic_subject: candidate.semantic_subject,
        admission_receipt,
        verification_receipt,
        ontology_digest: candidate.ontology_digest,
        manufacturer_digest: candidate.manufacturer_digest,
        authority: ExperienceAuthority::None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(value: &str) -> Digest {
        Digest::hash(value.as_bytes())
    }

    fn genuine() -> PromotionCandidate {
        candidate_from_execution(&ExperienceEvidence {
            semantic_subject: d("subject"),
            execution_receipt: d("exec-receipt"),
            execution_succeeded: true,
            ontology_digest: d("ontology"),
            manufacturer_digest: d("manufacturer"),
        })
        .unwrap()
    }

    fn qualify(c: &PromotionCandidate) -> Result<QualifiedCapability, ExperienceRefusal> {
        qualify_candidate(
            c,
            d("ontology"),
            d("manufacturer"),
            Some(d("admit")),
            Some(d("verify")),
        )
    }

    /// Audit probe P1: a candidate whose semantic subject was changed after
    /// manufacture kept its identity and qualified. In-crate code is the only code
    /// that can still write the fields; the recomputation refuses it.
    #[test]
    fn tampered_candidate_fields_are_refused_by_identity_recomputation() {
        let tampers: [fn(&mut PromotionCandidate); 5] = [
            |c| c.semantic_subject = d("forged-subject"),
            |c| c.source_execution_receipt = d("failed-exec"),
            |c| c.candidate_id = d("anything"),
            |c| {
                c.ontology_digest = d("ontology-2");
            },
            |c| {
                c.manufacturer_digest = d("manufacturer-2");
            },
        ];
        assert!(qualify(&genuine()).is_ok());
        for tamper in tampers {
            let mut c = genuine();
            tamper(&mut c);
            assert_eq!(
                qualify(&c),
                Err(ExperienceRefusal::CandidateIdentityMismatch)
            );
        }
    }
}
