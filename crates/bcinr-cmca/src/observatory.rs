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

use crate::fixed::Fixed;
use crate::allocator::{const_lt_u32, const_select_u32, const_eq_u32};

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
    /// assert_eq!(ObservatoryFlag::from_u32(4), Some(ObservatoryFlag::RecertificationCandidate));
    /// assert_eq!(ObservatoryFlag::from_u32(5), None);
    /// ```
    pub fn from_u32(val: u32) -> Option<Self> {
        let lookup = [
            Some(Self::NumericallyUncertain),
            Some(Self::GramDegenerate),
            Some(Self::Drifting),
            Some(Self::ScaleInert),
            Some(Self::RecertificationCandidate),
            None, None, None
        ];
        
        let in_bounds = const_lt_u32(val, 5);
        let idx = const_select_u32(in_bounds, val, 5) as usize;
        let res = lookup[idx & 7];
        
        res.filter(|_| in_bounds != 0)
    }
}

const FLAGS: [ObservatoryFlag; 8] = [
    ObservatoryFlag::NumericallyUncertain,
    ObservatoryFlag::GramDegenerate,
    ObservatoryFlag::Drifting,
    ObservatoryFlag::ScaleInert,
    ObservatoryFlag::RecertificationCandidate,
    ObservatoryFlag::RecertificationCandidate,
    ObservatoryFlag::RecertificationCandidate,
    ObservatoryFlag::RecertificationCandidate,
];

/// Wraps a raw observatory flag code into a `Result` type branchlessly.
///
/// A flag code of `4` (corresponding to `RecertificationCandidate`) represents success and
/// maps to `Ok(())`. Any other flag code represents a failure mode and maps to `Err(ObservatoryFlag)`.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::observatory::{wrap_observatory_result, ObservatoryFlag};
///
/// assert_eq!(wrap_observatory_result(4), Ok(()));
/// assert_eq!(wrap_observatory_result(2), Err(ObservatoryFlag::Drifting));
/// ```
///
/// # Branchless Contract
///
/// Under the hood, this function maps the integer code into the static flag mapping
/// array and uses a branchless array index selection based on the equality to `4`.
pub fn wrap_observatory_result(
    flag_code: u32,
) -> Result<(), ObservatoryFlag> {
    let flag = FLAGS[(flag_code as usize) & 7];
    let is_recert = const_eq_u32(flag_code, 4);
    let outcomes = [Err(flag), Ok(())];
    outcomes[is_recert as usize]
}

/// Evaluates calibration metrics and proposes an admission/stability flag branchlessly.
///
/// This is the primary telemetry evaluation routine. It takes estimates and lower bounds of
/// key covariance and distance metrics and compares them against predefined thresholds,
/// compiling all findings into a single outcome flag branchlessly.
///
/// # Arguments
///
/// * `kappa_hat` - The estimated condition number of the system.
/// * `kappa_under` - The lower bound (conservative estimate) of the condition number.
/// * `epsilon_on` - The threshold limit for the condition number.
/// * `_gamma_min_plus_hat` - The estimated minimum positive eigenvalue of the Gram matrix. (Unused but kept for contract symmetry).
/// * `gamma_min_plus_under` - The lower bound (conservative estimate) of the minimum positive eigenvalue.
/// * `epsilon_gram` - The threshold limit for the Gram eigenvalue.
/// * `d_js` - The measured divergence (e.g., Jensen-Shannon distance) representing drift.
/// * `epsilon_drift` - The threshold limit for drift divergence.
/// * `s_meas` - The measured scale parameter of the current sample.
/// * `s_leaf` - The target scale parameter at the leaf node.
///
/// # Return Value
///
/// Returns `Ok(())` if the system passes all safety gates (mapping to `RecertificationCandidate`).
/// Otherwise, returns an `Err(ObservatoryFlag)` indicating the first detected failure mode in the
/// prioritization queue (highest priority to lowest priority):
/// 1. `Drifting` (highest priority check)
/// 2. `ScaleInert`
/// 3. `NumericallyUncertain`
/// 4. `GramDegenerate`
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::fixed::Fixed;
/// use bcinr_cmca::observatory::{evaluate_calibration, ObservatoryFlag};
///
/// // Example: Drifting state
/// let result = evaluate_calibration(
///     Fixed::from_bits(131072),
///     Fixed::from_bits(131072),
///     Fixed::from_bits(65536),
///     Fixed::from_bits(131072),
///     Fixed::from_bits(131072),
///     Fixed::from_bits(65536),
///     Fixed::from_bits(131072), // d_js (2.0)
///     Fixed::from_bits(65536),  // epsilon_drift (1.0) -> Drift detected!
///     Fixed::ONE,
///     Fixed::from_bits(32768),
/// );
/// assert_eq!(result, Err(ObservatoryFlag::Drifting));
/// ```
///
/// # Branchless Contract
///
/// Checks are performed concurrently using bitwise masks. Prioritization is enforced using
/// sequential branchless selections `const_select_u32`.
pub fn evaluate_calibration(
    kappa_hat: Fixed,
    kappa_under: Fixed,
    epsilon_on: Fixed,
    _gamma_min_plus_hat: Fixed,
    gamma_min_plus_under: Fixed,
    epsilon_gram: Fixed,
    d_js: Fixed,
    epsilon_drift: Fixed,
    s_meas: Fixed,
    s_leaf: Fixed,
) -> Result<(), ObservatoryFlag> {
    
    // Conditions
    let is_drift = const_lt_u32(epsilon_drift.0, d_js.0);
    
    let is_scale_inert = const_eq_u32(s_meas.0, s_leaf.0);
    
    let kappa_hat_on = const_lt_u32(epsilon_on.0, kappa_hat.0) | const_eq_u32(epsilon_on.0, kappa_hat.0);
    let kappa_under_off = const_lt_u32(kappa_under.0, epsilon_on.0);
    let is_numerically_uncertain = kappa_hat_on & kappa_under_off;
    
    let kappa_under_on = const_lt_u32(epsilon_on.0, kappa_under.0) | const_eq_u32(epsilon_on.0, kappa_under.0);
    
    let gamma_under_off = const_lt_u32(gamma_min_plus_under.0, epsilon_gram.0);
    
    let is_gram_degenerate = kappa_under_on & gamma_under_off;
    
    let is_recert = kappa_under_on & (!gamma_under_off);
    
    let mut flag = 4u32; // Default to Ok
    flag = const_select_u32(is_recert, 4, flag);
    flag = const_select_u32(is_gram_degenerate, 1, flag);
    flag = const_select_u32(is_numerically_uncertain, 0, flag);
    flag = const_select_u32(is_scale_inert, 3, flag);
    flag = const_select_u32(is_drift, 2, flag);
    
    wrap_observatory_result(flag)
}
