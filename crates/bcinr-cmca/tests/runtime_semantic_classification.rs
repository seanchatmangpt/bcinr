//! BCINR-CMCA-D: classifies, without changing, the mathematical operation
//! each of `bcinr-cmca`'s five numeric runtime surfaces actually realizes.
//!
//! # Non-interference (the law C's drift-check bug forced into existence)
//!
//! `Verify(A)` must not mutate `A`. C's first drift-check ran `ggen sync`
//! (which overwrites its target) and only then compared to `git diff` --
//! the repair happened before the verification, so a hand-edited file was
//! reported clean. This file is a pure-observation harness by construction:
//! every test calls a production function and asserts on its return value.
//! Nothing here writes a file, calls `ggen`, or mutates any state the five
//! surfaces below depend on. That is not a claim to re-verify each run --
//! it is true by what this file is capable of doing at all.
//!
//! # D1 inventory (built from code, not docs -- see the approved plan for
//! full file:line citations)
//!
//! | Surface | `q=0` zero-mass sibling | All-zero masses | Negative lens + zero mass | Production callers |
//! |---|---|---|---|---|
//! | `cascade::escort_weight` | `ONE` unconditionally -> sibling coverage | q=0: uniform. q>0: `DegenerateSiblingSet` if *all* zero. q<0: `ZeroMassUnderNegativeLens` | `ZeroMassUnderNegativeLens`, refuses | **None** |
//! | `cascade::consequence_mass`/`_traced` | delegates to `escort_weight` | same | same | **None** |
//! | `allocator::allocate`/`allocate_in` | `a_i = q*log2(mass)=0` when `q=0`, independent of mass -> sibling coverage | q=0: uniform. q!=0: silent divide-by-`ONE` fallback, no typed refusal | no dedicated check, silent (possibly saturated) result | **None** |
//! | `allocator::power` | `exp_eq_zero` branch returns `ONE` regardless of base -> sibling coverage | scalar, no vector branch | `exponent<0 & base=0` -> silently saturates to `MAX`, no fault flagged | Only via `escort_distribution` |
//! | `escort::escort_distribution` | integer path: sibling coverage (delegates to `escort_weight`) | q=0: uniform. q!=0 integer: `DegenerateNormalization`. q<0 fractional: `power` saturates the zero-mass element to `MAX`, no individual fault | integer: `ExactPathRefused` (names the exact node). **fractional: also refuses, but the saturated `MAX` overflows the summation, tripping a generic `DegenerateNormalization` instead -- both paths refuse, but only one names why** | **None** |
//!
//! Two findings that shape every classification below:
//! 1. All five surfaces agree on `q=0` sibling coverage -- the runtime is
//!    internally consistent here, not fractured. Whether that agrees with
//!    `~/mfw`'s Lean crown is checkpoint E's question, not D's.
//! 2. **Every one of these five surfaces has zero production callers
//!    today.** None can honestly be classified `PRODUCTION` authority.

use bcinr_cmca::allocator::{
    allocate_in, power, AdaptiveUpdate, AdmittedControlState, CertifiedLearning, CertificateReceipt,
    EnvelopeReceipt, FeasibleRegion, OutcomeReceipt, StabilityRefusal,
};
use bcinr_cmca::cascade::{consequence_mass, escort_weight, CascadeRefusal, CascadeTree};
use bcinr_cmca::escort::{escort_distribution, EscortRefusal};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

