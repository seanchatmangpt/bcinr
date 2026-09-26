//! Regression bound for the DME route selector benchmark
//! (`benches/dme_route.rs`; receipt `docs/jira/v26.9.16/bench/dme-bench-receipt.json`).
//!
//! Two bounds, both on the real selector (no doubles):
//! 1. absolute: best-of-R wall time for select+verify over 1024 candidates stays under
//!    a ceiling generous enough for an unoptimized debug build on a loaded CI runner;
//! 2. scaling: best(1024) / best(64) stays under 32. Measured on the v26.9.26
//!    hardening subject (debug, Apple Silicon): 13.5 (three runs); a quadratic
//!    candidate scan grows this ratio toward 256.

use bcinr_cmca::{
    select_dme_route, verify_dme_route_decision, ConsequenceClass, DmeRouteRequest, RouteCandidate,
    RouteClass, WorkKnowledge, WorkStanding,
};
use std::time::{Duration, Instant};

fn workload(n: usize) -> DmeRouteRequest {
    let classes = [
        RouteClass::KnownDeterministic,
        RouteClass::UnknownLocal,
        RouteClass::UnknownIdleEstate,
        RouteClass::UnknownFrontier,
        RouteClass::Refused,
    ];
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let routes = (0..n)
        .map(|i| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            RouteCandidate {
                route: classes[i % classes.len()],
                cost_units: state % 10_000,
                capability_fit: i == 1 || state % 3 != 0,
                budget_units: 1_000,
                required_units: if i == 1 { 1 } else { (state >> 16) % 1_200 },
                evidence_fit: true,
                consequence_fit: true,
            }
        })
        .collect();
    DmeRouteRequest {
        request_id: "bench-2605".into(),
        semantic_subject: "urn:bench:subject".into(),
        standing: WorkStanding::Admitted,
        knowledge: WorkKnowledge::Unknown,
        consequence: ConsequenceClass::Construct,
        deadline_class: "DEFERABLE".into(),
        evidence_obligation: "EXACT_SUBJECT_RECEIPT".into(),
        frontier_escalation_admitted: true,
        routes,
    }
}

fn best_of(runs: usize, req: &DmeRouteRequest) -> Duration {
    (0..runs)
        .map(|_| {
            let start = Instant::now();
            let decision = select_dme_route(req).unwrap();
            assert!(verify_dme_route_decision(req, &decision));
            start.elapsed()
        })
        .min()
        .unwrap()
}

#[test]
fn select_and_verify_stay_within_recorded_regression_bound() {
    let small = workload(64);
    let large = workload(1024);
    let t_small = best_of(15, &small);
    let t_large = best_of(15, &large);
    let ratio = t_large.as_nanos() as f64 / t_small.as_nanos().max(1) as f64;
    println!("dme_route best-of-15: n=64 {t_small:?}, n=1024 {t_large:?}, ratio {ratio:.1}");
    assert!(
        t_large < Duration::from_millis(250),
        "absolute bound exceeded: {t_large:?}"
    );
    assert!(
        ratio < 32.0,
        "super-n-log-n scaling regression: ratio {ratio:.1}"
    );
}
