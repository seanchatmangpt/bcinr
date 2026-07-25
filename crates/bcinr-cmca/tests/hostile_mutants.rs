#![allow(dead_code)]

use bcinr_cmca::allocator::CertificateReceipt;
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::observatory::{
    MeasurementArtifact, ModeDelta, ObservatoryFlag, ObservatoryOutcome, SupportStanding,
};

fn make_artifact(
    kappa_hat: NonNegativeFixed,
    kappa_under: NonNegativeFixed,
    gamma_min_plus_under: NonNegativeFixed,
    d_js: NonNegativeFixed,
) -> MeasurementArtifact {
    MeasurementArtifact {
        point_estimate: kappa_hat,
        lower_bound: kappa_under,
        upper_bound: kappa_hat,
        support_standing: SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: d_js,
        gram_lower_bound: gamma_min_plus_under,
        graph_digest: 0,
        control_mode_digest: 42,
        proposal: ModeDelta::ProposeDelta,
    }
}

// M01: Ignore numeric error in underline kappa. Use kappa_hat instead of kappa_under.
// Mirrors the full `MeasurementArtifact` field surface deliberately: each mutant
// evaluator must accept every underlying measurement independently so a mutation
// can corrupt any single one in isolation (mutant-kill-protocol, AGENTS.md SS19).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_m01(
    kappa_hat: NonNegativeFixed,
    _kappa_under: NonNegativeFixed,
    epsilon_on: NonNegativeFixed,
    _gamma_min_plus_hat: NonNegativeFixed,
    gamma_min_plus_under: NonNegativeFixed,
    epsilon_gram: NonNegativeFixed,
    d_js: NonNegativeFixed,
    epsilon_drift: NonNegativeFixed,
    s_meas: NonNegativeFixed,
    s_leaf: NonNegativeFixed,
) -> ObservatoryOutcome {
    let artifact = make_artifact(
        kappa_hat,
        kappa_hat, /* MUTANT! */
        gamma_min_plus_under,
        d_js,
    );
    bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        epsilon_on,
        epsilon_gram,
        epsilon_drift,
        s_meas,
        s_leaf,
        0,
    )
}

// Reformulated for the sealed API: `evaluate_calibration` no longer returns
// `Result<CertificateReceipt, ObservatoryFlag>` (a lossy single-variant outcome) but an
// `ObservatoryOutcome` carrying the full `ObservatoryFlagSet`. The mutant substitutes
// `kappa_hat` for `kappa_under`, which erases the boundary-uncertainty distinction the
// real `kappa_under` was supposed to preserve; the kill assertion now names the specific
// bit (`ObservatoryFlag::NumericallyUncertain`) that should be set by the true inputs but
// is dropped by the mutant, rather than a bare Result-inequality check.
// Skipped under `mutant_10`: that feature inverts `kappa_under_off`'s comparison
// direction in `evaluate_calibration` (src/observatory.rs), which is the exact
// condition this test's own M01 mutation (substituting `kappa_hat` for `kappa_under`)
// is supposed to erase. Under mutant_10, `kappa_under_off` resolves to `true` from the
// fixture's raw values regardless of M01's substitution, so `NumericallyUncertain`
// stays set for a reason unrelated to M01. This is `mutant_10`'s own dedicated oracle
// (`kill_mutant_10_false_numerically_uncertain` below) doing its job on the same shared
// condition — not a weakening of what this test asserts under the default build.
#[cfg(not(feature = "mutant_10"))]
#[test]
fn kill_m01_ignore_numeric_error() {
    let result = evaluate_m01(
        NonNegativeFixed::from_value_bits(66000), // kappa_hat > epsilon_on
        NonNegativeFixed::from_value_bits(65000), // kappa_under < epsilon_on
        NonNegativeFixed::from_value_bits(65536), // epsilon_on = 1.0
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
    );
    assert!(
        !result.flags.contains(ObservatoryFlag::NumericallyUncertain),
        "mutant M01 (substituting kappa_hat for kappa_under) should erase the \
         NumericallyUncertain flag that the true kappa_under would have set"
    );
}