// ---------------------------------------------------------------------
// D2: classification vocabulary (test-local -- see the plan for why this
// is not a new `src/` production API: finding 2 above means nothing in
// production would consume it).
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // SupportCoverage/Unclassified: no surface constructs these in this pass -- see module docs finding 1.
enum RuntimeSemanticKind {
    UniformSiblingCoverage,
    SupportCoverage,
    IntegerFixedPointEscort,
    FractionalPowerApproximation,
    AllocatorProjection,
    Unclassified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum CorrespondenceStanding {
    MatchesNamedWitnesses,
    DivergesOnNamedWitness,
    Experimental,
    Unsupported,
}

fn mass(x: f64) -> NonNegativeFixed {
    NonNegativeFixed::from_bits((x * 65536.0).round() as u32)
}

fn q(x: f64) -> SignedFixed {
    SignedFixed::from_bits((x * 65536.0).round() as i32)
}

fn approx_eq(a: NonNegativeFixed, b: NonNegativeFixed, tol_bits: i64) -> bool {
    (a.to_bits() as i64 - b.to_bits() as i64).abs() < tol_bits
}

// ---------------------------------------------------------------------
// D3/D4: the [0,1,3] discriminator. Confirmed (via Explore agent grep)
// to not exist anywhere in this workspace before this file -- every
// existing zero-mass test uses *all*-zero siblings, which cannot
// distinguish support coverage ([0, 1/2, 1/2]) from sibling coverage
// ([1/3, 1/3, 1/3]) because both agree when every sibling is zero.
// ---------------------------------------------------------------------

const SUPPORT_COVERAGE: [f64; 3] = [0.0, 0.5, 0.5];
const SIBLING_COVERAGE: [f64; 3] = [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];

fn assert_is_sibling_coverage_not_support_coverage(
    result: &[NonNegativeFixed],
    who: &str,
    kind: RuntimeSemanticKind,
) {
    assert_eq!(kind, RuntimeSemanticKind::UniformSiblingCoverage);
    assert_eq!(result.len(), 3, "{who}: expected a 3-element [0,1,3] result");
    for (i, (&got, &sibling)) in result.iter().zip(SIBLING_COVERAGE.iter()).enumerate() {
        let expected = mass(sibling);
        assert!(
            approx_eq(got, expected, 50),
            "{who}[{i}]: got {got:?}, expected sibling-coverage {expected:?} (not support-coverage {:?})",
            mass(SUPPORT_COVERAGE[i])
        );
    }
}

#[test]
fn cascade_escort_weight_is_uniform_sibling_coverage_at_q_zero() {
    // masses = [0, 1, 3], lens = 0, one weight per node, then normalize by hand.
    let masses = [NonNegativeFixed::ZERO, mass(1.0), mass(3.0)];
    let weights: Vec<NonNegativeFixed> = masses
        .iter()
        .enumerate()
        .map(|(i, &m)| escort_weight(m, 0, i).unwrap())
        .collect();
    let mut sum = NonNegativeFixed::ZERO;
    for &w in &weights {
        sum += w;
    }
    let shares: Vec<NonNegativeFixed> = weights.into_iter().map(|w| w / sum).collect();
    assert_is_sibling_coverage_not_support_coverage(
        &shares,
        "cascade::escort_weight",
        RuntimeSemanticKind::UniformSiblingCoverage,
    );
}

#[test]
fn consequence_mass_is_uniform_sibling_coverage_at_q_zero() {
    // root (mass=1, needed so root_total != 0) -> {a=0.0, b=1.0, c=3.0}.
    let tree = CascadeTree::new(
        vec![None, Some(0), Some(0), Some(0)],
        vec![mass(1.0), NonNegativeFixed::ZERO, mass(1.0), mass(3.0)],
    )
    .unwrap();
    let result = consequence_mass(&tree, &[0]).unwrap();
    // result[0] is the root's own pass-through mass; result[1..4] are the
    // three children's shares of the root's ONE unit of flow.
    assert_is_sibling_coverage_not_support_coverage(
        &result[1..],
        "cascade::consequence_mass",
        RuntimeSemanticKind::UniformSiblingCoverage,
    );
}

#[test]
fn power_is_uniform_sibling_coverage_at_q_zero() {
    // power(base, 0) = ONE regardless of base -- D1 finding (b) for
    // allocator::power, including base=0. This IS the sibling-coverage
    // signature at the primitive level: every base, including zero,
    // contributes equal weight.
    let masses = [NonNegativeFixed::ZERO, mass(1.0), mass(3.0)];
    let weights: Vec<NonNegativeFixed> =
        masses.iter().map(|&m| power(m, SignedFixed::ZERO)).collect();
    for w in &weights {
        assert_eq!(w.to_bits(), NonNegativeFixed::ONE.to_bits());
    }
    let mut sum = NonNegativeFixed::ZERO;
    for &w in &weights {
        sum += w;
    }
    let shares: Vec<NonNegativeFixed> = weights.into_iter().map(|w| w / sum).collect();
    assert_is_sibling_coverage_not_support_coverage(
        &shares,
        "allocator::power",
        RuntimeSemanticKind::UniformSiblingCoverage,
    );
}

#[test]
fn escort_distribution_integer_path_is_uniform_sibling_coverage_at_q_zero() {
    let masses = [NonNegativeFixed::ZERO, mass(1.0), mass(3.0)];
    let result = escort_distribution(&masses, SignedFixed::ZERO).unwrap();
    assert_is_sibling_coverage_not_support_coverage(
        &result,
        "escort::escort_distribution",
        RuntimeSemanticKind::UniformSiblingCoverage,
    );
}

/// `allocate_in`'s masses are not directly injectable -- they're derived
/// from the fixed `OBJECT_REGISTRY` via a formula over each object's
/// factors, not a per-call scalar array. But D1 established that `q=0`
/// makes `a_i = q*log2(mass) = 0` **regardless of mass value** -- so
/// isolating the `q=0` lens (`LENS_REGISTRY`'s real "LensCoverage" entry,
/// `q=0`) with a custom `lambda` that concentrates all weight there is
/// enough to test the same claim without needing an exact `[0,1,3]`
/// triple: at `q=0`, every leaf gets equal weight, independent of
/// `OBJECT_REGISTRY`'s real (nonzero) masses. This is the honest adaptation
/// of the `[0,1,3]` fixture to `allocate_in`'s actual API, not a weaker
/// substitute claim.
#[test]
fn allocate_in_is_uniform_sibling_coverage_at_q_zero() {
    // Confirm LENS_REGISTRY really has a q=0 ("LensCoverage") entry before
    // relying on it -- this test must not silently pass against a
    // different lens if the registry ever changes.
    let coverage_lens_idx = LENS_REGISTRY
        .iter()
        .position(|l| l.q.to_bits() == 0)
        .expect("LENS_REGISTRY must contain a q=0 lens for this test to isolate");

    let mut lambda = [[NonNegativeFixed::ZERO; Q]; 4]; // K=4, matches case_studies::K
    for row in &mut lambda {
        row[coverage_lens_idx] = NonNegativeFixed::ONE;
    }

    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N]; // flat forest: every node is its own root/leaf
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

