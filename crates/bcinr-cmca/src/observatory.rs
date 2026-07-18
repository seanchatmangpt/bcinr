//! # Calibration Observatory and Telemetry Engine
//!
//! This module provides the telemetry and evaluation machinery to monitor
//! mathematical model stability and trigger recertification/admissions branchlessly.
//!
//! It implements the core inference phase of the MAPE-K loop, checking five key criteria:
//!
//! 1. **Numerical Uncertainty** ([`ObservatoryFlag::NumericallyUncertain`]): Triggers when the estimated condition
//!    number (`kappa_hat`) exceeds the stability limit (`epsilon_on`), but the lower bound
//!    (`kappa_under`) does not, meaning the actual condition number is boundary-uncertain.
//! 2. **Gram Degeneracy** ([`ObservatoryFlag::GramDegenerate`]): Triggers when the condition number lower bound
//!    (`kappa_under`) exceeds the stability threshold, but the minimum positive eigenvalue
//!    (`gamma_min_plus_under`) falls below the Gram threshold (`epsilon_gram`).
//! 3. **Non-stationary Drift** ([`ObservatoryFlag::Drifting`]): Triggers when the divergence metric (`d_js`)
//!    exceeds the drift boundary (`epsilon_drift`).
//! 4. **Scale Inertia** ([`ObservatoryFlag::ScaleInert`]): Triggers when measured scale (`s_meas`) matches the
//!    leaf scale (`s_leaf`) exactly, indicating a collapse of informative scaling variance.
//! 5. **Recertification Candidate** ([`ObservatoryFlag::RecertificationCandidate`]): The success flag returned when
//!    all active stability criteria are satisfied.
//!
//! All checks are combined into a branchless selection tree, guaranteeing $CC=1$.

use crate::allocator::{const_eq_u32, const_lt_u32, const_select_u32};
use crate::fixed::{NonNegativeFixed, SignedFixed};

/// Telemetry safety and calibration indicators for the runtime observatory.
///
/// These flags categorize the current stability status of the monitored substrate.
/// Except for [`ObservatoryFlag::RecertificationCandidate`], all flags represent a failure mode
/// that halts normal operation and demands quarantine or fallback execution.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ObservatoryFlag {
    /// The estimated condition number exceeds the safety threshold, but its lower bound does not.
    /// Indicates that the substrate is operating in a region of high numerical uncertainty.
    NumericallyUncertain,

    /// The lower bound condition number indicates a active state, but the minimum positive Gram eigenvalue
    /// is below the degeneracy threshold. Indicates loss of numerical rank/independence.
    GramDegenerate,

    /// The measured distance/divergence exceeds the allowed drift threshold, indicating that the underlying
    /// data distribution is non-stationary.
    Drifting,

    /// The measured scale is identical to the target leaf scale, indicating zero scaling update information.
    ScaleInert,

    /// The proposal does not propose a delta, meaning it is unadmitted for recertification.
    ModeDeltaUnadmitted,

    /// The successful validation state. The substrate passes all safety filters and is a candidate
    /// for recertification or normal deployment.
    RecertificationCandidate,
}

impl ObservatoryFlag {
    /// Converts a raw `u32` value into its corresponding `ObservatoryFlag`.
    ///
    /// The conversion is performed branchlessly, mapping out-of-bounds inputs to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bcinr_cmca::observatory::ObservatoryFlag;
    ///
    /// assert_eq!(ObservatoryFlag::from_u32(0), Some(ObservatoryFlag::NumericallyUncertain));
    /// assert_eq!(ObservatoryFlag::from_u32(4), Some(ObservatoryFlag::ModeDeltaUnadmitted));
    /// assert_eq!(ObservatoryFlag::from_u32(5), Some(ObservatoryFlag::RecertificationCandidate));
    /// assert_eq!(ObservatoryFlag::from_u32(6), None);
    /// ```
    pub fn from_u32(val: u32) -> Option<Self> {
        let lookup = [
            Some(Self::NumericallyUncertain),
            Some(Self::GramDegenerate),
            Some(Self::Drifting),
            Some(Self::ScaleInert),
            Some(Self::ModeDeltaUnadmitted),
            Some(Self::RecertificationCandidate),
            None,
            None,
        ];

        let in_bounds = const_lt_u32(val, 6);
        let idx = const_select_u32(in_bounds, val, 6) as usize;
        let res = lookup[idx & 7];

        res.filter(|_| in_bounds != 0)
    }
}

