/// Canonical bounded-integer logical time type for PDDL temporal planning.
///
/// `LogicalTime` represents discrete time units (milliseconds) in temporal plans.
/// It is used as:
/// - Observation timestamps in `ObservationSnapshot`
/// - Goal deadlines in `GoalEnvelope`
/// - Plan deadline constraints in `GroundTemporalProblem`
///
/// The underlying `u64` is bounded by domain/problem grounding, not by the type itself.
/// Cross-crate usage (PDDL → POWL → workflow runtime) relies on this single definition.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct LogicalTime(pub u64);

impl LogicalTime {
    /// Create a new `LogicalTime` from milliseconds.
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Get the time value in milliseconds.
    pub const fn as_millis(&self) -> u64 {
        self.0
    }

    /// Create `LogicalTime` representing zero (t=0).
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Convert `LogicalTime` to floating-point seconds for temporal plan comparison.
    pub fn as_seconds_f64(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Create `LogicalTime` from floating-point seconds.
    pub fn from_seconds_f64(seconds: f64) -> Self {
        Self((seconds * 1000.0).round() as u64)
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
    fn logical_time_ordering() {
        let t0 = LogicalTime::zero();
        let t1 = LogicalTime::from_millis(100);
        let t2 = LogicalTime::from_millis(200);

        assert!(t0 < t1);
        assert!(t1 < t2);
        assert!(t0 <= t0);
        assert_eq!(t0, LogicalTime::zero());
    }

    #[test]
    fn logical_time_f64_conversion() {
        let t = LogicalTime::from_seconds_f64(1.5);
        assert_eq!(t.as_millis(), 1500);
        assert!((t.as_seconds_f64() - 1.5).abs() < 0.001);
    }
}