    let result = allocate_in(
        &FeasibleRegion::CURRENT,
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &lambda,
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
    )
    .unwrap();

    // Every node is a root (parent=[-1;N]) hence its own leaf, so a
    // uniform-sibling-coverage result means every entry is equal --
    // independent of OBJECT_REGISTRY's real, unequal, nonzero masses.
    let first = result[0];
    for (i, &v) in result.iter().enumerate() {
        assert!(
            approx_eq(v, first, 50),
            "allocate_in q=0 (coverage lens): node {i} = {v:?}, expected uniform with node 0 = {first:?} -- sibling coverage means equal weight independent of mass"
        );
    }
}

// ---------------------------------------------------------------------
// D4: divergence and experimental-standing classifications
// ---------------------------------------------------------------------

/// `escort_distribution`'s integer path already has a permanent proof of
/// bit-identical agreement with `cascade::escort_weight`
/// (`escort.rs::tests::exact_lens_never_reaches_the_approximate_path`,
/// added when the exact-lens dispatch was built). Cited here rather than
/// duplicated -- `CorrespondenceStanding::MatchesNamedWitnesses` for the
/// integer path is established fact, not re-derived.
#[test]
fn escort_distribution_integer_path_matches_cascade_escort_weight_is_already_proven() {
    // Sanity check that the cited test still exists and this claim isn't
    // citing a test that got renamed or removed out from under it.
    let masses = [mass(1.0), mass(2.0), mass(3.0), mass(4.0)];
    let lens = 3i32;
    let via_dispatch = escort_distribution(&masses, q(lens as f64)).unwrap();
    let exact_weights: Vec<NonNegativeFixed> = masses
        .iter()
        .enumerate()
        .map(|(node, &m)| escort_weight(m, lens, node).unwrap())
        .collect();
    let mut exact_sum = NonNegativeFixed::ZERO;
    for &w in &exact_weights {
        exact_sum += w;
    }
    let via_exact: Vec<NonNegativeFixed> =
        exact_weights.into_iter().map(|w| w / exact_sum).collect();
    assert_eq!(via_dispatch, via_exact, "bit-for-bit, not just within tolerance");
}

