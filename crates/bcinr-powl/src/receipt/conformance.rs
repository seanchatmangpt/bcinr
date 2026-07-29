//! Conformance metrics and branchless predicate checking for POWL process replay.
//!
//! This module provides the types and methods for evaluating conformance metrics of a
//! replayed process trace against a process model. All conformance values are represented
//! in **Q16.16 fixed-point format**, where `0x0001_0000` corresponds to `1.0`.
//!
//! # Conformance Dimensions
//!
//! Conformance is evaluated across four distinct dimensions:
//!
//! 1. **Fitness ($F$):** Measures the degree to which the trace can be replayed by the model
//!    without transitions firing out of order or failing enablement.
//!    $$F = \frac{\text{Fitted Nodes}}{\text{Replayed Nodes}}$$
//! 2. **Precision ($P$):** Measures the degree to which the model does not allow behaviors
//!    unobserved in the trace (minimizing underfitting).
//!    $$P = \frac{\text{Replayed Nodes}}{\text{Replayed Nodes} + \text{Tokens Enabled but Not Taken}}$$
//! 3. **Generalization ($G$):** Evaluates the model's capability to generalize to unseen,
//!    yet consistent, behaviors rather than strictly overfitting to the observed trace.
//!    This is calculated using a Real Conformance Metric Estimator (RCME) formula:
//!    $$G = 1.0 - \text{clamp}\left(\frac{N_{\text{unique}}}{L + T_{\text{not\_taken}} + 1}, 0.0, 1.0\right)$$
//!    where $N_{\text{unique}}$ is the number of unique replayed nodes, $L$ is the tape length (total replayed events),
//!    and $T_{\text{not\_taken}}$ is the number of tokens that were enabled but never consumed (the count of non-taken options).
//! 4. **Simplicity ($S$):** Measures the simplicity of the model's structure under the observed replay,
//!    penalizing overly complex structures and unused transitions.
//!    This is calculated using a Real Conformance Metric Estimator (RCME) formula:
//!    $$S = \frac{K}{N_{\text{unique}} + T_{\text{not\_taken}} + T_{\text{active}} + K}$$
//!    where $K = 8$ is a scaling constant, and $T_{\text{active}}$ is the count of active tokens remaining in the model at finalization.
//!
//! # Q16.16 Fixed-Point Representation
//!
//! Fixed-point numbers are represented as `u32` integers:
//! - `0x0001_0000` represents $1.0$ (maximum value).
//! - `0x0000_8000` represents $0.5$.
//! - `0x0000_0000` represents $0.0$.
//!
//! This representation avoids floating-point operations at runtime, conforming to the
//! zero-allocation, branchless requirements of the BCINR substrate.
//!
//! # Branchless Predicate Checking
//!
//! Predicate checks utilize B-Calculus masks (`0xFFFF_FFFF` for true, `0x0000_0000` for false)
//! instead of conditional branching (`if` / `else`) to achieve constant-time, branchless evaluation.
//!
//! # Examples
//!
//! ```
//! use bcinr_powl::receipt::conformance::{ConformanceMetrics, ConformancePredicate};
//!
//! // Create a metrics report
//! let metrics = ConformanceMetrics {
//!     fitness: 0x0001_0000,        // 1.0 (perfect fitness)
//!     precision: 0x0001_0000,      // 1.0 (perfect precision)
//!     generalization: 0x0000_8000, // 0.50
//!     simplicity: 0x0000_9000,     // 0.5625
//! };
//!
//! // Check metrics against the strict predicate
//! let result = ConformancePredicate::STRICT.check(&metrics);
//! assert!(result.is_ok());
//!
//! // If any metric falls below the threshold, a conformance violation is returned
//! let failing_metrics = ConformanceMetrics {
//!     fitness: 0x0000_8000,        // 0.50 (below strict threshold of 1.0)
//!     precision: 0x0001_0000,
//!     generalization: 0x0000_8000,
//!     simplicity: 0x0000_8000,
//! };
//! let result = ConformancePredicate::STRICT.check(&failing_metrics);
//! assert!(result.is_err());
//! ```

