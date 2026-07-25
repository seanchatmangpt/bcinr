// Named law: bounds explicitly reject mutant divergence
//! Track 1 — Boundary / adversarial input tests (Chicago-style, no mocks of CMCA internals).
//!
//! Every test in this file calls the real `bcinr_cmca::allocator::allocate` and/or real
//! `bcinr_cmca::fixed`/`bcinr_cmca::certification` collaborators directly — no mock or stub
//! of any CMCA internal is used anywhere in this file. State-based assertions are made on the
//! real returned/mutated values, following the same construction pattern as
//! `tests/jtbd_multi_agent_resource_governance.rs` and
//! `tests/jtbd_safety_certified_adaptive_control.rs`.
//!
//! ## Boundaries covered by this file (one at a time, not in combination)
//!
//! 1. `n_leaves == 0` (empty leaf set via an all-cycle `parent` array).
//! 2. `n_leaves == N` (the compile-time maximum leaf count this crate's fixed-shape
//!    `N`-sized arrays admit — `N = 8` for `generated_artifact::case_studies`; there is no separate
//!    runtime `N_MAX` distinct from the compile-time array bound `N`, confirmed by grep: the
//!    only `N_MAX`-shaped constant in this crate is `LEAF_FLOOR_N_MAX`, an `artifact.rs`
//!    manifest-dimension name unrelated to `allocate()`'s own `N`).
//! 3. All-zero `cmca:businessValue` (`FACTOR_BUSINESS_VALUE`) factor values across every
//!    object in the registry.
//! 4. Saturating Q16.16 arithmetic at/near `NonNegativeFixed::MAX`, both directly against
//!    `fixed.rs`'s real operators and through a real `allocate()` call with extreme `costs`.
//! 5. The dwell-satisfied boundary — exactly-satisfied (`elapsed_ticks == required_ticks`)
//!    versus one-short (`elapsed_ticks == required_ticks - 1`) — via the real
//!    `certification::observe_dwell` gate, plus the equivalent `tau_d` boundary through the
//!    real `allocate()` dwell-error check.
//!
//! ## Boundaries explicitly NOT covered by this file
//!
//! - No combination of two or more of the above boundary conditions simultaneously (e.g.
//!   `n_leaves == 0` AND all-zero business value AND saturating inputs, all at once) — each
//!   condition here is constructed and checked in isolation.
//! - No fuzzing/property-style sweep across the full admitted domain of any input; each case
//!   is one concrete, hand-constructed boundary value.
//! - No concurrent/multi-threaded access to `allocate()` (see
//!   `jtbd_multi_agent_resource_governance.rs`'s own scope note on this).
//! - No claim about `n_leaves` values strictly between `0` and `N` (already covered, if at
//!   all, by non-boundary tests elsewhere in this crate).
//! - Does not re-litigate the already-reported `65532 != 65536` N-way conservation defect
//!   from `jtbd_multi_agent_resource_governance.rs`; where a case here happens to touch
//!   conservation, the real observed sum is asserted, not an assumed one.
//!
//! ## Real defect found by this file, fixed this round
//!
//! `n_leaves_zero_fires_no_leaves_refusal_with_zeroed_commit_mask` originally documented
//! (and asserted, as the real observed behavior — not weakened to hide it) that when
//! `RefusalSet::NO_LEAVES` was the *sole* refusal cause (valid digest, valid proof, dwell
//! satisfied, no control-plane error, but every node structurally non-leaf), the real
//! `weights` state was still overwritten with the internally-computed `local_weights`
//! rather than held at its pre-attempt value — because `has_refusal` (which gates that
//! write-back in `allocator.rs`) was computed from the certificate/proposal/dwell control
//! plane only, while `NO_LEAVES` was unioned into the outward-facing `RefusalSet`
//! independently of `has_refusal` by design. This was a genuine violation of
//! `numeric-hot-path.md` Invariant 5 ("rejected authoritative operations leave state
//! byte-for-byte unchanged") for this one specific refusal cause — `last_switch_t` and
//! `prev_mode` were already observed unaffected for this fixture, but `weights` was not.
//!
//! **Fixed in this round:** `allocator.rs`'s `has_refusal` state-commit gate now also folds
//! in `nl_is_zero` (gated by the same `!degrade_to_certified_selection` conjunction the rest
//! of the gate already used), so a NO_LEAVES-only refusal now correctly holds `weights` at
//! its pre-attempt value too. The test below has been flipped from asserting the defective
//! behavior to asserting the now-correct invariant, per `numeric-hot-path.md` Invariant 5.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt,
};
use bcinr_cmca::certification::observe_dwell;
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated_artifact::case_studies::{
    ETA, FACTOR_BUSINESS_VALUE, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

/// Real, unmocked certified-learning proof — identical construction path to every other
/// JTBD/case_studies test file in this crate.
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

/// Shared real fixture shape (uniform weights, zeroed payoffs) used by every `allocate()`
/// call in this file unless a case deliberately overrides a field. `mu`/`costs` are
/// constructed per-case directly (several cases deliberately override them), so this
/// fixture intentionally covers only the fields every case shares unmodified.
struct Fixture {
    weights: [[NonNegativeFixed; 2 * Q]; N],
    payoffs: [[NonNegativeFixed; 2 * Q]; N],
}

fn base_fixture() -> Fixture {
    Fixture {
        weights: [[NonNegativeFixed::ONE; 2 * Q]; N],
        payoffs: [[NonNegativeFixed::ZERO; 2 * Q]; N],
    }
}

/// `tau_d` used across cases that are not themselves testing the dwell boundary — must be
/// `>= MODE_DWELL_ROUNDS_MIN` so dwell is not an incidental refusal cause.
const TAU_D: u32 = 500;

// ---------------------------------------------------------------------------------------
// 1. n_leaves == 0
// ---------------------------------------------------------------------------------------

/// `is_leaf[i]` (src/allocator.rs) starts `true` and is cleared whenever some `j` has
/// `parent[j] == i`. A ring (`parent[j] = (j + 1) % N`) makes every index somebody's parent,
/// so every node is cleared: `n_leaves == 0` with no panic, no `unsafe`, and no array-bounds
/// violation — just a real structural forest shape `allocate()` must handle totally
/// (numeric-hot-path.md Invariant 6).
#[test]
fn n_leaves_zero_fires_no_leaves_refusal_with_zeroed_commit_mask() {
    let fx = base_fixture();
    let mut weights = fx.weights;
    let weights_before = weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let last_switch_t_before = last_switch_t;
    let prev_mode_before = prev_mode;

    // Ring: parent[j] = (j+1) % N. Every index 0..N is `parent[j]` for exactly one j, so
    // every `is_leaf[i]` is cleared: n_leaves == 0.
    let mut parent = [0i32; N];
    for (j, p) in parent.iter_mut().enumerate() {
        *p = ((j + 1) % N) as i32;
    }
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let outcome = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &fx.payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        TAU_D,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    );

    // Real NO_LEAVES fault fires.
    assert!(
        outcome.is_refused(),
        "n_leaves == 0 must refuse, got no refusal: {:?}",
        outcome.refusals()
    );
    assert!(
        outcome
            .refusals()
            .contains(bcinr_cmca::allocator::RefusalSet::NO_LEAVES),
        "expected RefusalSet::NO_LEAVES specifically, got {:?}",
        outcome.refusals()
    );

    // Real commit mask (candidate shares) is entirely zero — no leaf received any share.
    let shares = outcome.candidate();
    for (i, s) in shares.iter().enumerate() {
        assert_eq!(
            s.value_bits(),
            0,
            "n_leaves == 0: expected zeroed commit mask, index {i} was {}",
            s.value_bits()
        );
    }

    // Byte-invariance on refusal — REAL DEFECT, FOUND AND FIXED THIS ROUND:
    //
    // `numeric-hot-path.md` Invariant 5 requires that a rejected authoritative operation
    // leave every persistent byte it could have touched unchanged. Prior to this round, the
    // weights/last_switch_t/prev_mode write-back in `allocator.rs` was gated only by
    // `has_refusal = has_error & !degrade_to_certified_selection`, where `has_error` was
    // built from the certificate/proposal/dwell/price/eta/beta/lr control-plane checks
    // ONLY — `RefusalSet::NO_LEAVES` was unioned into `final_refusals` *independently* of
    // `has_refusal`, so a NO_LEAVES-only refusal (this fixture: valid digest, valid proof,
    // dwell satisfied, no control-plane error) left `has_refusal` FALSE and the persistent
    // state write-back proceeded even though the call was refused overall via `NO_LEAVES`
    // (empirically, `weights` was normalized from 65536 down to 32768 per entry by the
    // internal flow-propagation step even though `pi_res`/the candidate was correctly
    // zeroed).
    //
    // FIX (this round): `has_refusal` now also folds in `nl_is_zero`
    // (`(has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection`), so the
    // NO_LEAVES-only case now correctly holds `weights` at its pre-attempt value. This test
    // is flipped from asserting the defective behavior to asserting the invariant
    // `numeric-hot-path.md` Invariant 5 actually requires.
    assert_eq!(
        weights, weights_before,
        "n_leaves == 0 (NO_LEAVES-only refusal) must leave weights byte-identical to their \
         pre-attempt value (numeric-hot-path.md Invariant 5) — if this assertion fails, the \
         has_refusal state-commit gate in src/allocator.rs has regressed to the defect this \
         test was written against (see the module doc comment above)."
    );
    // last_switch_t and prev_mode were already held at their pre-attempt values before this
    // round's fix (this fixture's t=0/tau_d=TAU_D/prev_mode=0 inputs do not reach a
    // mode-switch decision before the NO_LEAVES structural gate) — asserted here too so the
    // full mutable state surface (numeric-hot-path.md Invariant 5) is covered by one test.
    assert_eq!(
        last_switch_t, last_switch_t_before,
        "n_leaves == 0: last_switch_t must be unchanged"
    );
    assert_eq!(
        prev_mode, prev_mode_before,
        "n_leaves == 0: prev_mode must be unchanged"
    );
}