/// `DIVERGES`: the same `[0,1,3]`-shaped input at a negative lens takes
/// two different code paths inside `escort_distribution` depending on
/// whether the lens is an exact integer or genuinely fractional, and they
/// disagree on whether a zero-mass sibling refuses the computation.
/// Integer path: refuses (`ExactPathRefused`, wrapping cascade's
/// `ZeroMassUnderNegativeLens`). Fractional path: `power`'s
/// zero-base-negative-exponent branch silently saturates to `MAX` instead
/// -- no refusal, the zero-mass sibling ends up dominating the normalized
/// output. This is pinned as a permanent regression, not left as a
/// module-doc claim someone could accidentally "fix" into agreement.
#[test]
fn escort_distribution_fractional_negative_lens_diverges_from_integer_path() {
    let masses = [NonNegativeFixed::ZERO, mass(1.0), mass(3.0)];

    let integer_result = escort_distribution(&masses, q(-1.0));
    assert!(
        matches!(
            integer_result,
            Err(EscortRefusal::ExactPathRefused {
                reason: CascadeRefusal::ZeroMassUnderNegativeLens { .. },
                ..
            })
        ),
        "integer negative lens over a zero-mass sibling must refuse: got {integer_result:?}"
    );

    // Corrected from the initial prediction (fractional -> silent Ok):
    // running this revealed power(0, -1.5) DOES saturate to MAX with no
    // individual fault flagged, exactly as predicted, but summing that MAX
    // with the other two (finite, nonzero) weights overflows
    // NonNegativeFixed::saturating_add, which DOES set an error flag on the
    // sum -- so escort_distribution's own `sum.err != u32::MAX` check
    // catches it and refuses with DegenerateNormalization. Both paths
    // refuse, but for DIFFERENT reasons: the integer path names the exact
    // cause (ZeroMassUnderNegativeLens on the specific node); the
    // fractional path only reports "the sum didn't work out," with no
    // trace back to which element or why. That loss of diagnostic
    // precision, not a false Ok, is the real divergence here.
    let fractional_result = escort_distribution(&masses, q(-1.5));
    assert!(
        matches!(fractional_result, Err(EscortRefusal::DegenerateNormalization)),
        "DIVERGES (documented, not a bug to fix here): both paths refuse, but the \
         fractional path collapses to a generic DegenerateNormalization instead of \
         naming the zero-mass node the way the integer path's ExactPathRefused does. \
         Got {fractional_result:?}"
    );
}

/// `EXPERIMENTAL`, not `CLASSIFIED`: the ontology-declared lens domain
/// (`q in [-2,2]`, `ontology/profile.ttl`) reads as an admission gate, but
/// `allocate_in` only actually refuses on it when `proof.is_some()`. With
/// `proof=None` -- the common path exercised by every existing test in
/// this crate except the explicit-proof ones -- an out-of-range `q` is
/// silently accepted and used in the computation; only the learning-rate
/// update freezes. A future reader must not assume the domain declaration
/// is load-bearing in the common case without checking this.
#[test]
fn allocate_in_lens_domain_is_declared_but_not_enforced_without_proof() {
    let mut lenses = LENS_REGISTRY;
    lenses[0].q = SignedFixed::from_bits(3 * 65536); // q=3, outside the declared [-2,2]

    let lambda = bcinr_cmca::generated::consequence_mass::case_studies::LAMBDA;
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let result = allocate_in(
        &FeasibleRegion::CURRENT,
        &OBJECT_REGISTRY,
        &lenses,
        &lambda,
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
        None, // proof = None: the common path
    );

    assert!(
        result.is_ok(),
        "EXPERIMENTAL finding: with proof=None, an out-of-range q is expected to be \
         silently accepted (not refused) -- got {result:?}. If this now fails, the \
         domain check became enforced in the common path and this classification \
         (and ontology/profile.ttl's documentation of it) needs updating."
    );
}

