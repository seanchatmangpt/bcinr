//! # Cascade Resource Allocator
//!
//! This module provides the core implementation of the resource allocation engine for the
//! Covariance Monitoring and Calibration Assessment (CMCA) substrate.
//!
//! Under the strict mandates of the Radon Law, this allocator enforces:
//! - **Zero Heap Allocations**: All computations are performed on stack-allocated structures.
//! - **Constant-Time Execution ($CC=1$)**: Absolutely no input-dependent loops, conditional jumps,
//!   or branches.
//! - **Typed Refusals**: Any out-of-envelope or invalid operational state yields a specific
//!   [`StabilityRefusal`] code without panic or unwinding.
//!
//! ## Core Mathematical Algorithms
//!
//! The resource allocation algorithm executes in four distinct phases:
//!
//! ### 1. Cascade Allocation
//! The allocator distributes resource flows hierarchically down a forest structure of $N$ nodes.
//! Let the tree structure be defined by a parent vector $P \in \mathbb{I}^N$ where $P_i$ denotes the parent
//! index of node $i$, or $-1$ if $i$ is a root.
//!
//! The initial resource flow is distributed to the roots of the forest.
//! For a given policy lens $q \in \{0, \dots, Q-1\}$ and target index $k \in \{0, \dots, K-1\}$, the initial
//! root weights are:
//!
//! $$ W_{\text{root}}(i) = \exp_2\left( q_{\text{val}} \cdot \log_2(M_{k, i}) - A_{\text{max\_root}} \right) $$
//!
//! where $M_{k, i}$ is the clipped semantic mass of node $i$, and $A_{\text{max\_root}}$ is a normalization scalar
//! to prevent arithmetic overflow. The initial root allocation flow is then:
//!
//! $$ \text{alloc\_flow}[r] = \frac{W_{\text{root}}(r)}{\sum_{j \in \text{roots}} W_{\text{root}}(j)} $$
//!
//! Non-root nodes are initialized with zero flow. The allocator then executes exactly $N$ iterations of a
//! straight-line propagation function ([`flow_step`]).
//!
//! At each step, for every node $v$:
//! - If $v$ is a leaf, the incoming flow is collected into the final allocation vector.
//! - If $v$ has children, the incoming flow is split into a direct leaf allocation part ($F_v$) and a child
//!   propagation part ($D_v$):
//!
//!   $$ F_v = (1 - \rho_v) \cdot \text{alloc\_flow}[v] $$
//!   $$ D_v = \rho_v \cdot \text{alloc\_flow}[v] $$
//!
//!   where $\rho_v \in [0, 1]$ is the local routing parameter.
//! - The direct part $F_v$ is distributed to all descendant leaves under $v$ proportional to the leaf weights:
//!
//!   $$ \text{flat\_alloc}[x] \leftarrow \text{flat\_alloc}[x] + F_v \cdot \frac{W_{\text{leaf}}(v, x)}{\sum_{y \in \text{leaves}(v)} W_{\text{leaf}}(v, y)} $$
//!
//! - The descendant part $D_v$ is distributed to direct children of $v$ proportional to the child weights:
//!
//!   $$ \text{alloc\_flow}[c] \leftarrow \text{alloc\_flow}[c] + D_v \cdot \frac{W_{\text{child}}(v, c)}{\sum_{d \in \text{children}(v)} W_{\text{child}}(v, d)} $$
//!
//! ### 2. Multiplicative Weights Update (MWU) Step Updates
//! For each internal node $v$, routing weights between direct leaf allocation and child propagation
//! are adjusted dynamically based on payoff feedback.
//! The updates are controlled by a local divergence metric $\kappa_v$ (relative entropy) computed via [`compute_kappa`]:
//!
//! $$ \kappa_v = \sum_{c \in \text{children}(v)} s_{\text{leaf}}(c) \cdot \log_2\left( \frac{s_{\text{leaf}}(c)}{s_{\text{meas}}(c)} \right) $$
//!
//! If $\kappa_v > \epsilon_{\kappa}$, the weights are updated using learning rate $\beta$:
//!
//! $$ w_{t+1}(v, d) = w_t(v, d) \cdot \exp\left( \beta \cdot \text{payoff}(v, d) \right) $$
//!
//! followed by normalization.
//!
//! ### 3. Stable Projections
//! Combined allocation is projected based on resource prices $\mu_x$ and operational costs $c_x$:
//!
//! $$ P_{\mu}(x) = \frac{\pi_{\text{combined}}(x) \cdot \exp(-\mu_x \cdot c_x)}{\sum_{y \in \text{leaves}} \pi_{\text{combined}}(y) \cdot \exp(-\mu_y \cdot c_y)} $$
//!
//! ### 4. Explore Floors
//! A uniform exploration floor is mixed into the final allocation vector to guarantee minimal search and prevent
//! numerical singularity:
//!
//! $$ \pi_{\text{res}}(x) = \eta \cdot \frac{1}{n_L} + (1 - \eta) \cdot P_{\mu}(x) $$
//!
//! where $n_L$ is the number of leaf nodes in the tree.
//!
//! ## Algorithmic Complexity
//!
//! - **Time Complexity**: $O(K \cdot Q \cdot N^2)$ operations, where $N=8$, $K=4$, $Q=4$ are constants.
//!   Thus, execution time is strictly bounded and $O(1)$.
//! - **Space Complexity**: $O(1)$ auxiliary stack space. No heap allocations.
//! - **Cyclomatic Complexity**: $CC = 1$ (no conditional control-flow branches).

#![allow(non_upper_case_globals, unused_assignments, unused_mut, dead_code)]

macro_rules! unroll_8_static {
    ($var:ident, $body:expr) => {
        {
            { const $var: usize = 0; $body }
            { const $var: usize = 1; $body }
            { const $var: usize = 2; $body }
            { const $var: usize = 3; $body }
            { const $var: usize = 4; $body }
            { const $var: usize = 5; $body }
            { const $var: usize = 6; $body }
            { const $var: usize = 7; $body }
        }
    };
}

macro_rules! unroll_9_static {
    ($var:ident, $body:expr) => {
        {
            { const $var: usize = 0; $body }
            { const $var: usize = 1; $body }
            { const $var: usize = 2; $body }
            { const $var: usize = 3; $body }
            { const $var: usize = 4; $body }
            { const $var: usize = 5; $body }
            { const $var: usize = 6; $body }
            { const $var: usize = 7; $body }
            { const $var: usize = 8; $body }
        }
    };
}