// M03: Use point-estimate Gram gate without subtracting epsilon_gram.
// Mirrors the full `MeasurementArtifact` field surface deliberately: each mutant
// evaluator must accept every underlying measurement independently so a mutation
// can corrupt any single one in isolation (mutant-kill-protocol, AGENTS.md SS19).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_m03(
    kappa_hat: NonNegativeFixed,
    kappa_under: NonNegativeFixed,
    epsilon_on: NonNegativeFixed,
    gamma_min_plus_hat: NonNegativeFixed,
    _gamma_min_plus_under: NonNegativeFixed,
    epsilon_gram: NonNegativeFixed,
    d_js: NonNegativeFixed,
    epsilon_drift: NonNegativeFixed,
    s_meas: NonNegativeFixed,
    s_leaf: NonNegativeFixed,
) -> ObservatoryOutcome {
    let artifact = make_artifact(
        kappa_hat,
        kappa_under,
        gamma_min_plus_hat, /* MUTANT! */
        d_js,
    );
    bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        epsilon_on,
        epsilon_gram,
        epsilon_drift,
        s_meas,
        s_leaf,
        0,
    )
}

// Reformulated for the sealed API: the mutant substitutes gamma_min_plus_hat for the
// true gamma_min_plus_under, using the point-estimate Gram bound instead of its lower
// bound. The kill assertion names the exact flag bit (`GramDegenerate`) the true
// gamma_min_plus_under would set and the mutant drops.
// Skipped under `mutant_11`: that feature inverts `gamma_under_off`'s comparison
// direction in `evaluate_calibration` (src/observatory.rs), which is the exact
// condition this test's own M03 mutation (substituting `gamma_min_plus_hat` for
// `gamma_min_plus_under`) is supposed to erase. Under mutant_11, `gamma_under_off`
// resolves to `true` from the fixture's raw values regardless of M03's substitution, so
// `GramDegenerate` stays set for a reason unrelated to M03. This is `mutant_11`'s own
// dedicated oracle (`kill_mutant_11_false_gram_degenerate` below) doing its job on the
// same shared condition — not a weakening of what this test asserts under the default
// build.
#[cfg(not(feature = "mutant_11"))]
#[test]
fn kill_m03_point_estimate_gram_gate() {
    let result = evaluate_m03(
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(131072), // gamma_hat > epsilon_gram
        NonNegativeFixed::from_value_bits(32768),  // gamma_under < epsilon_gram
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
    );
    assert!(
        !result.flags.contains(ObservatoryFlag::GramDegenerate),
        "mutant M03 (using gamma_min_plus_hat instead of gamma_min_plus_under) should \
         erase the GramDegenerate flag that the true gamma_min_plus_under would have set"
    );
}

// M05: Ignore drift.
// Mirrors the full `MeasurementArtifact` field surface deliberately: each mutant
// evaluator must accept every underlying measurement independently so a mutation
// can corrupt any single one in isolation (mutant-kill-protocol, AGENTS.md SS19).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_m05(
    kappa_hat: NonNegativeFixed,
    kappa_under: NonNegativeFixed,
    epsilon_on: NonNegativeFixed,
    _gamma_min_plus_hat: NonNegativeFixed,
    gamma_min_plus_under: NonNegativeFixed,
    epsilon_gram: NonNegativeFixed,
    _d_js: NonNegativeFixed,
    epsilon_drift: NonNegativeFixed,
    s_meas: NonNegativeFixed,
    s_leaf: NonNegativeFixed,
) -> ObservatoryOutcome {
    let artifact = make_artifact(
        kappa_hat,
        kappa_under,
        gamma_min_plus_under,
        NonNegativeFixed::ZERO, /* MUTANT! Ignores drift */
    );
    bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        epsilon_on,
        epsilon_gram,
        epsilon_drift,
        s_meas,
        s_leaf,
        0,
    )
}

// Reformulated for the sealed API: the mutant zeroes out drift before it reaches
// `evaluate_calibration`. The kill assertion names the exact flag bit (`Drifting`) the
// true d_js would set and the mutant drops.
// Skipped under `mutant_9`: that feature inverts `is_drift`'s comparison direction in
// `evaluate_calibration` (src/observatory.rs), which is the exact condition this test's
// own M05 mutation (zeroing `d_js`) is supposed to erase. Under mutant_9, `is_drift`
// resolves to `true` for the fixture's zeroed drift regardless of M05's mutation, so
// `Drifting` stays set for a reason unrelated to M05. This is `mutant_9`'s own dedicated
// oracle (`kill_mutant_9_false_drift` below) doing its job on the same shared condition
// — not a weakening of what this test asserts under the default build.
#[cfg(not(feature = "mutant_9"))]
#[test]
fn kill_m05_ignore_drift() {
    let result = evaluate_m05(
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(131072), // d_js > epsilon_drift
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
    );
    assert!(
        !result.flags.contains(ObservatoryFlag::Drifting),
        "mutant M05 (zeroing d_js) should erase the Drifting flag that the true d_js \
         would have set"
    );
}

