//! Fractional-exponent escort distribution: `L_q(i) = p_i^q / SUM_j p_j^q`,
//! for real-valued `q`.
//!
//! # Ancestry
//!
//! This normalized-power construction is not novel to this crate. It is an
//! *escort mapping* -- see Harper, "Escort Evolutionary Game Theory"
//! (Physica D, 2009), which develops escort maps `phi(x) = x^q` over the
//! simplex, escort replicator dynamics, and the `q == 0` orthogonal-projection
//! special case -- and the exponent `q` plays the role of a multifractal
//! partition-sum exponent, per Halsey, Jensen, Kadanoff, Procaccia & Shraiman,
//! "Fractal measures and their singularities" (Phys. Rev. A, 1986), where
//! varying `q` shifts which subset of a measure dominates a partition sum
//! `chi(q) = sum_i p_i^q`.
//!
//! What this crate adds on top of that ancestry: a fixed five-lens reference
//! profile (not yet formalized -- see `~/mfw`'s planned
//! `MFW/CMCA/ReferenceEscort.lean`), typed refusals instead of silent
//! degradation, a fixed-point (not floating-point) realization, an explicit
//! declared lens domain (below), integration with [`crate::cascade`]'s
//! hierarchical allocation, and a still-open, not-yet-resolved semantic
//! decision between *support coverage* (uniform over positive-mass support
//! only, excluding zero-mass elements) and *sibling coverage* (uniform over
//! every eligible sibling, zero-mass included). **Current BCINR behavior is
//! sibling coverage**: [`crate::cascade::escort_weight`]'s `lens == 0`
//! branch returns `NonNegativeFixed::ONE` unconditionally, regardless of
//! mass, so a zero-mass sibling gets the same weight as every other one.
//! This module's current behavior is not a claim that it already conforms
//! to whatever `~/mfw`'s Lean crown ultimately settles for `q == 0` -- if
//! that crown settles on support coverage instead, this is the exact
//! behavior that migrates, not something already aligned with it. The
//! citation above documents
//! mathematical ancestry, not completed conformance.
//!
//! # Relationship to `cascade::escort_weight` and `allocator::power`
//!
//! [`crate::cascade::escort_weight`] computes `m^q` exactly, by repeated
//! `saturating_mul` -- but only for integer `q` (`lens: i32`), and that
//! module's own docs are explicit that this is deliberate: "no `powf`, no
//! libm, no floating point anywhere... bit-identical on every platform."
//! [`escort_distribution`] now dispatches to it automatically whenever `q`
//! has no fractional part: an integer lens never reaches the approximate
//! path, regardless of which entry point a caller uses.
//!
//! [`crate::allocator::power`] remains this module's fallback for genuinely
//! fractional `q` (e.g. `q = 0.5` or `q = -0.5`) -- a branchless
//! `base^exponent` via fixed-point `log2`/`exp2` approximation, at the real
//! cost of being an approximation, not the exact, bit-identical repeated
//! multiplication `escort_weight` gives you for integer lenses. Measured,
//! not assumed: at `q = 3` over a small representative mass set, the two
//! disagree by up to 704/65536 (~1.07% relative) per share -- see
//! `tests::power_disagrees_with_the_exact_path_at_a_measured_bound`.
//!
//! # Declared lens domain
//!
//! [`escort_distribution`] refuses any `q` whose magnitude exceeds
//! [`crate::cascade::MAX_LENS_MAGNITUDE`] -- the same bound
//! `cascade::escort_weight` enforces for integer lenses, now applied to
//! fractional `q` too. This is a provisional domain, not one derived from a
//! specification: before this gate existed, this module silently accepted
//! any `SignedFixed` and either produced an answer or saturated without
//! signaling it. Expect this bound to be superseded once `~/mfw`'s
//! `ReferenceLens` fixes the admitted domain formally.

extern crate alloc;

use alloc::vec::Vec;

use crate::allocator::power;
use crate::cascade::{self, CascadeRefusal, MAX_LENS_MAGNITUDE};
use crate::fixed::{NonNegativeFixed, SignedFixed};

/// Why [`escort_distribution`] refused to produce a distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscortRefusal {
    /// `masses` was empty -- there is no distribution over zero elements.
    EmptyInput,
    /// `q`'s magnitude exceeded [`MAX_LENS_MAGNITUDE`] (see the module-level
    /// "Declared lens domain" section).
    UnsupportedLens { lens: SignedFixed },
    /// `power(mass, q)` for the mass at `index` carried a numeric fault
    /// (`NonNegativeFixed::err != u32::MAX`): the value produced is not the
    /// value the mathematics calls for. Only reachable for genuinely
    /// fractional `q` -- integer `q` is routed to `cascade::escort_weight`
    /// instead, see [`EscortRefusal::ExactPathRefused`].
    NumericFault { index: usize, error_code: u32 },
    /// `q` was an exact integer within `cascade::escort_weight`'s domain,
    /// and the exact path refused for the mass at `index`. Carries the
    /// original [`CascadeRefusal`] rather than collapsing it into
    /// `NumericFault`, since `CascadeRefusal` already distinguishes several
    /// refusal shapes (underflow, zero mass under a negative lens, ...)
    /// worth keeping intact.
    ExactPathRefused { index: usize, reason: CascadeRefusal },
    /// Every element's `p_i^q` came out zero (typically: all masses zero
    /// under `q > 0`, or a very negative `q` driving every weight to zero),
    /// so the normalization `w_i / SUM w_j` has no denominator. Refused
    /// rather than silently returning zeros.
    DegenerateNormalization,
}

