//! bcinr-pm-bridge — PM Conformance Loop (Innovation 3)
//!
//! ## Path B implementation
//!
//! Chosen because wasm4pm's `InductiveMiner` and `token_replay_pure()` are callable
//! from plain Rust but require nightly + wasm-bindgen, which break bcinr's
//! stable-MSRV-1.70 and `#![forbid(unsafe_code)]` invariants.  Full wiring is gated
//! behind the optional `pm_engine` feature (see `Cargo.toml`).
//!
//! ## What is implemented (stable, no new deps)
//!
//! - [`OcelExporter`]: converts a [`WorldManufactureReceipt`] into an OCEL 2.0 JSON
//!   object, one event per plan step.
//! - [`OcelLog`]: typed wrapper around the OCEL JSON value.
//! - [`ConformanceStub`]: lightweight step-replay counter.  Computes
//!   `fitness = admitted_steps / total_steps` (Q16.16 fixed-point) from the receipt
//!   without calling any wasm4pm algorithm.  Returns a [`ConformanceReport`] with a
//!   clear `integration_gap` explaining exactly what wiring is needed.
//! - [`manufacture_with_conformance`]: thin orchestrator that calls
//!   `bcinr_pddl::manufacture_world`, wraps the receipt, and returns
//!   `(WorldManufactureReceipt, Option<ConformanceReport>)`.
//!
//! ## Integration gap (what is left for `pm_engine`)
//!
//! To enable real Inductive Miner discovery + token-based replay:
//! 1. Activate feature `pm_engine` in this crate.
//! 2. Add `wasm4pm = { path = "/Users/sac/wasm4pm/wasm4pm" }` to `[dependencies]`
//!    (under the feature gate).
//! 3. Call `wasm4pm::more_discovery::discover_inductive_miner_from_log()` with an
//!    event log built from [`OcelLog::events`].
//! 4. Call `wasm4pm::conformance::token_replay_pure()` with the discovered net.
//! 5. Replace the stub fitness in [`ConformanceReport::fitness_q1616`] with the
//!    result.
//! This requires switching the workspace to nightly and accepting wasm-bindgen as a
//! transitive dependency.

pub mod exporter;
pub mod report;
pub mod stub;

pub use exporter::{OcelExporter, OcelLog, export_ocel_log};
pub use report::ConformanceReport;
pub use stub::ConformanceStub;

use bcinr_pddl::{WorldManufactureReceipt, manufacture_world};

// ─── Top-level orchestrator ───────────────────────────────────────────────────

/// Manufacture a world and compute a conformance report in one call.
///
/// Always succeeds (errors are surfaced inside [`WorldManufactureReceipt::admitted`]).
/// The [`ConformanceReport`] is `Some` when the receipt is admitted and contains at
/// least one plan step; it is `None` when planning failed or produced an empty plan.
///
/// ## Example
///
/// ```rust
/// # use bcinr_pm_bridge::manufacture_with_conformance;
/// let domain = "(define (domain d) (:predicates (p)) (:action a :parameters () :precondition (p) :effect (not (p))))";
/// let problem = "(define (problem pr) (:domain d) (:init (p)) (:goal (not (p))))";
/// let (receipt, report) = manufacture_with_conformance(domain, problem, "case-1");
/// // receipt.admitted may be true or false depending on planning outcome.
/// let _ = report; // Some(ConformanceReport{..}) or None
/// ```
pub fn manufacture_with_conformance(
    domain_text: &str,
    problem_text: &str,
    case_id: &str,
) -> (WorldManufactureReceipt, Option<ConformanceReport>) {
    let receipt = manufacture_world(domain_text, problem_text, case_id, &[]);

    let report = if receipt.admitted && !receipt.plan.steps.is_empty() {
        let log = export_ocel_log(&receipt);
        let stub = ConformanceStub::from_receipt(&receipt);
        Some(stub.into_report(log))
    } else {
        None
    };

    (receipt, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOMAIN: &str = r#"(define (domain blocks)
  (:requirements :strips)
  (:predicates (on-table ?x) (clear ?x) (on ?x ?y) (holding ?x) (arm-empty))
  (:action pick-up
    :parameters (?x)
    :precondition (and (clear ?x) (on-table ?x) (arm-empty))
    :effect (and (not (on-table ?x)) (not (clear ?x)) (not (arm-empty)) (holding ?x)))
  (:action put-down
    :parameters (?x)
    :precondition (holding ?x)
    :effect (and (on-table ?x) (clear ?x) (arm-empty) (not (holding ?x)))))"#;

    const PROBLEM: &str = r#"(define (problem pick-put)
  (:domain blocks)
  (:objects a)
  (:init (on-table a) (clear a) (arm-empty))
  (:goal (on-table a)))"#;

    #[test]
    fn manufacture_with_conformance_returns_tuple() {
        let (receipt, _report) = manufacture_with_conformance(DOMAIN, PROBLEM, "test-1");
        // Receipt is always returned regardless of planning outcome.
        assert!(!receipt.domain_name.is_empty());
    }

    #[test]
    fn export_ocel_log_from_receipt_roundtrip() {
        let receipt = manufacture_world(DOMAIN, PROBLEM, "test-ocel", &[]);
        let log = export_ocel_log(&receipt);
        // export_ocel_log uses problem_name as case_id.
        assert_eq!(log.case_id, receipt.problem_name);
        // Event count must equal plan step count.
        assert_eq!(log.events.len(), receipt.plan.steps.len());
    }
}