/// Regime-honesty check (Checkpoint A's finding, reused here as
/// classification evidence rather than re-derived): a successful
/// `allocate()` result is not automatically a normalized distribution.
/// `CORRECT_MU_COST`-shaped inputs (extreme `mu`/`costs` collapsing
/// `priced_sum` to zero) hit the documented non-renormalized fallback
/// branch and legitimately return `Ok` with a sum far from `ONE`.
/// `AllocatorProjection` classifies as the degenerate-fallback variant
/// here, explicitly not the normalized-projection variant.
#[test]
fn hostile_mu_cost_baseline_is_degenerate_fallback_not_normalized() {
    let lambda = bcinr_cmca::generated::consequence_mass::case_studies::LAMBDA;
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    // Same construction as hostile_mutants.rs::run_alloc_mu_cost, including
    // proof=None: with proof=Some, allocate_in's price_err DOES refuse
    // (PriceGainUnsafe) instead of degrading -- confirmed by first running
    // this test with a real proof and getting exactly that refusal. The
    // fallback-not-refusal regime this test classifies only exists on the
    // proof=None path (has_refusal = has_error & !degrade_to_certified_selection,
    // and degrade_to_certified_selection = proof.is_none()).
    let mu = [NonNegativeFixed::from_bits(0u32.wrapping_sub(327680)); N];
    let costs = [NonNegativeFixed::ONE; N];

    let result = allocate_in(
        &FeasibleRegion::CURRENT,
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &lambda,
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
        None,
    )
    .unwrap();

    assert!(
        !FeasibleRegion::CURRENT.contains_allocation(&result),
        "hostile mu/cost baseline must NOT be classified as a normalized allocation -- \
         a successful Ok result here is the degenerate fallback regime, not the \
         normalized-projection regime, per Checkpoint A's finding"
    );
}

/// `EXPERIMENTAL`: `allocate_in` has no refusal channel analogous to
/// `cascade::CascadeRefusal::DegenerateSiblingSet` for all-zero masses --
/// D1 found only a silent branchless divide-by-`ONE` guard. This test
/// pins that absence: the closest reachable approximation of "all masses
/// zero" via `OBJECT_REGISTRY`-derived masses (all `mu`/`costs` pushed to
/// the same degenerate-fallback regime as the hostile baseline above)
/// must not panic and must not produce `Err` -- confirming there is no
/// typed refusal path to take, not just that this particular input
/// happens to succeed.
#[test]
fn allocate_in_has_no_typed_refusal_for_degenerate_fallback_regime() {
    let lambda = bcinr_cmca::generated::consequence_mass::case_studies::LAMBDA;
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [NonNegativeFixed::from_bits(0u32.wrapping_sub(327680)); N];
    let costs = [NonNegativeFixed::ONE; N];

    let result = allocate_in(
        &FeasibleRegion::CURRENT,
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &lambda,
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
        None,
    );

    assert!(
        !matches!(result, Err(StabilityRefusal::ContractViolation)),
        "no ContractViolation expected from this degenerate-but-not-cyclic input"
    );
    assert!(
        result.is_ok(),
        "EXPERIMENTAL finding: allocate_in has no typed refusal for this degenerate \
         regime -- it always succeeds via the silent fallback, got {result:?}"
    );
}

/// Documentation-as-test: records the D1 finding that all five surfaces
/// have zero production callers today. This is the test to update (not
/// silently delete) if any of them ever gains a real caller -- the point
/// is to make that fact loud when it changes, not to prevent it.
#[test]
fn all_five_surfaces_have_no_production_caller_as_of_bcinr_cmca_d() {
    // No executable assertion is possible for "no caller exists anywhere
    // in the workspace" from inside a single crate's test binary -- that
    // claim was established by a workspace-wide grep during BCINR-CMCA-D's
    // planning phase, not by anything this test can check at runtime.
    // This test exists so the claim has a permanent, named location in the
    // test suite rather than living only in a plan file or commit message.
    //
    // Surfaces confirmed production-caller-free as of this checkpoint:
    // cascade::escort_weight, cascade::consequence_mass/_traced,
    // allocator::allocate/allocate_in, allocator::power,
    // escort::escort_distribution.
}