/// `q`'s integer value, if `q` has no fractional part -- `None` for
/// genuinely fractional `q`. Q16.16: the low 16 bits are the fractional
/// part, so "no fractional part" is exactly "those bits are zero."
#[inline]
fn exact_integer_lens(q: SignedFixed) -> Option<i32> {
    if q.to_bits() & 0xFFFF == 0 {
        Some(q.to_num())
    } else {
        None
    }
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

    // Declared domain check, once per call (a property of `q` alone, not of
    // any one mass). `unsigned_abs` handles `i32::MIN` correctly, unlike a
    // signed `abs()`.
    if q.to_bits().unsigned_abs() > MAX_LENS_MAGNITUDE << 16 {
        return Err(EscortRefusal::UnsupportedLens { lens: q });
    }
    let exact_lens = exact_integer_lens(q);

    let mut weighted: Vec<NonNegativeFixed> = Vec::with_capacity(masses.len());
    for (index, &mass) in masses.iter().enumerate() {
        let w = if let Some(lens) = exact_lens {
            cascade::escort_weight(mass, lens, index)
                .map_err(|reason| EscortRefusal::ExactPathRefused { index, reason })?
        } else {
            let w = power(mass, q);
            if w.err != u32::MAX {
                return Err(EscortRefusal::NumericFault {
                    index,
                    error_code: w.err,
                });
            }
            w
        };
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

    /// `escort_distribution` no longer calls `power` for an integer lens
    /// (see `exact_lens_never_reaches_the_approximate_path` below), so this
    /// probes the raw primitives directly: it quantifies the real precision
    /// cost of `power`'s `log2`/`exp2` approximation against
    /// `cascade::escort_weight`'s exact repeated multiplication, instead of
    /// just asserting the cost is acceptable. This is diagnostic evidence
    /// about `power`'s approximation error, not a conformance check --
    /// pending `~/mfw`'s generated reference vectors, it is the closest
    /// thing this module has to one.
    #[test]
    fn power_disagrees_with_the_exact_path_at_a_measured_bound() {
        let p = [mass(1.0), mass(2.0), mass(3.0), mass(4.0)];
        let lens: i32 = 3;

        let power_weights: alloc::vec::Vec<NonNegativeFixed> =
            p.iter().map(|&m| power(m, q(lens as f32))).collect();
        let mut power_sum = NonNegativeFixed::ZERO;
        for &w in &power_weights {
            power_sum += w;
        }
        let via_power: alloc::vec::Vec<NonNegativeFixed> =
            power_weights.into_iter().map(|w| w / power_sum).collect();

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

    /// The module-level claim this test file exists to hold: an integer `q`
    /// passed through the public `escort_distribution` entry point produces
    /// the *exact* `cascade::escort_weight` result, not `power`'s
    /// approximation of it -- bit-identical, not merely within tolerance.
    #[test]
    fn exact_lens_never_reaches_the_approximate_path() {
        let p = [mass(1.0), mass(2.0), mass(3.0), mass(4.0)];
        let lens: i32 = 3;

        let via_dispatch = escort_distribution(&p, q(lens as f32)).unwrap();

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

        assert_eq!(
            via_dispatch, via_exact,
            "an integer lens must reach the exact path bit-for-bit, not just within tolerance"
        );
    }

    #[test]
    fn lens_beyond_the_declared_magnitude_is_refused() {
        let p = [mass(1.0), mass(2.0)];
        let too_large = SignedFixed::from_num((MAX_LENS_MAGNITUDE + 1) as i32);
        assert_eq!(
            escort_distribution(&p, too_large),
            Err(EscortRefusal::UnsupportedLens { lens: too_large })
        );
    }

    #[test]
    fn lens_at_the_declared_magnitude_boundary_is_admitted() {
        // Masses close to 1.0: `mass^16` must not overflow the exact path's
        // fixed-point range, or this would test `escort_weight`'s own
        // overflow refusal instead of the domain gate this test targets.
        let p = [mass(0.9), mass(1.0)];
        let at_bound = SignedFixed::from_num(MAX_LENS_MAGNITUDE as i32);
        assert!(escort_distribution(&p, at_bound).is_ok());
    }
}