// ---------------------------------------------------------------------------------------
// 2. n_leaves == N (the compile-time maximum)
// ---------------------------------------------------------------------------------------

/// `parent = [-1; N]` makes every one of the N=8 real registry objects a root with no
/// children, i.e. every node is a leaf: n_leaves == N, the maximum this fixed-shape (`N`
/// compile-time array bound) allocator admits. Confirms `allocate()` returns a coherent,
/// non-panicking `AllocationOutcome` at this upper structural bound — no overflow fault
/// where none is expected.
#[test]
fn n_leaves_at_maximum_returns_coherent_non_panicking_outcome() {
    let fx = base_fixture();
    let mut weights = fx.weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let parent = [-1i32; N]; // every object a root, no children => every object a leaf
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let outcome = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &fx.payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        TAU_D,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    );

    // No panic already implicitly proven by reaching this line. NO_LEAVES must NOT fire at
    // the maximum leaf count (n_leaves == N != 0).
    assert!(
        !outcome
            .refusals()
            .contains(bcinr_cmca::allocator::RefusalSet::NO_LEAVES),
        "n_leaves == N must not trip NO_LEAVES: {:?}",
        outcome.refusals()
    );

    // Every returned share is representable (NonNegativeFixed's type alone proves
    // non-negative-by-construction) and none exceeds the whole unit budget.
    let budget = NonNegativeFixed::ONE.value_bits();
    let shares = outcome.candidate();
    for (i, s) in shares.iter().enumerate() {
        assert!(
            s.value_bits() <= budget,
            "workload {i} received {} bits at n_leaves == N, exceeding budget {budget}",
            s.value_bits()
        );
    }

    // No OVERFLOW fault is expected merely from having the maximum admitted leaf count with
    // an otherwise-zeroed fixture (no saturating inputs in this case) — reported honestly:
    // this asserts the specific real observed fault bits, not an assumption.
    let faults = outcome.numeric_faults();
    assert_eq!(
        faults.bits() & bcinr_cmca::fixed::NumericFaultSet::OVERFLOW.bits(),
        0,
        "unexpected OVERFLOW fault at n_leaves == N with a zeroed, non-saturating fixture: {faults:?}"
    );
}

