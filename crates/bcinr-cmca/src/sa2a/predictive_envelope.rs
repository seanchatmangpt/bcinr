//! Predictive consequence envelopes for CMCA's authority-free SA2A boundary.
//!
//! This module applies the transferable part of arXiv:2608.25754 to CMCA:
//! forecast future multi-resource pressure, bind that forecast to an exact
//! subject/model/observation/provenance identity, and turn it into deterministic
//! advisory pressure.
//!
//! Prediction is deliberately not allocation authority. Forecast demand may
//! exceed actual capacity; that is useful overload evidence. It never enlarges
//! a ResourceEnvelope, SELECTs a candidate, CONSTRUCTs an effect, or DOs work.

use super::resource_envelope::{Allocation, ResourceEnvelope};

pub const PRESSURE_PPM_ONE: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceAxis {
    Cpu,
    Memory,
    Io,
}

/// Unadmitted predictor output.
///
/// Public construction is intentional: predictors are proposal sources. Every
/// field that matters to standing is independently rechecked by
/// `admit_predictive_envelope`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictiveConsequenceEnvelope {
    pub subject_id: u64,
    pub horizon_start: u64,
    pub horizon_end: u64,
    pub expected: Allocation,
    pub lower: Allocation,
    pub upper: Allocation,
    pub confidence_ppm: u32,
    pub observation_digest: u64,
    pub model_digest: u64,
    pub provenance_digest: u64,
}

/// Caller-owned admission facts. A predictor cannot self-authorize these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictiveAdmission {
    pub subject_id: u64,
    pub observation_digest: u64,
    pub model_digest: u64,
    pub provenance_digest: u64,
    pub min_confidence_ppm: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictiveRefusal {
    SubjectMismatch { expected: u64, actual: u64 },
    ObservationDigestMismatch,
    ModelDigestMismatch,
    ProvenanceDigestMismatch,
    InvalidHorizon,
    InvalidConfidence { confidence_ppm: u32 },
    InvalidConfidenceFloor { floor_ppm: u32 },
    ConfidenceBelowFloor { floor_ppm: u32, actual_ppm: u32 },
    LowerExceedsExpected { axis: ResourceAxis },
    ExpectedExceedsUpper { axis: ResourceAxis },
}

impl core::fmt::Display for PredictiveRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PredictiveRefusal {}

/// Admitted forecast evidence. It remains advisory and carries no allocation or
/// actuation capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmittedPredictiveConsequenceEnvelope {
    envelope: PredictiveConsequenceEnvelope,
}

impl AdmittedPredictiveConsequenceEnvelope {
    pub const fn envelope(&self) -> &PredictiveConsequenceEnvelope {
        &self.envelope
    }

    /// Convert expected and conservative-upper demand into bounded pressure
    /// against *actual* capacity. Values saturate at 1.0 (1_000_000 ppm).
    ///
    /// This reports pressure only. It does not return a new ResourceEnvelope
    /// and therefore cannot manufacture budget from a forecast.
    pub fn pressure_against(&self, actual: &ResourceEnvelope) -> PredictivePressure {
        PredictivePressure {
            expected: pressure_vector(self.envelope.expected, actual),
            upper: pressure_vector(self.envelope.upper, actual),
        }
    }

