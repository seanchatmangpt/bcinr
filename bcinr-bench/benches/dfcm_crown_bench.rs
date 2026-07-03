//! DfCM crown bench: one fixed, bounded (ops × capacity) matrix exercising
//! topology -> planning -> analysis -> admission -> receipt -> replay, all
//! within the 8/64 bound. See docs/DFCM_CONTRACT.md for the contract this
//! enforces: the whole 16-cell suite must complete in <= 5s wall-clock
//! (gated separately by crates/bcinr-pddl/tests/dfcm_crown_suite.rs — this
//! bench file measures it, the test gates it).
//!
//! Uses divan, matching pddl_quick_bench.rs, for the same reason: low
//! per-benchmark overhead keeps the whole suite inside its own wall-clock
//! budget instead of fighting Criterion's adaptive sampling.

use bcinr_pddl::run_dfcm_crown_suite;
use std::time::Instant;

#[divan::bench(sample_count = 3, sample_size = 1)]
fn dfcm_crown_suite_bench() {
    let receipt = divan::black_box(run_dfcm_crown_suite().expect("dfcm crown suite"));
    assert!(receipt.suite_passed_5s_gate, "DfCM crown suite exceeded the 5s gate");
}

fn main() {
    let start = Instant::now();
    divan::main();

    // Run once more outside the divan harness purely to print the headline
    // receipt fields — divan's own timing is the source of truth for the
    // benchmark numbers; this is just human-readable confirmation.
    if let Ok(receipt) = run_dfcm_crown_suite() {
        eprintln!(
            "dfcm_crown_bench receipt: wall_clock_ms={} topology_ns={} planning_ns={} analysis_ns={} admission_ns={} receipt_ns={} replay_ns={} max_ops={} max_parallelism={} passed_5s_gate={}",
            receipt.wall_clock_ms,
            receipt.topology_ns,
            receipt.planning_ns,
            receipt.analysis_ns,
            receipt.admission_ns,
            receipt.receipt_ns,
            receipt.replay_ns,
            receipt.max_ops,
            receipt.max_parallelism,
            receipt.suite_passed_5s_gate,
        );
        eprintln!(
            "dfcm_crown_bench cold/warm split: cold_topology_once_ns={} warm_replay_existing_receipt_ns={} (vs. full-reexecution replay_ns={} above)",
            receipt.cold_topology_once_ns,
            receipt.warm_replay_existing_receipt_ns,
            receipt.replay_ns,
        );
        eprintln!(
            "dfcm_crown_bench L3 admission substage: fact_load_ns={} query_ns={} effects_apply_ns={} proof_receipt_build_ns={} trace_build_ns={}",
            receipt.admission_substage.fact_load_ns,
            receipt.admission_substage.query_ns,
            receipt.admission_substage.effects_apply_ns,
            receipt.admission_substage.proof_receipt_build_ns,
            receipt.admission_substage.trace_build_ns,
        );
        eprintln!(
            "dfcm_crown_bench L3 replay substage: fact_load_ns={} query_ns={} effects_apply_ns={} proof_receipt_build_ns={} trace_build_ns={}",
            receipt.replay_substage.fact_load_ns,
            receipt.replay_substage.query_ns,
            receipt.replay_substage.effects_apply_ns,
            receipt.replay_substage.proof_receipt_build_ns,
            receipt.replay_substage.trace_build_ns,
        );
        eprintln!(
            "dfcm_crown_bench L3 analysis substage: resource_key_collect_ns={} base_plan_ns={} perturb_minus_ns={} perturb_plus_ns={} sensitivity_compute_ns={} result_build_ns={}",
            receipt.analysis_substage.resource_key_collect_ns,
            receipt.analysis_substage.base_plan_ns,
            receipt.analysis_substage.perturb_minus_ns,
            receipt.analysis_substage.perturb_plus_ns,
            receipt.analysis_substage.sensitivity_compute_ns,
            receipt.analysis_substage.result_build_ns,
        );
    }

    eprintln!("dfcm_crown_bench wall clock: {:?}", start.elapsed());
}