macro_rules! unroll_4_static {
    ($var:ident, $body:expr) => {
        {
            { const $var: usize = 0; $body }
            { const $var: usize = 1; $body }
            { const $var: usize = 2; $body }
            { const $var: usize = 3; $body }
        }
    };
}

macro_rules! unroll_32_static {
    ($var:ident, $body:expr) => {
        {
            { const $var: usize = 0; $body }
            { const $var: usize = 1; $body }
            { const $var: usize = 2; $body }
            { const $var: usize = 3; $body }
            { const $var: usize = 4; $body }
            { const $var: usize = 5; $body }
            { const $var: usize = 6; $body }
            { const $var: usize = 7; $body }
            { const $var: usize = 8; $body }
            { const $var: usize = 9; $body }
            { const $var: usize = 10; $body }
            { const $var: usize = 11; $body }
            { const $var: usize = 12; $body }
            { const $var: usize = 13; $body }
            { const $var: usize = 14; $body }
            { const $var: usize = 15; $body }
            { const $var: usize = 16; $body }
            { const $var: usize = 17; $body }
            { const $var: usize = 18; $body }
            { const $var: usize = 19; $body }
            { const $var: usize = 20; $body }
            { const $var: usize = 21; $body }
            { const $var: usize = 22; $body }
            { const $var: usize = 23; $body }
            { const $var: usize = 24; $body }
            { const $var: usize = 25; $body }
            { const $var: usize = 26; $body }
            { const $var: usize = 27; $body }
            { const $var: usize = 28; $body }
            { const $var: usize = 29; $body }
            { const $var: usize = 30; $body }
            { const $var: usize = 31; $body }
        }
    };
}

macro_rules! unroll_5_static {
    ($var:ident, $body:expr) => {
        {
            { const $var: usize = 0; $body }
            { const $var: usize = 1; $body }
            { const $var: usize = 2; $body }
            { const $var: usize = 3; $body }
            { const $var: usize = 4; $body }
        }
    };
}

use crate::fixed::Fixed;
use crate::generated::case_studies::{PackedSemanticState, LensSpec, N, K, Q};

/// Refusal reasons returned by the allocator when stability invariants are violated.
///
/// In compliance with the substrate rules, these are typed error codes rather than
/// text logs to avoid allocation and branching in the hot path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StabilityRefusal {
    CertificateMissing,
    BlockGainBoundExceeded,
    ContractionMarginInsufficient,
    LearningRateOutsideEnvelope,
    ModeDwellTimeViolated,
    QRangeDestabilizing,
    MassClampUnsafe,
    PriceGainUnsafe,
    StandingProjectionGainUnsafe,
    RuntimeEnvelopeViolated,
    CertificateDigestMismatch,
    ControlModeUncertified,
    ControlModeSwitchTooFast,
    YieldGainBoundViolated,
    RewardBoundViolated,
    ResourceResponseBoundViolated,
    StandingResetBoundViolated,
    LearningFrozen,
}

impl StabilityRefusal {
    /// Parses a raw `u32` value into a `StabilityRefusal` branchlessly.
    ///
    /// # Complexity
    /// $O(1)$ constant time with no branches.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bcinr_cmca::allocator::StabilityRefusal;
    ///
    /// assert_eq!(StabilityRefusal::from_u32(0), Some(StabilityRefusal::CertificateMissing));
    /// assert_eq!(StabilityRefusal::from_u32(18), None);
    /// ```
    pub fn from_u32(val: u32) -> Option<Self> {
        let lookup = [
            Some(Self::CertificateMissing),
            Some(Self::BlockGainBoundExceeded),
            Some(Self::ContractionMarginInsufficient),
            Some(Self::LearningRateOutsideEnvelope),
            Some(Self::ModeDwellTimeViolated),
            Some(Self::QRangeDestabilizing),
            Some(Self::MassClampUnsafe),
            Some(Self::PriceGainUnsafe),
            Some(Self::StandingProjectionGainUnsafe),
            Some(Self::RuntimeEnvelopeViolated),
            Some(Self::CertificateDigestMismatch),
            Some(Self::ControlModeUncertified),
            Some(Self::ControlModeSwitchTooFast),
            Some(Self::YieldGainBoundViolated),
            Some(Self::RewardBoundViolated),
            Some(Self::ResourceResponseBoundViolated),
            Some(Self::StandingResetBoundViolated),
            Some(Self::LearningFrozen),
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None
        ];
        
        let in_bounds = const_lt_u32(val, 18);
        let idx = const_select_u32(in_bounds, val, 18) as usize;
        let res = lookup[idx & 31];
        
        res
    }
}

const REFUSALS: [StabilityRefusal; 32] = [
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::BlockGainBoundExceeded,
    StabilityRefusal::ContractionMarginInsufficient,
    StabilityRefusal::LearningRateOutsideEnvelope,
    StabilityRefusal::ModeDwellTimeViolated,
    StabilityRefusal::QRangeDestabilizing,
    StabilityRefusal::MassClampUnsafe,
    StabilityRefusal::PriceGainUnsafe,
    StabilityRefusal::StandingProjectionGainUnsafe,
    StabilityRefusal::RuntimeEnvelopeViolated,
    StabilityRefusal::CertificateDigestMismatch,
    StabilityRefusal::ControlModeUncertified,
    StabilityRefusal::ControlModeSwitchTooFast,
    StabilityRefusal::YieldGainBoundViolated,
    StabilityRefusal::RewardBoundViolated,
    StabilityRefusal::ResourceResponseBoundViolated,
    StabilityRefusal::StandingResetBoundViolated,
    StabilityRefusal::LearningFrozen,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
    StabilityRefusal::CertificateMissing,
];

// Bounded leaf reciprocal lookup table (nl from 1 to 8)
const LEAF_RECIP: [Fixed; 9] = [
    Fixed(0),
    Fixed(65536), // 1.0
    Fixed(32768), // 0.5
    Fixed(21845), // 0.33333
    Fixed(16384), // 0.25
    Fixed(13107), // 0.2
    Fixed(10922), // 0.16667
    Fixed(9362),  // 0.14285
    Fixed(8192),  // 0.125
];

