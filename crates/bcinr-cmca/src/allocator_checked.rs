//! Admitted public boundary for the bounded certified CMCA allocator.
//!
//! `allocator.rs` remains the straight-line implementation kernel. It contains
//! historical raw-value selections which are safe only when every arithmetic
//! operation is proved representable. This module is therefore the public
//! authority boundary: it validates the complete numeric operation envelope,
//! runs the kernel against local state, verifies every returned error channel,
//! and commits mutations only after success.

pub use crate::allocator_legacy::{
    check_hierarchy_acyclic, const_eq_u32, const_lt_u32, const_select_bool, const_select_u32,
    wrap_result, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    CertifiedSelectionOnly, EnvelopeReceipt, HierarchyRefusal, OutcomeReceipt, StabilityRefusal,
};
pub(crate) use crate::allocator_legacy::{clip, const_max_i32};

use crate::fixed::{NonNegativeFixed, SignedFixed};
use crate::generated::consequence_mass::case_studies::{
    LensSpec, PackedSemanticState, FACTOR_ACCESS_FREQUENCY, FACTOR_BUSINESS_VALUE,
    FACTOR_DOWNSTREAM_CONSEQUENCE, FACTOR_RECOMPUTATION_COST, FACTOR_RETRIEVAL_DEMAND,
    FACTOR_SCHEDULING_DEMAND, FACTOR_SEARCH_DEMAND, FACTOR_STANDING, FACTOR_VERIFICATION_COST, K,
    MEASURE_CACHE, MEASURE_RETRIEVAL, MEASURE_SCHEDULING, MEASURE_SEARCH, N, Q,
};

const OK: u32 = u32::MAX;
const Q_MIN: i32 = -2 * 65_536;
const Q_MAX: i32 = 2 * 65_536;
const MAX_NORMALIZED_SPREAD: i64 =
    crate::generated_profile::ESCORT_DYNAMIC_RANGE_LIMIT as i64 * 65_536;

/// Allocate through the certified bounded kernel after proving that the
/// concrete request lies inside its Q16.16 executable envelope.
///
/// The mathematical CMCA domain is not narrowed to the Q16.16 profile. A
/// mathematically supported request outside this executable envelope returns
/// [`StabilityRefusal::NumericRangeExceeded`]. Invalid mathematical inputs
/// return [`StabilityRefusal::UnsupportedDomain`].
#[allow(clippy::too_many_arguments)]
pub fn allocate(
    states: &[PackedSemanticState; N],
    lenses: &[LensSpec; Q],
    lambda: &[[NonNegativeFixed; Q]; K],
    eta: NonNegativeFixed,
    parent: &[i32; N],
    weights: &mut [[NonNegativeFixed; 2 * Q]; N],
    payoffs: &[[NonNegativeFixed; 2 * Q]; N],
    zeta: NonNegativeFixed,
    epsilon_kappa: NonNegativeFixed,
    mu: &[NonNegativeFixed; N],
    costs: &[NonNegativeFixed; N],
    t: u32,
    last_switch_t: &mut u32,
    prev_mode: &mut u32,
    tau_d: u32,
    digest: [u8; 32],
    proof: Option<&AdaptiveUpdate<CertifiedLearning>>,
) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
    admit_request(
        states,
        lenses,
        lambda,
        eta,
        parent,
        weights,
        payoffs,
        zeta,
        epsilon_kappa,
        mu,
        costs,
    )?;

    let mut candidate_weights = *weights;
    let mut candidate_last_switch = *last_switch_t;
    let mut candidate_mode = *prev_mode;
    let result = crate::allocator_legacy::allocate(
        states,
        lenses,
        lambda,
        eta,
        parent,
        &mut candidate_weights,
        payoffs,
        zeta,
        epsilon_kappa,
        mu,
        costs,
        t,
        &mut candidate_last_switch,
        &mut candidate_mode,
        tau_d,
        digest,
        proof,
    )?;

    for value in result.iter().chain(candidate_weights.iter().flatten()) {
        admit_fixed(*value)?;
    }

    *weights = candidate_weights;
    *last_switch_t = candidate_last_switch;
    *prev_mode = candidate_mode;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn admit_request(
    states: &[PackedSemanticState; N],
    lenses: &[LensSpec; Q],
    lambda: &[[NonNegativeFixed; Q]; K],
    eta: NonNegativeFixed,
    parent: &[i32; N],
    weights: &[[NonNegativeFixed; 2 * Q]; N],
    payoffs: &[[NonNegativeFixed; 2 * Q]; N],
    zeta: NonNegativeFixed,
    epsilon_kappa: NonNegativeFixed,
    mu: &[NonNegativeFixed; N],
    costs: &[NonNegativeFixed; N],
) -> Result<(), StabilityRefusal> {
    for &p in parent {
        if p < -1 || p >= N as i32 {
            return Err(StabilityRefusal::UnsupportedDomain);
        }
    }
    if !parent.iter().any(|p| *p == -1) {
        return Err(StabilityRefusal::UnsupportedDomain);
    }
    check_hierarchy_acyclic(parent).map_err(|_| StabilityRefusal::UnsupportedDomain)?;

    for state in states {
        for factor in state.factors {
            admit_fixed(factor)?;
        }
    }
    for lens in lenses {
        admit_signed(lens.q)?;
        if !(Q_MIN..=Q_MAX).contains(&lens.q.val) {
            return Err(StabilityRefusal::UnsupportedDomain);
        }
    }
    for value in lambda
        .iter()
        .flatten()
        .chain(weights.iter().flatten())
        .chain(payoffs.iter().flatten())
        .chain(mu.iter())
        .chain(costs.iter())
        .chain([&eta, &zeta, &epsilon_kappa])
    {
        admit_fixed(*value)?;
    }

    if eta.val > NonNegativeFixed::ONE.val {
        return Err(StabilityRefusal::UnsupportedDomain);
    }
    if lambda
        .iter()
        .flatten()
        .any(|value| value.val > NonNegativeFixed::ONE.val)
        || weights
            .iter()
            .flatten()
            .any(|value| value.val > NonNegativeFixed::ONE.val)
    {
        return Err(StabilityRefusal::NumericRangeExceeded);
    }

    let masses = checked_node_masses(states)?;
    admit_escort_groups(parent, &masses, lenses)?;
    admit_update_exponentials(weights, payoffs, zeta)?;
    admit_price_exponentials(mu, costs)?;
    Ok(())
}