/// Opaque bitset preserving every simultaneously-true telemetry condition observed by
/// [`evaluate_calibration`].
///
/// Per `.claude/rules/cmca/authority-and-c3.md` Invariant 2, a telemetry evaluation whose
/// underlying conditions can be simultaneously true must never collapse to a single enum
/// variant as its sole representation — that would silently discard a second true
/// condition. This type is the full-set representation; [`ObservatoryFlagSet::primary_flag`]
/// is a separately named, separately tested priority projection layered on top, not a
/// replacement for the set.
///
/// The Observatory (this module) NEVER constructs a `CertificateReceipt` from this set or
/// from any other value — see `crate::certification::seal_certificate` for the only lawful
/// minting path. This type carries telemetry standing only.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ObservatoryFlagSet(u32);

impl ObservatoryFlagSet {
    pub const EMPTY: Self = Self(0);

    const BIT_NUMERICALLY_UNCERTAIN: u32 = 1 << 0;
    const BIT_GRAM_DEGENERATE: u32 = 1 << 1;
    const BIT_DRIFT: u32 = 1 << 2;
    const BIT_SCALE_INERT: u32 = 1 << 3;
    const BIT_UNADMITTED: u32 = 1 << 4;
    const BIT_RECERTIFICATION_SUGGESTED: u32 = 1 << 5;

    #[inline(always)]
    const fn bit_for(flag: ObservatoryFlag) -> u32 {
        match flag {
            ObservatoryFlag::NumericallyUncertain => Self::BIT_NUMERICALLY_UNCERTAIN,
            ObservatoryFlag::GramDegenerate => Self::BIT_GRAM_DEGENERATE,
            ObservatoryFlag::Drifting => Self::BIT_DRIFT,
            ObservatoryFlag::ScaleInert => Self::BIT_SCALE_INERT,
            ObservatoryFlag::ModeDeltaUnadmitted => Self::BIT_UNADMITTED,
            ObservatoryFlag::RecertificationCandidate => Self::BIT_RECERTIFICATION_SUGGESTED,
        }
    }

    /// Builds a set from independent branchless condition masks (each `0` or `1`), one per
    /// named telemetry dimension. This is the only production constructor: every condition
    /// that was independently true at evaluation time is preserved, regardless of how many
    /// others were also true.
    #[inline(always)]
    pub(crate) const fn from_conditions(
        numerically_uncertain: u32,
        gram_degenerate: u32,
        drift: u32,
        scale_inert: u32,
        unadmitted: u32,
        recertification_suggested: u32,
    ) -> Self {
        let bits = (numerically_uncertain & 1).wrapping_mul(Self::BIT_NUMERICALLY_UNCERTAIN)
            | (gram_degenerate & 1).wrapping_mul(Self::BIT_GRAM_DEGENERATE)
            | (drift & 1).wrapping_mul(Self::BIT_DRIFT)
            | (scale_inert & 1).wrapping_mul(Self::BIT_SCALE_INERT)
            | (unadmitted & 1).wrapping_mul(Self::BIT_UNADMITTED)
            | (recertification_suggested & 1).wrapping_mul(Self::BIT_RECERTIFICATION_SUGGESTED);
        Self(bits)
    }

    /// Test/fixture helper: insert a single named flag into the set. Used to construct
    /// multi-true fixtures without re-deriving raw condition masks. `cfg(test)`-scoped:
    /// its only call sites are test-fixture builders in this crate's `#[cfg(test)]`
    /// modules (`jump.rs`, `proposal.rs`, `shadow.rs`).
    #[cfg(test)]
    #[inline(always)]
    pub(crate) const fn insert(self, flag: ObservatoryFlag) -> Self {
        Self(self.0 | Self::bit_for(flag))
    }

    #[inline(always)]
    pub const fn contains(self, flag: ObservatoryFlag) -> bool {
        (self.0 & Self::bit_for(flag)) != 0
    }

