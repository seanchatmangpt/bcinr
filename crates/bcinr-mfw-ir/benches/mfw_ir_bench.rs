//! Divan benchmarks for the `bcinr-mfw-ir` shared IR — the crate's first
//! benchmark suite. Covers the three real production hot paths:
//!
//! - `event_set`: the fixed-capacity 512-bit (8-word) bitset the planner
//!   uses per epoch — insert/contains/union/subset/intersects/len/
//!   iter_stable over half-full and dense sets (the fixed 8-word shape is
//!   the crate's data-independent O(1) claim, so those are the shapes that
//!   price it).
//! - `digest`: BLAKE3 hash and 2-way mix — the identity primitive every
//!   receipt and epoch id flows through.
//! - `dme_epoch`: `DmeEpoch::root` construction and `advance_epoch` on both
//!   admitted outcomes (Successor with a strictly-descending residual set,
//!   and Closed on the empty residual set).
//!
//! Convention matches `bcinr-powl/benches/*` and
//! `bcinr-cmca/benches/allocation_bench.rs`: Divan, `harness = false`,
//! `black_box` on inputs AND outputs. Base fixtures are built once
//! (`OnceLock`) and borrowed per iteration; mutating calls (`insert`) and
//! consuming calls (`advance_epoch` takes ownership of nothing but the
//! meter descends) start from a clone/const-empty base per iteration —
//! that base cost is part of the real call contract and is priced with the
//! operation. `advance_epoch` benchmarks clone the parent epoch (a 4- or
//! 1-element `Vec<Digest>` copy); the `dme_epoch_root` benchmark prices
//! `DmeEpoch::root` itself so the reader can subtract construction cost.
//!
//! Numbers are receipts, not claims.

use std::sync::OnceLock;

use bcinr_mfw_ir::{
    advance_epoch, DescentMeter, Digest, DmeEpoch, EpochAuthority, EventSet, ReceiptFeedback,
    MAX_EPOCH_EVENTS,
};
use divan::black_box;

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// EventSet fixtures
// ---------------------------------------------------------------------------

/// Half-dense set: every 2nd event id present (256 members).
fn half_set() -> &'static EventSet {
    static S: OnceLock<EventSet> = OnceLock::new();
    S.get_or_init(|| {
        let mut set = EventSet::empty();
        let mut id = 0;
        while id < MAX_EPOCH_EVENTS {
            set.insert(id);
            id += 2;
        }
        set
    })
}

/// Dense set: every 3rd event id present (171 members, different word
/// pattern from the half set so pair operations mix dense/sparse words).
fn third_set() -> &'static EventSet {
    static S: OnceLock<EventSet> = OnceLock::new();
    S.get_or_init(|| {
        let mut set = EventSet::empty();
        let mut id = 0;
        while id < MAX_EPOCH_EVENTS {
            set.insert(id);
            id += 3;
        }
        set
    })
}

/// Proper subset of the half set: every 4th id (a strict subset of the
/// every-2nd pattern).
fn quarter_set() -> &'static EventSet {
    static S: OnceLock<EventSet> = OnceLock::new();
    S.get_or_init(|| {
        let mut set = EventSet::empty();
        let mut id = 0;
        while id < MAX_EPOCH_EVENTS {
            set.insert(id);
            id += 4;
        }
        set
    })
}

#[divan::bench]
fn event_set_insert_single() {
    // Start from const empty (free) — the single-insert contract cost.
    let mut set = EventSet::empty();
    set.insert(black_box(197usize));
    let _ = black_box(set);
}

#[divan::bench]
fn event_set_contains_hit() {
    // 2 divides every even id in the half set: guaranteed hit.
    let out = black_box(half_set()).contains(black_box(198usize));
    let _ = black_box(out);
}

#[divan::bench]
fn event_set_contains_miss() {
    // 5 is coprime to 2 and to MAX_EPOCH_EVENTS region: representative miss.
    let out = black_box(half_set()).contains(black_box(5usize));
    let _ = black_box(out);
}

#[divan::bench]
fn event_set_union() {
    let out = black_box(half_set()).union(black_box(third_set()));
    let _ = black_box(out);
}

#[divan::bench]
fn event_set_is_subset_of() {
    let out = black_box(quarter_set()).is_subset_of(black_box(half_set()));
    let _ = black_box(out);
}

#[divan::bench]
fn event_set_intersects() {
    let out = black_box(quarter_set()).intersects(black_box(third_set()));
    let _ = black_box(out);
}

