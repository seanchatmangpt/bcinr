//! DfCM crown suite: a fixed, bounded (ops × capacity) matrix, each cell
//! exercising topology → planning → analysis → admission → receipt → replay
//! entirely within the 8/64 bound (≤ 64 durative-action ground instances, ≤
//! 64 POWL tape ops per cell — see `docs/DFCM_CONTRACT.md`). Backs both
//! `bcinr-bench/benches/dfcm_crown_bench.rs` and the wall-clock gate test in
//! `tests/dfcm_crown_suite.rs`, so the same suite is what's benchmarked and
//! what's gated.
//!
//! This proves composition stays inside one fixed wall-clock envelope. It
//! does not claim to be the fastest planner.

use std::time::Instant;

use crate::execute::{compute_plan_chain, execute_temporal_plan};
use crate::ground::GroundTemporalProblem;
use crate::powl_bridge::temporal_plan_to_powl_tape;
use crate::schedule_analysis::analyze_schedule;
use crate::{domain_from_pddl, problem_from_pddl, Pddl8Error};

/// Headline metrics for one run of the full crown suite (all 16 cells).
#[derive(Debug, Clone, Copy)]
pub struct DfcmBenchReceipt {
    pub wall_clock_ms: u128,
    pub topology_ns: u128,
    pub planning_ns: u128,
    pub analysis_ns: u128,
    pub admission_ns: u128,
    pub receipt_ns: u128,
    pub replay_ns: u128,
    pub max_ops: u8,
    pub max_parallelism: u8,
    pub suite_passed_5s_gate: bool,
}

const OPS_MATRIX: [usize; 4] = [8, 16, 32, 64];
const CAPACITY_MATRIX: [usize; 4] = [1, 2, 4, 8];

const DOMAIN: &str = r#"
(define (domain dfcm-crown)
  (:requirements :durative-actions :numeric-fluents :typing)
  (:types worker)
  (:predicates (idle ?w - worker) (busy ?w - worker) (done ?w - worker))
  (:functions (available-workers))
  (:durative-action assign-worker
    :parameters (?w - worker)
    :duration (= ?duration 1)
    :condition (and (at start (idle ?w)) (at start (>= (available-workers) 1)))
    :effect (and
      (at start (decrease (available-workers) 1))
      (at start (not (idle ?w))) (at start (busy ?w))
      (at end (increase (available-workers) 1))
      (at end (not (busy ?w))) (at end (done ?w)))))
"#;

fn problem_text(n_workers: usize, capacity: usize) -> String {
    let workers: Vec<String> = (1..=n_workers).map(|i| format!("w{i}")).collect();
    let objects = format!("{} - worker", workers.join(" "));
    let idle: Vec<String> = workers.iter().map(|w| format!("(idle {w})")).collect();
    let done: Vec<String> = workers.iter().map(|w| format!("(done {w})")).collect();
    format!(
        "(define (problem dfcm-crown-{n_workers}-{capacity})\n  (:domain dfcm-crown)\n  (:objects {objects})\n  (:init {} (= (available-workers) {capacity}))\n  (:goal (and {})))",
        idle.join(" "),
        done.join(" ")
    )
}

/// Run the full 16-cell (ops × capacity) matrix once, recording wall-clock
/// and per-stage timings, gated against `docs/DFCM_CONTRACT.md`'s ≤ 5s rule.
pub fn run_dfcm_crown_suite() -> Result<DfcmBenchReceipt, Pddl8Error> {
    let suite_start = Instant::now();

    let mut topology_ns: u128 = 0;
    let mut planning_ns: u128 = 0;
    let mut analysis_ns: u128 = 0;
    let mut admission_ns: u128 = 0;
    let mut receipt_ns: u128 = 0;
    let mut replay_ns: u128 = 0;
    let mut max_ops_seen: u8 = 0;
    let mut max_parallelism_seen: u8 = 0;

    let domain = domain_from_pddl(DOMAIN)?;

    for &n_workers in &OPS_MATRIX {
        for &capacity in &CAPACITY_MATRIX {
            let problem = problem_from_pddl(&problem_text(n_workers, capacity))?;
            let gtp = GroundTemporalProblem::build(&domain, &problem)?;

            let t0 = Instant::now();
            let plan = gtp.find_temporal_plan()?;
            planning_ns += t0.elapsed().as_nanos();

            let t1 = Instant::now();
            let ops = temporal_plan_to_powl_tape(&plan);
            topology_ns += t1.elapsed().as_nanos();
            max_ops_seen = max_ops_seen.max(ops.len().min(u8::MAX as usize) as u8);

            let t2 = Instant::now();
            let analysis = analyze_schedule(&gtp, &["available-workers".to_string()])?;
            analysis_ns += t2.elapsed().as_nanos();
            max_parallelism_seen = max_parallelism_seen.max(analysis.max_parallelism);

            // admission_ns times execute_temporal_plan as a whole — admission-gate
            // checking dominates its cost, but this is not a precise sub-component
            // split (see docs/DFCM_CONTRACT.md's measurement honesty note).
            let t3 = Instant::now();
            let (receipt, _ocel) =
                execute_temporal_plan(&plan, &domain, &problem, "dfcm-crown", &[])?;
            admission_ns += t3.elapsed().as_nanos();

            // receipt_ns times an isolated chain-hash recomputation — a real,
            // independently-measured operation, not a slice of the call above.
            let t4 = Instant::now();
            let _chain = compute_plan_chain(&plan.steps);
            receipt_ns += t4.elapsed().as_nanos();

            // replay_ns: re-execute the same plan/case_id and confirm the
            // receipt chain is reproducible — determinism, not just speed.
            let t5 = Instant::now();
            let (replay_receipt, _ocel) =
                execute_temporal_plan(&plan, &domain, &problem, "dfcm-crown", &[])?;
            replay_ns += t5.elapsed().as_nanos();
            debug_assert_eq!(receipt.chain_hash, replay_receipt.chain_hash);
        }
    }

    let wall_clock_ms = suite_start.elapsed().as_millis();
    Ok(DfcmBenchReceipt {
        wall_clock_ms,
        topology_ns,
        planning_ns,
        analysis_ns,
        admission_ns,
        receipt_ns,
        replay_ns,
        max_ops: max_ops_seen,
        max_parallelism: max_parallelism_seen,
        suite_passed_5s_gate: wall_clock_ms <= 5000,
    })
}
