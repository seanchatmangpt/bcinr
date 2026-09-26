//! Criterion benchmark for the DME route selector (A2A-2605).
//!
//! Workload: an admitted UNKNOWN request with `n` candidate routes spread over all
//! five route classes, with deterministic pseudo-random costs/budgets and ~1/3 of
//! candidates unlawful. Measures `select_dme_route` (canonicalize + select + seal)
//! and `verify_dme_route_decision` (full replay). Recorded numbers and the regression
//! bound live in `docs/jira/v26.9.16/bench/dme-bench-receipt.json`.

use bcinr_cmca::{
    select_dme_route, verify_dme_route_decision, ConsequenceClass, DmeRouteRequest, RouteCandidate,
    RouteClass, WorkKnowledge, WorkStanding,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn workload(n: usize) -> DmeRouteRequest {
    let classes = [
        RouteClass::KnownDeterministic,
        RouteClass::UnknownLocal,
        RouteClass::UnknownIdleEstate,
        RouteClass::UnknownFrontier,
        RouteClass::Refused,
    ];
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    // Candidate 1 is always lawful so every size has a selectable route.
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

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("dme_route");
    for n in [4usize, 64, 1024] {
        let req = workload(n);
        let decision = select_dme_route(&req).expect("workload has a lawful route");
        group.bench_with_input(BenchmarkId::new("select", n), &req, |b, r| {
            b.iter(|| select_dme_route(black_box(r)))
        });
        group.bench_with_input(BenchmarkId::new("verify", n), &req, |b, r| {
            b.iter(|| verify_dme_route_decision(black_box(r), black_box(&decision)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
