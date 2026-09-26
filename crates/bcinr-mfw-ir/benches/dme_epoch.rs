//! Criterion benchmark for recursive DME epoch closure (A2A-2606) and experience
//! compile-back (A2A-2612).
//!
//! Workload: a root epoch with `n` residual obligations advanced by feedback that
//! discharges one obligation (the worst case for set-descent checking: `n - 1`
//! membership checks against an `n`-set). Recorded numbers and the regression bound
//! live in `docs/jira/v26.9.16/bench/dme-bench-receipt.json`.

use bcinr_mfw_ir::{
    advance_epoch, candidate_from_execution, qualify_candidate, DescentMeter, Digest, DmeEpoch,
    EpochAuthority, ExperienceEvidence, ReceiptFeedback,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn d(i: u64) -> Digest {
    Digest::hash(&i.to_le_bytes())
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("dme_epoch");
    for n in [4u64, 64, 512] {
        let residuals: Vec<Digest> = (0..n).map(d).collect();
        let root = DmeEpoch::root(d(u64::MAX), residuals.clone()).unwrap();
        let fb = ReceiptFeedback {
            receipt_digest: d(u64::MAX - 1),
            observed_ontology_digest: root.ontology_digest,
            residual_obligations: residuals[1..].iter().rev().copied().collect(),
            authority: EpochAuthority::None,
        };
        group.bench_with_input(BenchmarkId::new("root", n), &residuals, |b, r| {
            b.iter(|| DmeEpoch::root(d(u64::MAX), black_box(r.clone())))
        });
        group.bench_with_input(BenchmarkId::new("advance", n), &fb, |b, f| {
            b.iter(|| {
                let mut meter = DescentMeter::new(8);
                advance_epoch(black_box(&root), black_box(f), &mut meter)
            })
        });
    }
    let evidence = ExperienceEvidence {
        semantic_subject: d(1),
        execution_receipt: d(2),
        execution_succeeded: true,
        ontology_digest: d(3),
        manufacturer_digest: d(4),
    };
    group.bench_function("experience_candidate_and_qualify", |b| {
        b.iter(|| {
            let cand = candidate_from_execution(black_box(&evidence)).unwrap();
            qualify_candidate(&cand, d(3), d(4), Some(d(5)), Some(d(6)))
        })
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
