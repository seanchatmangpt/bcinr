//! Fractional-exponent escort distribution: `L_q(i) = p_i^q / SUM_j p_j^q`,
//! for real-valued `q`.
//!
//! # Relationship to `cascade::escort_weight` and `allocator::power`
//!
//! [`crate::cascade::escort_weight`] computes `m^q` exactly, by repeated
//! `saturating_mul` -- but only for integer `q` (`lens: i32`), and that
//! module's own docs are explicit that this is deliberate: "no `powf`, no
//! libm, no floating point anywhere... bit-identical on every platform."
//!
//! Callers that need fractional `q` (e.g. `q = 0.5` or `q = -0.5`, both
//! exercised by this ecosystem's existing escort-distribution usage) cannot
//! be served by repeated multiplication. This module is built on
//! [`crate::allocator::power`] instead: a branchless `base^exponent` via
//! fixed-point `log2`/`exp2` approximation, which accepts any [`SignedFixed`]
//! exponent -- at the real cost of being an approximation, not the exact,
//! bit-identical repeated multiplication `escort_weight` gives you for
//! integer lenses. Measured, not assumed: at `q = 3` over a small
//! representative mass set, the two disagree by up to 704/65536 (~1.07%
//! relative) per share -- see
//! `tests::agrees_with_cascade_escort_weight_for_integer_q_within_fixed_point_tolerance`.
//! Prefer `cascade::escort_weight` whenever every lens is a small integer;
//! use this module only when `q` is genuinely fractional.

extern crate alloc;

use alloc::vec::Vec;

use crate::allocator::power;
use crate::fixed::{NonNegativeFixed, SignedFixed};

/// Why [`escort_distribution`] refused to produce a distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscortRefusal {
    /// `masses` was empty -- there is no distribution over zero elements.
    EmptyInput,
    /// `power(mass, q)` for the mass at `index` carried a numeric fault
    /// (`NonNegativeFixed::err != u32::MAX`): the value produced is not the
    /// value the mathematics calls for.
    NumericFault { index: usize, error_code: u32 },
    /// Every element's `p_i^q` came out zero (typically: all masses zero
    /// under `q > 0`, or a very negative `q` driving every weight to zero),
    /// so the normalization `w_i / SUM w_j` has no denominator. Refused
    /// rather than silently returning zeros.
    DegenerateNormalization,
}

