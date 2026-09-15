//! Production replay receipt: the allocator's Fortran-grade determinism
//! contract, asserted at the bit level for repeated identical invocations.
//!
//! The single-kernel law ("hosts obtain the answer from the certified
//! kernel; transport choice cannot alter it") is only meaningful if the
//! kernel itself is replayable: the same admitted inputs must produce
//! bit-identical allocations AND bit-identical post-call state, every
//! time, in the same process. This is the receipt a Fortune-5 audit trail
//! points at when it claims a decision can be re-derived later.
//!
//! Distinct from `tests/differential.rs` (fixed-point vs f64 oracle,
//! within tolerance) and `tests/falsification_adversarial.rs` (loop-count
//! independence): this file asserts EXACT bit identity of two runs from
//! cloned identical state.

#![cfg(feature = "std")]

use bcinr_cmca::allocator::allocate;
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

/// Non-degenerate inputs: a depth-2 tree (internal nodes 0,1,2), the same
/// differentiated positive payoffs as `single_lens_allocation`'s MWU test,
/// and a zeta inside the admission envelope, so the MWU weight update
/// actually fires and post-call state is meaningfully non-initial.
fn run_once() -> (
    [NonNegativeFixed; N],
    [[NonNegativeFixed; 2 * Q]; N],
    u32,
    u32,
) {
    let parent: [i32; N] = {
        let mut p = [-1i32; N];
        p[1] = 0;
        p[2] = 0;
        p[3] = 0;
        p[4] = 1;
        p[5] = 1;
        p[6] = 2;
        p[7] = 2;
        p
    };
    let mut payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    for (v, row) in payoffs.iter_mut().enumerate() {
        for (e, slot) in row.iter_mut().enumerate() {
            *slot = NonNegativeFixed::from_num((v * (2 * Q) + e + 1) as u32);
        }
    }

    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let allocation = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        NonNegativeFixed::from_bits(328), // zeta ~0.005, inside ZETA_W_MAX
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        7,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        None,
    )
    .expect("identical admitted inputs must never be refused");

    (allocation, weights, last_switch_t, prev_mode)
}

#[test]
fn repeated_identical_invocations_are_bit_identical_allocations() {
    let (alloc_a, _, _, _) = run_once();
    let (alloc_b, _, _, _) = run_once();
    for i in 0..N {
        assert_eq!(
            alloc_a[i].to_bits(),
            alloc_b[i].to_bits(),
            "object {i}: allocation differs across identical invocations -- \
             the replay contract is broken"
        );
    }
}

#[test]
fn repeated_identical_invocations_leave_bit_identical_state() {
    let (_, weights_a, switch_a, mode_a) = run_once();
    let (_, weights_b, switch_b, mode_b) = run_once();
    for v in 0..N {
        for e in 0..(2 * Q) {
            assert_eq!(
                weights_a[v][e].to_bits(),
                weights_b[v][e].to_bits(),
                "weights[{v}][{e}] differ across identical invocations -- \
                 post-call MWU state is not replayable"
            );
        }
    }
    assert_eq!(
        (switch_a, mode_a),
        (switch_b, mode_b),
        "dwell/mode bookkeeping differs across identical invocations"
    );
}

#[test]
fn a_third_replay_after_recloning_still_matches_the_first() {
    // The audit shape: derive once, seal the answer, re-derive later from
    // a freshly initialized state -- the sealed answer must still hold.
    let (first, _, _, _) = run_once();
    let (third, _, _, _) = run_once();
    assert_eq!(
        first.map(|f| f.to_bits()),
        third.map(|f| f.to_bits()),
        "a later replay from re-initialized state diverged from the sealed \
         first derivation"
    );
}