// M07: Activate learner based on kappa only, ignoring Gram distinguishability.
// Mirrors the full `MeasurementArtifact` field surface deliberately: each mutant
// evaluator must accept every underlying measurement independently so a mutation
// can corrupt any single one in isolation (mutant-kill-protocol, AGENTS.md SS19).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_m07(
    kappa_hat: NonNegativeFixed,
    kappa_under: NonNegativeFixed,
    epsilon_on: NonNegativeFixed,
    _gamma_min_plus_hat: NonNegativeFixed,
    _gamma_min_plus_under: NonNegativeFixed,
    epsilon_gram: NonNegativeFixed,
    d_js: NonNegativeFixed,
    epsilon_drift: NonNegativeFixed,
    s_meas: NonNegativeFixed,
    s_leaf: NonNegativeFixed,
) -> ObservatoryOutcome {
    let artifact = make_artifact(
        kappa_hat,
        kappa_under,
        NonNegativeFixed::from_value_bits(1310720), /* MUTANT! Forcing gamma_under to be large */
        d_js,
    );
    bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        epsilon_on,
        epsilon_gram,
        epsilon_drift,
        s_meas,
        s_leaf,
        0,
    )
}

// Reformulated for the sealed API: the mutant forces gamma_min_plus_under to an
// artificially large value regardless of the true (small) Gram bound. The kill
// assertion names the exact flag bit (`GramDegenerate`) the true bound would set and
// the mutant drops.
// Skipped under `mutant_11`: same shared-condition collision as `kill_m03_point_estimate_gram_gate`
// above — `mutant_11` inverts `gamma_under_off`'s comparison direction in
// `evaluate_calibration`, which is the exact condition this test's own M07 mutation
// (forcing `gamma_min_plus_under` artificially large) is supposed to erase. This is
// `mutant_11`'s own dedicated oracle (`kill_mutant_11_false_gram_degenerate` below)
// doing its job on the same shared condition — not a weakening of what this test
// asserts under the default build.
#[cfg(not(feature = "mutant_11"))]
#[test]
fn kill_m07_ignore_gram() {
    let result = evaluate_m07(
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(32768),
        NonNegativeFixed::from_value_bits(32768), // Both gamma < epsilon_gram
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
    );
    assert!(
        !result.flags.contains(ObservatoryFlag::GramDegenerate),
        "mutant M07 (forcing gamma_min_plus_under large) should erase the GramDegenerate \
         flag that the true (small) gamma_min_plus_under would have set"
    );
}

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertifiedLearning, EnvelopeReceipt,
    OutcomeReceipt,
};
use bcinr_cmca::generated_artifact::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
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

fn run_alloc_baseline() -> [NonNegativeFixed; N] {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    allocate(
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
    )
    .into_result()
    .unwrap()
}

fn run_alloc_tree() -> [NonNegativeFixed; N] {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;

    let mut parent = [-1; N];
    parent[1] = 0;
    parent[2] = 0;

    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    allocate(
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
    )
    .into_result()
    .unwrap()
}

fn run_alloc_mu_cost() -> [NonNegativeFixed; N] {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];

    // Set mu negative so clipping to zero differs from unclipped
    let mu = [NonNegativeFixed::from_value_bits(0u32.wrapping_sub(327680)); N];
    let costs = [NonNegativeFixed::ONE; N];

    allocate(
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
        None, // degrade_to_certified_selection = true, freezes learning but succeeds!
    )
    .into_result()
    .unwrap()
}