    #[inline(always)]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Telemetry standing usable for downstream admission: admissible if and only if no
    /// failure-mode flag is set (i.e. at most `RECERTIFICATION_SUGGESTED` is present).
    #[inline(always)]
    pub const fn telemetry_admissible(self) -> bool {
        const FAILURE_MASK: u32 = ObservatoryFlagSet::BIT_NUMERICALLY_UNCERTAIN
            | ObservatoryFlagSet::BIT_GRAM_DEGENERATE
            | ObservatoryFlagSet::BIT_DRIFT
            | ObservatoryFlagSet::BIT_SCALE_INERT
            | ObservatoryFlagSet::BIT_UNADMITTED;
        (self.0 & FAILURE_MASK) == 0
    }

    /// Documented, tested priority projection onto a single primary flag, for callers that
    /// need one headline condition (e.g. a status line). This NEVER replaces the full set
    /// above — a caller that needs to know about co-occurring conditions must inspect
    /// `contains` directly.
    ///
    /// Priority order (highest first), matching the historical single-flag selection this
    /// type replaces: `Drifting` > `ScaleInert` > `NumericallyUncertain` > `GramDegenerate`
    /// > `ModeDeltaUnadmitted` > `RecertificationCandidate`.
    pub const fn primary_flag(self) -> ObservatoryFlag {
        if self.contains(ObservatoryFlag::Drifting) {
            ObservatoryFlag::Drifting
        } else if self.contains(ObservatoryFlag::ScaleInert) {
            ObservatoryFlag::ScaleInert
        } else if self.contains(ObservatoryFlag::NumericallyUncertain) {
            ObservatoryFlag::NumericallyUncertain
        } else if self.contains(ObservatoryFlag::GramDegenerate) {
            ObservatoryFlag::GramDegenerate
        } else if self.contains(ObservatoryFlag::ModeDeltaUnadmitted) {
            ObservatoryFlag::ModeDeltaUnadmitted
        } else {
            ObservatoryFlag::RecertificationCandidate
        }
    }
}

/// The Observatory's evaluation outcome: a lawfully-constructed [`ModeProposal`] plus the
/// full, lossless telemetry standing that produced it.
///
/// Replaces the historical `Result<CertificateReceipt, ObservatoryFlag>` return of
/// `evaluate_calibration`. The Observatory never mints a `CertificateReceipt` — that
/// authority belongs solely to `crate::certification::seal_certificate`, reached (if at
/// all) only after `crate::proposal::admit_proposal`, `crate::shadow`, `crate::jump`, and
/// `crate::stability` have each independently run on the proposal this outcome carries.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObservatoryOutcome {
    pub proposal: crate::proposal::ModeProposal,
    pub flags: ObservatoryFlagSet,
}