/// Wraps the resource allocation array and an error status code into a branchless `Result`.
///
/// If `err_code == u32::MAX`, returns `Ok(pi_res)`.
/// Otherwise, maps the code to a [`StabilityRefusal`] and returns `Err`.
///
/// # Inputs
/// - `pi_res`: The computed resource allocation distribution array.
/// - `err_code`: The status/error code.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::fixed::Fixed;
/// use bcinr_cmca::allocator::{wrap_result, StabilityRefusal};
///
/// let pi = [Fixed::ZERO; 8];
/// let ok_res = wrap_result(pi, u32::MAX);
/// assert_eq!(ok_res, Ok(pi));
///
/// let err_res = wrap_result(pi, 0);
/// assert_eq!(err_res, Err(StabilityRefusal::CertificateMissing));
/// ```
///
/// # Branchless Contract
pub fn wrap_result(
    pi_res: [Fixed; N],
    err_code: u32,
) -> Result<[Fixed; N], StabilityRefusal> {
    let err_val = REFUSALS[(err_code as usize) & 31];
    let is_ok = const_eq_u32(err_code, u32::MAX);
    let outcomes = [Err(err_val), Ok(pi_res)];
    outcomes[(is_ok as usize) & 1]
}

/// Selects branchlessly between two `u32` values based on a condition mask.
///
/// If `condition != 0`, returns `a`. Otherwise, returns `b`.
///
/// # Inputs
/// - `condition`: A mask/condition value.
/// - `a`: Return value if condition is non-zero.
/// - `b`: Return value if condition is zero.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::allocator::const_select_u32;
///
/// assert_eq!(const_select_u32(1, 42, 100), 42);
/// assert_eq!(const_select_u32(0, 42, 100), 100);
/// ```
///
/// # Branchless Contract
#[inline(always)]
pub fn const_select_u32(condition: u32, a: u32, b: u32) -> u32 {
    let cond = core::hint::black_box(condition);
    let cond_val = (cond | cond.wrapping_neg()) >> 31;
    let mask = 0u32.wrapping_sub(cond_val);
    (core::hint::black_box(a) & mask) | (core::hint::black_box(b) & !mask)
}

/// Performs a branchless "less than" comparison between two `u32` values.
///
/// Returns `1` if `a < b`, and `0` otherwise.
///
/// # Inputs
/// - `a`: First value.
/// - `b`: Second value.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::allocator::const_lt_u32;
///
/// assert_eq!(const_lt_u32(5, 10), 1);
/// assert_eq!(const_lt_u32(10, 5), 0);
/// assert_eq!(const_lt_u32(5, 5), 0);
/// ```
///
/// # Branchless Contract
#[inline(always)]
pub fn const_lt_u32(a: u32, b: u32) -> u32 {
    let a_bb = core::hint::black_box(a);
    let b_bb = core::hint::black_box(b);
    ((a_bb ^ ((a_bb ^ b_bb) | (a_bb.wrapping_sub(b_bb) ^ b_bb))) >> 31) & 1
}

/// Performs a branchless "equals" check between two `u32` values.
///
/// Returns `1` if `a == b`, and `0` otherwise.
///
/// # Inputs
/// - `a`: First value.
/// - `b`: Second value.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::allocator::const_eq_u32;
///
/// assert_eq!(const_eq_u32(42, 42), 1);
/// assert_eq!(const_eq_u32(42, 100), 0);
/// ```
///
/// # Branchless Contract
#[inline(always)]
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}

/// Selects branchlessly between two boolean values based on a condition mask.
///
/// If `condition != 0`, returns `a`. Otherwise, returns `b`.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::allocator::const_select_bool;
///
/// assert_eq!(const_select_bool(1, true, false), true);
/// assert_eq!(const_select_bool(0, true, false), false);
/// ```
#[inline(always)]
pub fn const_select_bool(condition: u32, a: bool, b: bool) -> bool {
    const_select_u32(condition, a as u32, b as u32) != 0
}

/// Computes the maximum of two `i32` values branchlessly.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
#[inline(always)]
fn const_max_i32(a: i32, b: i32) -> i32 {
    let diff_64 = (a as i64).wrapping_sub(b as i64);
    let is_lt = (diff_64 >> 63) & 1;
    const_select_u32(is_lt as u32, b as u32, a as u32) as i32
}

/// Marker struct indicating certified learning mode is active.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct CertifiedLearning;

/// Marker struct indicating selection-only mode is active.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct CertifiedSelectionOnly;

/// Proof token certifying that the control state has been admitted.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct AdmittedControlState;

/// Proof token certifying receipt of a valid security certificate.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct CertificateReceipt;

/// Proof token certifying receipt of a valid envelope.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvelopeReceipt;

/// Proof token certifying receipt of a valid outcome.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeReceipt;

/// A proof token certifying that an adaptive update is authorized.
///
/// Constructed via `AdaptiveUpdate::new` when the control mode and environmental bounds
/// are validated.
#[derive(Debug, PartialEq, Eq)]
pub struct AdaptiveUpdate<Mode> {
    _mode: core::marker::PhantomData<Mode>,
}

impl<Mode> Clone for AdaptiveUpdate<Mode> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            _mode: core::marker::PhantomData,
        }
    }
}
impl<Mode> Copy for AdaptiveUpdate<Mode> {}

impl AdaptiveUpdate<CertifiedLearning> {
    /// Constructs a new `AdaptiveUpdate` receipt under certified learning mode.
    ///
    /// Validates that the temperature does not exceed the profile ceiling and the
    /// distinguishability meets the profile floor.
    ///
    /// # Complexity
    /// $O(1)$ constant time, branchless.
    #[inline(always)]
    pub fn new(
        _state: AdmittedControlState,
        _cert: CertificateReceipt,
        _env: EnvelopeReceipt,
        _outcome: OutcomeReceipt,
        temperature: Fixed,
        distinguishability: Fixed,
    ) -> Option<Self> {
        let temp_ceil = ((crate::generated::stability_profile::PROFILE.temperature_ceiling.raw * 65536) / 1_000_000_000) as u32;
        let dist_floor = ((crate::generated::stability_profile::PROFILE.distinguishability_floor.raw * 65536) / 1_000_000_000) as u32;

        let temp_ok = const_lt_u32(temp_ceil, temperature.0) == 0;
        let dist_ok = const_lt_u32(distinguishability.0, dist_floor) == 0;
        let ok = temp_ok & dist_ok;

        let outcomes = [None, Some(Self { _mode: core::marker::PhantomData })];
        outcomes[(ok as usize) & 1]
    }
}

