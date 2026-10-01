//! Powerless adapter from admitted predictive evidence to CMCA-facing measure evidence.
//!
//! This module deliberately does not write allocator state. It binds forecast
//! pressure and empirical forecast standing into one typed artifact that a
//! later semantic projection may consume.

use super::forecast_standing::{standing_digest, ForecastStanding};
use super::predictive_envelope::{
    AdmittedPredictiveConsequenceEnvelope, PredictivePressure, PressureVector,
};
use super::resource_envelope::ResourceEnvelope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictiveMeasureArtifact {
    pub subject_id: u64,
    pub forecast_digest: u64,
    pub standing_digest: u64,
    pub expected_pressure: PressureVector,
    pub upper_pressure: PressureVector,
    pub mean_absolute_error: u64,
    pub root_mean_squared_error: u64,
    pub interval_coverage_ppm: u32,
    pub shock_misses: u64,
    pub shock_false_positives: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictiveMeasureAdmission {
    pub subject_id: u64,
    pub forecast_digest: u64,
    pub standing_digest: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmittedPredictiveMeasure {
    artifact: PredictiveMeasureArtifact,
}

impl AdmittedPredictiveMeasure {
    pub const fn artifact(&self) -> &PredictiveMeasureArtifact {
        &self.artifact
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictiveMeasureRefusal {
    SubjectMismatch,
    ForecastDigestMismatch,
    StandingDigestMismatch,
}

impl core::fmt::Display for PredictiveMeasureRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PredictiveMeasureRefusal {}

pub fn build_predictive_measure(
    forecast: &AdmittedPredictiveConsequenceEnvelope,
    actual_capacity: &ResourceEnvelope,
    standing: ForecastStanding,
) -> PredictiveMeasureArtifact {
    let envelope = forecast.envelope();
    let PredictivePressure { expected, upper } = forecast.pressure_against(actual_capacity);
    PredictiveMeasureArtifact {
        subject_id: envelope.subject_id,
        forecast_digest: predictive_envelope_digest(forecast),
        standing_digest: standing_digest(&standing),
        expected_pressure: expected,
        upper_pressure: upper,
        mean_absolute_error: standing.mean_absolute_error,
        root_mean_squared_error: standing.root_mean_squared_error,
        interval_coverage_ppm: standing.interval_coverage_ppm,
        shock_misses: standing.shock_misses,
        shock_false_positives: standing.shock_false_positives,
    }
}

pub fn admit_predictive_measure(
    artifact: PredictiveMeasureArtifact,
    admission: PredictiveMeasureAdmission,
) -> Result<AdmittedPredictiveMeasure, PredictiveMeasureRefusal> {
    if artifact.subject_id != admission.subject_id {
        return Err(PredictiveMeasureRefusal::SubjectMismatch);
    }
    if artifact.forecast_digest != admission.forecast_digest {
        return Err(PredictiveMeasureRefusal::ForecastDigestMismatch);
    }
    if artifact.standing_digest != admission.standing_digest {
        return Err(PredictiveMeasureRefusal::StandingDigestMismatch);
    }

    Ok(AdmittedPredictiveMeasure { artifact })
}

pub fn predictive_envelope_digest(forecast: &AdmittedPredictiveConsequenceEnvelope) -> u64 {
    let envelope = forecast.envelope();
    let mut digest = mix64(envelope.subject_id, envelope.horizon_start);
    digest = mix64(digest, envelope.horizon_end);
    digest = mix64(digest, envelope.expected.cpu);
    digest = mix64(digest, envelope.expected.memory);
    digest = mix64(digest, envelope.expected.io);
    digest = mix64(digest, envelope.lower.cpu);
    digest = mix64(digest, envelope.lower.memory);
    digest = mix64(digest, envelope.lower.io);
    digest = mix64(digest, envelope.upper.cpu);
    digest = mix64(digest, envelope.upper.memory);
    digest = mix64(digest, envelope.upper.io);
    digest = mix64(digest, envelope.confidence_ppm as u64);
    digest = mix64(digest, envelope.observation_digest);
    digest = mix64(digest, envelope.model_digest);
    mix64(digest, envelope.provenance_digest)
}

#[inline(always)]
fn mix64(a: u64, b: u64) -> u64 {
    let mut x = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x
}
