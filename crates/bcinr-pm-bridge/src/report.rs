//! report — [`ConformanceReport`] returned by the PM Conformance Loop.

use crate::exporter::OcelLog;
use serde::{Deserialize, Serialize};

// ─── ConformanceReport ────────────────────────────────────────────────────────

/// Conformance report produced by [`crate::stub::ConformanceStub`].
///
/// ## Fitness encoding
///
/// `fitness_q1616` uses Q16.16 fixed-point: `0x0001_0000` == 1.0, `0x0000_8000` == 0.5.
/// This matches the encoding used by `bcinr-powl-receipt::conformance::ConformanceMetrics`.
///
/// ## Integration gap
///
/// When `integration_gap` is `Some`, the fitness is a lightweight approximation
/// (admitted_steps / total_steps).  After activating the `pm_engine` feature and
/// wiring `wasm4pm::conformance::token_replay_pure()`, this field becomes `None`
/// and `fitness_q1616` reflects true token-replay fitness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// Fitness in Q16.16 fixed-point.  `0x0001_0000` == 1.0 (all steps admitted).
    pub fitness_q1616: u32,
    /// Number of admitted (non-denied) plan steps.
    pub admitted_steps: usize,
    /// Total plan steps evaluated.
    pub total_steps: usize,
    /// Whether the receipt's `goal_reached` flag is set.
    pub goal_reached: bool,
    /// OCEL log exported from the same receipt.
    pub ocel_log: OcelLog,
    /// Explanation of what wiring is missing for real Inductive Miner conformance.
    ///
    /// `None` when `pm_engine` is active and real replay was performed.
    /// `Some(reason)` when using the lightweight stub.
    pub integration_gap: Option<String>,
}

impl ConformanceReport {
    /// Return the fitness as a float in `[0.0, 1.0]`.
    #[inline]
    pub fn fitness_f64(&self) -> f64 {
        self.fitness_q1616 as f64 / 65536.0
    }

    /// `true` when fitness equals 1.0 (all steps admitted).
    #[inline]
    pub fn is_perfect_fitness(&self) -> bool {
        self.fitness_q1616 == 0x0001_0000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::OcelLog;
    use serde_json::json;

    fn dummy_log() -> OcelLog {
        OcelLog {
            case_id: "t".to_owned(),
            events: vec![],
            raw_json: json!({}),
        }
    }

    #[test]
    fn fitness_f64_full() {
        let r = ConformanceReport {
            fitness_q1616: 0x0001_0000,
            admitted_steps: 1,
            total_steps: 1,
            goal_reached: true,
            ocel_log: dummy_log(),
            integration_gap: None,
        };
        assert!((r.fitness_f64() - 1.0).abs() < 1e-9);
        assert!(r.is_perfect_fitness());
    }

    #[test]
    fn fitness_f64_half() {
        let r = ConformanceReport {
            fitness_q1616: 0x0000_8000,
            admitted_steps: 1,
            total_steps: 2,
            goal_reached: false,
            ocel_log: dummy_log(),
            integration_gap: Some("stub".to_owned()),
        };
        assert!((r.fitness_f64() - 0.5).abs() < 1e-4);
        assert!(!r.is_perfect_fitness());
    }
}