fn checked_node_masses(
    states: &[PackedSemanticState; N],
) -> Result<[[NonNegativeFixed; N]; K], StabilityRefusal> {
    let mut masses = [[NonNegativeFixed::ZERO; N]; K];
    let minimum = NonNegativeFixed::from_bits(6);
    let maximum = NonNegativeFixed::from_bits(65_536_000);

    for (index, state) in states.iter().enumerate() {
        let f_recomp = state.factors[FACTOR_RECOMPUTATION_COST];
        let f_verify = state.factors[FACTOR_VERIFICATION_COST];
        let f_stand = state.factors[FACTOR_STANDING];
        let f_access = state.factors[FACTOR_ACCESS_FREQUENCY];
        let f_search = state.factors[FACTOR_SEARCH_DEMAND];
        let f_retrieve = state.factors[FACTOR_RETRIEVAL_DEMAND];
        let f_sched = state.factors[FACTOR_SCHEDULING_DEMAND];
        let f_bval = state.factors[FACTOR_BUSINESS_VALUE];
        let f_conseq = state.factors[FACTOR_DOWNSTREAM_CONSEQUENCE];

        masses[MEASURE_CACHE][index] = checked_clip(
            checked_mul(
                checked_mul(
                    checked_add(
                        checked_mul(f_recomp, NonNegativeFixed::from_num(5))?,
                        f_verify,
                    )?,
                    f_access,
                )?,
                f_stand,
            )?,
            minimum,
            maximum,
        )?;
        masses[MEASURE_SEARCH][index] = checked_clip(
            checked_mul(
                checked_mul(checked_add(f_bval, f_conseq)?, f_search)?,
                f_stand,
            )?,
            minimum,
            maximum,
        )?;
        masses[MEASURE_RETRIEVAL][index] =
            checked_clip(checked_mul(f_bval, f_retrieve)?, minimum, maximum)?;
        masses[MEASURE_SCHEDULING][index] =
            checked_clip(checked_mul(f_bval, f_sched)?, minimum, maximum)?;
    }
    Ok(masses)
}

fn admit_escort_groups(
    parent: &[i32; N],
    masses: &[[NonNegativeFixed; N]; K],
    lenses: &[LensSpec; Q],
) -> Result<(), StabilityRefusal> {
    let mut descendant = [[false; N]; N];
    for root in 0..N {
        descendant[root][root] = true;
        for node in 0..N {
            let mut cursor = node;
            for _ in 0..N {
                match parent[cursor] {
                    -1 => break,
                    p if p as usize == root => {
                        descendant[root][node] = true;
                        break;
                    }
                    p => cursor = p as usize,
                }
            }
        }
    }
    let is_leaf = core::array::from_fn::<_, N, _>(|node| {
        !parent.iter().any(|candidate| *candidate == node as i32)
    });

    for mass_set in masses {
        for lens in lenses {
            admit_group(
                (0..N).filter(|index| parent[*index] == -1),
                mass_set,
                lens.q,
            )?;
            for node in 0..N {
                admit_group(
                    (0..N).filter(|index| parent[*index] == node as i32),
                    mass_set,
                    lens.q,
                )?;
                admit_group(
                    (0..N).filter(|index| is_leaf[*index] && descendant[node][*index]),
                    mass_set,
                    lens.q,
                )?;
            }
        }
    }
    Ok(())
}

