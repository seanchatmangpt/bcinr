//! [`FeasibleRegion`]: `allocate_in`'s input bounds, extracted from inline
//! constants without changing allocation behavior. See `super::allocate_in`'s
//! docs for how each bound is actually used, and
//! [`FeasibleRegion::contains_allocation`]'s docs for why this is not also a
//! universally-enforced output contract.

use crate::fixed::NonNegativeFixed;
use crate::generated::consequence_mass::case_studies::N;

use super::StabilityRefusal;

/// The numeric bounds `allocate_in` clamps or gates against, named rather
/// than inline. Two different kinds of bound, not one region:
///
/// `beta_max`, `m_min`, `m_max` are **clamp targets** -- `allocate_in`
/// applies them unconditionally (`beta = min(zeta, beta_max)`,
/// `node_masses[k][i] = clip(node_masses[k][i], m_min, m_max)`). There is no
/// input value for these three that produces a refusal instead of a silent
/// reshape; that has always been true of this function and this checkpoint
/// does not change it.
///
/// `mu_max` is the one bound among these four that already gates: any
/// `mu[i] > mu_max` sets `price_err`, which folds into `has_error` and
/// ultimately produces [`StabilityRefusal::PriceGainUnsafe`] (confirmed by
/// reading `allocate_in`'s `err_val` selection chain -- its final fallback
/// slot, index 7, *is* `PriceGainUnsafe`, not an unrelated default).
///
/// [`FeasibleRegion::admit_inputs`] therefore only checks `mu` against
/// `mu_max`: it is the only one of the four with an admission decision to
/// make. [`FeasibleRegion::contains_allocation`] checks a *related but
/// separate* output property -- see its own docs for why it is not the
/// universal postcondition an earlier version of this type assumed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FeasibleRegion {
    pub(super) beta_max: NonNegativeFixed,
    pub(super) m_min: NonNegativeFixed,
    pub(super) m_max: NonNegativeFixed,
    pub(super) mu_max: NonNegativeFixed,
}

impl FeasibleRegion {
    /// The bounds `allocate()` has always used, extracted rather than
    /// changed. `allocate(x) == allocate_in(&FeasibleRegion::CURRENT, x)`
    /// for every input, by construction: `allocate` is defined as that call.
    pub const CURRENT: Self = Self {
        beta_max: NonNegativeFixed::from_bits(6553),
        m_min: NonNegativeFixed::from_bits(6),
        m_max: NonNegativeFixed::from_bits(65536000),
        mu_max: NonNegativeFixed::from_bits(6553600),
    };

    /// Absolute tolerance `contains_allocation` allows on the sum, matching
    /// the 1% figure `usecase_trading_determinism.rs`'s ad hoc test used
    /// before this contract existed (`NonNegativeFixed::ONE.val / 100`).
    const SUM_TOLERANCE_BITS: u32 = NonNegativeFixed::ONE.val / 100;

    /// Checks the one input bound among these four that `allocate_in`
    /// actually refuses on (see the struct docs): every `mu[i]` must not
    /// exceed `mu_max`. Mirrors `allocate_in`'s own `price_err` check
    /// exactly (`mu_max.val < mu[i].val`, strict), so a caller can decide
    /// not to call `allocate_in` at all rather than call it and discard an
    /// `Err`.
    pub fn admit_inputs(&self, mu: &[NonNegativeFixed; N]) -> Result<(), StabilityRefusal> {
        for &m in mu {
            if self.mu_max.val < m.val {
                return Err(StabilityRefusal::PriceGainUnsafe);
            }
        }
        Ok(())
    }

    /// Checks: every component nonnegative (guaranteed by
    /// `NonNegativeFixed`'s type, not checked here), none exceeds `ONE`, and
    /// the sum equals `ONE` within [`Self::SUM_TOLERANCE_BITS`].
    ///
    /// **Not a universal postcondition of `allocate_in`.** It was written as
    /// one, wired into `allocate_in` as an automatic refusal gate, and that
    /// broke a real, correct, pre-existing test:
    /// `hostile_mutants.rs::verify_correctness_baselines`'s
    /// `CORRECT_MU_COST = [4096; N]` is a legitimate `allocate()` output
    /// (large `mu`/`costs` collapse `priced_sum` to zero, which forces the
    /// fallback `psd = ONE` branch -- the leaf sum in that branch is not
    /// renormalized to `ONE`, by design). Checkpoint A's plan explicitly
    /// forbids changing allocation behavior, so the automatic gate was
    /// reverted; this method is now a caller-invoked check for the
    /// non-degenerate regime (`priced_sum != 0`, the common case, e.g. the
    /// scenario `usecase_trading_determinism.rs`'s ad hoc 1%-tolerance
    /// assertion covers) -- call it when you know your inputs are in that
    /// regime, not as a blanket assumption about every `Ok` result.
    pub fn contains_allocation(&self, allocation: &[NonNegativeFixed; N]) -> bool {
        let mut sum: u64 = 0;
        for &a in allocation {
            if a.val > NonNegativeFixed::ONE.val {
                return false;
            }
            sum += a.val as u64;
        }
        let target = NonNegativeFixed::ONE.val as u64;
        sum.abs_diff(target) <= Self::SUM_TOLERANCE_BITS as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The normalized regime: components summing to (approximately) `ONE`.
    #[test]
    fn contains_allocation_true_for_a_normalized_distribution() {
        let allocation: [NonNegativeFixed; N] =
            core::array::from_fn(|_| NonNegativeFixed::from_bits(65536 / N as u32));
        assert!(FeasibleRegion::CURRENT.contains_allocation(&allocation));
    }

    /// The degenerate regime this method's docs cite by name:
    /// `hostile_mutants.rs::CORRECT_MU_COST`, a real, correct `allocate()`
    /// output whose sum is deliberately not close to `ONE` (large
    /// `mu`/`costs` collapse `priced_sum` to zero, forcing the
    /// non-renormalized `psd = ONE` fallback branch). `contains_allocation`
    /// must say `false` here -- if it said `true`, the method's own docs
    /// about what it does and does not guarantee would be wrong.
    #[test]
    fn contains_allocation_false_for_the_documented_degenerate_baseline() {
        const CORRECT_MU_COST: [u32; N] = [4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096];
        let allocation: [NonNegativeFixed; N] =
            core::array::from_fn(|i| NonNegativeFixed::from_bits(CORRECT_MU_COST[i]));
        assert!(!FeasibleRegion::CURRENT.contains_allocation(&allocation));
    }

    /// [`FeasibleRegion::admit_inputs`] mirrors `allocate_in`'s own
    /// `price_err` check: `mu_max` is a strict ceiling (`mu_max.val < m.val`
    /// refuses; `mu.val == mu_max.val` is admitted).
    #[test]
    fn admit_inputs_refuses_mu_strictly_above_mu_max() {
        let mut mu = [NonNegativeFixed::ZERO; N];
        mu[3] = NonNegativeFixed::from_bits(FeasibleRegion::CURRENT.mu_max.val + 1);
        assert_eq!(
            FeasibleRegion::CURRENT.admit_inputs(&mu),
            Err(StabilityRefusal::PriceGainUnsafe)
        );
    }

    #[test]
    fn admit_inputs_admits_mu_at_or_below_mu_max() {
        let mut mu = [NonNegativeFixed::ZERO; N];
        mu[3] = FeasibleRegion::CURRENT.mu_max;
        assert!(FeasibleRegion::CURRENT.admit_inputs(&mu).is_ok());
    }
}
