//! Forecast-backed reservation proposals.
//!
//! Admission here means only that a proposal is internally consistent with an
//! actual ResourceEnvelope and an externally supplied authorization binding.
//! It does not reserve, mutate, or spend capacity.

use super::resource_envelope::{Allocation, ResourceEnvelope};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservationProposal {
    pub subject_id: u64,
    pub horizon_start: u64,
    pub horizon_end: u64,
    pub requested: Allocation,
    pub forecast_digest: u64,
    pub reason_digest: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservationAdmission {
    pub subject_id: u64,
    pub forecast_digest: u64,
    pub authorization_binding_digest: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmittedReservationProposal {
    proposal: ReservationProposal,
    authorization_binding_digest: u64,
}

impl AdmittedReservationProposal {
    pub const fn proposal(&self) -> &ReservationProposal {
        &self.proposal
    }

    pub const fn authorization_binding_digest(&self) -> u64 {
        self.authorization_binding_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationRefusal {
    SubjectMismatch,
    ForecastDigestMismatch,
    AuthorizationBindingMismatch,
    InvalidHorizon,
    EnvelopeExceeded {
        requested: Allocation,
        actual: ResourceEnvelope,
    },
}

impl core::fmt::Display for ReservationRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ReservationRefusal {}

pub fn admit_reservation_proposal(
    proposal: ReservationProposal,
    actual_capacity: &ResourceEnvelope,
    admission: ReservationAdmission,
    expected_authorization_binding_digest: u64,
) -> Result<AdmittedReservationProposal, ReservationRefusal> {
    if proposal.subject_id != admission.subject_id {
        return Err(ReservationRefusal::SubjectMismatch);
    }
    if proposal.forecast_digest != admission.forecast_digest {
        return Err(ReservationRefusal::ForecastDigestMismatch);
    }
    if admission.authorization_binding_digest != expected_authorization_binding_digest {
        return Err(ReservationRefusal::AuthorizationBindingMismatch);
    }
    if proposal.horizon_start >= proposal.horizon_end {
        return Err(ReservationRefusal::InvalidHorizon);
    }
    if !actual_capacity.admits(&proposal.requested) {
        return Err(ReservationRefusal::EnvelopeExceeded {
            requested: proposal.requested,
            actual: *actual_capacity,
        });
    }

    Ok(AdmittedReservationProposal {
        proposal,
        authorization_binding_digest: admission.authorization_binding_digest,
    })
}