/// Computes `base^exponent` branchlessly using fixed-point log2 and exp2 approximations.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
#[inline(always)]
pub(crate) fn power(base: Fixed, exponent: Fixed) -> Fixed {
    let base_is_zero = const_eq_u32(base.0, 0);
    let log_val = base.log2();
    let exp_signed = exponent.0 as i32;
    let log_signed = log_val.0 as i32;
    let product = (((exp_signed as i64).wrapping_mul(log_signed as i64)) >> 16) as i32;
    let pow_val = Fixed(product as u32).exp2();
    let exp_val = exponent.0 as i32;
    let exp_gt_zero = (((0i32.wrapping_sub(exp_val)) >> 31) & 1) as u32;
    let exp_eq_zero = const_eq_u32(exponent.0, 0);
    let zero_res = const_select_u32(exp_eq_zero, Fixed::ONE.0,
                    const_select_u32(exp_gt_zero, 0, u32::MAX));
    Fixed(const_select_u32(base_is_zero, zero_res, pow_val.0))
}

/// Clamps a fixed-point value within `[min_val, max_val]` branchlessly.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
#[inline(always)]
pub(crate) fn clip(val: Fixed, min_val: Fixed, max_val: Fixed) -> Fixed {
    let lt_min = const_lt_u32(val.0, min_val.0);
    let val_or_min = const_select_u32(lt_min, min_val.0, val.0);
    let gt_max = const_lt_u32(max_val.0, val_or_min);
    Fixed(const_select_u32(gt_max, max_val.0, val_or_min))
}

/// Computes the divergence metric $\kappa_v$ for a subtree at node `v` under policy `k`.
///
/// $\kappa_v$ measures the Kullback-Leibler (KL) divergence between direct child allocations
/// and subtree leaf distributions.
///
/// # Complexity
/// $O(N^2)$ operations, which is $O(1)$ since $N=8$.
#[inline(never)]
pub(crate) fn compute_kappa(
    v: usize,
    _q_idx: usize,
    k: usize,
    parent: &[i32; N],
    _is_leaf: &[bool; N],
    is_subtree_leaf_v: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    node_masses: &[[Fixed; N]; K],
    q_val: Fixed,
) -> Fixed {
    let k_masked = k & 3;
    
    let mut x = [0i32; N];
    unroll_8_static!(i, {
        let mut log_m = 0u32;
        unroll_4_static!(k_idx, {
            let matches = const_eq_u32(k_masked as u32, k_idx as u32);
            log_m = const_select_u32(matches, node_masses[k_idx & 3][i & 7].log2().0, log_m);
        });
        let q_signed = q_val.0 as i32;
        x[i & 7] = (((q_signed as i64).wrapping_mul(log_m as i32 as i64)) >> 16) as i32;
    });
    
    let mut x_max_meas = i32::MIN;
    unroll_8_static!(j, {
        let is_child = const_eq_u32(parent[j & 7] as u32, v as u32);
        let x_safe = const_select_u32(is_child, x[j & 7] as u32, i32::MIN as u32) as i32;
        x_max_meas = const_max_i32(x_max_meas, x_safe);
    });
    
    let mut sum_exp_meas = Fixed::ZERO;
    unroll_8_static!(j, {
        let is_child = const_eq_u32(parent[j & 7] as u32, v as u32);
        let a_prime = x[j & 7].wrapping_sub(x_max_meas);
        let exp_val = Fixed(a_prime as u32).exp2();
        sum_exp_meas += Fixed(const_select_u32(is_child, exp_val.0, 0));
    });
    let l_meas = x_max_meas.wrapping_add(sum_exp_meas.log2().0 as i32);

    let mut x_max_leaf = i32::MIN;
    unroll_8_static!(x_idx, {
        let is_sub = is_subtree_leaf_v[x_idx & 7];
        let x_safe = const_select_u32(is_sub as u32, x[x_idx & 7] as u32, i32::MIN as u32) as i32;
        x_max_leaf = const_max_i32(x_max_leaf, x_safe);
    });
    
    let mut sum_exp_leaf = Fixed::ZERO;
    unroll_8_static!(x_idx, {
        let is_sub = is_subtree_leaf_v[x_idx & 7];
        let a_prime = x[x_idx & 7].wrapping_sub(x_max_leaf);
        let exp_val = Fixed(a_prime as u32).exp2();
        sum_exp_leaf += Fixed(const_select_u32(is_sub as u32, exp_val.0, 0));
    });
    let l_leaf = x_max_leaf.wrapping_add(sum_exp_leaf.log2().0 as i32);

    let mut kappa = Fixed::ZERO;
    unroll_8_static!(c, {
        let is_child = const_eq_u32(parent[c & 7] as u32, v as u32);
        
        let mut x_max_c = i32::MIN;
        unroll_8_static!(x_idx, {
            let is_sub_c = is_subtree_leaf[c & 7][x_idx & 7];
            let x_safe = const_select_u32(is_sub_c as u32, x[x_idx & 7] as u32, i32::MIN as u32) as i32;
            x_max_c = const_max_i32(x_max_c, x_safe);
        });
        
        let mut sum_exp_c = Fixed::ZERO;
        unroll_8_static!(x_idx, {
            let is_sub_c = is_subtree_leaf[c & 7][x_idx & 7];
            let a_prime = x[x_idx & 7].wrapping_sub(x_max_c);
            let exp_val = Fixed(a_prime as u32).exp2();
            sum_exp_c += Fixed(const_select_u32(is_sub_c as u32, exp_val.0, 0));
        });
        let y_c = x_max_c.wrapping_add(sum_exp_c.log2().0 as i32);
        
        let log_s_leaf = y_c.wrapping_sub(l_leaf);
        let log_s_meas = x[c & 7].wrapping_sub(l_meas);
        let log_diff = log_s_leaf.wrapping_sub(log_s_meas);
        
        let s_leaf_val = Fixed(log_s_leaf as u32).exp2();
        let term = s_leaf_val * Fixed(log_diff as u32);
        let term_safe = Fixed(const_select_u32(const_eq_u32(s_leaf_val.0, 0), 0, term.0));
        
        kappa += Fixed(const_select_u32(is_child, term_safe.0, 0));
    });
    
    kappa
}

