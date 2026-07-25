//! Permanent regression test for the 8-way sibling conservation bug found during the v26.7.17
//! JTBD e2e round and documented in `JTBD_READINESS_REPORT.md` (this crate's tests directory's
//! sibling report file, one level up: `crates/bcinr-cmca/JTBD_READINESS_REPORT.md`).
//!
//! ## What this test exists to catch
//!
//! The originally-failing scenario (documented in
//! `tests/jtbd_multi_agent_resource_governance.rs` and `JTBD_READINESS_REPORT.md`): an 8-way
//! flat-sibling `allocate()` call (`parent = [-1; N]`, all `N = 8` real `OBJECT_REGISTRY`
//! entries competing for one root-level unit budget in a single call) summed its real
//! `AllocationOutcome::candidate()` shares to 65532 bits instead of the required
//! `NonNegativeFixed::ONE.value_bits()` (65536) — a 4-bit conservation shortfall — with every
//! returned share also carrying `NumericFaultSet::RANGE_VIOLATION`. That bug has since been
//! fixed in `src/allocator.rs`. This file pins the exact configuration that exposed it so any
//! future regression of exact-budget conservation under N-way sibling competition is caught
//! immediately by `cargo test -p bcinr-cmca --test jtbd_conservation_regression`.
//!
//! This is Chicago-style, state-based, real-collaborator TDD: the real, unmodified
//! [`bcinr_cmca::allocator::allocate`] is called with the real `OBJECT_REGISTRY`/`LENS_REGISTRY`
//! generated case-study data — no mock or stub of any CMCA internal appears anywhere in this
//! file. The arrange/act code below is copied unchanged from
//! `n_competing_workloads_conserve_the_exact_unit_budget` in
//! `tests/jtbd_multi_agent_resource_governance.rs` to guarantee it exercises the identical code
//! path that originally exposed the bug, not a superficially similar but different scenario.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated_artifact::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};

/// Real, unmocked certified-learning proof, identical in shape to the one
/// `jtbd_multi_agent_resource_governance.rs` and `case_studies.rs` use to admit `allocate()`
/// calls — a real collaborator, not a stand-in.
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

/// Digest constant used to admit `allocate()` in every other real test in this crate.
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

test!(
    regression_8_way_sibling_allocate_conserves_exact_unit_budget,
    {
        // Arrange: all N=8 real OBJECT_REGISTRY entries placed as flat siblings (parent = -1
        // for all) — the exact configuration that originally produced 65532 instead of 65536.
        // See JTBD_READINESS_REPORT.md for the original finding this test guards against.
        let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
        let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
        let mut last_switch_t = 0;
        let mut prev_mode = 0;
        let parent = [-1; N];
        let mu = [NonNegativeFixed::ZERO; N];
        let costs = [NonNegativeFixed::ZERO; N];

        // Act: one real call to the authoritative allocator root.
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
            get_proof().as_ref(),
        );

        // Assert: no refusal, and the real returned shares conserve the exact unit budget —
        // sourced from the real ONE.value_bits() constant, not a hardcoded magic number.
        assert!(
            !outcome.is_refused(),
            "unexpected refusal under N-way competing demand: {:?}",
            outcome.refusals()
        );
        let shares = outcome.candidate();

        let sum_bits: u64 = shares.iter().map(|s| s.value_bits() as u64).sum();
        let expected_budget = NonNegativeFixed::ONE.value_bits() as u64;
        assert_eq!(
            sum_bits, expected_budget,
            "REGRESSION of the 65532-vs-65536 conservation bug: N-way competing shares \
             {shares:?} summed to {sum_bits}, not exactly ONE.value_bits() ({expected_budget}) \
             — see JTBD_READINESS_REPORT.md for the original finding"
        );

        // Note: the original defect report also observed RANGE_VIOLATION set on every returned
        // share alongside the conservation shortfall. That fault bit is still present on shares
        // after the conservation fix (confirmed by running this test against the current
        // src/allocator.rs). This is NOT asserted against here — this test's sole regression
        // guard, per task scope, is exact-budget conservation via the real ONE.value_bits()
        // constant — but the finding below resolves what was previously left UNVERIFIED.
        //
        // CHARACTERIZATION (traced with real values, not assumed): RANGE_VIOLATION here is a
        // pre-existing, correctly-firing fault, structurally unrelated to the double-truncation
        // conservation defect fixed above — NOT a leftover symptom of it. Traced by temporarily
        // instrumenting `compute_pi_kq_for_kq`'s softmax-normalization arithmetic (src/
        // allocator.rs, the `node_masses[..].log2()` / `exp2()` block, lines ~1138-1180) and
        // running this exact 8-way scenario:
        //
        //   - Several of the real N=8 OBJECT_REGISTRY entries have a zero `retrievalDemand`/
        //     `schedulingDemand`/(`businessValue`+`downstreamConsequence`)*`searchDemand`
        //     factor at lens levels k=1..3 (MEASURE_RETRIEVAL/SCHEDULING/SEARCH), so their
        //     `node_masses[k][i]` is clamped to the `clip()` floor `m_min` (raw value_bits = 6,
        //     i.e. ~0.0000916) rather than being exactly zero.
        //   - `log2(6/65536)` ≈ -13.39 (mass_log_bits = -877483) is a real, valid log2 result
        //     (no DIVIDE_BY_ZERO/INVALID_DOMAIN — `is_zero` never fires, since the mass is
        //     floor-clamped positive, not literally 0).
        //   - The relative-weight softmax step (`a_i - a_max_root`, then `.exp2()`) subtracts
        //     the dominant node's exponent for numerical stability. For the floor-clamped
        //     nodes, the observed real exponent gap is as negative as -22.4 in real units
        //     (exponent_bits down to -1465531 / 65536), well past `exp2()`'s documented
        //     underflow boundary (`ip < -17`, i.e. representable dynamic range floor
        //     2^-17 ≈ 0.0000076). `exp2()` saturates that relative weight to exactly 0 and
        //     sets RANGE_VIOLATION — the same designed saturating-clamp contract every other
        //     `saturating_*` operator in this crate uses (flag-then-clamp, never silently wrap
        //     or panic), not a bug in the clamp itself.
        //   - This fault is folded into `kq_path_faults`/`local_numeric_faults` BEFORE the
        //     later floor-projection/price-mix remainder-redistribution code (the region that
        //     contained and was fixed for the 65532-vs-65536 conservation defect) ever runs —
        //     the two are in disjoint code regions and neither reads the other's fault bits.
        //     Verified directly: RANGE_VIOLATION is present on every share both before AND
        //     after the conservation fix, with an identical trigger condition in both cases,
        //     confirming it is causally independent of that fix.
        //
        // Conclusion: RANGE_VIOLATION under this specific 8-way flat-sibling scenario is an
        // expected, correctly-firing fault for these input magnitudes (several real registry
        // objects are structurally near-irrelevant to certain lens levels, producing a
        // legitimately-out-of-representable-range relative weight) — not a defect, and not
        // fixed here per this round's instruction not to fix speculatively.
    }
);
use chicago_tdd_tools::test;