fn admit_group(
    indices: impl Iterator<Item = usize>,
    masses: &[NonNegativeFixed; N],
    q: SignedFixed,
) -> Result<(), StabilityRefusal> {
    let mut minimum: Option<i64> = None;
    let mut maximum: Option<i64> = None;
    for index in indices {
        let mass = masses[index];
        if mass.val == 0 {
            return Err(StabilityRefusal::UnsupportedDomain);
        }
        let logarithm = mass.log2();
        admit_signed(logarithm)?;
        let product = (q.val as i64)
            .checked_mul(logarithm.val as i64)
            .ok_or(StabilityRefusal::NumericRangeExceeded)?
            >> 16;
        if product < i32::MIN as i64 || product > i32::MAX as i64 {
            return Err(StabilityRefusal::NumericRangeExceeded);
        }
        minimum = Some(minimum.map_or(product, |value| value.min(product)));
        maximum = Some(maximum.map_or(product, |value| value.max(product)));
    }
    if let (Some(minimum), Some(maximum)) = (minimum, maximum) {
        if maximum - minimum > MAX_NORMALIZED_SPREAD {
            return Err(StabilityRefusal::NumericRangeExceeded);
        }
    }
    Ok(())
}

fn admit_update_exponentials(
    weights: &[[NonNegativeFixed; 2 * Q]; N],
    payoffs: &[[NonNegativeFixed; 2 * Q]; N],
    zeta: NonNegativeFixed,
) -> Result<(), StabilityRefusal> {
    let beta = if zeta.val < 6_553 {
        zeta
    } else {
        NonNegativeFixed::from_bits(6_553)
    };
    for (weight, payoff) in weights.iter().flatten().zip(payoffs.iter().flatten()) {
        let exponent = checked_mul(beta, *payoff)?;
        let signed = SignedFixed {
            val: exponent.val as i32,
            err: exponent.err,
        };
        let growth = signed.exp();
        admit_fixed(growth)?;
        admit_fixed(checked_mul(*weight, growth)?)?;
    }
    Ok(())
}

fn admit_price_exponentials(
    prices: &[NonNegativeFixed; N],
    costs: &[NonNegativeFixed; N],
) -> Result<(), StabilityRefusal> {
    for (&price, &cost) in prices.iter().zip(costs) {
        let product = checked_mul(price, cost)?;
        if product.val > i32::MAX as u32 {
            return Err(StabilityRefusal::NumericRangeExceeded);
        }
        let exponent = SignedFixed {
            val: 0i32.wrapping_sub(product.val as i32),
            err: product.err,
        };
        admit_fixed(exponent.exp())?;
    }
    Ok(())
}

fn checked_add(
    left: NonNegativeFixed,
    right: NonNegativeFixed,
) -> Result<NonNegativeFixed, StabilityRefusal> {
    let value = left.saturating_add(right);
    admit_fixed(value)?;
    Ok(value)
}

fn checked_mul(
    left: NonNegativeFixed,
    right: NonNegativeFixed,
) -> Result<NonNegativeFixed, StabilityRefusal> {
    let value = left.saturating_mul(right);
    admit_fixed(value)?;
    Ok(value)
}

fn checked_clip(
    value: NonNegativeFixed,
    minimum: NonNegativeFixed,
    maximum: NonNegativeFixed,
) -> Result<NonNegativeFixed, StabilityRefusal> {
    admit_fixed(value)?;
    Ok(if value.val < minimum.val {
        NonNegativeFixed {
            val: minimum.val,
            err: value.err,
        }
    } else if value.val > maximum.val {
        NonNegativeFixed {
            val: maximum.val,
            err: value.err,
        }
    } else {
        value
    })
}

fn admit_fixed(value: NonNegativeFixed) -> Result<(), StabilityRefusal> {
    if value.err == OK {
        Ok(())
    } else {
        Err(StabilityRefusal::from_u32(value.err).unwrap_or(StabilityRefusal::ContractViolation))
    }
}

fn admit_signed(value: SignedFixed) -> Result<(), StabilityRefusal> {
    if value.err == OK {
        Ok(())
    } else {
        Err(StabilityRefusal::from_u32(value.err).unwrap_or(StabilityRefusal::ContractViolation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisoned_values_refuse_before_kernel_execution() {
        let value = NonNegativeFixed {
            val: 1,
            err: StabilityRefusal::NumericRangeExceeded as u32,
        };
        assert_eq!(
            admit_fixed(value),
            Err(StabilityRefusal::NumericRangeExceeded)
        );
    }

    #[test]
    fn normalized_spread_boundary_is_profile_derived() {
        let masses = [NonNegativeFixed::ONE; N];
        assert!(admit_group(0..N, &masses, SignedFixed::from_num(2)).is_ok());
    }
}