/// Q16.16 fixed-point conformance metrics.
///
/// Encoding:
/// - `0x0001_0000` == 1.0 (perfect conformance)
/// - `0x0000_8000` == 0.5
/// - `0x0000_0000` == 0.0 (no conformance)
///
/// All four dimensions are computed branchlessly during trace replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceMetrics {
    /// Fitness ($F$): The ratio of replayed events that fit the process model's allowed transitions.
    ///
    /// Range: `0x0000_0000` to `0x0001_0000` (inclusive).
    pub fitness: u32,
    /// Precision ($P$): The ratio of executed transitions to total enabled transitions during replay.
    ///
    /// Range: `0x0000_0000` to `0x0001_0000` (inclusive).
    pub precision: u32,
    /// Generalization ($G$): The RCME proxy estimator representing the model's capacity to generalize to unseen traces.
    ///
    /// Range: `0x0000_0000` to `0x0001_0000` (inclusive).
    pub generalization: u32,
    /// Simplicity ($S$): The RCME proxy estimator representing structural simplicity based on token passing.
    ///
    /// Range: `0x0000_0000` to `0x0001_0000` (inclusive).
    pub simplicity: u32,
}

/// Threshold predicate for conformance checking.
///
/// Contains the minimum acceptable values for the four conformance dimensions.
/// A predicate is met if each measured metric is greater than or equal to the minimum
/// threshold (inclusive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformancePredicate {
    /// Minimum acceptable fitness threshold (Q16.16).
    pub min_fitness: u32,
    /// Minimum acceptable precision threshold (Q16.16).
    pub min_precision: u32,
    /// Minimum acceptable generalization threshold (Q16.16).
    pub min_generalization: u32,
    /// Minimum acceptable simplicity threshold (Q16.16).
    pub min_simplicity: u32,
}

impl ConformancePredicate {
    /// Strict predicate: requiring perfect fitness and precision (`1.0`), and moderate
    /// generalization and simplicity (`0.5`).
    ///
    /// - Fitness $\ge 1.0$ (`0x0001_0000`)
    /// - Precision $\ge 1.0$ (`0x0001_0000`)
    /// - Generalization $\ge 0.5$ (`0x0000_8000`)
    /// - Simplicity $\ge 0.5$ (`0x0000_8000`)
    pub const STRICT: Self = Self {
        min_fitness: 0x0001_0000,
        min_precision: 0x0001_0000,
        min_generalization: 0x0000_8000,
        min_simplicity: 0x0000_8000,
    };

    /// Lenient predicate: allowing moderate fitness and precision (`0.5`), and low
    /// generalization and simplicity (`0.25`).
    ///
    /// - Fitness $\ge 0.5$ (`0x0000_8000`)
    /// - Precision $\ge 0.5$ (`0x0000_8000`)
    /// - Generalization $\ge 0.25$ (`0x0000_4000`)
    /// - Simplicity $\ge 0.25$ (`0x0000_4000`)
    pub const LENIENT: Self = Self {
        min_fitness: 0x0000_8000,
        min_precision: 0x0000_8000,
        min_generalization: 0x0000_4000,
        min_simplicity: 0x0000_4000,
    };