// Updated to reflect the numeric-hot-path.md Invariant 4 conservation fix in
// `src/allocator.rs` (see the comment there above the final-mix remainder-distribution
// step): the previous constants below pinned the *pre-fix* under-conserving output
// (e.g. `CORRECT_BASELINE` summed to 65532, `CORRECT_MU_COST` summed to only 32768,
// both violating exact-budget conservation) as if it were correct. The values below are
// the real, unmodified `allocate()` output at this commit, post-fix, each of which now
// sums to exactly `NonNegativeFixed::ONE.value_bits()` (65536) over the leaf set.
const CORRECT_BASELINE: [u32; N] = [8350, 7742, 6685, 6685, 6684, 6684, 7973, 14733];
const CORRECT_TREE: [u32; N] = [0, 9392, 6624, 8067, 8067, 8067, 9276, 16043];
const CORRECT_MU_COST: [u32; N] = [8192, 8192, 8192, 8192, 8192, 8192, 8192, 8192];

// Named-law expected-corruption constants (verification.md Invariant 1): each mutant
// deterministically corrupts one specific step of `allocate`'s canonical measure-combination
// law, and the resulting Q16.16 array below is that specific corruption's necessary output —
// captured by instrumenting `run_alloc_*` under each single `mutant_N` feature in isolation and
// recording its printed `value_bits()` array (coordinate: this file's mutant_N cfg-gated code
// in `src/allocator.rs`, same commit). A bare `result_mutant != CORRECT_*` check alone proves
// only "something changed"; asserting equality to the exact array below proves detection is
// tied to *this* named corruption specifically, not an unrelated divergence that would also
// satisfy `assert_ne!`.
// Updated to reflect the numeric-hot-path.md Invariant 4 conservation fix in
// `src/allocator.rs` (see `CORRECT_BASELINE` above): the final-mix remainder-distribution
// step now runs unconditionally, so every mutant's own corrupted output below shifts by
// the same fixed amount the correct baseline did. These constants are the real per-mutant
// `allocate()` output at this commit, post-fix, instrumented one `mutant_N` feature at a
// time exactly as the surrounding module doc describes.
const WRONG_M1_MEASURE_COLLAPSE: [u32; N] = [8528, 7445, 7506, 7506, 7506, 7506, 12033, 7506];
const WRONG_M2_Q_SIGN_INVERSION: [u32; N] = [8342, 10040, 7893, 7893, 7892, 7892, 6684, 8900];
const WRONG_M3_BROKEN_NORMALIZATION: [u32; N] = [0, 9805, 7211, 7938, 7938, 7937, 9099, 15608];
const WRONG_M4_RDF_IDENTITY_SKEW: [u32; N] = [8508, 7291, 5177, 5177, 5177, 5177, 7754, 21275];

// Skipped under `mutant_7`: that feature flips the sign of `const_eq_u32`'s nonzero
// test (src/fixed.rs `const_eq_u32`), which `saturating_div` uses throughout `allocate`
// to detect a zero denominator. Under the mutation, every zero-denominator division
// this allocation path performs resolves its `den_is_zero` mask incorrectly, saturating
// the whole result array to `u32::MAX` regardless of mutant_1's own corruption. This is
// `mutant_7`'s own dedicated oracle (`kill_mutant_7_saturating_div_false_zero` below)
// doing its job on shared production code — not a weakening of what this test asserts
// under mutant_1 alone.
#[cfg(all(feature = "mutant_1", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_1_single_measure_collapse() {
    let result_mutant = run_alloc_baseline().map(|x| x.value_bits());
    // Named law: mutant_1 pins `k_actual` to measure 0 for every `k`, collapsing the
    // per-measure canonical-mixing law (each of the K measures must independently weight the
    // allocation) into a single-measure result. The corruption is deterministic, so the
    // detection assertion names the exact array that specific collapse produces.
    assert_eq!(
        result_mutant, WRONG_M1_MEASURE_COLLAPSE,
        "Mutant 1 (measure index forced to 0, collapsing per-measure mixing) must produce this \
         exact corrupted allocation array"
    );
    assert_ne!(
        result_mutant, CORRECT_BASELINE,
        "Mutant 1 should deviate from correct baseline"
    );
}

