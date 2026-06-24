//! conformance — Q16.16 fixed-point conformance metrics and branchless predicate checking.
//!
//! All numeric values are Q16.16 fixed-point: `0x0001_0000` == 1.0.
//! The `mask_ge` comparison is derived from the B-Calculus `lt_mask_u32` pattern in
//! `bcinr-logic/src/mask.rs`: produce all-ones when `a >= b`, all-zeros otherwise,
//! with no conditional branch.

/// Q16.16 fixed-point conformance metrics.
///
/// Value encoding: `0x0001_0000` == 1.0, `0x8000_0000` == 0.5, `0x0000_0000` == 0.0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceMetrics {
    pub fitness: u32,
    pub precision: u32,
    pub generalization: u32,
    pub simplicity: u32,
}

/// Threshold predicate for conformance checking.
///
/// Each field is a Q16.16 minimum (inclusive). `check` returns `Ok(())` iff every
/// dimension of the supplied [`ConformanceMetrics`] meets its threshold — all
/// comparisons are branchless.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformancePredicate {
    pub min_fitness: u32,
    pub min_precision: u32,
    pub min_generalization: u32,
    pub min_simplicity: u32,
}

impl ConformancePredicate {
    /// Strict predicate: all four dimensions must be ≥ 1.0 (fitness/precision) or ≥ 0.5
    /// (generalization/simplicity).
    pub const STRICT: Self = Self {
        min_fitness: 0xFFFF_0000,
        min_precision: 0xFFFF_0000,
        min_generalization: 0x8000_0000,
        min_simplicity: 0x8000_0000,
    };

    /// Lenient predicate: all four dimensions must be ≥ 0.5 (fitness/precision) or ≥ 0.25
    /// (generalization/simplicity).
    pub const LENIENT: Self = Self {
        min_fitness: 0x8000_0000,
        min_precision: 0x8000_0000,
        min_generalization: 0x4000_0000,
        min_simplicity: 0x4000_0000,
    };

    /// Branchless conformance check.
    ///
    /// Returns the first dimension that fails its threshold.  All four comparisons
    /// use `mask_ge` so the CPU never branches on metric values — only the final
    /// `select_u32`-style reduction materialises the error path.
    pub fn check(&self, m: &ConformanceMetrics) -> Result<(), ConformanceViolation> {
        // Branchless ≥ comparisons — all four computed unconditionally.
        let fit_ok = mask_ge(m.fitness, self.min_fitness);
        let pre_ok = mask_ge(m.precision, self.min_precision);
        let gen_ok = mask_ge(m.generalization, self.min_generalization);
        let sim_ok = mask_ge(m.simplicity, self.min_simplicity);

        // Combine: all-ones means every dimension passed.
        // We report the first failure in enum discriminant order.
        // Use branchless select to encode "which dimension failed first",
        // then a single branch on the aggregate to stay panic-free.
        let all_pass = fit_ok & pre_ok & gen_ok & sim_ok;

        if all_pass == 0xFFFF_FFFF {
            return Ok(());
        }

        // Determine the first failing dimension using branchless priority encoding.
        // Priority: Fitness > Precision > Generalization > Simplicity.
        // select_u32(mask, if_true, if_false) from mask.rs: (mask & a) | (!mask & b).
        let dim = branchless_first_failure(fit_ok, pre_ok, gen_ok, sim_ok);

        Err(ConformanceViolation {
            dim,
            measured: m.clone(),
        })
    }
}

/// Branchless `a >= b` comparison for `u32`.
///
/// Maps from the B-Calculus `lt_mask_u32` pattern:
/// `lt_mask_u32(a, b)` produces all-ones when `a < b`.
/// Negating: `ge_mask = !lt_mask`.
///
/// `lt_mask_u32(a, b)`: the borrow bit of `a.wrapping_sub(b)` is 1 iff `a < b`.
/// Broadcast the borrow to all bits via arithmetic right shift (or negation).
#[inline(always)]
pub fn mask_ge(a: u32, b: u32) -> u32 {
    // Widen to i64 to capture the sign of (a as i64) - (b as i64).
    // Since a, b <= 0xFFFF_FFFF, the 64-bit signed difference is in
    // [-0xFFFF_FFFF, 0xFFFF_FFFF]; it is negative iff a < b.
    // Arithmetic right shift by 63 broadcasts the sign bit to all bits,
    // yielding -1 (all ones) when a < b, 0 otherwise.
    let lt_mask = (((a as i64) - (b as i64)) >> 63) as u32;
    // ge_mask = !lt_mask: all-ones iff a >= b.
    !lt_mask
}

/// Encode the first failing dimension as a `ConformanceDimension` without branching
/// on the *value* of metrics — only on bitmasks.
#[inline(always)]
fn branchless_first_failure(
    fit_ok: u32,
    pre_ok: u32,
    gen_ok: u32,
    _sim_ok: u32,
) -> ConformanceDimension {
    // Use select_u32 pattern: (mask & a) | (!mask & b).
    // Priority: Fitness first.
    // We build a discriminant u32 branchlessly, then transmute via exhaustive match.
    //
    // Discriminant encoding: Fitness=0, Precision=1, Generalization=2, Simplicity=3.
    // If fitness failed  → 0
    // Else if precision  → 1
    // Else if gen        → 2
    // Else               → 3
    let d_if_fit_pass = select_u32(pre_ok, select_u32(gen_ok, 3, 2), 1);
    let d = select_u32(fit_ok, d_if_fit_pass, 0);

    match d {
        0 => ConformanceDimension::Fitness,
        1 => ConformanceDimension::Precision,
        2 => ConformanceDimension::Generalization,
        _ => ConformanceDimension::Simplicity,
    }
}

/// B-Calculus select: returns `a` when `mask == 0xFFFF_FFFF`, `b` when `mask == 0`.
#[inline(always)]
const fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}

/// A conformance violation: records which dimension failed and the full measured metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceViolation {
    pub dim: ConformanceDimension,
    pub measured: ConformanceMetrics,
}

/// Conformance dimension discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceDimension {
    Fitness = 0,
    Precision = 1,
    Generalization = 2,
    Simplicity = 3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_ge_all_ones_when_equal() {
        assert_eq!(mask_ge(5, 5), 0xFFFF_FFFF);
    }

    #[test]
    fn mask_ge_all_ones_when_greater() {
        assert_eq!(mask_ge(10, 5), 0xFFFF_FFFF);
    }

    #[test]
    fn mask_ge_all_zeros_when_less() {
        assert_eq!(mask_ge(4, 5), 0x0000_0000);
    }

    #[test]
    fn strict_predicate_passes_at_threshold() {
        let m = ConformanceMetrics {
            fitness: 0xFFFF_0000,
            precision: 0xFFFF_0000,
            generalization: 0x8000_0000,
            simplicity: 0x8000_0000,
        };
        assert!(ConformancePredicate::STRICT.check(&m).is_ok());
    }

    #[test]
    fn strict_predicate_fails_on_precision() {
        let m = ConformanceMetrics {
            fitness: 0xFFFF_0000,
            precision: 0x7000_0000, // below 0xFFFF_0000
            generalization: 0x8000_0000,
            simplicity: 0x8000_0000,
        };
        let err = ConformancePredicate::STRICT.check(&m).unwrap_err();
        assert_eq!(err.dim, ConformanceDimension::Precision);
    }
}
