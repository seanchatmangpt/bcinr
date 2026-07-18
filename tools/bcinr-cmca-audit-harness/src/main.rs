//! Object-code audit harness for bcinr-cmca's authoritative allocation root.
//!
//! This binary exists solely so the `bcinr-cmca::allocator::allocate` authoritative root
//! (and its direct callees) can be inspected as *linked executable machine code* rather
//! than as a raw `.rlib` archive member. A prior audit attempt found the rustc rlib
//! member format undecodable with otool-classic on this machine; a real linked
//! executable does not have that problem.
//!
//! `main()` calls `allocate()` exactly once with fixed sample inputs (mirroring the
//! crate's own doctest fixture in `src/allocator.rs`), then folds the result into a
//! checksum that is printed to stdout. The print is load-bearing: it is what prevents
//! the optimizer from treating the whole computation as dead code and eliding it.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

fn main() {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t: u32 = 0;
    let mut prev_mode: u32 = 0;
    let parent = [-1i32; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let proof = AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        CertifiedLearning::admit_learning(),
    );

    let outcome = allocate(
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

    // Fold the outcome into a single checksum. This is a genuine data dependency on the
    // full result (candidate array + numeric faults + refusals), so the optimizer cannot
    // discard the `allocate()` call as dead code, and cannot constant-fold it away either
    // (CERTIFICATE_DIGEST and the registries are opaque `extern`-linked statics from the
    // crate's own generated module, not literals visible to this binary's optimizer pass
    // in a way that would let it precompute the whole call at compile time across the
    // crate boundary in a release build... note: LTO may still do so; see report).
    let mut checksum: u64 = 0;
    for c in outcome.candidate() {
        checksum = checksum.wrapping_add(c.value_bits() as u64);
        checksum = checksum.rotate_left(7);
    }
    checksum ^= outcome.numeric_faults().bits() as u64;
    checksum = checksum.wrapping_add(outcome.refusals().bits() as u64);
    checksum ^= last_switch_t as u64;
    checksum ^= (prev_mode as u64) << 32;

    println!(
        "bcinr-cmca-audit-harness checksum={checksum:#018x} refused={}",
        outcome.is_refused()
    );
}
