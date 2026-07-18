//! Allocation gate: asserts zero heap allocations across a call to the
//! authoritative `allocator::allocate()` root.
//!
//! Gated behind the dev-only `alloc-gate` feature (see `src/alloc_counter.rs`
//! and `Cargo.toml`). Installs `CountingAlloc` as this test binary's
//! `#[global_allocator]` — an integration test file is its own binary crate,
//! so this does not affect any other test target or the library build.
//!
//! Run: `cargo test -p bcinr-cmca --features alloc-gate --test alloc_gate -- --nocapture`

#![cfg(feature = "alloc-gate")]

use bcinr_cmca::alloc_counter::counting_alloc::{snapshot, CountingAlloc};

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

fn get_proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
    AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        CertifiedLearning::admit_learning(),
    )
}

#[test]
fn allocate_root_performs_zero_heap_allocations() {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];
    let proof = get_proof();

    // Warm up: force one-time lazy-static-ish machinery (none is expected in
    // this crate, but taking the snapshot immediately before the timed call,
    // rather than at process start, keeps the assertion scoped to exactly the
    // call under test regardless of what test-harness setup allocated before
    // this test function ran).
    let (count_before, bytes_before) = snapshot();

    let result = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        proof.as_ref(),
    );

    let (count_after, bytes_after) = snapshot();

    // Keep `result` alive/used so the call is not optimized away and so a
    // panic path inside `allocate` (none expected) would still be visible.
    std::hint::black_box(&result);

    let allocs = count_after - count_before;
    let bytes = bytes_after - bytes_before;

    assert_eq!(
        allocs, 0,
        "allocator::allocate() performed {allocs} heap allocation(s) ({bytes} bytes) — \
         authoritative root must be allocation-free per AGENTS.md Sec 3"
    );
}