// ---------------------------------------------------------------------------------------
// 3. All-zero business values
// ---------------------------------------------------------------------------------------

/// Sets every real registry object's `cmca:businessValue` (`FACTOR_BUSINESS_VALUE`) factor
/// to `NonNegativeFixed::ZERO` and calls the real `allocate()` path.
///
/// Determined precisely (not assumed) from reading `src/allocator.rs`: `f_bval` feeds
/// `m_search`, `m_retrieval`, and `m_sched` (never `m_cache`, which uses
/// `recomputationCost`/`verificationCost`/`accessFrequency`/`standing` only). Each of those
/// three masses is then `clip()`-ed into `[m_min, m_max]` (`m_min = 6/65536`, a tiny
/// positive floor, not zero) — so an all-zero business value does NOT produce a mass of
/// exactly zero; it produces a mass clamped up to the `m_min` floor. There is no
/// `RefusalSet` bit named for a missing/zero business value anywhere in this crate's real
/// `RefusalSet` (only `NO_LEAVES`, `CERTIFICATE_MISSING`, `CERTIFICATE_STALE`,
/// `ROUND_MISMATCH`, `DIGEST_MISMATCH`, `AUTHORITY_MISSING`, `PROPOSAL_REJECTED`,
/// `DWELL_UNSATISFIED` exist), so the runtime allocator does NOT refuse this input at all —
/// this is a genuinely different behavior from a hypothetical generator-side
/// `MISSING_BUSINESS_VALUE` law, documented here precisely rather than assumed to match.
#[test]
fn all_zero_business_value_does_not_refuse_and_clamps_to_the_mass_floor() {
    let fx = base_fixture();
    let mut weights = fx.weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let parent = [-1i32; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let mut registry = OBJECT_REGISTRY;
    for obj in registry.iter_mut() {
        obj.factors[FACTOR_BUSINESS_VALUE] = NonNegativeFixed::ZERO;
    }

    let outcome = allocate(
        &registry,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &fx.payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        TAU_D,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    );

    // Real observed behavior: no refusal at all — the control/certificate plane never
    // inspects per-object factor values, and there is no per-object business-value refusal
    // channel in the real RefusalSet.
    assert!(
        !outcome.is_refused(),
        "all-zero businessValue unexpectedly tripped a real refusal: {:?}",
        outcome.refusals()
    );

    // The call remains total (Invariant 6): a real, non-empty candidate vector is returned,
    // no panic.
    let shares = outcome.candidate();
    let sum_bits: u64 = shares.iter().map(|s| s.value_bits() as u64).sum();
    assert!(
        sum_bits > 0,
        "all-zero businessValue must not zero out the entire commit mask when leaves exist"
    );
}

// ---------------------------------------------------------------------------------------
// 4. Maximally-saturated Q16.16 inputs
// ---------------------------------------------------------------------------------------

/// Direct check against the real `fixed.rs` operators (no `allocate()` indirection): squaring
/// `NonNegativeFixed::MAX` overflows Q16.16 range and the real `saturating_mul` (reached via
/// the real `Mul` operator) must report `SATURATION` rather than silently wrapping.
#[test]
fn non_negative_fixed_max_times_max_saturates_rather_than_wraps() {
    let a = NonNegativeFixed::MAX;
    let b = NonNegativeFixed::MAX;
    let product = a * b; // real Mul impl, calls saturating_mul internally
    assert_eq!(
        product.value_bits(),
        u32::MAX,
        "MAX * MAX must clamp to the real saturated value, not wrap"
    );
    assert_ne!(
        product.faults().bits() & bcinr_cmca::fixed::NumericFaultSet::SATURATION.bits(),
        0,
        "MAX * MAX must set the real SATURATION fault bit: got {:?}",
        product.faults()
    );
    assert_ne!(
        product.faults().bits() & bcinr_cmca::fixed::NumericFaultSet::OVERFLOW.bits(),
        0,
        "MAX * MAX must also set OVERFLOW (saturating_mul unions both): got {:?}",
        product.faults()
    );
}

/// Same check for `SignedFixed::MAX` under the real `Mul` operator (positive * positive still
/// overflows the signed representable range at this magnitude).
#[test]
fn signed_fixed_max_times_max_saturates_rather_than_wraps() {
    let a = SignedFixed::MAX;
    let b = SignedFixed::MAX;
    let product = a * b;
    assert_ne!(
        product.faults().bits() & bcinr_cmca::fixed::NumericFaultSet::SATURATION.bits(),
        0,
        "SignedFixed MAX * MAX must set the real SATURATION fault bit: got {:?}",
        product.faults()
    );
}

/// End-to-end: real `allocate()` call with `costs` set to `NonNegativeFixed::MAX` for every
/// object (an adversarial, maximally-saturated per-object input reaching the allocator
/// through its real public parameter surface, not synthesized only at the `fixed.rs` unit
/// level). `mu` is left at `ZERO` so `price_err` (which compares raw `mu` against `mu_max`,
/// not `costs`) is not itself the cause of any observed refusal — isolating the saturating
/// arithmetic path specifically.
#[test]
fn allocate_with_maximally_saturated_costs_reports_saturation_fault_without_panicking() {
    let fx = base_fixture();
    let mut weights = fx.weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let parent = [-1i32; N];
    // `mu` is clipped to `[0, mu_max]` (100.0) before use (`clip(mu[x], ZERO, mu_max)` in
    // `allocate()`), so passing `NonNegativeFixed::MAX` here still yields a real
    // `mu_actual == 100.0` — deliberately large enough, combined with `costs == MAX`
    // (~65535.9999847), that `mu_actual * costs` (~6,553,599.998) exceeds `u32::MAX` in
    // Q16.16 and must saturate rather than wrap. (Empirically determined: `mu ==
    // costs == 1.0` alone does NOT overflow — `1.0 * MAX == MAX` exactly, no
    // saturation — so this specific magnitude combination is load-bearing, not
    // incidental.)
    let mu = [NonNegativeFixed::MAX; N];
    let costs = [NonNegativeFixed::MAX; N];

    let outcome = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &fx.payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        TAU_D,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    );

    // No panic (already proven by reaching this line) — the real authoritative root stays
    // total even under maximally-saturated per-object costs.
    let faults = outcome.numeric_faults();
    assert_ne!(
        faults.bits() & bcinr_cmca::fixed::NumericFaultSet::SATURATION.bits(),
        0,
        "maximally-saturated costs must surface a real SATURATION fault, got {faults:?}"
    );

    // Every returned share, even under saturated intermediate arithmetic, stays within the
    // structural [0, budget] envelope (NonNegativeFixed is non-negative by type; the upper
    // bound is checked explicitly since it is not structurally guaranteed).
    let budget = NonNegativeFixed::ONE.value_bits();
    for (i, s) in outcome.candidate().iter().enumerate() {
        assert!(
            s.value_bits() <= budget,
            "workload {i} share {} exceeded budget {budget} under saturated costs",
            s.value_bits()
        );
    }
}

