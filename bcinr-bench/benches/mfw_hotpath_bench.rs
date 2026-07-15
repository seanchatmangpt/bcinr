//! Microbenchmarks for the MFW retrofit's genuinely hot-path primitives.
//!
//! Before this file, `bcinr-bench` had zero coverage of `bcinr-mfw-ir`'s
//! `EventSet`/`ExecutableConcurrencyComplex::admits` or `bcinr-pddl`'s
//! `q_lens` — and didn't even declare `bcinr-mfw-ir`/`bcinr-powl` as
//! dependencies, so no bench file could reach these types at all. Per
//! CLAUDE.md's own "Add algorithm" convention ("... add a benchmark in
//! `bcinr-bench/`"), this fills that gap for the three primitives named in
//! the MFW retrofit gap-hunt as the most call-volume-sensitive:
//!
//! - [`ExecutableConcurrencyComplex::admits`] — called once per ready-set
//!   candidate inside `StableMaximalSelector::select`'s inner loop on
//!   every POWL scheduler tick.
//! - [`q_lens`] — called per exploit-rail search step from `QLensRail`, on
//!   every best-first step.
//! - [`EventSet`]'s fixed-size bitset operations, which back both of the
//!   above.
//!
//! Uses `divan` (not Criterion) to match `pddl_quick_bench.rs`'s low
//! per-benchmark overhead, keeping this suite fast enough to run
//! routinely rather than only on demand.

use bcinr_mfw_ir::{Digest, EventSet, ExecutableConcurrencyComplex, MinimalNonFace};
use bcinr_pddl::mfw::{q_lens, PositiveDistribution, PositiveMass, QValue};
use std::collections::BTreeMap;
use std::time::Instant;

// ---------------------------------------------------------------------------
// EventSet: fixed-size bitset ops
// ---------------------------------------------------------------------------

#[divan::bench]
fn event_set_insert() -> EventSet {
    let mut s = EventSet::empty();
    for i in (0..512).step_by(7) {
        s.insert(divan::black_box(i));
    }
    s
}

#[divan::bench]
fn event_set_contains() -> bool {
    let mut s = EventSet::empty();
    for i in (0..512).step_by(7) {
        s.insert(i);
    }
    let mut found = false;
    for i in 0..512 {
        found ^= s.contains(divan::black_box(i));
    }
    found
}

#[divan::bench]
fn event_set_is_subset_of() -> bool {
    let mut a = EventSet::empty();
    let mut b = EventSet::empty();
    for i in (0..512).step_by(11) {
        a.insert(i);
        b.insert(i);
    }
    for i in (0..512).step_by(3) {
        b.insert(i);
    }
    divan::black_box(&a).is_subset_of(divan::black_box(&b))
}

#[divan::bench]
fn event_set_union() -> EventSet {
    let mut a = EventSet::empty();
    let mut b = EventSet::empty();
    for i in (0..512).step_by(5) {
        a.insert(i);
    }
    for i in (0..512).step_by(7) {
        b.insert(i);
    }
    divan::black_box(&a).union(divan::black_box(&b))
}

// ---------------------------------------------------------------------------
// ExecutableConcurrencyComplex::admits — the per-candidate admission gate
// ---------------------------------------------------------------------------

/// A complex with `n` two-element minimal nonfaces `{2i, 2i+1}` — a
/// reasonably realistic shape for `PddlConcurrencyAnalyzer`'s real output
/// (one nonface per dependent action pair).
fn complex_with_nonfaces(n: usize) -> ExecutableConcurrencyComplex {
    let mut minimal_nonfaces = Vec::with_capacity(n);
    let conflict_witnesses = BTreeMap::new();
    for i in 0..n {
        let members = EventSet::empty().with(2 * i).with(2 * i + 1);
        let witness_digest = Digest::hash(&(i as u64).to_le_bytes());
        minimal_nonfaces.push(MinimalNonFace {
            members,
            witness_digest,
        });
    }
    ExecutableConcurrencyComplex {
        event_count: 2 * n,
        minimal_nonfaces,
        conflict_witnesses,
        digest: Digest::hash(b"bench-complex"),
    }
}

#[divan::bench(args = [8, 64, 256])]
fn concurrency_admits(n: usize) -> bool {
    let complex = complex_with_nonfaces(n);
    // A candidate that avoids every nonface -- the common "admitted" case
    // a scheduler's inner loop hits repeatedly while building a ready set.
    let candidate = EventSet::empty().with(1).with(3).with(5);
    divan::black_box(&complex).admits(divan::black_box(&candidate))
}

// ---------------------------------------------------------------------------
// q_lens — the exploit rail's per-tick frontier-measure normalization
// ---------------------------------------------------------------------------

#[divan::bench(args = [4, 32, 128])]
fn q_lens_bench(n: usize) {
    let entries: Vec<(usize, PositiveMass)> = (0..n)
        .map(|i| (i, PositiveMass::new((i + 1) as f64).unwrap()))
        .collect();
    let dist = PositiveDistribution::new(entries).unwrap();
    let q = QValue::new(1.5).unwrap();
    divan::black_box(q_lens(q, divan::black_box(&dist)).unwrap());
}

fn main() {
    let start = Instant::now();
    divan::main();
    eprintln!("mfw_hotpath_bench wall clock: {:?}", start.elapsed());
}