    /// Close the loop with an observation receipt. This is evidence for
    /// falsification/retraining; it does not mutate the predictor or allocator.
    pub fn observe(&self, observed: Allocation) -> ForecastReceipt {
        ForecastReceipt {
            subject_id: self.envelope.subject_id,
            horizon_start: self.envelope.horizon_start,
            horizon_end: self.envelope.horizon_end,
            observation_digest: self.envelope.observation_digest,
            model_digest: self.envelope.model_digest,
            provenance_digest: self.envelope.provenance_digest,
            observed,
            absolute_error: Allocation {
                cpu: observed.cpu.abs_diff(self.envelope.expected.cpu),
                memory: observed.memory.abs_diff(self.envelope.expected.memory),
                io: observed.io.abs_diff(self.envelope.expected.io),
            },
            within_interval: inside_interval(
                observed,
                self.envelope.lower,
                self.envelope.upper,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PressureVector {
    pub cpu_ppm: u32,
    pub memory_ppm: u32,
    pub io_ppm: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictivePressure {
    pub expected: PressureVector,
    pub upper: PressureVector,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastReceipt {
    pub subject_id: u64,
    pub horizon_start: u64,
    pub horizon_end: u64,
    pub observation_digest: u64,
    pub model_digest: u64,
    pub provenance_digest: u64,
    pub observed: Allocation,
    pub absolute_error: Allocation,
    pub within_interval: bool,
}

/// Independently admit predictor output against exact caller-owned identity and
/// confidence constraints.
///
/// Capacity is intentionally absent from this admission gate. A valid forecast
/// may predict overload. Actual capacity remains authoritative later at the
/// existing ResourceEnvelope boundary.
pub fn admit_predictive_envelope(
    candidate: PredictiveConsequenceEnvelope,
    admission: PredictiveAdmission,
) -> Result<AdmittedPredictiveConsequenceEnvelope, PredictiveRefusal> {
    if candidate.subject_id != admission.subject_id {
        return Err(PredictiveRefusal::SubjectMismatch {
            expected: admission.subject_id,
            actual: candidate.subject_id,
        });
    }
    if candidate.observation_digest != admission.observation_digest {
        return Err(PredictiveRefusal::ObservationDigestMismatch);
    }
    if candidate.model_digest != admission.model_digest {
        return Err(PredictiveRefusal::ModelDigestMismatch);
    }
    if candidate.provenance_digest != admission.provenance_digest {
        return Err(PredictiveRefusal::ProvenanceDigestMismatch);
    }
    if candidate.horizon_start >= candidate.horizon_end {
        return Err(PredictiveRefusal::InvalidHorizon);
    }
    if candidate.confidence_ppm > PRESSURE_PPM_ONE {
        return Err(PredictiveRefusal::InvalidConfidence {
            confidence_ppm: candidate.confidence_ppm,
        });
    }
    if admission.min_confidence_ppm > PRESSURE_PPM_ONE {
        return Err(PredictiveRefusal::InvalidConfidenceFloor {
            floor_ppm: admission.min_confidence_ppm,
        });
    }
    if candidate.confidence_ppm < admission.min_confidence_ppm {
        return Err(PredictiveRefusal::ConfidenceBelowFloor {
            floor_ppm: admission.min_confidence_ppm,
            actual_ppm: candidate.confidence_ppm,
        });
    }

    check_bounds(candidate.lower, candidate.expected, candidate.upper)?;

    Ok(AdmittedPredictiveConsequenceEnvelope {
        envelope: candidate,
    })
}

fn check_bounds(
    lower: Allocation,
    expected: Allocation,
    upper: Allocation,
) -> Result<(), PredictiveRefusal> {
    check_axis(lower.cpu, expected.cpu, upper.cpu, ResourceAxis::Cpu)?;
    check_axis(
        lower.memory,
        expected.memory,
        upper.memory,
        ResourceAxis::Memory,
    )?;
    check_axis(lower.io, expected.io, upper.io, ResourceAxis::Io)?;
    Ok(())
}

fn check_axis(
    lower: u64,
    expected: u64,
    upper: u64,
    axis: ResourceAxis,
) -> Result<(), PredictiveRefusal> {
    if lower > expected {
        return Err(PredictiveRefusal::LowerExceedsExpected { axis });
    }
    if expected > upper {
        return Err(PredictiveRefusal::ExpectedExceedsUpper { axis });
    }
    Ok(())
}

fn pressure_vector(demand: Allocation, actual: &ResourceEnvelope) -> PressureVector {
    PressureVector {
        cpu_ppm: component_pressure_ppm(demand.cpu, actual.cpu),
        memory_ppm: component_pressure_ppm(demand.memory, actual.memory),
        io_ppm: component_pressure_ppm(demand.io, actual.io),
    }
}

fn component_pressure_ppm(demand: u64, capacity: u64) -> u32 {
    if demand == 0 {
        return 0;
    }
    if capacity == 0 {
        return PRESSURE_PPM_ONE;
    }

    let scaled = (demand as u128) * (PRESSURE_PPM_ONE as u128);
    let ratio = scaled / (capacity as u128);
    core::cmp::min(ratio, PRESSURE_PPM_ONE as u128) as u32
}


const fn inside_interval(observed: Allocation, lower: Allocation, upper: Allocation) -> bool {
    observed.cpu >= lower.cpu
        && observed.cpu <= upper.cpu
        && observed.memory >= lower.memory
        && observed.memory <= upper.memory
        && observed.io >= lower.io
        && observed.io <= upper.io
}