/// Performs a single straight-line flow propagation step down the node forest.
///
/// Divides the incoming flow into flat and descendant parts, distributing them
/// branchlessly according to normalized leaf and child weights.
///
/// # Complexity
/// $O(N^2)$ operations, which is $O(1)$ since $N=8$.
#[inline(never)]
fn flow_step(
    parent: &[i32; N],
    is_leaf: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    rho: &[Fixed; N],
    child_w: &[[Fixed; N]; N],
    cw_sum: &[Fixed; N],
    leaf_w: &[[Fixed; N]; N],
    lw_sum: &[Fixed; N],
    alloc_flow: &mut [Fixed; N],
    flat_alloc: &mut [Fixed; N],
) {
    unroll_8_static!(v, {
        let has_children = !is_leaf[v & 7];
        
        let flat_part = Fixed(const_select_u32(has_children as u32, ((Fixed::ONE - rho[v & 7]) * alloc_flow[v & 7]).0, 0));
        let desc_part = Fixed(const_select_u32(has_children as u32, (rho[v & 7] * alloc_flow[v & 7]).0, 0));
        
        #[allow(unused_variables)]
        let l_cond = const_eq_u32(lw_sum[v & 7].0, 0);
        #[cfg(feature = "mutant_3")]
        let lw_denom = Fixed::ONE.0;
        #[cfg(not(feature = "mutant_3"))]
        let lw_denom = const_select_u32(l_cond, Fixed::ONE.0, lw_sum[v & 7].0);
        
        let c_cond = const_eq_u32(cw_sum[v & 7].0, 0);
        let cw_denom = const_select_u32(c_cond, Fixed::ONE.0, cw_sum[v & 7].0);

        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7] & has_children;
            let flat_addition = flat_part * leaf_w[v & 7][x & 7].saturating_div(Fixed(lw_denom));
            flat_alloc[x & 7] += Fixed(const_select_u32(is_sub as u32, flat_addition.0, 0));
            
            let is_child = (parent[x & 7] == v as i32) & has_children;
            let flow_addition = desc_part * child_w[v & 7][x & 7].saturating_div(Fixed(cw_denom));
            alloc_flow[x & 7] += Fixed(const_select_u32(is_child as u32, flow_addition.0, 0));
        });
        
        alloc_flow[v & 7] = Fixed(const_select_u32(has_children as u32, 0, alloc_flow[v & 7].0));
    });
}

/// Computes the allocation vector $\pi_{k, q}$ for a specific model `k` and lens spec `q`.
///
/// Traverses down the hierarchy by initializing roots and propagating flow via repeated
/// straight-line iterations of [`flow_step`].
///
/// # Complexity
/// $O(N^2)$ operations, which is $O(1)$ since $N=8$.
#[inline(never)]
fn compute_pi_kq_for_kq(
    k_actual: usize,
    q_idx: usize,
    q_val_mutated: Fixed,
    parent: &[i32; N],
    is_leaf: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    node_masses: &[[Fixed; N]; K],
    local_weights: &[[Fixed; 2 * Q]; N],
) -> [Fixed; N] {
    let mut a_roots = [0i32; N];
    let mut a_max_root = i32::MIN;
    unroll_8_static!(i, {
        let is_r = parent[i & 7] == -1;
        let a_i = (((q_val_mutated.0 as i32 as i64).wrapping_mul(node_masses[k_actual & 3][i & 7].log2().0 as i32 as i64)) >> 16) as i32;
        a_roots[i & 7] = const_select_u32(is_r as u32, a_i as u32, i32::MIN as u32) as i32;
        a_max_root = const_max_i32(a_max_root, a_roots[i & 7]);
    });
    
    let mut root_w = [Fixed::ZERO; N];
    let mut root_w_sum = Fixed::ZERO;
    unroll_8_static!(i, {
        root_w[i & 7] = Fixed(const_select_u32((parent[i & 7] == -1) as u32, Fixed(a_roots[i & 7].wrapping_sub(a_max_root) as u32).exp2().0, 0));
        root_w_sum += root_w[i & 7];
    });
    
    let mut alloc_flow = [Fixed::ZERO; N];
    unroll_8_static!(i, {
        let is_r = parent[i & 7] == -1;
        let r_cond = const_eq_u32(root_w_sum.0, 0);
        let flow_val = root_w[i & 7].saturating_div(Fixed(const_select_u32(r_cond, Fixed::ONE.0, root_w_sum.0)));
        alloc_flow[i & 7] = Fixed(const_select_u32(is_r as u32, flow_val.0, 0));
    });
    
    let mut rho = [Fixed::ZERO; N];
    let mut child_w = [[Fixed::ZERO; N]; N];
    let mut cw_sum = [Fixed::ZERO; N];
    let mut leaf_w = [[Fixed::ZERO; N]; N];
    let mut lw_sum = [Fixed::ZERO; N];
    
    unroll_8_static!(v, {
        let w_sum = local_weights[v & 7][(2 * q_idx) & 7] + local_weights[v & 7][(2 * q_idx + 1) & 7];
        rho[v & 7] = Fixed(const_select_u32(const_eq_u32(w_sum.0, 0), 32768, local_weights[v & 7][(2 * q_idx + 1) & 7].saturating_div(w_sum).0));
        
        let mut a_c = [0i32; N];
        let mut a_max_c = i32::MIN;
        unroll_8_static!(c, {
            let is_c = parent[c & 7] == v as i32;
            a_c[c & 7] = const_select_u32(is_c as u32, (((q_val_mutated.0 as i32 as i64).wrapping_mul(node_masses[k_actual & 3][c & 7].log2().0 as i32 as i64)) >> 16) as u32, i32::MIN as u32) as i32;
            a_max_c = const_max_i32(a_max_c, a_c[c & 7]);
        });
        unroll_8_static!(c, {
            let matches = a_c[c & 7] != i32::MIN;
            child_w[v & 7][c & 7] = Fixed(const_select_u32(matches as u32, Fixed(a_c[c & 7].wrapping_sub(a_max_c) as u32).exp2().0, 0));
            cw_sum[v & 7] += child_w[v & 7][c & 7];
        });
        
        let mut a_l = [0i32; N];
        let mut a_max_l = i32::MIN;
        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7];
            a_l[x & 7] = const_select_u32(is_sub as u32, (((q_val_mutated.0 as i32 as i64).wrapping_mul(node_masses[k_actual & 3][x & 7].log2().0 as i32 as i64)) >> 16) as u32, i32::MIN as u32) as i32;
            a_max_l = const_max_i32(a_max_l, a_l[x & 7]);
        });
        unroll_8_static!(x, {
            let matches = a_l[x & 7] != i32::MIN;
            leaf_w[v & 7][x & 7] = Fixed(const_select_u32(matches as u32, Fixed(a_l[x & 7].wrapping_sub(a_max_l) as u32).exp2().0, 0));
            lw_sum[v & 7] += leaf_w[v & 7][x & 7];
        });
    });
    
    let mut flat_alloc = [Fixed::ZERO; N];
    
    // Call flow_step 8 times sequentially to avoid stack frame nesting
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);
    flow_step(parent, is_leaf, is_subtree_leaf, &rho, &child_w, &cw_sum, &leaf_w, &lw_sum, &mut alloc_flow, &mut flat_alloc);

    let mut res = [Fixed::ZERO; N];
    unroll_8_static!(x, res[x & 7] = flat_alloc[x & 7] + alloc_flow[x & 7]);
    res
}

