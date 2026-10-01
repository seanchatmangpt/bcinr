//! Predictor-independent resource, horizon, trace, and forecast semantics.
//!
//! This module is intentionally powerless. It represents observations and
//! forecasts; it does not allocate resources or perform effects.

pub const CONFIDENCE_PPM_ONE: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResourceAxisId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceVector<const R: usize> {
    pub values: [u64; R],
}

impl<const R: usize> ResourceVector<R> {
    pub const fn zero() -> Self {
        Self { values: [0; R] }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceAxisRegistry<const R: usize> {
    pub axes: [ResourceAxisId; R],
}

impl<const R: usize> ResourceAxisRegistry<R> {
    pub fn validate(&self) -> Result<(), ForecastModelRefusal> {
        let mut i = 0usize;
        while i < R {
            let mut j = i + 1;
            while j < R {
                if self.axes[i] == self.axes[j] {
                    return Err(ForecastModelRefusal::DuplicateAxis { index: j });
                }
                j += 1;
            }
            i += 1;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastHorizon {
    pub start: u64,
    pub end: u64,
}

impl ForecastHorizon {
    pub const fn is_valid(&self) -> bool {
        self.start < self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastHorizonSet<const H: usize> {
    pub horizons: [ForecastHorizon; H],
}

impl<const H: usize> ForecastHorizonSet<H> {
    pub fn validate(&self) -> Result<(), ForecastModelRefusal> {
        let mut i = 0usize;
        while i < H {
            if !self.horizons[i].is_valid() {
                return Err(ForecastModelRefusal::InvalidHorizon { index: i });
            }
            i += 1;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastCell {
    pub expected: u64,
    pub lower: u64,
    pub upper: u64,
    pub confidence_ppm: u32,
}

impl ForecastCell {
    pub const fn is_valid(&self) -> bool {
        self.lower <= self.expected
            && self.expected <= self.upper
            && self.confidence_ppm <= CONFIDENCE_PPM_ONE
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastMatrix<const R: usize, const H: usize> {
    pub subject_id: u64,
    pub axes: [ResourceAxisId; R],
    pub horizons: [ForecastHorizon; H],
    pub cells: [[ForecastCell; H]; R],
    pub observation_digest: u64,
    pub model_digest: u64,
    pub provenance_digest: u64,
}

impl<const R: usize, const H: usize> ForecastMatrix<R, H> {
    pub fn validate(&self, registry: &ResourceAxisRegistry<R>) -> Result<(), ForecastModelRefusal> {
        registry.validate()?;
        ForecastHorizonSet {
            horizons: self.horizons,
        }
        .validate()?;

        let mut axis = 0usize;
        while axis < R {
            if self.axes[axis] != registry.axes[axis] {
                return Err(ForecastModelRefusal::AxisMismatch { index: axis });
            }

            let mut horizon = 0usize;
            while horizon < H {
                let cell = self.cells[axis][horizon];
                if cell.lower > cell.expected {
                    return Err(ForecastModelRefusal::LowerExceedsExpected { axis, horizon });
                }
                if cell.expected > cell.upper {
                    return Err(ForecastModelRefusal::ExpectedExceedsUpper { axis, horizon });
                }
                if cell.confidence_ppm > CONFIDENCE_PPM_ONE {
                    return Err(ForecastModelRefusal::InvalidConfidence {
                        axis,
                        horizon,
                        confidence_ppm: cell.confidence_ppm,
                    });
                }
                horizon += 1;
            }

            axis += 1;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceSample<const R: usize> {
    pub at: u64,
    pub resources: ResourceVector<R>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceWindow<const R: usize, const N: usize> {
    pub subject_id: u64,
    pub samples: [TraceSample<R>; N],
    pub provenance_digest: u64,
}

impl<const R: usize, const N: usize> TraceWindow<R, N> {
    pub fn validate(&self) -> Result<(), ForecastModelRefusal> {
        let mut i = 1usize;
        while i < N {
            if self.samples[i - 1].at >= self.samples[i].at {
                return Err(ForecastModelRefusal::NonMonotonicTrace { index: i });
            }
            i += 1;
        }
        Ok(())
    }

    pub fn digest(&self) -> u64 {
        let mut digest = mix64(self.subject_id, self.provenance_digest);
        let mut i = 0usize;
        while i < N {
            digest = mix64(digest, self.samples[i].at);
            let mut axis = 0usize;
            while axis < R {
                digest = mix64(digest, self.samples[i].resources.values[axis]);
                axis += 1;
            }
            i += 1;
        }
        digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictorArtifact<const R: usize, const H: usize> {
    pub model_digest: u64,
    pub runtime_profile_digest: u64,
    pub axes: [ResourceAxisId; R],
    pub horizons: [ForecastHorizon; H],
}

/// A predictor is a proposal source. Implementations do not receive an
/// allocation or actuation capability.
pub trait Predictor<const R: usize, const H: usize, const N: usize> {
    fn artifact(&self) -> PredictorArtifact<R, H>;

    fn predict(
        &self,
        trace: &TraceWindow<R, N>,
    ) -> Result<ForecastMatrix<R, H>, ForecastModelRefusal>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForecastModelRefusal {
    DuplicateAxis {
        index: usize,
    },
    AxisMismatch {
        index: usize,
    },
    InvalidHorizon {
        index: usize,
    },
    InvalidConfidence {
        axis: usize,
        horizon: usize,
        confidence_ppm: u32,
    },
    LowerExceedsExpected {
        axis: usize,
        horizon: usize,
    },
    ExpectedExceedsUpper {
        axis: usize,
        horizon: usize,
    },
    NonMonotonicTrace {
        index: usize,
    },
    SubjectMismatch,
    ObservationDigestMismatch,
    ModelDigestMismatch,
    ProvenanceDigestMismatch,
    RuntimeProfileMismatch,
}

impl core::fmt::Display for ForecastModelRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ForecastModelRefusal {}

/// Independently validates a predictor result against the exact input trace and
/// predictor artifact. Identical model/input/profile identities therefore have
/// an explicit replay boundary.
pub fn admit_forecast_matrix<const R: usize, const H: usize, const N: usize>(
    trace: &TraceWindow<R, N>,
    artifact: &PredictorArtifact<R, H>,
    runtime_profile_digest: u64,
    matrix: ForecastMatrix<R, H>,
) -> Result<ForecastMatrix<R, H>, ForecastModelRefusal> {
    trace.validate()?;

    if matrix.subject_id != trace.subject_id {
        return Err(ForecastModelRefusal::SubjectMismatch);
    }
    if matrix.observation_digest != trace.digest() {
        return Err(ForecastModelRefusal::ObservationDigestMismatch);
    }
    if matrix.model_digest != artifact.model_digest {
        return Err(ForecastModelRefusal::ModelDigestMismatch);
    }
    if matrix.provenance_digest != trace.provenance_digest {
        return Err(ForecastModelRefusal::ProvenanceDigestMismatch);
    }
    if runtime_profile_digest != artifact.runtime_profile_digest {
        return Err(ForecastModelRefusal::RuntimeProfileMismatch);
    }
    if matrix.horizons != artifact.horizons {
        return Err(ForecastModelRefusal::InvalidHorizon { index: 0 });
    }

    let registry = ResourceAxisRegistry {
        axes: artifact.axes,
    };
    matrix.validate(&registry)?;
    Ok(matrix)
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
