#![allow(clippy::needless_range_loop)]
#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

mod reference;

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt, StabilityRefusal,
};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated::consequence_mass::case_studies::{
    LensSpec, PackedSemanticState, FACTOR_ACCESS_FREQUENCY, FACTOR_BUSINESS_VALUE,
    FACTOR_DOWNSTREAM_CONSEQUENCE, FACTOR_RECOMPUTATION_COST, FACTOR_RETRIEVAL_DEMAND,
    FACTOR_SCHEDULING_DEMAND, FACTOR_SEARCH_DEMAND, FACTOR_STANDING, FACTOR_VERIFICATION_COST,
    K, MEASURE_CACHE, MEASURE_RETRIEVAL, MEASURE_SCHEDULING, MEASURE_SEARCH, N, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
use bcinr_cmca::generated_profile::{
    ALLOCATION_ERROR_OPERATION_BUDGET, ESCORT_DYNAMIC_RANGE_LIMIT,
    TRANSCENDENTAL_MAX_ERROR_ULPS,
};
use proptest::prelude::*;
use reference::allocate_f64;

const SCALE: f64 = 65_536.0;
const Q16_MAX: f64 = u32::MAX as f64 / SCALE;

#[derive(Debug)]
enum OracleOutcome {
    Representable([f64; N]),
    OutOfNumericEnvelope,
    UnsupportedDomain,
}

fn proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
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

fn to_f64(value: NonNegativeFixed) -> f64 {
    value.val as f64 / SCALE
}

fn signed_to_f64(value: SignedFixed) -> f64 {
    value.val as f64 / SCALE
}

fn to_fixed(value: f64) -> NonNegativeFixed {
    let scaled = (value * SCALE).round();
    if scaled >= u32::MAX as f64 {
        NonNegativeFixed::MAX
    } else if scaled <= 0.0 {
        NonNegativeFixed::ZERO
    } else {
        NonNegativeFixed::from_bits(scaled as u32)
    }
}

fn to_signed_fixed(value: f64) -> SignedFixed {
    SignedFixed::from_bits((value * SCALE).round() as i32)
}

fn parent_strategy() -> impl Strategy<Value = [i32; N]> {
    (
        Just(-1i32),
        any::<bool>().prop_map(|root| if root { -1 } else { 0 }),
        (0..3).prop_map(|value| if value == 2 { -1 } else { value }),
        (0..4).prop_map(|value| if value == 3 { -1 } else { value }),
        (0..5).prop_map(|value| if value == 4 { -1 } else { value }),
        (0..6).prop_map(|value| if value == 5 { -1 } else { value }),
        (0..7).prop_map(|value| if value == 6 { -1 } else { value }),
        (0..8).prop_map(|value| if value == 7 { -1 } else { value }),
    )
        .prop_map(|values| {
            let (p0, p1, p2, p3, p4, p5, p6, p7) = values;
            [p0, p1, p2, p3, p4, p5, p6, p7]
        })
}

fn oracle_masses(states: &[PackedSemanticState; N]) -> Option<[[f64; N]; K]> {
    let mut masses = [[0.0; N]; K];
    let minimum = 6.0 / SCALE;
    let maximum = 1_000.0;
    for (index, state) in states.iter().enumerate() {
        let f = |slot| to_f64(state.factors[slot]);
        let candidates = [
            ((f(FACTOR_RECOMPUTATION_COST) * 5.0 + f(FACTOR_VERIFICATION_COST))
                * f(FACTOR_ACCESS_FREQUENCY))
                * f(FACTOR_STANDING),
            f(FACTOR_BUSINESS_VALUE) * f(FACTOR_RETRIEVAL_DEMAND),
            f(FACTOR_BUSINESS_VALUE) * f(FACTOR_SCHEDULING_DEMAND),
            ((f(FACTOR_BUSINESS_VALUE) + f(FACTOR_DOWNSTREAM_CONSEQUENCE))
                * f(FACTOR_SEARCH_DEMAND))
                * f(FACTOR_STANDING),
        ];
        if candidates
            .iter()
            .any(|value| !value.is_finite() || *value > Q16_MAX)
        {
            return None;
        }
        masses[MEASURE_CACHE][index] = candidates[0].clamp(minimum, maximum);
        masses[MEASURE_RETRIEVAL][index] = candidates[1].clamp(minimum, maximum);
        masses[MEASURE_SCHEDULING][index] = candidates[2].clamp(minimum, maximum);
        masses[MEASURE_SEARCH][index] = candidates[3].clamp(minimum, maximum);
    }
    Some(masses)
}

fn group_representable(indices: impl Iterator<Item = usize>, masses: &[f64; N], q: f64) -> bool {
    let scores: Vec<f64> = indices.map(|index| q * masses[index].log2()).collect();
    if scores.is_empty() {
        return true;
    }
    let minimum = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    maximum - minimum <= ESCORT_DYNAMIC_RANGE_LIMIT as f64
}

fn classify_envelope(
    states: &[PackedSemanticState; N],
    lenses: &[LensSpec; Q],
    parent: &[i32; N],
    mu: &[f64; N],
    costs: &[f64; N],
) -> Result<(), OracleOutcome> {
    if parent.iter().any(|p| *p < -1 || *p >= N as i32)
        || !parent.iter().any(|p| *p == -1)
        || lenses
            .iter()
            .any(|lens| !(-2.0..=2.0).contains(&signed_to_f64(lens.q)))
    {
        return Err(OracleOutcome::UnsupportedDomain);
    }
    let Some(masses) = oracle_masses(states) else {
        return Err(OracleOutcome::OutOfNumericEnvelope);
    };

    let is_leaf = core::array::from_fn::<_, N, _>(|node| {
        !parent.iter().any(|candidate| *candidate == node as i32)
    });
    let mut descendant = [[false; N]; N];
    for ancestor in 0..N {
        descendant[ancestor][ancestor] = true;
        for node in 0..N {
            let mut cursor = node;
            for _ in 0..N {
                match parent[cursor] {
                    -1 => break,
                    p if p as usize == ancestor => {
                        descendant[ancestor][node] = true;
                        break;
                    }
                    p => cursor = p as usize,
                }
            }
        }
    }

    for mass_set in &masses {
        for lens in lenses {
            let q = signed_to_f64(lens.q);
            if !group_representable(
                (0..N).filter(|index| parent[*index] == -1),
                mass_set,
                q,
            ) {
                return Err(OracleOutcome::OutOfNumericEnvelope);
            }
            for node in 0..N {
                if !group_representable(
                    (0..N).filter(|index| parent[*index] == node as i32),
                    mass_set,
                    q,
                ) || !group_representable(
                    (0..N).filter(|index| is_leaf[*index] && descendant[node][*index]),
                    mass_set,
                    q,
                ) {
                    return Err(OracleOutcome::OutOfNumericEnvelope);
                }
            }
        }
    }

    if mu
        .iter()
        .zip(costs)
        .any(|(price, cost)| price * cost * core::f64::consts::LOG2_E > 16.0)
    {
        return Err(OracleOutcome::OutOfNumericEnvelope);
    }
    Ok(())
}

fn comparison_bound() -> f64 {
    (TRANSCENDENTAL_MAX_ERROR_ULPS * ALLOCATION_ERROR_OPERATION_BUDGET) as f64 / SCALE
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(
        std::env::var("PROPTEST_CASES").unwrap_or_else(|_| "32".into()).parse().unwrap()
    ))]

    #[test]
    fn differential_oracle_compares_outcome_pairs(
        factors in prop::collection::vec(prop::collection::vec(0.0..1.0, 8), N),
        bvals in prop::collection::vec(0.0..1000.0, N),
        consequences in prop::collection::vec(0.0..1000.0, N),
        lens_exponents in prop::collection::vec(-1.99..1.99, Q),
        lambda_rows in prop::collection::vec(prop::collection::vec(0.0..1.0, Q), K),
        eta in 0.1..0.9,
        parent in parent_strategy(),
        raw_weights in prop::collection::vec(0.1..1.0, N * 2 * Q),
        raw_payoffs in prop::collection::vec(0.0..1.0, N * 2 * Q),
        zeta in 0.001..0.0125,
        epsilon_kappa in 0.001..0.05,
        mu in prop::collection::vec(0.0..10.0, N),
        costs in prop::collection::vec(0.0..1.0, N),
        t in 0..100u32,
        tau_d in 461..1000u32,
    ) {
        let mut states = [PackedSemanticState { id: 0, factors: [NonNegativeFixed::ZERO; 10] }; N];
        for index in 0..N {
            states[index].id = index as u32;
            for slot in 0..8 {
                states[index].factors[slot] = to_fixed(factors[index][slot]);
            }
            states[index].factors[8] = to_fixed(bvals[index]);
            states[index].factors[9] = to_fixed(consequences[index]);
        }

        let mut lenses = [LensSpec { id: 0, q: SignedFixed::ZERO }; Q];
        for index in 0..Q {
            lenses[index] = LensSpec { id: index as u32, q: to_signed_fixed(lens_exponents[index]) };
        }

        let mut lambda_fixed = [[NonNegativeFixed::ZERO; Q]; K];
        let mut lambda_f64 = [[0.0; Q]; K];
        for model in 0..K {
            let sum: f64 = lambda_rows[model].iter().sum();
            for lens in 0..Q {
                let value = if sum > 0.0 { lambda_rows[model][lens] / sum } else { 1.0 / Q as f64 };
                lambda_fixed[model][lens] = to_fixed(value);
                lambda_f64[model][lens] = value;
            }
        }

        let mut weights_fixed = [[NonNegativeFixed::ZERO; 2 * Q]; N];
        let mut weights_f64 = [[0.0; 2 * Q]; N];
        let mut payoffs_fixed = [[NonNegativeFixed::ZERO; 2 * Q]; N];
        let mut payoffs_f64 = [[0.0; 2 * Q]; N];
        for node in 0..N {
            for edge in 0..2 * Q {
                let offset = node * 2 * Q + edge;
                weights_fixed[node][edge] = to_fixed(raw_weights[offset]);
                weights_f64[node][edge] = raw_weights[offset];
                payoffs_fixed[node][edge] = to_fixed(raw_payoffs[offset]);
                payoffs_f64[node][edge] = raw_payoffs[offset];
            }
            for lens in 0..Q {
                let sum_f64 = weights_f64[node][2 * lens] + weights_f64[node][2 * lens + 1];
                weights_f64[node][2 * lens] /= sum_f64;
                weights_f64[node][2 * lens + 1] /= sum_f64;
                let sum_fixed = weights_fixed[node][2 * lens] + weights_fixed[node][2 * lens + 1];
                weights_fixed[node][2 * lens] = weights_fixed[node][2 * lens].saturating_div(sum_fixed);
                weights_fixed[node][2 * lens + 1] = weights_fixed[node][2 * lens + 1].saturating_div(sum_fixed);
            }
        }

        let mu_fixed = core::array::from_fn(|index| to_fixed(mu[index]));
        let costs_fixed = core::array::from_fn(|index| to_fixed(costs[index]));
        let mut fixed_last = 0;
        let mut fixed_mode = 0;
        let before_weights = weights_fixed;
        let fixed = allocate(
            &states, &lenses, &lambda_fixed, to_fixed(eta), &parent, &mut weights_fixed,
            &payoffs_fixed, to_fixed(zeta), to_fixed(epsilon_kappa), &mu_fixed, &costs_fixed,
            t, &mut fixed_last, &mut fixed_mode, tau_d, CERTIFICATE_DIGEST, proof().as_ref(),
        );

        let oracle = match classify_envelope(&states, &lenses, &parent, &mu.clone().try_into().unwrap(), &costs.clone().try_into().unwrap()) {
            Ok(()) => {
                let mut reference_weights = weights_f64;
                let mut reference_last = 0;
                let mut reference_mode = 0;
                OracleOutcome::Representable(allocate_f64(
                    &states, &lenses, &lambda_f64, eta, &parent, &mut reference_weights,
                    &payoffs_f64, zeta, epsilon_kappa, &mu.clone().try_into().unwrap(),
                    &costs.clone().try_into().unwrap(), t, &mut reference_last, &mut reference_mode,
                    tau_d,
                ))
            }
            Err(outcome) => outcome,
        };

        match (fixed, oracle) {
            (Ok(actual), OracleOutcome::Representable(expected)) => {
                let is_leaf = core::array::from_fn::<_, N, _>(|node| !parent.iter().any(|p| *p == node as i32));
                for node in 0..N {
                    if is_leaf[node] {
                        let difference = (to_f64(actual[node]) - expected[node]).abs();
                        prop_assert!(difference <= comparison_bound(), "node={node}, fixed={}, reference={}, difference={}, bound={}", to_f64(actual[node]), expected[node], difference, comparison_bound());
                    }
                }
            }
            (Err(StabilityRefusal::NumericRangeExceeded), OracleOutcome::OutOfNumericEnvelope) => {
                prop_assert_eq!(weights_fixed, before_weights, "refusal mutated persistent weights");
                prop_assert_eq!(fixed_last, 0);
                prop_assert_eq!(fixed_mode, 0);
            }
            (Err(StabilityRefusal::UnsupportedDomain), OracleOutcome::UnsupportedDomain) => {}
            (actual, expected) => prop_assert!(false, "outcome mismatch: fixed={actual:?}, oracle={expected:?}"),
        }
    }
}