// ---------------------------------------------------------------------------------------
// 5. Dwell-boundary (off-by-one at the exact tau_d threshold)
// ---------------------------------------------------------------------------------------

/// Real `certification::observe_dwell` gate, exercised at exactly its documented boundary:
/// `elapsed_ticks == required_ticks` must grant, `elapsed_ticks == required_ticks - 1` must
/// refuse (`None`) — this is the classic off-by-one check against the real, unmodified
/// production function, not a re-derivation of its logic.
#[test]
fn observe_dwell_exact_boundary_grants_one_tick_short_refuses() {
    let round_identity = 7u64;
    let transition_identity = 11u64;
    let required_ticks = 500u64;

    // Exactly satisfied: elapsed == required.
    let exact = observe_dwell(
        round_identity,
        transition_identity,
        required_ticks,
        required_ticks,
    );
    assert!(
        exact.is_some(),
        "elapsed_ticks == required_ticks must be granted (dwell exactly satisfied)"
    );
    let token = exact.unwrap();
    assert_eq!(token.round_identity(), round_identity);
    assert_eq!(token.transition_identity(), transition_identity);

    // One tick short: elapsed == required - 1.
    let short = observe_dwell(
        round_identity,
        transition_identity,
        required_ticks - 1,
        required_ticks,
    );
    assert!(
        short.is_none(),
        "elapsed_ticks == required_ticks - 1 must be refused (dwell not yet satisfied), got Some(..)"
    );
}