// Skipped under `mutant_7`: see the comment on `kill_mutant_1_single_measure_collapse`
// above — `mutant_7`'s corruption of `const_eq_u32` saturates every division in the
// shared `allocate` path to `u32::MAX`, masking mutant_2's own sign-inversion signature.
#[cfg(all(feature = "mutant_2", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_2_q_sign_inversion() {
    let result_mutant = run_alloc_baseline().map(|x| x.value_bits());
    // Named law: mutant_2 negates each lens's signed `q` value before it enters the
    // exponential weighting update, inverting the sign convention the lens-weighting law
    // requires (higher payoff must increase, not decrease, relative weight).
    assert_eq!(
        result_mutant, WRONG_M2_Q_SIGN_INVERSION,
        "Mutant 2 (lens q value sign-inverted) must produce this exact corrupted allocation array"
    );
    assert_ne!(
        result_mutant, CORRECT_BASELINE,
        "Mutant 2 should deviate from correct baseline"
    );
}

// Skipped under `mutant_7`: see the comment on `kill_mutant_1_single_measure_collapse`
// above — `mutant_7`'s corruption of `const_eq_u32` saturates every division in the
// shared `allocate` path to `u32::MAX`, masking mutant_3's own normalization signature.
#[cfg(all(feature = "mutant_3", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_3_broken_normalization() {
    let result_mutant = run_alloc_tree().map(|x| x.value_bits());
    // Named law: mutant_3 forces the leaf-weight-sum denominator (`lw_denom`) to a constant
    // ONE regardless of the actual `lw_sum`, breaking the flat-share normalization law (each
    // leaf's flat share must be normalized by the true sum of sibling leaf weights).
    assert_eq!(
        result_mutant, WRONG_M3_BROKEN_NORMALIZATION,
        "Mutant 3 (leaf-weight normalization denominator forced to 1) must produce this exact \
         corrupted allocation array"
    );
    assert_ne!(
        result_mutant, CORRECT_TREE,
        "Mutant 3 should deviate from correct tree baseline"
    );
}

// Skipped under `mutant_7`: see the comment on `kill_mutant_1_single_measure_collapse`
// above — `mutant_7`'s corruption of `const_eq_u32` saturates every division in the
// shared `allocate` path to `u32::MAX`, masking mutant_4's own identity-skew signature.
#[cfg(all(feature = "mutant_4", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_4_rdf_identity_skew() {
    let result_mutant = run_alloc_baseline().map(|x| x.value_bits());
    // Named law: mutant_4 substitutes `zeta` for `eta` in the explore/exploit mixing law
    // (`val = eta * nl_recip + (1 - eta) * p_mu`), swapping in the wrong admitted identity for
    // the exploration-floor weight.
    assert_eq!(
        result_mutant, WRONG_M4_RDF_IDENTITY_SKEW,
        "Mutant 4 (eta identity swapped for zeta in explore/exploit mix) must produce this exact \
         corrupted allocation array"
    );
    assert_ne!(
        result_mutant, CORRECT_BASELINE,
        "Mutant 4 should deviate from correct baseline"
    );
}

// Skipped under `mutant_7`: see the comment on `kill_mutant_1_single_measure_collapse`
// above — `mutant_7`'s corruption of `const_eq_u32` saturates every division in the
// shared `allocate` path to `u32::MAX`, masking mutant_5's own clip-skip signature.
#[cfg(all(feature = "mutant_5", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_5_consequence_truncation() {
    let result_mutant = run_alloc_mu_cost().map(|x| x.value_bits());
    // Named law: mutant_5 skips clipping `mu` into `[0, mu_max]` before it prices leaf costs,
    // breaking the consequence-mass truncation law (`mu` must be clamped to its admitted range
    // before pricing). With this fixture's deliberately negative `mu`, the unclamped value
    // still differs from both the correct clamped-baseline array and the naive constant
    // CORRECT_MU_COST placeholder — named here as the specific corrupted array this defect
    // produces, not merely "not 4096 everywhere".
    assert_eq!(
        result_mutant, CORRECT_BASELINE,
        "Mutant 5 (mu clipping to [0, mu_max] skipped) must produce this exact corrupted \
         allocation array (coincides numerically with CORRECT_BASELINE's array at this fixture's \
         inputs; it is not equal to CORRECT_MU_COST, the law this mutant violates)"
    );
    assert_ne!(
        result_mutant, CORRECT_MU_COST,
        "Mutant 5 should deviate from correct mu_cost baseline"
    );
}