fn boundary_fixture(high_business_value: f64, lens: f64) -> (
    [PackedSemanticState; N],
    [LensSpec; Q],
    [[NonNegativeFixed; Q]; K],
    [i32; N],
    [[NonNegativeFixed; 2 * Q]; N],
    [[NonNegativeFixed; 2 * Q]; N],
) {
    let mut states = [PackedSemanticState { id: 0, factors: [NonNegativeFixed::ONE; 10] }; N];
    for (index, state) in states.iter_mut().enumerate() {
        state.id = index as u32;
        state.factors[FACTOR_BUSINESS_VALUE] = if index == 0 { to_fixed(high_business_value) } else { NonNegativeFixed::ZERO };
        state.factors[FACTOR_DOWNSTREAM_CONSEQUENCE] = NonNegativeFixed::ZERO;
    }
    let lenses = core::array::from_fn(|index| LensSpec { id: index as u32, q: to_signed_fixed(lens) });
    let lambda = [[NonNegativeFixed::from_bits(16_384); Q]; K];
    let parent = [-1; N];
    let weights = [[NonNegativeFixed::from_bits(32_768); 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    (states, lenses, lambda, parent, weights, payoffs)
}

#[test]
fn five_permanent_numeric_boundary_witnesses_refuse_deterministically() {
    let seeds = [
        (1_000.0, 2.0),
        (100.0, 2.0),
        (10.0, 2.0),
        (1.0, 2.0),
        (1_000.0, -2.0),
    ];
    for (seed, (high, lens)) in seeds.into_iter().enumerate() {
        let (states, lenses, lambda, parent, initial_weights, payoffs) = boundary_fixture(high, lens);
        let mut first_weights = initial_weights;
        let mut first_last = 0;
        let mut first_mode = 0;
        let first = allocate(
            &states, &lenses, &lambda, to_fixed(0.1), &parent, &mut first_weights,
            &payoffs, NonNegativeFixed::ZERO, NonNegativeFixed::ZERO,
            &[NonNegativeFixed::ZERO; N], &[NonNegativeFixed::ZERO; N], 0,
            &mut first_last, &mut first_mode, 500, CERTIFICATE_DIGEST, proof().as_ref(),
        );
        let mut second_weights = initial_weights;
        let mut second_last = 0;
        let mut second_mode = 0;
        let second = allocate(
            &states, &lenses, &lambda, to_fixed(0.1), &parent, &mut second_weights,
            &payoffs, NonNegativeFixed::ZERO, NonNegativeFixed::ZERO,
            &[NonNegativeFixed::ZERO; N], &[NonNegativeFixed::ZERO; N], 0,
            &mut second_last, &mut second_mode, 500, CERTIFICATE_DIGEST, proof().as_ref(),
        );
        assert_eq!(first, Err(StabilityRefusal::NumericRangeExceeded), "seed {seed}");
        assert_eq!(second, first, "seed {seed} is nondeterministic");
        assert_eq!(first_weights, initial_weights, "seed {seed} mutated state");
        assert_eq!(second_weights, initial_weights, "seed {seed} mutated state");
    }
}