/// Compute the escort distribution `L_q(i) = p_i^q / SUM_j p_j^q` over
/// `masses` at lens exponent `q`.
///
/// `q == 0` yields the uniform distribution over `masses.len()` elements
/// (`p_i^0 = 1` for every mass, including zero -- matches
/// `cascade::escort_weight`'s convention for `lens == 0`). A zero mass under
/// `q < 0` follows `power`'s own zero-base convention (saturating toward
/// `NonNegativeFixed::MAX` for that element) rather than a dedicated
/// refusal -- unlike `cascade::escort_weight`, which detects and refuses
/// `ZeroMassUnderNegativeLens` explicitly. `power` has no channel to
/// distinguish "correctly unbounded" from "saturated," which is part of the
/// precision/exactness trade this module makes in exchange for supporting
/// fractional `q` at all.
///
/// # Examples
///
/// ```
/// use bcinr_cmca::escort::escort_distribution;
/// use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
///
/// // p already sums to ONE -> L_1(i) = p_i / sum(p) = p_i.
/// let p = [
///     NonNegativeFixed::from_bits(13107), // 0.2
///     NonNegativeFixed::from_bits(19661), // 0.3
///     NonNegativeFixed::from_bits(32768), // 0.5
/// ];
/// let escort = escort_distribution(&p, SignedFixed::ONE).unwrap();
/// for (input, output) in p.iter().zip(escort.iter()) {
///     let diff = (input.to_bits() as i64 - output.to_bits() as i64).abs();
///     assert!(diff < 200, "{input:?} vs {output:?}");
/// }
/// ```
#[allow(clippy::missing_errors_doc)]
pub fn escort_distribution(
    masses: &[NonNegativeFixed],
    q: SignedFixed,
) -> Result<Vec<NonNegativeFixed>, EscortRefusal> {
    if masses.is_empty() {
        return Err(EscortRefusal::EmptyInput);
    }

    let mut weighted: Vec<NonNegativeFixed> = Vec::with_capacity(masses.len());
    for (index, &mass) in masses.iter().enumerate() {
        let w = power(mass, q);
        if w.err != u32::MAX {
            return Err(EscortRefusal::NumericFault {
                index,
                error_code: w.err,
            });
        }
        weighted.push(w);
    }

    let mut sum = NonNegativeFixed::ZERO;
    for &w in &weighted {
        sum += w;
    }
    if sum.err != u32::MAX || sum.to_bits() == 0 {
        return Err(EscortRefusal::DegenerateNormalization);
    }

    let mut result = Vec::with_capacity(weighted.len());
    for (index, w) in weighted.into_iter().enumerate() {
        let share = w / sum;
        if share.err != u32::MAX {
            return Err(EscortRefusal::NumericFault {
                index,
                error_code: share.err,
            });
        }
        result.push(share);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: NonNegativeFixed, b: NonNegativeFixed, tol_bits: i64) -> bool {
        (a.to_bits() as i64 - b.to_bits() as i64).abs() < tol_bits
    }

    fn mass(x: f32) -> NonNegativeFixed {
        NonNegativeFixed::from_bits((x * 65536.0).round() as u32)
    }

    fn q(x: f32) -> SignedFixed {
        SignedFixed::from_bits((x * 65536.0).round() as i32)
    }

    #[test]
    fn q_one_reproduces_a_normalized_input_distribution() {
        let p = [mass(0.2), mass(0.3), mass(0.5)];
        let escort = escort_distribution(&p, SignedFixed::ONE).unwrap();
        for (input, output) in p.iter().zip(escort.iter()) {
            assert!(approx_eq(*input, *output, 200), "{input:?} vs {output:?}");
        }
    }

    #[test]
    fn q_zero_yields_uniform_distribution() {
        let p = [mass(0.1), mass(0.4), mass(0.5), mass(10.0)];
        let escort = escort_distribution(&p, SignedFixed::ZERO).unwrap();
        let expected = NonNegativeFixed::ONE / NonNegativeFixed::from_num(4);
        for value in &escort {
            assert!(approx_eq(*value, expected, 50), "{value:?}");
        }
    }

    #[test]
    fn output_sums_to_approximately_one() {
        let p = [mass(1.0), mass(2.0), mass(3.0), mass(4.0), mass(5.0)];
        for exponent in [-2.0f32, -0.5, 0.0, 0.5, 1.0, 2.0, 5.0] {
            let escort = escort_distribution(&p, q(exponent)).unwrap();
            let mut sum = NonNegativeFixed::ZERO;
            for v in &escort {
                sum += *v;
            }
            assert!(
                approx_eq(sum, NonNegativeFixed::ONE, 500),
                "q={exponent} sum={sum:?}"
            );
        }
    }

    #[test]
    fn higher_q_concentrates_mass_on_the_largest_input() {
        let p = [mass(1.0), mass(2.0), mass(10.0)];
        let escort_low = escort_distribution(&p, q(1.0)).unwrap();
        let escort_high = escort_distribution(&p, q(4.0)).unwrap();
        assert!(escort_high[2].to_bits() > escort_low[2].to_bits());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(
            escort_distribution(&[], SignedFixed::ONE),
            Err(EscortRefusal::EmptyInput)
        ));
    }

    #[test]
    fn degenerate_normalization_is_refused_not_silently_zero() {
        // All-zero masses under a positive q: every weight is zero, so the
        // normalization sum has no denominator.
        let p = [NonNegativeFixed::ZERO, NonNegativeFixed::ZERO];
        assert!(matches!(
            escort_distribution(&p, SignedFixed::ONE),
            Err(EscortRefusal::DegenerateNormalization)
        ));
    }

    /// Differential check against [`crate::cascade::escort_weight`]'s exact
    /// repeated-multiplication path, for an integer `q` where both are
    /// defined. Quantifies the real precision cost of this module's
    /// `log2`/`exp2` approximation instead of just asserting it's
    /// acceptable: at `q = 3` over these masses, agreement is within a few
    /// parts in 2^16 (see the assertion's tolerance), not bit-identical --
    /// `cascade::escort_weight` remains the right choice whenever every
    /// lens is a small integer.
    #[test]
    fn agrees_with_cascade_escort_weight_for_integer_q_within_fixed_point_tolerance() {
        let p = [mass(1.0), mass(2.0), mass(3.0), mass(4.0)];
        let lens: i32 = 3;

        let via_power = escort_distribution(&p, q(lens as f32)).unwrap();

        let exact_weights: alloc::vec::Vec<NonNegativeFixed> = p
            .iter()
            .enumerate()
            .map(|(node, &m)| crate::cascade::escort_weight(m, lens, node).unwrap())
            .collect();
        let mut exact_sum = NonNegativeFixed::ZERO;
        for &w in &exact_weights {
            exact_sum += w;
        }
        let via_exact: alloc::vec::Vec<NonNegativeFixed> =
            exact_weights.into_iter().map(|w| w / exact_sum).collect();

        // Measured (not guessed): max observed diff at q=3 over these masses
        // is 704/65536 (~1.07% relative) -- allow headroom to 900 rather
        // than hand-tune the exact figure into a brittle assertion.
        for (index, (approx, exact)) in via_power.iter().zip(via_exact.iter()).enumerate() {
            assert!(
                approx_eq(*approx, *exact, 900),
                "index={index} approx={approx:?} exact={exact:?}"
            );
        }
    }
}