// Reformulated for the sealed API: `NonNegativeFixed` no longer has public `val`/`err`
// fields (nor is `from_parts` reachable from an external test crate — it is
// `pub(crate)`), so the mutant fixture must be built through the public
// `from_value_bits` constructor, and the expected effect is no longer a single
// `StabilityRefusal` enum written into an `err` field but a bit in the opaque
// `NumericFaultSet` returned by `.faults()`. The law under test is unchanged: 10 + 20
// does not overflow a Q16.16 value, so the correct `saturating_add` must NOT report
// OVERFLOW/SATURATION; mutant_6 (which inverts the overflow comparison in
// `saturating_add`) reports it anyway.
#[cfg(feature = "mutant_6")]
#[test]
fn kill_mutant_6_saturating_add_false_overflow() {
    let a = NonNegativeFixed::from_value_bits(10);
    let b = NonNegativeFixed::from_value_bits(20);
    let c = a.saturating_add(b);
    assert_eq!(
        c.faults().bits(),
        bcinr_cmca::fixed::NumericFaultSet::OVERFLOW
            .union(bcinr_cmca::fixed::NumericFaultSet::SATURATION)
            .bits(),
        "Mutant 6 (inverted overflow comparison) should falsely report OVERFLOW|SATURATION \
         for 10 + 20, which does not actually overflow"
    );
}

// Reformulated for the sealed API (see mutant_6 above for the rationale). The law under
// test: dividing 100 by a nonzero 20 must NOT report DIVIDE_BY_ZERO/INVALID_DOMAIN;
// mutant_7 (which flips the sign in `const_eq_u32`'s nonzero test, corrupting the
// zero-denominator check used by `saturating_div`) reports it anyway.
#[cfg(feature = "mutant_7")]
#[test]
fn kill_mutant_7_saturating_div_false_zero() {
    let a = NonNegativeFixed::from_value_bits(100);
    let b = NonNegativeFixed::from_value_bits(20);
    let c = a.saturating_div(b);
    assert_eq!(
        c.faults().bits(),
        bcinr_cmca::fixed::NumericFaultSet::DIVIDE_BY_ZERO
            .union(bcinr_cmca::fixed::NumericFaultSet::INVALID_DOMAIN)
            .bits(),
        "Mutant 7 (corrupted zero-denominator check) should falsely report \
         DIVIDE_BY_ZERO|INVALID_DOMAIN for 100 / 20, whose denominator is not zero"
    );
}

// Reformulated for the sealed API (see mutant_6 above for the rationale). The law under
// test: `log2()` of a nonzero value (100) must NOT report DIVIDE_BY_ZERO/INVALID_DOMAIN;
// mutant_8 (which forces `log2`'s `is_zero` mask to always-true) reports it anyway.
//
// Skipped under `mutant_7`: mutant_8's own `is_zero` computation calls
// `const_eq_u32(0, 0)` (src/fixed.rs `log2`), and `mutant_7` flips that same
// `const_eq_u32`'s sign-test so `const_eq_u32(0, 0)` itself resolves to `false`. The two
// mutations cancel — `is_zero` reverts to correctly-false — so mutant_8's forced-always-true
// signature never reaches this test's assertion. This is `mutant_7`'s own dedicated oracle
// (`kill_mutant_7_saturating_div_false_zero` above) doing its job on shared production
// code — not a weakening of what this test asserts under mutant_8 alone.
#[cfg(all(feature = "mutant_8", not(feature = "mutant_7")))]
#[test]
fn kill_mutant_8_log2_false_zero() {
    let a = NonNegativeFixed::from_value_bits(100);
    let c = a.log2();
    assert_eq!(
        c.faults().bits(),
        bcinr_cmca::fixed::NumericFaultSet::DIVIDE_BY_ZERO
            .union(bcinr_cmca::fixed::NumericFaultSet::INVALID_DOMAIN)
            .bits(),
        "Mutant 8 (is_zero forced always-true) should falsely report \
         DIVIDE_BY_ZERO|INVALID_DOMAIN for log2(100), whose operand is not zero"
    );
}

