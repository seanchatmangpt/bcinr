//! Regression bound for the DME epoch benchmark
//! (`benches/dme_epoch.rs`; receipt `docs/jira/v26.9.16/bench/dme-bench-receipt.json`).
//!
//! Real `advance_epoch` on a root with n residuals and feedback discharging one
//! obligation. Bounds (round 2, v26.9.26): best-of-9 for n=4096 under 60 ms (debug
//! build) and best(4096)/best(512) under 20. Measured on the round-2 subject (debug,
//! Apple Silicon, 3 runs): n=4096 9.06-9.07 ms, ratio 8.1-8.8. The quadratic
//! `Vec::contains` membership mutant measured n=4096 76.9-79.4 ms, ratio 36.4-42.0,
//! so it fails both bounds on every run (the old n=64/n=512 guard killed it 3-4 of 6).

use bcinr_mfw_ir::{
    advance_epoch, DescentMeter, Digest, DmeEpoch, EpochAdvance, EpochAuthority, ReceiptFeedback,
};
use std::time::{Duration, Instant};

const ABSOLUTE_BOUND_MS: u64 = 60;
const RATIO_BOUND: f64 = 20.0;

fn d(i: u64) -> Digest {
    Digest::hash(&i.to_le_bytes())
}

fn best_of(runs: usize, n: u64) -> Duration {
    let residuals: Vec<Digest> = (0..n).map(d).collect();
    let root = DmeEpoch::root(d(u64::MAX), residuals.clone()).unwrap();
    let fb = ReceiptFeedback {
        receipt_digest: d(u64::MAX - 1),
        observed_ontology_digest: root.ontology_digest(),
        residual_obligations: residuals[1..].iter().rev().copied().collect(),
        authority: EpochAuthority::None,
    };
    (0..runs)
        .map(|_| {
            let mut meter = DescentMeter::new(8);
            let start = Instant::now();
            let out = advance_epoch(&root, &fb, &mut meter).unwrap();
            let elapsed = start.elapsed();
            assert!(matches!(out, EpochAdvance::Successor(_)));
            elapsed
        })
        .min()
        .unwrap()
}

#[test]
fn advance_epoch_stays_within_recorded_regression_bound() {
    // n = 512 and n = 4096 (not 64): at n = 64 the debug-build timing is bimodal
    // (fixed per-call overhead dominates), which made the old 512/64 ratio guard kill
    // the quadratic-membership mutant only 3-4 times in 6. At 4096 the quadratic
    // mutant does ~8x more work per element than n log n, so both the ratio and the
    // absolute large-n bound separate it deterministically.
    let t_small = best_of(9, 512);
    let t_large = best_of(9, 4096);
    let ratio = t_large.as_nanos() as f64 / t_small.as_nanos().max(1) as f64;
    println!("dme_epoch best-of-9: n=512 {t_small:?}, n=4096 {t_large:?}, ratio {ratio:.1}");
    assert!(
        t_large < Duration::from_millis(ABSOLUTE_BOUND_MS),
        "absolute bound exceeded: {t_large:?}"
    );
    assert!(
        ratio < RATIO_BOUND,
        "super-n-log-n scaling regression: ratio {ratio:.1}"
    );
}