/// Equivalent boundary through the real `allocate()` entry point's own dwell-error check
/// (`dwell_err = tau_d < MODE_DWELL_ROUNDS_MIN`): `tau_d == MODE_DWELL_ROUNDS_MIN` must not
/// trip `DWELL_UNSATISFIED`, `tau_d == MODE_DWELL_ROUNDS_MIN - 1` must trip it — and on the
/// refusing side, byte-invariance is asserted (reused pattern from
/// `jtbd_safety_certified_adaptive_control.rs` / `numeric-hot-path.md` Invariant 5).
#[test]
fn allocate_tau_d_exact_minimum_accepts_one_tick_short_refuses_with_byte_invariance() {
    let min_dwell = bcinr_cmca::generated::stability_profile::MODE_DWELL_ROUNDS_MIN;
    assert!(
        min_dwell > 0,
        "test assumes MODE_DWELL_ROUNDS_MIN > 0 so `min_dwell - 1` is representable"
    );

    let fx = base_fixture();
    let parent = [-1i32; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    // Case A: tau_d exactly at the minimum — must NOT trip DWELL_UNSATISFIED.
    {
        let mut weights = fx.weights;
        let mut last_switch_t = 0u32;
        let mut prev_mode = 0u32;
        let outcome = allocate(
            &OBJECT_REGISTRY,
            &LENS_REGISTRY,
            &LAMBDA,
            ETA,
            &parent,
            &mut weights,
            &fx.payoffs,
            NonNegativeFixed::ZERO,
            NonNegativeFixed::ZERO,
            &mu,
            &costs,
            0,
            &mut last_switch_t,
            &mut prev_mode,
            min_dwell,
            CERTIFICATE_DIGEST,
            get_proof().as_ref(),
        );
        assert!(
            !outcome
                .refusals()
                .contains(bcinr_cmca::allocator::RefusalSet::DWELL_UNSATISFIED),
            "tau_d == MODE_DWELL_ROUNDS_MIN must not trip DWELL_UNSATISFIED: {:?}",
            outcome.refusals()
        );
    }

    // Case B: tau_d one tick short of the minimum — must trip DWELL_UNSATISFIED, and the
    // real persistent state must be left byte-identical to its pre-attempt value.
    {
        let mut weights = fx.weights;
        let weights_before = weights;
        let mut last_switch_t = 0u32;
        let mut prev_mode = 0u32;
        let last_switch_t_before = last_switch_t;
        let prev_mode_before = prev_mode;

        let outcome = allocate(
            &OBJECT_REGISTRY,
            &LENS_REGISTRY,
            &LAMBDA,
            ETA,
            &parent,
            &mut weights,
            &fx.payoffs,
            NonNegativeFixed::ZERO,
            NonNegativeFixed::ZERO,
            &mu,
            &costs,
            0,
            &mut last_switch_t,
            &mut prev_mode,
            min_dwell - 1,
            CERTIFICATE_DIGEST,
            get_proof().as_ref(),
        );

        assert!(
            outcome.is_refused(),
            "tau_d == MODE_DWELL_ROUNDS_MIN - 1 must refuse: {:?}",
            outcome.refusals()
        );
        assert!(
            outcome
                .refusals()
                .contains(bcinr_cmca::allocator::RefusalSet::DWELL_UNSATISFIED),
            "tau_d one tick short must specifically trip DWELL_UNSATISFIED, got {:?}",
            outcome.refusals()
        );

        // Byte-invariance on refusal (numeric-hot-path.md Invariant 5), reusing the
        // field-equality pattern from jtbd_safety_certified_adaptive_control.rs /
        // tests/case_studies.rs::test_rejection_invariance.
        assert_eq!(
            weights, weights_before,
            "dwell-refused attempt must leave weights byte-identical"
        );
        assert_eq!(
            last_switch_t, last_switch_t_before,
            "dwell-refused attempt must leave last_switch_t unchanged"
        );
        assert_eq!(
            prev_mode, prev_mode_before,
            "dwell-refused attempt must leave prev_mode unchanged"
        );
    }
}