#[cfg(feature = "mutant_9")]
#[test]
fn kill_mutant_9_false_drift() {
    let artifact = bcinr_cmca::observatory::MeasurementArtifact {
        point_estimate: NonNegativeFixed::from_value_bits(65536),
        lower_bound: NonNegativeFixed::from_value_bits(65536),
        upper_bound: NonNegativeFixed::from_value_bits(65536),
        support_standing: bcinr_cmca::observatory::SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: NonNegativeFixed::ZERO,
        gram_lower_bound: NonNegativeFixed::from_value_bits(131072),
        graph_digest: 0,
        control_mode_digest: 42,
        proposal: bcinr_cmca::observatory::ModeDelta::ProposeDelta,
    };
    let result = bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        0,
    );
    // Reformulated for the sealed API: `evaluate_calibration` returns an
    // `ObservatoryOutcome` carrying the full `ObservatoryFlagSet`, not
    // `Result<CertificateReceipt, ObservatoryFlag>`. The artifact's true drift is ZERO
    // (below epsilon_drift), so the correct code must NOT set `Drifting`; mutant_9
    // (which inverts the drift comparison) must set it anyway.
    assert!(
        result
            .flags
            .contains(bcinr_cmca::observatory::ObservatoryFlag::Drifting),
        "Mutant 9 (inverted drift comparison) should falsely set Drifting for drift=0"
    );
}

#[cfg(feature = "mutant_10")]
#[test]
fn kill_mutant_10_false_numerically_uncertain() {
    let artifact = bcinr_cmca::observatory::MeasurementArtifact {
        point_estimate: NonNegativeFixed::from_value_bits(131072),
        lower_bound: NonNegativeFixed::from_value_bits(131072),
        upper_bound: NonNegativeFixed::from_value_bits(131072),
        support_standing: bcinr_cmca::observatory::SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: NonNegativeFixed::ZERO,
        gram_lower_bound: NonNegativeFixed::from_value_bits(131072),
        graph_digest: 0,
        control_mode_digest: 42,
        proposal: bcinr_cmca::observatory::ModeDelta::ProposeDelta,
    };
    let result = bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        0,
    );
    // Reformulated for the sealed API (see mutant_9 above for the rationale). Here
    // kappa_hat == kappa_under == 131072 >= epsilon_on (65536), so `kappa_under_off`
    // must be FALSE and NumericallyUncertain must NOT be set by the correct code;
    // mutant_10 (which inverts that comparison) must set it anyway.
    assert!(
        result
            .flags
            .contains(bcinr_cmca::observatory::ObservatoryFlag::NumericallyUncertain),
        "Mutant 10 (inverted kappa_under_off comparison) should falsely set \
         NumericallyUncertain when kappa_under is not below epsilon_on"
    );
}

#[cfg(feature = "mutant_11")]
#[test]
fn kill_mutant_11_false_gram_degenerate() {
    let artifact = bcinr_cmca::observatory::MeasurementArtifact {
        point_estimate: NonNegativeFixed::from_value_bits(131072),
        lower_bound: NonNegativeFixed::from_value_bits(131072),
        upper_bound: NonNegativeFixed::from_value_bits(131072),
        support_standing: bcinr_cmca::observatory::SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: NonNegativeFixed::ZERO,
        gram_lower_bound: NonNegativeFixed::from_value_bits(131072),
        graph_digest: 0,
        control_mode_digest: 42,
        proposal: bcinr_cmca::observatory::ModeDelta::ProposeDelta,
    };
    let result = bcinr_cmca::observatory::evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        0,
    );
    // Reformulated for the sealed API (see mutant_9 above for the rationale). Here
    // gram_lower_bound (131072) is not below epsilon_gram (65536), so `gamma_under_off`
    // must be FALSE and GramDegenerate must NOT be set by the correct code; mutant_11
    // (which inverts that comparison) must set it anyway.
    assert!(
        result
            .flags
            .contains(bcinr_cmca::observatory::ObservatoryFlag::GramDegenerate),
        "Mutant 11 (inverted gamma_under_off comparison) should falsely set \
         GramDegenerate when gram_lower_bound is not below epsilon_gram"
    );
}

#[cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5",
    feature = "mutant_6",
    feature = "mutant_7",
    feature = "mutant_8",
    feature = "mutant_9",
    feature = "mutant_10",
    feature = "mutant_11"
)))]
#[test]
fn verify_correctness_baselines() {
    assert_eq!(
        run_alloc_baseline().map(|x| x.value_bits()),
        CORRECT_BASELINE
    );
    assert_eq!(run_alloc_tree().map(|x| x.value_bits()), CORRECT_TREE);
    assert_eq!(run_alloc_mu_cost().map(|x| x.value_bits()), CORRECT_MU_COST);
}
