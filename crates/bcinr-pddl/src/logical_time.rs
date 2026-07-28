/// Canonical non-negative millisecond time for planning and resource admission.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LogicalTime(pub u64);

/// Refusal raised while admitting external floating-point time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeConversionError {
    NonFinite,
    Negative,
    Overflow,
}

impl std::fmt::Display for TimeConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite => write!(f, "time is NaN or infinite"),
            Self::Negative => write!(f, "time is negative"),
            Self::Overflow => write!(f, "time exceeds logical-time range"),
        }
    }
}
impl std::error::Error for TimeConversionError {}

impl LogicalTime {
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }
    pub const fn as_millis(self) -> u64 {
        self.0
    }
    pub const fn zero() -> Self {
        Self(0)
    }
    pub fn as_seconds_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }

    pub fn try_from_seconds_f64(seconds: f64) -> Result<Self, TimeConversionError> {
        if !seconds.is_finite() {
            return Err(TimeConversionError::NonFinite);
        }
        if seconds < 0.0 {
            return Err(TimeConversionError::Negative);
        }
        let millis = seconds * 1000.0;
        if millis > u64::MAX as f64 {
            return Err(TimeConversionError::Overflow);
        }
        Ok(Self(millis.round() as u64))
    }

    /// Compatibility constructor for already-trusted values. Boundary code
    /// must use [`Self::try_from_seconds_f64`].
    pub fn from_seconds_f64(seconds: f64) -> Self {
        Self::try_from_seconds_f64(seconds).expect("invalid external logical time")
    }
}

impl std::fmt::Display for LogicalTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_seconds_admit() {
        assert_eq!(
            LogicalTime::try_from_seconds_f64(1.5).unwrap().as_millis(),
            1500
        );
    }

    #[test]
    fn invalid_seconds_refuse() {
        assert_eq!(
            LogicalTime::try_from_seconds_f64(f64::NAN),
            Err(TimeConversionError::NonFinite)
        );
        assert_eq!(
            LogicalTime::try_from_seconds_f64(-1.0),
            Err(TimeConversionError::Negative)
        );
    }
}
