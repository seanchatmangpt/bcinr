//! Regression bound for the DME epoch benchmark
//! (`benches/dme_epoch.rs`; receipt `docs/jira/v26.9.16/bench/dme-bench-receipt.json`).
//!
//! Real `advance_epoch` on a root with n residuals and feedback discharging one
//! obligation. Bounds: best-of-R for n=512 under a debug-build ceiling, and
//! best(512)/best(64) under 14. Measured on the v26.9.26 hardening subject (debug,
//! Apple Silicon): sorted/binary-search set descent ratio 7.9-9.0; the quadratic
//! `Vec::contains` membership check it replaced measured 18.5 and fails this bound.

use bcinr_mfw_ir::{
    advance_epoch, DescentMeter, Digest, DmeEpoch, EpochAdvance, EpochAuthority, ReceiptFeedback,
};
use std::time::{Duration, Instant};

fn d(i: u64) -> Digest {
    Digest::hash(&i.to_le_bytes())
}

fn best_of(runs: usize, n: u64) -> Duration {
    let residuals: Vec<Digest> = (0..n).map(d).collect();
    let root = DmeEpoch::root(d(u64::MAX), residuals.clone()).unwrap();
    let fb = ReceiptFeedback {
        receipt_digest: d(u64::MAX - 1),
        observed_ontology_digest: root.ontology_digest,
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
    let t_small = best_of(15, 64);
    let t_large = best_of(15, 512);
    let ratio = t_large.as_nanos() as f64 / t_small.as_nanos().max(1) as f64;
    println!("dme_epoch best-of-15: n=64 {t_small:?}, n=512 {t_large:?}, ratio {ratio:.1}");
    assert!(
        t_large < Duration::from_millis(100),
        "absolute bound exceeded: {t_large:?}"
    );
    assert!(
        ratio < 14.0,
        "super-n-log-n scaling regression: ratio {ratio:.1}"
    );
}