/// Allocates resources down the node forest branchlessly, performing MWU step updates,
/// stable projections, and explore floors.
///
/// This is the entry point for the Cascade Allocation engine.
///
/// # Mathematical Behavior
///
/// 1. **Divergence Guard & MWU**: For each internal node, computes the divergence $\kappa_v$ between child
///    allocations and subtree leaf distributions. If $\kappa_v > \epsilon_{\kappa}$ and learning is authorized
///    by `proof`, updates routing weights multiplicatively using payoffs scaled by learning rate $\beta$.
/// 2. **Cascade flow propagation**: Distributes flow from roots to leaves over the hierarchy of $N$ nodes.
/// 3. **Stable projection**: Scales leaf allocations by $\exp(-\mu_x \cdot c_x)$ and normalizes.
/// 4. **Explore floor mixture**: Restricts allocations from dropping below $\frac{\eta}{n_L}$ by mixing the
///    projection with a uniform distribution.
///
/// # Inputs
/// - `states`: Packed semantic states for the $N$ nodes.
/// - `lenses`: Lenses defining policy priorities.
/// - `lambda`: Weighting matrix mapping models and lenses to overall priority.
/// - `eta`: Explore floor parameter $\eta \in [0, 1]$.
/// - `parent`: Forest structure represented by parent indices (where `-1` indicates root).
/// - `weights`: Multiplicative routing weights (updated in place).
/// - `payoffs`: Environment payoff feedback for each decision slot.
/// - `zeta`: Learning rate parameter.
/// - `epsilon_kappa`: Divergence update threshold.
/// - `mu`: Resource prices vector.
/// - `costs`: Operational costs vector.
/// - `t`: Current epoch index.
/// - `last_switch_t`: Epoch of the last policy switch (updated in place).
/// - `prev_mode`: Currently active policy mode index (updated in place).
/// - `tau_d`: Minimum dwell rounds constraint.
/// - `digest`: Security certificate digest.
/// - `proof`: Verification proof authorizing learning updates.
///
/// # Outputs
/// Returns the resource allocation probability distribution over the $N$ nodes if successful,
/// or a [`StabilityRefusal`] code otherwise.
///
/// # Complexity
/// - **Time Complexity**: $O(K \cdot Q \cdot N^2)$ operations ($O(1)$ constant time).
/// - **Space Complexity**: $O(1)$ auxiliary stack space.
/// - **Cyclomatic Complexity**: $CC = 1$.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::fixed::Fixed;
/// use bcinr_cmca::allocator::{allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, CertifiedLearning};
/// use bcinr_cmca::generated::case_studies::{OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, N, Q};
/// use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
///
/// let mut weights = [[Fixed::ONE; 2 * Q]; N];
/// let payoffs = [[Fixed::ZERO; 2 * Q]; N];
/// let mut last_switch_t = 0;
/// let mut prev_mode = 0;
/// let parent = [-1; N];
/// let mu = [Fixed::ZERO; N];
/// let costs = [Fixed::ZERO; N];
///
/// let proof = AdaptiveUpdate::new(
///     AdmittedControlState,
///     CertificateReceipt,
///     EnvelopeReceipt,
///     OutcomeReceipt,
///     Fixed::ZERO,
///     Fixed::ONE,
/// );
///
/// let result = allocate(
///     &OBJECT_REGISTRY,
///     &LENS_REGISTRY,
///     &LAMBDA,
///     ETA,
///     &parent,
///     &mut weights,
///     &payoffs,
///     Fixed::ZERO,
///     Fixed::ZERO,
///     &mu,
///     &costs,
///     0,
///     &mut last_switch_t,
///     &mut prev_mode,
///     500,
///     CERTIFICATE_DIGEST,
///     proof.as_ref(),
/// );
/// assert!(result.is_ok());
/// ```
///
/// # Branchless Contract
pub fn allocate(
    states: &[PackedSemanticState; N],
    lenses: &[LensSpec; Q],
    lambda: &[[Fixed; Q]; K],
    eta: Fixed,
    parent: &[i32; N],
    weights: &mut [[Fixed; 2 * Q]; N],
    payoffs: &[[Fixed; 2 * Q]; N],
    zeta: Fixed,
    epsilon_kappa: Fixed,
    mu: &[Fixed; N],
    costs: &[Fixed; N],
    t: u32,
    last_switch_t: &mut u32,
    prev_mode: &mut u32,
    tau_d: u32,
    digest: [u8; 32],
    proof: Option<&AdaptiveUpdate<CertifiedLearning>>,
) -> Result<[Fixed; N], StabilityRefusal> {
    let mut local_weights = *weights;
    let mut local_last_switch_t = *last_switch_t;
    let mut local_prev_mode = *prev_mode;

    let beta_max = Fixed(6553);
    let m_min = Fixed(6);
    let m_max = Fixed(65536000);
    let mu_max = Fixed(6553600);

    let proof_some = proof.is_some();
    let degrade_to_certified_selection = proof.is_none();

    let mut digest_match = 1u32;
    unroll_32_static!(i, {
        digest_match &= const_eq_u32(digest[i & 31] as u32, crate::generated::stability_profile::CERTIFICATE_DIGEST[i & 31] as u32);
    });
    let digest_err = const_eq_u32(digest_match, 0) != 0;

    let mut gd_ok = true;
    unroll_5_static!(i, {
        let mut sum_g_d = 0u128;
        unroll_5_static!(j, {
            let g_raw = crate::generated::stability_profile::GAIN_MATRIX[i][j].raw as u128;
            let d_raw = crate::generated::stability_profile::WEIGHT_VECTOR[j].raw as u128;
            sum_g_d += g_raw * d_raw;
        });
        let lhs = sum_g_d / 1_000_000_000;
        let d_i_raw = crate::generated::stability_profile::WEIGHT_VECTOR[i].raw as u128;
        let delta_raw = crate::generated::stability_profile::CONTRACTION_MARGIN.raw as u128;
        let rhs = d_i_raw - (delta_raw * d_i_raw / 1_000_000_000);
        gd_ok = gd_ok & (lhs <= rhs);
    });

    let zeta_w_max_q16 = ((crate::generated::stability_profile::ZETA_W_MAX.raw * 65536) / 1_000_000_000) as u32;
    let eta_g_min_q16 = ((crate::generated::stability_profile::ETA_G_MIN.raw * 65536) / 1_000_000_000) as u32;

    let lr_err = const_lt_u32(zeta_w_max_q16, zeta.0) != 0;
    let dwell_err = const_lt_u32(tau_d, crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN) != 0;
    
    let mut q_err = false;
    unroll_4_static!(q_idx, {
        let q_val = lenses[q_idx & 3].q.0 as i32;
        q_err = q_err | (q_val < -131072) | (q_val > 131072);
    });

    let mut price_err = false;
    unroll_8_static!(i, {
        price_err = price_err | (const_lt_u32(mu_max.0, mu[i & 7].0) != 0);
    });

    let eta_err = const_lt_u32(eta.0, eta_g_min_q16) != 0;

    let is_zeta_less = const_lt_u32(zeta.0, beta_max.0);
    let beta = Fixed(const_select_u32(is_zeta_less, zeta.0, beta_max.0));
    let beta_m_max_q16 = ((crate::generated::stability_profile::BETA_M_MAX.raw * 65536) / 1_000_000_000) as u32;
    let beta_err = const_lt_u32(beta_m_max_q16, beta.0) != 0;

    let has_error = !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
    let freeze_learning = has_error & degrade_to_certified_selection;

    let mut is_leaf = [true; N];
    unroll_8_static!(i, {
        unroll_8_static!(j, {
            let is_match = parent[j & 7] == i as i32;
            is_leaf[i & 7] = is_leaf[i & 7] & !is_match;
        });
    });
    
    #[allow(non_snake_case)]
    let mut P = [[-1i32; N]; 8];
    unroll_8_static!(j, {
        P[0][j] = parent[j];
    });

    // Level 1
    unroll_8_static!(j, {
        let parent_node = P[0][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[1][j] = p_next;
    });

    // Level 2
    unroll_8_static!(j, {
        let parent_node = P[1][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[2][j] = p_next;
    });

    // Level 3
    unroll_8_static!(j, {
        let parent_node = P[2][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[3][j] = p_next;
    });

    // Level 4
    unroll_8_static!(j, {
        let parent_node = P[3][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[4][j] = p_next;
    });

    // Level 5
    unroll_8_static!(j, {
        let parent_node = P[4][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[5][j] = p_next;
    });

    // Level 6
    unroll_8_static!(j, {
        let parent_node = P[5][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[6][j] = p_next;
    });

    // Level 7
    unroll_8_static!(j, {
        let parent_node = P[6][j];
        let mut p_next = -1i32;
        unroll_8_static!(p_idx, {
            let matches = const_eq_u32(parent_node as u32, p_idx as u32);
            p_next = const_select_u32(matches, parent[p_idx] as u32, p_next as u32) as i32;
        });
        P[7][j] = p_next;
    });

    #[allow(non_snake_case)]
    let P_bb = core::hint::black_box(P);

    let mut is_descendant = [[false; N]; N];
    unroll_8_static!(i, {
        unroll_8_static!(j, {
            let mut matched = const_eq_u32(j as u32, i as u32);
            unroll_8_static!(level, {
                matched |= const_eq_u32(P_bb[level][j] as u32, i as u32);
            });
            is_descendant[i][j] = matched != 0;
        });
    });

    let is_descendant = core::hint::black_box(is_descendant);

    let mut is_subtree_leaf = [[false; N]; N];
    unroll_8_static!(i, {
        unroll_8_static!(k, { is_subtree_leaf[i & 7][k & 7] = is_leaf[k & 7] & is_descendant[i & 7][k & 7]; });
    });
    
    let mut node_masses = [[Fixed::ZERO; N]; K];
    unroll_8_static!(i, {
        let state = &states[i & 7];
        let m0 = (state.factors[0] * Fixed::from_num(5) + state.factors[1]) * state.factors[4] * state.factors[2];
        let m1 = (state.factors[8] + state.factors[9]) * state.factors[5] * state.factors[2];
        let m2 = state.factors[8] * state.factors[6];
        let m3 = state.factors[8] * state.factors[7];
        
        node_masses[0][i & 7] = m0;
        node_masses[1][i & 7] = m1;
        node_masses[2][i & 7] = m2;
        node_masses[3][i & 7] = m3;
    });
    
    unroll_4_static!(k, {
        unroll_8_static!(i, { node_masses[k & 3][i & 7] = clip(node_masses[k & 3][i & 7], m_min, m_max); });
    });

    let mut root_idx = 0usize;
    unroll_8_static!(i, {
        let is_root = parent[i & 7] == -1;
        root_idx = const_select_u32(is_root as u32, i as u32, root_idx as u32) as usize;
    });

    // Load root weights branchlessly
    let mut root_weights = [Fixed::ZERO; 2 * Q];
    unroll_8_static!(idx, {
        let matches = const_eq_u32(root_idx as u32, idx as u32);
        unroll_8_static!(e, {
            root_weights[e & 7] = Fixed(const_select_u32(matches, local_weights[idx & 7][e & 7].0, root_weights[e & 7].0));
        });
    });

    let mut max_w = Fixed::ZERO;
    let mut dom_mode = 0u32;
    unroll_8_static!(e, {
        let w = root_weights[e & 7];
        let is_greater = const_lt_u32(max_w.0, w.0);
        max_w = Fixed(const_select_u32(is_greater, w.0, max_w.0));
        dom_mode = const_select_u32(is_greater, e as u32, dom_mode);
    });

    let switch_wanted = dom_mode != local_prev_mode;
    let can_switch = t.wrapping_sub(local_last_switch_t) >= tau_d;
    let update_allowed = !(switch_wanted & !can_switch) & !freeze_learning & proof_some;

    unroll_8_static!(v, {
        let has_children = !is_leaf[v & 7];
        
        let mut is_subtree_leaf_v = [false; N];
        unroll_8_static!(x, {
            is_subtree_leaf_v[x] = is_subtree_leaf[v & 7][x & 7];
        });

        unroll_4_static!(q_idx, {
            let mut q_val_mutated = lenses[q_idx & 3].q;
            #[cfg(feature = "mutant_2")]
            {
                q_val_mutated = Fixed(0u32.wrapping_sub(q_val_mutated.0));
            }
            let kappa = compute_kappa(
                v,
                q_idx,
                0,
                parent,
                &is_leaf,
                &is_subtree_leaf_v,
                &is_subtree_leaf,
                &node_masses,
                q_val_mutated,
            );
            let update_active = const_lt_u32(epsilon_kappa.0, kappa.0);
            let w_flat = local_weights[v & 7][(2 * q_idx) & 7];
            let w_desc = local_weights[v & 7][(2 * q_idx + 1) & 7];
            let is_updating = has_children & (update_active != 0) & update_allowed;
            local_weights[v & 7][(2 * q_idx) & 7] = Fixed(const_select_u32(is_updating as u32, (w_flat * (beta * payoffs[v & 7][(2 * q_idx) & 7]).exp()).0, w_flat.0));
            local_weights[v & 7][(2 * q_idx + 1) & 7] = Fixed(const_select_u32(is_updating as u32, (w_desc * (beta * payoffs[v & 7][(2 * q_idx + 1) & 7]).exp()).0, w_desc.0));
        });
        
        unroll_4_static!(q_idx, {
            let w_flat = local_weights[v & 7][(2 * q_idx) & 7];
            let w_desc = local_weights[v & 7][(2 * q_idx + 1) & 7];
            let sum_div = w_flat + w_desc;
            local_weights[v & 7][(2 * q_idx) & 7] = Fixed(const_select_u32(update_allowed as u32, w_flat.saturating_div(sum_div).0, w_flat.0));
            local_weights[v & 7][(2 * q_idx + 1) & 7] = Fixed(const_select_u32(update_allowed as u32, w_desc.saturating_div(sum_div).0, w_desc.0));
        });
    });

    let mut new_dom_mode = 0u32;
    let mut new_max_w = Fixed::ZERO;
    
    // Reload root weights
    unroll_8_static!(idx, {
        let matches = const_eq_u32(root_idx as u32, idx as u32);
        unroll_8_static!(e, {
            root_weights[e & 7] = Fixed(const_select_u32(matches, local_weights[idx & 7][e & 7].0, root_weights[e & 7].0));
        });
    });
    
    unroll_8_static!(e, {
        let w = root_weights[e & 7];
        let is_greater = const_lt_u32(new_max_w.0, w.0);
        new_max_w = Fixed(const_select_u32(is_greater, w.0, new_max_w.0));
        new_dom_mode = const_select_u32(is_greater, e as u32, new_dom_mode);
    });

    let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;
    local_last_switch_t = const_select_u32(did_switch as u32, t, local_last_switch_t);
    local_prev_mode = const_select_u32(did_switch as u32, new_dom_mode, local_prev_mode);

    let mut pi_kq = [[[Fixed::ZERO; N]; Q]; K];
    
    unroll_4_static!(k, {
        #[cfg(feature = "mutant_1")]
        const k_actual: usize = 0;
        #[cfg(not(feature = "mutant_1"))]
        const k_actual: usize = k;
        
        unroll_4_static!(q_idx, {
            let q_val_mutated = lenses[q_idx & 3].q;
            #[cfg(feature = "mutant_2")]
            let q_val_mutated = Fixed(0u32.wrapping_sub(q_val_mutated.0));
            
            let res_kq = compute_pi_kq_for_kq(
                k_actual,
                q_idx,
                q_val_mutated,
                parent,
                &is_leaf,
                &is_subtree_leaf,
                &node_masses,
                &local_weights,
            );
            unroll_8_static!(x, pi_kq[k & 3][q_idx & 3][x & 7] = res_kq[x & 7]);
        });
    });
    
    let mut pi_combined = [Fixed::ZERO; N];
    unroll_4_static!(k, {
        unroll_4_static!(q_idx, {
            unroll_8_static!(x, {
                let term = lambda[k & 3][q_idx & 3] * pi_kq[k & 3][q_idx & 3][x & 7];
                pi_combined[x & 7] += term;
            });
        });
    });
    
    let mut pi_res = [Fixed::ZERO; N];
    let mut priced_sum = Fixed::ZERO;
    unroll_8_static!(x, {
        #[cfg(feature = "mutant_5")]
        let mu_actual = mu[x & 7];
        #[cfg(not(feature = "mutant_5"))]
        let mu_actual = clip(mu[x & 7], Fixed::ZERO, mu_max);
        
        let p = pi_combined[x & 7] * Fixed(0u32.wrapping_sub((mu_actual * costs[x & 7]).0)).exp();
        priced_sum += Fixed(const_select_u32(is_leaf[x & 7] as u32, p.0, 0));
    });
    let psd = Fixed(const_select_u32(const_eq_u32(priced_sum.0, 0), Fixed::ONE.0, priced_sum.0));
    let mut nl = 0u32;
    unroll_8_static!(i, { nl += is_leaf[i & 7] as u32; });
    unroll_8_static!(x, {
        #[cfg(feature = "mutant_5")]
        let mu_actual = mu[x & 7];
        #[cfg(not(feature = "mutant_5"))]
        let mu_actual = clip(mu[x & 7], Fixed::ZERO, mu_max);
        
        let p_mu = (pi_combined[x & 7] * Fixed(0u32.wrapping_sub((mu_actual * costs[x & 7]).0)).exp()).saturating_div(psd);
        
        #[cfg(feature = "mutant_4")]
        let eta_actual = zeta;
        #[cfg(not(feature = "mutant_4"))]
        let eta_actual = eta;
        
        // Use lookup table to avoid division
        let mut nl_recip = Fixed::ZERO;
        unroll_9_static!(idx, {
            let matches = const_eq_u32(nl, idx as u32);
            nl_recip = Fixed(const_select_u32(matches, LEAF_RECIP[idx].0, nl_recip.0));
        });
        
        let val = (eta_actual * nl_recip) + ((Fixed::ONE - eta_actual) * p_mu);
        let pi_val = pi_res[x & 7];
        pi_res[x & 7] = Fixed(const_select_u32(is_leaf[x & 7] as u32, val.0, pi_val.0));
    });

    let has_refusal = has_error & !degrade_to_certified_selection;
    unroll_8_static!(v, {
        unroll_8_static!(e, { weights[v & 7][e & 7] = Fixed(const_select_u32(has_refusal as u32, weights[v & 7][e & 7].0, local_weights[v & 7][e & 7].0)); });
    });
    *last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
    *prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);

    let err_val = const_select_u32(q_err as u32, 5, const_select_u32(dwell_err as u32, 4, const_select_u32((lr_err | beta_err | eta_err) as u32, 3, const_select_u32((!gd_ok) as u32, 1, const_select_u32(digest_err as u32, 10, 7)))));
    wrap_result(pi_res, const_select_u32(has_refusal as u32, err_val, u32::MAX))
}