    /// Branchless conformance check.
    ///
    /// Compares the provided [`ConformanceMetrics`] against this predicate's thresholds.
    /// Returns `Ok(())` if all metrics meet or exceed the thresholds. Otherwise, returns a
    /// [`ConformanceViolation`] specifying the first failing dimension (in priority order:
    /// Fitness, Precision, Generalization, Simplicity).
    ///
    /// This function evaluates all dimensions unconditionally using branchless comparison
    /// (`mask_ge`) to prevent timing side channels. The final result is selected branchlessly,
    /// and a single branch is executed only to construct the error variant or return `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::receipt::conformance::{ConformanceMetrics, ConformancePredicate, ConformanceDimension};
    ///
    /// let metrics = ConformanceMetrics {
    ///     fitness: 0x0001_0000,        // 1.0 (meets STRICT threshold 1.0)
    ///     precision: 0x0000_C000,      // 0.75 (below STRICT threshold 1.0)
    ///     generalization: 0x0000_8000, // 0.5 (meets STRICT threshold 0.5)
    ///     simplicity: 0x0000_8000,     // 0.5 (meets STRICT threshold 0.5)
    /// };
    ///
    /// let result = ConformancePredicate::STRICT.check(&metrics);
    /// assert!(result.is_err());
    /// let violation = result.unwrap_err();
    /// assert_eq!(violation.dim, ConformanceDimension::Precision);
    /// ```
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
/// Returns `0xFFFF_FFFF` (all ones) if `a >= b`, and `0x0000_0000` (all zeros) if `a < b`.
///
/// This uses sign-bit broadcasting over 64-bit differences to perform the comparison without
/// conditional branching instructions, preventing timing-based leakage.
///
/// # Examples
///
/// ```
/// use bcinr_powl::receipt::conformance::mask_ge;
///
/// assert_eq!(mask_ge(10, 5), 0xFFFF_FFFF); // true (greater)
/// assert_eq!(mask_ge(5, 5), 0xFFFF_FFFF);  // true (equal)
/// assert_eq!(mask_ge(3, 5), 0x0000_0000);  // false (less)
/// ```
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

/// A conformance violation.
///
/// Created when a [`ConformancePredicate`] check fails. Contains the first dimension that
/// failed the threshold and the full measured metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceViolation {
    /// The first dimension that failed to meet the predicate threshold.
    pub dim: ConformanceDimension,
    /// The measured metrics at the time of the violation.
    pub measured: ConformanceMetrics,
}

/// Conformance dimension discriminant.
///
/// Represents one of the four conformance evaluation criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceDimension {
    /// Fitness criteria.
    Fitness = 0,
    /// Precision criteria.
    Precision = 1,
    /// Generalization criteria.
    Generalization = 2,
    /// Simplicity criteria.
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
            fitness: 0x0001_0000,
            precision: 0x0001_0000,
            generalization: 0x0000_8000,
            simplicity: 0x0000_8000,
        };
        assert!(ConformancePredicate::STRICT.check(&m).is_ok());
    }

    #[test]
    fn strict_predicate_fails_on_precision() {
        let m = ConformanceMetrics {
            fitness: 0x0001_0000,
            precision: 0x0000_7000, // below 0x0001_0000
            generalization: 0x0000_8000,
            simplicity: 0x0000_8000,
        };
        let err = ConformancePredicate::STRICT.check(&m).unwrap_err();
        assert_eq!(err.dim, ConformanceDimension::Precision);
    }

    /// `branchless_first_failure`'s final `_ => ConformanceDimension::Simplicity`
    /// arm is reached only when fitness, precision, *and* generalization all
    /// pass and simplicity alone fails. Before this test, that arm had zero
    /// coverage anywhere in this crate: every other failing-predicate test
    /// (this file's `strict_predicate_fails_on_precision`, `replay.rs`'s
    /// `strict_predicate_fails_on_a_perfect_trace_due_to_mocked_dimensions`,
    /// `tests/replay_law.rs`'s `strict_predicate_fails_on_low_precision`)
    /// reports `Precision` or `Generalization`, never a pure
    /// simplicity-only failure. A bug that swapped the `3`/`2` literals in
    /// `select_u32(gen_ok, 3, 2)` would misreport this exact case as
    /// `Generalization` (and, symmetrically, misreport a pure
    /// generalization-only failure as `Simplicity`) — this test would catch
    /// that; the existing suite would not.
    #[test]
    fn strict_predicate_fails_on_simplicity_alone() {
        let m = ConformanceMetrics {
            fitness: 0x0001_0000,        // meets min_fitness (0x0001_0000)
            precision: 0x0001_0000,      // meets min_precision (0x0001_0000)
            generalization: 0x0000_8000, // meets min_generalization (0x0000_8000)
            simplicity: 0x0000_0000,     // below min_simplicity (0x0000_8000)
        };
        let err = ConformancePredicate::STRICT.check(&m).unwrap_err();
        assert_eq!(err.dim, ConformanceDimension::Simplicity);
    }
}