#[divan::bench]
fn event_set_len() {
    let out = black_box(half_set()).len();
    let _ = black_box(out);
}

#[divan::bench]
fn event_set_iter_stable_full() {
    // Ascending popcount-order iteration over all 256 members; the fold
    // consumes the iterator so the cost cannot be optimized away.
    let mut sum = 0usize;
    for id in black_box(half_set()).iter_stable() {
        sum = sum.wrapping_add(id);
    }
    let _ = black_box(sum);
}

// ---------------------------------------------------------------------------
// Digest — BLAKE3 identity primitives
// ---------------------------------------------------------------------------

/// Fixed 64-byte representative payload (one hash-block, the common receipt
/// frame size).
const PAYLOAD_64B: [u8; 64] = {
    let mut bytes = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        bytes[i] = (i as u8).wrapping_mul(31).wrapping_add(7);
        i += 1;
    }
    bytes
};

#[divan::bench]
fn digest_hash_64b() {
    let out = Digest::hash(black_box(&PAYLOAD_64B));
    let _ = black_box(out);
}

#[divan::bench]
fn digest_mix() {
    let a = Digest::hash(b"bcinr-mfw-ir-bench-a");
    let b = Digest::hash(b"bcinr-mfw-ir-bench-b");
    let out = black_box(&a).mix(black_box(&b));
    let _ = black_box(out);
}

// ---------------------------------------------------------------------------
// dme_epoch — root construction + both advance outcomes
// ---------------------------------------------------------------------------

fn ontology_digest() -> Digest {
    Digest::hash(b"ontology-v1")
}

fn residual(i: usize) -> Digest {
    Digest::hash(format!("residual-{i}").as_bytes())
}

/// Parent epoch holding 4 residual obligations + matching feedback carrying
/// 3 (strictly descending, unique) — the Successor path.
fn successor_fixture() -> &'static (DmeEpoch, ReceiptFeedback) {
    static F: OnceLock<(DmeEpoch, ReceiptFeedback)> = OnceLock::new();
    F.get_or_init(|| {
        let parent = DmeEpoch::root(
            ontology_digest(),
            vec![residual(0), residual(1), residual(2), residual(3)],
        )
        .expect("4-residual root must be in domain");
        let feedback = ReceiptFeedback {
            receipt_digest: Digest::hash(b"receipt-frame"),
            observed_ontology_digest: ontology_digest(),
            residual_obligations: vec![residual(0), residual(1), residual(2)],
            authority: EpochAuthority::None,
        };
        (parent, feedback)
    })
}

/// Parent epoch holding 1 residual + feedback carrying none — the Closed path.
fn closed_fixture() -> &'static (DmeEpoch, ReceiptFeedback) {
    static F: OnceLock<(DmeEpoch, ReceiptFeedback)> = OnceLock::new();
    F.get_or_init(|| {
        let parent = DmeEpoch::root(ontology_digest(), vec![residual(0)])
            .expect("1-residual root must be in domain");
        let feedback = ReceiptFeedback {
            receipt_digest: Digest::hash(b"receipt-frame-final"),
            observed_ontology_digest: ontology_digest(),
            residual_obligations: vec![],
            authority: EpochAuthority::None,
        };
        (parent, feedback)
    })
}

#[divan::bench]
fn dme_epoch_root_4_residuals() {
    // Prices construction itself (range checks + epoch id BLAKE3 digest),
    // so `advance_epoch_*` numbers can be read net of setup cost.
    let out = DmeEpoch::root(
        black_box(ontology_digest()),
        black_box(vec![residual(0), residual(1), residual(2), residual(3)]),
    );
    let _ = black_box(out);
}

#[divan::bench]
fn advance_epoch_successor() {
    let (parent, feedback) = successor_fixture();
    let parent = parent.clone(); // 4-digest Vec copy, part of the call contract
    let mut meter = DescentMeter::new(64);
    let out = advance_epoch(black_box(&parent), black_box(feedback), &mut meter);
    let _ = black_box(out);
}

#[divan::bench]
fn advance_epoch_closed() {
    let (parent, feedback) = closed_fixture();
    let parent = parent.clone(); // 1-digest Vec copy, part of the call contract
    let mut meter = DescentMeter::new(64);
    let out = advance_epoch(black_box(&parent), black_box(feedback), &mut meter);
    let _ = black_box(out);
}