/// Evaluates calibration metrics and proposes an admission/stability flag branchlessly.
///
/// This is the primary telemetry evaluation routine. It takes estimates and lower bounds of
/// key covariance and distance metrics and compares them against predefined thresholds,
/// compiling all findings into a single outcome flag branchlessly.
///
/// # Arguments
///
/// * `artifact` - The `MeasurementArtifact` containing telemetry metrics and proposal.
/// * `epsilon_on` - The threshold limit for the condition number.
/// * `epsilon_gram` - The threshold limit for the Gram eigenvalue.
/// * `epsilon_drift` - The threshold limit for drift divergence.
/// * `s_meas` - The measured scale parameter of the current sample.
/// * `s_leaf` - The target scale parameter at the leaf node.
///
/// # Return Value
///
/// Returns an [`ObservatoryOutcome`] carrying a lawfully-constructed [`crate::proposal::ModeProposal`]
/// and the full [`ObservatoryFlagSet`] of every telemetry condition that was true. This
/// function NEVER constructs a `CertificateReceipt` — see [`ObservatoryOutcome`]'s docs for
/// the certification path this hands off to. Callers wanting a single headline condition
/// should call `outcome.flags.primary_flag()`, which reproduces the historical single-flag
/// priority (`Drifting` > `ScaleInert` > `NumericallyUncertain` > `GramDegenerate` >
/// `ModeDeltaUnadmitted` > `RecertificationCandidate`) as a documented projection, not the
/// sole representation.
///
/// # Branchless Contract
///
/// Checks are performed concurrently using bitwise masks. The flag set is a bitwise-OR
/// composition of independent condition masks — no sequential override discards a
/// simultaneously-true condition.
pub fn evaluate_calibration(
    artifact: &MeasurementArtifact,
    epsilon_on: NonNegativeFixed,
    epsilon_gram: NonNegativeFixed,
    epsilon_drift: NonNegativeFixed,
    s_meas: NonNegativeFixed,
    s_leaf: NonNegativeFixed,
    round_identity: u64,
) -> ObservatoryOutcome {
    let kappa_hat = artifact.point_estimate;
    let kappa_under = artifact.lower_bound;
    let gamma_min_plus_under = artifact.gram_lower_bound;
    let d_js = artifact.drift;

    // Conditions
    #[cfg(not(feature = "mutant_9"))]
    let is_drift = const_lt_u32(epsilon_drift.value_bits(), d_js.value_bits());
    #[cfg(feature = "mutant_9")]
    let is_drift = const_lt_u32(d_js.value_bits(), epsilon_drift.value_bits()); // Mutated: drift check inverted

    let is_scale_inert = const_eq_u32(s_meas.value_bits(), s_leaf.value_bits());

    let kappa_hat_on = const_lt_u32(epsilon_on.value_bits(), kappa_hat.value_bits())
        | const_eq_u32(epsilon_on.value_bits(), kappa_hat.value_bits());
    #[cfg(not(feature = "mutant_10"))]
    let kappa_under_off = const_lt_u32(kappa_under.value_bits(), epsilon_on.value_bits());
    #[cfg(feature = "mutant_10")]
    let kappa_under_off = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits()); // Mutated: inverted
    let is_numerically_uncertain = kappa_hat_on & kappa_under_off;

    let kappa_under_on = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits())
        | const_eq_u32(epsilon_on.value_bits(), kappa_under.value_bits());

    #[cfg(not(feature = "mutant_11"))]
    let gamma_under_off =
        const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits());
    #[cfg(feature = "mutant_11")]
    let gamma_under_off =
        const_lt_u32(epsilon_gram.value_bits(), gamma_min_plus_under.value_bits()); // Mutated: inverted

    let is_gram_degenerate = kappa_under_on & gamma_under_off;

    let is_unadmitted = const_eq_u32(artifact.proposal as u32, ModeDelta::Retain as u32);

    let is_recert = kappa_under_on & (!gamma_under_off) & (!is_unadmitted);

    // Full-set composition: every independently-true condition is preserved (Invariant 2,
    // authority-and-c3.md). No sequential override here — that only happens, documented and
    // tested separately, inside `ObservatoryFlagSet::primary_flag`.
    let flags = ObservatoryFlagSet::from_conditions(
        is_numerically_uncertain,
        is_gram_degenerate,
        is_drift,
        is_scale_inert,
        is_unadmitted,
        is_recert,
    );

    let proposed_delta = SignedFixed::from_value_bits(const_select_u32(is_recert, 1, 0) as i32);

    let proposal = crate::proposal::ModeProposal::propose(
        proposed_delta,
        artifact.graph_digest,
        artifact.control_mode_digest,
        round_identity,
        flags,
    );

    ObservatoryOutcome { proposal, flags }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SupportStanding {
    pub is_supported: bool,
    pub smoothing_applied: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModeDelta {
    Retain,
    ProposeDelta,
}

#[derive(Copy, Clone, Debug)]
pub struct MeasurementArtifact {
    pub point_estimate: NonNegativeFixed,
    pub lower_bound: NonNegativeFixed,
    pub upper_bound: NonNegativeFixed,
    pub support_standing: SupportStanding,
    pub effective_sample_size: NonNegativeFixed,
    pub dependence_standing: u32,
    pub numeric_error: NonNegativeFixed,
    pub drift: NonNegativeFixed,
    pub gram_lower_bound: NonNegativeFixed,
    pub graph_digest: u64,
    pub control_mode_digest: u64,
    pub proposal: ModeDelta,
}

use crate::allocator::const_max_i32;
use crate::generated::case_studies::{K, N};
use crate::{unroll_4_static, unroll_8_static};

/// Measures the divergence metric $\kappa_v$ and produces a MeasurementArtifact branchlessly.
// Fixed-shape bounded parameters (N/K/Q bounded arrays and scalars); same rationale as
// `allocator::allocate`. Documented allow per AGENTS.md's "no undocumented allow" rule.
#[allow(clippy::too_many_arguments)]
#[inline(never)]
pub fn measure_kappa(
    v: usize,
    _q_idx: usize,
    k: usize,
    parent: &[i32; N],
    _is_leaf: &[bool; N],
    is_subtree_leaf_v: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    node_masses: &[[NonNegativeFixed; N]; K],
    q_val: SignedFixed,
) -> MeasurementArtifact {
    let k_masked = k & 3;

    let mut x = [0i32; N];
    unroll_8_static!(I, {
        let mut log_m = 0u32;
        unroll_4_static!(K_IDX, {
            let matches = const_eq_u32(k_masked as u32, K_IDX as u32);
            log_m = const_select_u32(
                matches,
                node_masses[K_IDX & 3][I & 7].log2().value_bits() as u32,
                log_m,
            );
        });
        let q_signed = q_val.value_bits();
        x[I & 7] = (((q_signed as i64).wrapping_mul(log_m as i32 as i64)) >> 16) as i32;
    });

    let mut x_max_meas = i32::MIN;
    unroll_8_static!(J, {
        let is_child = const_eq_u32(parent[J & 7] as u32, v as u32);
        let x_safe = const_select_u32(is_child, x[J & 7] as u32, i32::MIN as u32) as i32;
        x_max_meas = const_max_i32(x_max_meas, x_safe);
    });

    let mut sum_exp_meas = NonNegativeFixed::ZERO;
    unroll_8_static!(J, {
        let is_child = const_eq_u32(parent[J & 7] as u32, v as u32);
        let a_prime = x[J & 7].wrapping_sub(x_max_meas);
        let exp_val = SignedFixed::from_value_bits(a_prime).exp2();
        sum_exp_meas +=
            NonNegativeFixed::from_value_bits(const_select_u32(is_child, exp_val.value_bits(), 0));
    });
    let l_meas = x_max_meas.wrapping_add(sum_exp_meas.log2().value_bits());

    let mut x_max_leaf = i32::MIN;
    unroll_8_static!(X_IDX, {
        let is_sub = is_subtree_leaf_v[X_IDX & 7];
        let x_safe = const_select_u32(is_sub as u32, x[X_IDX & 7] as u32, i32::MIN as u32) as i32;
        x_max_leaf = const_max_i32(x_max_leaf, x_safe);
    });

    let mut sum_exp_leaf = NonNegativeFixed::ZERO;
    unroll_8_static!(X_IDX, {
        let is_sub = is_subtree_leaf_v[X_IDX & 7];
        let a_prime = x[X_IDX & 7].wrapping_sub(x_max_leaf);
        let exp_val = SignedFixed::from_value_bits(a_prime).exp2();
        sum_exp_leaf += NonNegativeFixed::from_value_bits(const_select_u32(
            is_sub as u32,
            exp_val.value_bits(),
            0,
        ));
    });
    let l_leaf = x_max_leaf.wrapping_add(sum_exp_leaf.log2().value_bits());

    let mut kappa_i64 = 0i64;
    unroll_8_static!(C, {
        let is_child = const_eq_u32(parent[C & 7] as u32, v as u32);

        let mut x_max_c = i32::MIN;
        unroll_8_static!(X_IDX, {
            let is_sub_c = is_subtree_leaf[C & 7][X_IDX & 7];
            let x_safe =
                const_select_u32(is_sub_c as u32, x[X_IDX & 7] as u32, i32::MIN as u32) as i32;
            x_max_c = const_max_i32(x_max_c, x_safe);
        });

        let mut sum_exp_c = NonNegativeFixed::ZERO;
        unroll_8_static!(X_IDX, {
            let is_sub_c = is_subtree_leaf[C & 7][X_IDX & 7];
            let a_prime = x[X_IDX & 7].wrapping_sub(x_max_c);
            let exp_val = SignedFixed::from_value_bits(a_prime).exp2();
            sum_exp_c += NonNegativeFixed::from_value_bits(const_select_u32(
                is_sub_c as u32,
                exp_val.value_bits(),
                0,
            ));
        });
        let l_c = x_max_c.wrapping_add(sum_exp_c.log2().value_bits());

        let log_ratio = l_c.wrapping_sub(l_meas);
        let s_leaf_c = NonNegativeFixed::from_value_bits(
            SignedFixed::from_value_bits(l_c.wrapping_sub(l_leaf))
                .exp2()
                .value_bits(),
        );

        let term = (s_leaf_c.value_bits() as i64).wrapping_mul(log_ratio as i64);
        let term_selected = const_select_u32(is_child, (term >> 16) as u32, 0) as i32 as i64;
        kappa_i64 = kappa_i64.wrapping_add(term_selected);
    });

    let kappa_clipped = const_select_u32((kappa_i64 < 0) as u32, 0, kappa_i64 as u32);
    let kappa = NonNegativeFixed::from_value_bits(kappa_clipped);

    MeasurementArtifact {
        point_estimate: kappa,
        lower_bound: kappa,
        upper_bound: kappa,
        support_standing: SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: NonNegativeFixed::ZERO,
        gram_lower_bound: NonNegativeFixed::ZERO,
        graph_digest: 0,
        control_mode_digest: 0,
        proposal: ModeDelta::Retain,
    }
}

#[cfg(test)]
mod flag_set_tests {
    use super::*;

    #[test]
    fn full_set_survives_when_two_conditions_are_simultaneously_true() {
        // drift=1 and scale_inert=1 true at once; is_recert/is_unadmitted false.
        let flags = ObservatoryFlagSet::from_conditions(0, 0, 1, 1, 0, 0);
        assert!(flags.contains(ObservatoryFlag::Drifting));
        assert!(flags.contains(ObservatoryFlag::ScaleInert));
        assert!(!flags.contains(ObservatoryFlag::NumericallyUncertain));
        assert!(!flags.contains(ObservatoryFlag::GramDegenerate));
        // primary_flag necessarily reports only one of the two true conditions...
        assert_eq!(flags.primary_flag(), ObservatoryFlag::Drifting);
        // ...but the full set still proves the second condition was not discarded.
        assert!(flags.contains(ObservatoryFlag::ScaleInert));
    }

    #[test]
    fn full_set_survives_all_four_failure_conditions_simultaneously() {
        let flags = ObservatoryFlagSet::from_conditions(1, 1, 1, 1, 1, 0);
        assert!(flags.contains(ObservatoryFlag::NumericallyUncertain));
        assert!(flags.contains(ObservatoryFlag::GramDegenerate));
        assert!(flags.contains(ObservatoryFlag::Drifting));
        assert!(flags.contains(ObservatoryFlag::ScaleInert));
        assert!(flags.contains(ObservatoryFlag::ModeDeltaUnadmitted));
        assert_eq!(flags.primary_flag(), ObservatoryFlag::Drifting);
        assert!(!flags.telemetry_admissible());
    }

    #[test]
    fn primary_flag_priority_order_is_documented_and_tested() {
        // Named priority-projection test per authority-and-c3.md Invariant 2: exercise
        // every adjacent pair in the documented tie-break order.
        assert_eq!(
            ObservatoryFlagSet::from_conditions(1, 1, 1, 0, 0, 0).primary_flag(),
            ObservatoryFlag::Drifting
        );
        assert_eq!(
            ObservatoryFlagSet::from_conditions(1, 1, 0, 1, 0, 0).primary_flag(),
            ObservatoryFlag::ScaleInert
        );
        assert_eq!(
            ObservatoryFlagSet::from_conditions(1, 1, 0, 0, 0, 0).primary_flag(),
            ObservatoryFlag::NumericallyUncertain
        );
        assert_eq!(
            ObservatoryFlagSet::from_conditions(0, 1, 0, 0, 0, 0).primary_flag(),
            ObservatoryFlag::GramDegenerate
        );
        assert_eq!(
            ObservatoryFlagSet::from_conditions(0, 0, 0, 0, 1, 0).primary_flag(),
            ObservatoryFlag::ModeDeltaUnadmitted
        );
        assert_eq!(
            ObservatoryFlagSet::from_conditions(0, 0, 0, 0, 0, 1).primary_flag(),
            ObservatoryFlag::RecertificationCandidate
        );
    }

    #[test]
    fn clean_recert_only_flag_is_telemetry_admissible() {
        let flags = ObservatoryFlagSet::from_conditions(0, 0, 0, 0, 0, 1);
        assert!(flags.telemetry_admissible());
    }

    #[test]
    fn empty_set_reports_no_conditions() {
        let flags = ObservatoryFlagSet::EMPTY;
        assert!(!flags.contains(ObservatoryFlag::Drifting));
        assert_eq!(
            flags.primary_flag(),
            ObservatoryFlag::RecertificationCandidate
        );
    }
}
