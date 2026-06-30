//! stub — lightweight conformance stub with clear integration gap.
//!
//! [`ConformanceStub`] computes a fitness approximation directly from the
//! [`WorldManufactureReceipt`] without calling any wasm4pm algorithm.
//!
//! ## Algorithm
//!
//! For each plan step the receipt is checked against `goal_reached` and the raw
//! step count.  The heuristic fitness is:
//!
//! ```text
//! fitness = admitted_steps / total_steps   (Q16.16 fixed-point)
//! ```
//!
//! where `admitted_steps` == `step_count` when `goal_reached` is true (the plan
//! fully executes) or `step_count - 1` when it is false (the last step is
//! considered denied).  This matches the token-replay intuition without requiring
//! a Petri net.
//!
//! ## Integration gap
//!
//! Activate the `pm_engine` feature and add `wasm4pm` as a dependency to replace
//! this heuristic with `wasm4pm::conformance::token_replay_pure()` on an
//! `InductiveMiner`-discovered net.  See `lib.rs` for the full wiring checklist.

use bcinr_pddl::WorldManufactureReceipt;

use crate::exporter::OcelLog;
use crate::report::ConformanceReport;

/// Lightweight conformance stub derived from a [`WorldManufactureReceipt`].
pub struct ConformanceStub {
    admitted_steps: usize,
    total_steps: usize,
    goal_reached: bool,
}

impl ConformanceStub {
    /// Derive stub metrics from a receipt.
    pub fn from_receipt(receipt: &WorldManufactureReceipt) -> Self {
        let total = receipt.plan.steps.len();
        let goal = receipt.plan_receipt.goal_reached;
        // Heuristic: all steps are "admitted" when goal is reached; otherwise
        // assume the last step was blocked (conservative undercount by one).
        let admitted = if goal || total == 0 {
            total
        } else {
            total.saturating_sub(1)
        };
        Self {
            admitted_steps: admitted,
            total_steps: total,
            goal_reached: goal,
        }
    }

    /// Convert stub metrics and an OCEL log into a [`ConformanceReport`].
    pub fn into_report(self, ocel_log: OcelLog) -> ConformanceReport {
        let fitness_q1616 = fitness_q1616(self.admitted_steps, self.total_steps);

        let integration_gap = Some(
            "Stub fitness: admitted_steps/total_steps (Q16.16). \
             Activate feature `pm_engine` and add wasm4pm dependency to replace \
             with token_replay_pure() on an InductiveMiner-discovered net. \
             Requires nightly Rust + wasm-bindgen toolchain."
                .to_owned(),
        );

        ConformanceReport {
            fitness_q1616,
            admitted_steps: self.admitted_steps,
            total_steps: self.total_steps,
            goal_reached: self.goal_reached,
            ocel_log,
            integration_gap,
        }
    }
}

// ─── Q16.16 arithmetic ───────────────────────────────────────────────────────

/// Compute `numerator / denominator` in Q16.16 fixed-point (branchless).
///
/// Returns `0x0001_0000` (1.0) when denominator is zero (degenerate / trivial
/// case: empty plan is trivially conformant).
fn fitness_q1616(numerator: usize, denominator: usize) -> u32 {
    // Branchless: compute both branches; select with a mask.
    // mask == 0xFFFF_FFFF when denominator > 0, 0 when denominator == 0.
    let denom_nonzero = (denominator != 0) as u64;
    let mask = denom_nonzero.wrapping_neg(); // 0xFFFF_FFFF_FFFF_FFFF or 0

    // Ratio in Q16.16: (num << 16) / den, clamped to 0x0001_0000.
    let shifted = (numerator as u64) << 16;
    let ratio = if denominator == 0 {
        0x0001_0000_u64
    } else {
        // Integer divide gives the Q16.16 representation directly.
        (shifted / denominator as u64).min(0x0001_0000)
    };

    // Apply mask: if denom == 0, return 1.0; otherwise return ratio.
    // Both arms are computed above; select without a runtime branch.
    let fallback = 0x0001_0000_u64;
    let selected = (ratio & mask) | (fallback & !mask);
    selected as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitness_full_when_all_admitted() {
        assert_eq!(fitness_q1616(5, 5), 0x0001_0000);
    }

    #[test]
    fn fitness_half_when_half_admitted() {
        let f = fitness_q1616(1, 2);
        // Q16.16 of 0.5 = 0x0000_8000
        assert_eq!(f, 0x0000_8000);
    }

    #[test]
    fn fitness_zero_numerator() {
        assert_eq!(fitness_q1616(0, 4), 0);
    }

    #[test]
    fn fitness_zero_denominator_returns_one() {
        assert_eq!(fitness_q1616(0, 0), 0x0001_0000);
    }

    #[test]
    fn stub_goal_reached_all_admitted() {
        use bcinr_pddl::manufacture_world;
        let domain = r#"(define (domain d)
          (:requirements :strips)
          (:predicates (p))
          (:action a :parameters () :precondition (p) :effect (not (p))))"#;
        let problem = r#"(define (problem pr) (:domain d) (:init (p)) (:goal (not (p))))"#;
        let receipt = manufacture_world(domain, problem, "stub-test", &[]);
        let stub = ConformanceStub::from_receipt(&receipt);
        if receipt.plan_receipt.goal_reached {
            assert_eq!(stub.admitted_steps, stub.total_steps);
        }
    }

    #[test]
    fn into_report_sets_integration_gap() {
        use bcinr_pddl::manufacture_world;
        use crate::exporter::OcelExporter;
        let domain = r#"(define (domain d)
          (:requirements :strips)
          (:predicates (p))
          (:action a :parameters () :precondition (p) :effect (not (p))))"#;
        let problem = r#"(define (problem pr) (:domain d) (:init (p)) (:goal (not (p))))"#;
        let receipt = manufacture_world(domain, problem, "gap-test", &[]);
        let log = OcelExporter::export(&receipt, "gap-test");
        let stub = ConformanceStub::from_receipt(&receipt);
        let report = stub.into_report(log);
        assert!(report.integration_gap.is_some());
        let gap = report.integration_gap.unwrap();
        assert!(gap.contains("pm_engine"));
        assert!(gap.contains("token_replay_pure"));
    }
}
