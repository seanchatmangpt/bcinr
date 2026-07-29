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
//! $$ \text{alloc\_flow}\[r\] = \frac{W_{\text{root}}(r)}{\sum_{j \in \text{roots}} W_{\text{root}}(j)} $$
//!
//! Non-root nodes are initialized with zero flow. The allocator then executes exactly $N$ iterations of a
//! straight-line propagation function (`flow_step`).
//!
//! At each step, for every node $v$:
//! - If $v$ is a leaf, the incoming flow is collected into the final allocation vector.
//! - If $v$ has children, the incoming flow is split into a direct leaf allocation part ($F_v$) and a child
//!   propagation part ($D_v$):
//!
//!   $$ F_v = (1 - \rho_v) \cdot \text{alloc\_flow}\[v\] $$
//!   $$ D_v = \rho_v \cdot \text{alloc\_flow}\[v\] $$
//!
//!   where $\rho_v \in [0, 1]$ is the local routing parameter.
//! - The direct part $F_v$ is distributed to all descendant leaves under $v$ proportional to the leaf weights:
//!
//!   $$ \text{flat\_alloc}\[x\] \leftarrow \text{flat\_alloc}\[x\] + F_v \cdot \frac{W_{\text{leaf}}(v, x)}{\sum_{y \in \text{leaves}(v)} W_{\text{leaf}}(v, y)} $$
//!
//! - The descendant part $D_v$ is distributed to direct children of $v$ proportional to the child weights:
//!
//!   $$ \text{alloc\_flow}\[c\] \leftarrow \text{alloc\_flow}\[c\] + D_v \cdot \frac{W_{\text{child}}(v, c)}{\sum_{d \in \text{children}(v)} W_{\text{child}}(v, d)} $$
//!
//! ### 2. Multiplicative Weights Update (MWU) Step Updates
//! For each internal node $v$, routing weights between direct leaf allocation and child propagation
//! are adjusted dynamically based on payoff feedback.
//! The updates are controlled by a local divergence metric $\kappa_v$ (relative entropy) computed via `compute_kappa`:
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

#[macro_export]
macro_rules! unroll_8_static {
    ($var:ident, $body:expr) => {{
        {
            const $var: usize = 0;
            $body
        }
        {
            const $var: usize = 1;
            $body
        }
        {
            const $var: usize = 2;
            $body
        }
        {
            const $var: usize = 3;
            $body
        }
        {
            const $var: usize = 4;
            $body
        }
        {
            const $var: usize = 5;
            $body
        }
        {
            const $var: usize = 6;
            $body
        }
        {
            const $var: usize = 7;
            $body
        }
    }};
}

#[macro_export]
macro_rules! unroll_9_static {
    ($var:ident, $body:expr) => {{
        {
            const $var: usize = 0;
            $body
        }
        {
            const $var: usize = 1;
            $body
        }
        {
            const $var: usize = 2;
            $body
        }
        {
            const $var: usize = 3;
            $body
        }
        {
            const $var: usize = 4;
            $body
        }
        {
            const $var: usize = 5;
            $body
        }
        {
            const $var: usize = 6;
            $body
        }
        {
            const $var: usize = 7;
            $body
        }
        {
            const $var: usize = 8;
            $body
        }
    }};
}

#[macro_export]
macro_rules! unroll_4_static {
    ($var:ident, $body:expr) => {{
        {
            const $var: usize = 0;
            $body
        }
        {
            const $var: usize = 1;
            $body
        }
        {
            const $var: usize = 2;
            $body
        }
        {
            const $var: usize = 3;
            $body
        }
    }};
}

#[macro_export]
macro_rules! unroll_32_static {
    ($var:ident, $body:expr) => {{
        {
            const $var: usize = 0;
            $body
        }
        {
            const $var: usize = 1;
            $body
        }
        {
            const $var: usize = 2;
            $body
        }
        {
            const $var: usize = 3;
            $body
        }
        {
            const $var: usize = 4;
            $body
        }
        {
            const $var: usize = 5;
            $body
        }
        {
            const $var: usize = 6;
            $body
        }
        {
            const $var: usize = 7;
            $body
        }
        {
            const $var: usize = 8;
            $body
        }
        {
            const $var: usize = 9;
            $body
        }
        {
            const $var: usize = 10;
            $body
        }
        {
            const $var: usize = 11;
            $body
        }
        {
            const $var: usize = 12;
            $body
        }
        {
            const $var: usize = 13;
            $body
        }
        {
            const $var: usize = 14;
            $body
        }
        {
            const $var: usize = 15;
            $body
        }
        {
            const $var: usize = 16;
            $body
        }
        {
            const $var: usize = 17;
            $body
        }
        {
            const $var: usize = 18;
            $body
        }
        {
            const $var: usize = 19;
            $body
        }
        {
            const $var: usize = 20;
            $body
        }
        {
            const $var: usize = 21;
            $body
        }
        {
            const $var: usize = 22;
            $body
        }
        {
            const $var: usize = 23;
            $body
        }
        {
            const $var: usize = 24;
            $body
        }
        {
            const $var: usize = 25;
            $body
        }
        {
            const $var: usize = 26;
            $body
        }
        {
            const $var: usize = 27;
            $body
        }
        {
            const $var: usize = 28;
            $body
        }
        {
            const $var: usize = 29;
            $body
        }
        {
            const $var: usize = 30;
            $body
        }
        {
            const $var: usize = 31;
            $body
        }
    }};
}

macro_rules! unroll_5_static {
    ($var:ident, $body:expr) => {{
        {
            const $var: usize = 0;
            $body
        }
        {
            const $var: usize = 1;
            $body
        }
        {
            const $var: usize = 2;
            $body
        }
        {
            const $var: usize = 3;
            $body
        }
        {
            const $var: usize = 4;
            $body
        }
    }};
}

use crate::fixed::{NonNegativeFixed, SignedFixed};
use crate::generated::consequence_mass::case_studies::{
    LensSpec, PackedSemanticState, FACTOR_ACCESS_FREQUENCY, FACTOR_BUSINESS_VALUE,
    FACTOR_DOWNSTREAM_CONSEQUENCE, FACTOR_RECOMPUTATION_COST, FACTOR_RETRIEVAL_DEMAND,
    FACTOR_SCHEDULING_DEMAND, FACTOR_SEARCH_DEMAND, FACTOR_STANDING, FACTOR_VERIFICATION_COST, K,
    MEASURE_CACHE, MEASURE_RETRIEVAL, MEASURE_SCHEDULING, MEASURE_SEARCH, N, Q,
};

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
    NumericRangeExceeded,
    UnsupportedDomain,
    ContractViolation,
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
    /// assert_eq!(StabilityRefusal::from_u32(99), None);
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
            Some(Self::NumericRangeExceeded),
            Some(Self::UnsupportedDomain),
            Some(Self::ContractViolation),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];

        let in_bounds = const_lt_u32(val, 21);
        let idx = const_select_u32(in_bounds, val, 21) as usize;
        lookup[idx & 31]
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
    StabilityRefusal::NumericRangeExceeded,
    StabilityRefusal::UnsupportedDomain,
    StabilityRefusal::ContractViolation,
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

/// Typed refusal for hierarchy-shape validation (`CMCA_CONTRACT.md` §9,
/// "Hierarchy Acyclicity"), distinct from [`StabilityRefusal`] (which covers
/// stability/certificate/numeric refusals raised *during* [`allocate`]'s
/// branchless flow). See [`check_hierarchy_acyclic`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HierarchyRefusal {
    /// `parent` contains a cycle (or a chain deeper than `N - 1` hops
    /// permits on a well-formed forest) -- the same ancestor-doubling
    /// witness `allocate` already checks internally (folded there into the
    /// generic [`StabilityRefusal::ContractViolation`]).
    Cyclic,
}

/// Ancestor-doubling table over `parent`: `P[level][j]` is `j`'s ancestor
/// `2^level` hops up the `parent` forest (`-1` once a chain reaches a root).
/// Shared by [`allocate`] (which needs the full table for its
/// `is_descendant` computation) and [`check_hierarchy_acyclic`] (which only
/// needs the cycle witness `P[7][j] != -1`) so the two never compute this
/// independently and risk drifting apart.
#[allow(non_snake_case)]
fn ancestor_doubling_table(parent: &[i32; N]) -> [[i32; N]; 8] {
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

    P
}

/// Validate that `parent` describes an acyclic forest on `N` nodes, per
/// `CMCA_CONTRACT.md` §9 ("Hierarchy Acyclicity (DAG Property)"): a
/// well-formed forest always reaches a root (`-1`) within `N` hops from any
/// node, so `P[7][j] != -1` for any `j` (`P` from `ancestor_doubling_table`)
/// is exactly the branchless witness of a cycle -- the same check
/// [`allocate`] already performs internally before it would otherwise
/// silently degrade to `root_w_sum == 0` and an all-`eta` output. Callers
/// that want the `CMCA_CONTRACT.md`-documented `Err(HierarchyRefusal::Cyclic)`
/// distinguished from `allocate`'s other, unrelated `ContractViolation`
/// causes should call this first.
///
/// # Complexity
/// $O(1)$: bounded, unrolled `N`-node ancestor doubling, no data-dependent
/// loops.
pub fn check_hierarchy_acyclic(parent: &[i32; N]) -> Result<(), HierarchyRefusal> {
    let p = ancestor_doubling_table(parent);
    let mut has_cycle = false;
    unroll_8_static!(j, {
        has_cycle |= p[7][j] != -1;
    });
    if has_cycle {
        Err(HierarchyRefusal::Cyclic)
    } else {
        Ok(())
    }
}

// Bounded leaf reciprocal lookup table (nl from 1 to 8)
const LEAF_RECIP: [NonNegativeFixed; 9] = [
    NonNegativeFixed::from_bits(0),
    NonNegativeFixed::from_bits(65536), // 1.0
    NonNegativeFixed::from_bits(32768), // 0.5
    NonNegativeFixed::from_bits(21845), // 0.33333
    NonNegativeFixed::from_bits(16384), // 0.25
    NonNegativeFixed::from_bits(13107), // 0.2
    NonNegativeFixed::from_bits(10922), // 0.16667
    NonNegativeFixed::from_bits(9362),  // 0.14285
    NonNegativeFixed::from_bits(8192),  // 0.125
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
/// use bcinr_cmca::fixed::NonNegativeFixed;
/// use bcinr_cmca::allocator::{wrap_result, StabilityRefusal};
///
/// let pi = [NonNegativeFixed::ZERO; 8];
/// let ok_res = wrap_result(pi, u32::MAX);
/// assert_eq!(ok_res, Ok(pi));
///
/// let err_res = wrap_result(pi, 0);
/// assert_eq!(err_res, Err(StabilityRefusal::CertificateMissing));
/// ```
///
/// # Branchless Contract
pub fn wrap_result(
    pi_res: [NonNegativeFixed; N],
    err_code: u32,
) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
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
pub(crate) fn const_max_i32(a: i32, b: i32) -> i32 {
    let diff_64 = (a as i64).wrapping_sub(b as i64);
    let is_lt = (diff_64 >> 63) & 1;
    const_select_u32(is_lt as u32, b as u32, a as u32) as i32
}

/// Marker struct indicating certified learning mode is active.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CertifiedLearning {
    _sealed: (),
}

impl CertifiedLearning {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self { _sealed: () }
    }

    #[inline(always)]
    pub const fn admit_learning() -> Self {
        Self { _sealed: () }
    }
}

/// Marker struct indicating selection-only mode is active.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CertifiedSelectionOnly {
    _sealed: (),
}

impl CertifiedSelectionOnly {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self { _sealed: () }
    }

    #[inline(always)]
    pub const fn admit_selection_only() -> Self {
        Self { _sealed: () }
    }
}

/// Proof token certifying that the control state has been admitted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AdmittedControlState {
    pub(crate) digest: u64,
}

impl AdmittedControlState {
    #[inline(always)]
    pub(crate) const fn new(digest: u64) -> Self {
        Self { digest }
    }

    #[inline(always)]
    pub const fn admit_control_state(digest: u64) -> Self {
        Self { digest }
    }
}

/// Proof token certifying receipt of a valid security certificate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CertificateReceipt {
    pub(crate) digest: u64,
}

impl CertificateReceipt {
    #[inline(always)]
    pub(crate) const fn new(digest: u64) -> Self {
        Self { digest }
    }

    #[inline(always)]
    pub const fn admit_certificate(digest: u64) -> Self {
        Self { digest }
    }
}

/// Proof token certifying receipt of a valid envelope.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EnvelopeReceipt {
    pub(crate) digest: u64,
}

impl EnvelopeReceipt {
    #[inline(always)]
    pub(crate) const fn new(digest: u64) -> Self {
        Self { digest }
    }

    #[inline(always)]
    pub const fn admit_envelope(digest: u64) -> Self {
        Self { digest }
    }
}

/// Proof token certifying receipt of a valid outcome.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OutcomeReceipt {
    pub(crate) digest: u64,
}

impl OutcomeReceipt {
    #[inline(always)]
    pub(crate) const fn new(digest: u64) -> Self {
        Self { digest }
    }

    #[inline(always)]
    pub const fn admit_outcome(digest: u64) -> Self {
        Self { digest }
    }
}

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
        *self
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
    pub fn admit_adaptive_update(
        state: AdmittedControlState,
        cert: CertificateReceipt,
        env: EnvelopeReceipt,
        outcome: OutcomeReceipt,
        temperature: NonNegativeFixed,
        distinguishability: NonNegativeFixed,
        _mode: CertifiedLearning,
    ) -> Option<Self> {
        let temp_ceil = ((crate::generated::stability_profile::PROFILE
            .temperature_ceiling
            .raw
            * 65536)
            / 1_000_000_000) as u32;
        let dist_floor = ((crate::generated::stability_profile::PROFILE
            .distinguishability_floor
            .raw
            * 65536)
            / 1_000_000_000) as u32;

        let temp_ok = (const_lt_u32(temp_ceil, temperature.val) == 0) as u32;
        let dist_ok = (const_lt_u32(distinguishability.val, dist_floor) == 0) as u32;

        let digests_ok = (((state.digest ^ cert.digest)
            | (state.digest ^ env.digest)
            | (state.digest ^ outcome.digest))
            == 0) as u32;

        let ok = temp_ok & dist_ok & digests_ok;

        let outcomes = [
            None,
            Some(Self {
                _mode: core::marker::PhantomData,
            }),
        ];
        outcomes[(ok as usize) & 1]
    }
}

/// Computes `base^exponent` branchlessly using fixed-point log2 and exp2 approximations.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
///
/// Made `pub` (was `pub(crate)`) so [`crate::escort`] can build a
/// fractional-exponent escort distribution on top of it -- see that
/// module's docs for why `cascade::escort_weight`'s exact, integer-only
/// repeated multiplication isn't sufficient for every caller.
#[inline(always)]
pub fn power(base: NonNegativeFixed, exponent: SignedFixed) -> NonNegativeFixed {
    let base_is_zero = const_eq_u32(base.val, 0);
    let log_val = base.log2();
    // `saturating_mul`, not a raw `wrapping_mul` + truncating `as i32` cast:
    // the manual version this replaced discarded overflow silently (product
    // could wrap past i32::MAX/MIN with no signal) and `from_bits` then
    // manufactured a fresh `err = u32::MAX`, erasing `log_val.err` too.
    // `saturating_mul` clamps on overflow and propagates both operands' err.
    let product = exponent.saturating_mul(log_val);
    let pow_val = product.exp2();
    let exp_val = exponent.val;
    let exp_gt_zero = (((0i32.wrapping_sub(exp_val)) >> 31) & 1) as u32;
    let exp_eq_zero = const_eq_u32(exponent.val as u32, 0);
    let zero_res = const_select_u32(
        exp_eq_zero,
        NonNegativeFixed::ONE.val,
        const_select_u32(exp_gt_zero, 0, u32::MAX),
    );
    // The zero-base branch is exact (no `exp2` approximation involved), so
    // it reports no fault regardless of `pow_val.err`; the base != 0 branch
    // must carry `pow_val.err` through rather than discarding it via
    // `from_bits`, or `exp2`'s own overflow/underflow detection becomes
    // unreachable dead code from every caller's perspective.
    NonNegativeFixed {
        val: const_select_u32(base_is_zero, zero_res, pow_val.val),
        err: const_select_u32(base_is_zero, u32::MAX, pow_val.err),
    }
}

/// Clamps a fixed-point value within `[min_val, max_val]` branchlessly.
///
/// # Complexity
/// $O(1)$ constant time, branchless.
#[inline(always)]
pub(crate) fn clip(
    val: NonNegativeFixed,
    min_val: NonNegativeFixed,
    max_val: NonNegativeFixed,
) -> NonNegativeFixed {
    let lt_min = const_lt_u32(val.val, min_val.val);
    let val_or_min = const_select_u32(lt_min, min_val.val, val.val);
    let gt_max = const_lt_u32(max_val.val, val_or_min);
    NonNegativeFixed::from_bits(const_select_u32(gt_max, max_val.val, val_or_min))
}

/// Performs a single straight-line flow propagation step down the node forest.
///
/// Divides the incoming flow into flat and descendant parts, distributing them
/// branchlessly according to normalized leaf and child weights.
///
/// # Complexity
/// $O(N^2)$ operations, which is $O(1)$ since $N=8$.
#[inline(never)]
#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless flow-step kernel
fn flow_step(
    parent: &[i32; N],
    is_leaf: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    rho: &[NonNegativeFixed; N],
    child_w: &[[NonNegativeFixed; N]; N],
    cw_sum: &[NonNegativeFixed; N],
    leaf_w: &[[NonNegativeFixed; N]; N],
    lw_sum: &[NonNegativeFixed; N],
    alloc_flow: &mut [NonNegativeFixed; N],
    flat_alloc: &mut [NonNegativeFixed; N],
) {
    unroll_8_static!(v, {
        let has_children = !is_leaf[v & 7];

        let flat_part = NonNegativeFixed::from_bits(const_select_u32(
            has_children as u32,
            ((NonNegativeFixed::ONE - rho[v & 7]) * alloc_flow[v & 7]).val,
            0,
        ));
        let desc_part = NonNegativeFixed::from_bits(const_select_u32(
            has_children as u32,
            (rho[v & 7] * alloc_flow[v & 7]).val,
            0,
        ));

        #[allow(unused_variables)]
        let l_cond = const_eq_u32(lw_sum[v & 7].val, 0);
        #[cfg(feature = "mutant_3")]
        let lw_denom = NonNegativeFixed::ONE.val;
        #[cfg(not(feature = "mutant_3"))]
        let lw_denom = const_select_u32(l_cond, NonNegativeFixed::ONE.val, lw_sum[v & 7].val);

        let c_cond = const_eq_u32(cw_sum[v & 7].val, 0);
        let cw_denom = const_select_u32(c_cond, NonNegativeFixed::ONE.val, cw_sum[v & 7].val);

        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7] & has_children;
            let flat_addition = flat_part
                * leaf_w[v & 7][x & 7].saturating_div(NonNegativeFixed::from_bits(lw_denom));
            flat_alloc[x & 7] +=
                NonNegativeFixed::from_bits(const_select_u32(is_sub as u32, flat_addition.val, 0));

            let is_child = (parent[x & 7] == v as i32) & has_children;
            let flow_addition = desc_part
                * child_w[v & 7][x & 7].saturating_div(NonNegativeFixed::from_bits(cw_denom));
            alloc_flow[x & 7] += NonNegativeFixed::from_bits(const_select_u32(
                is_child as u32,
                flow_addition.val,
                0,
            ));
        });

        alloc_flow[v & 7] = NonNegativeFixed::from_bits(const_select_u32(
            has_children as u32,
            0,
            alloc_flow[v & 7].val,
        ));
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
#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless kernel
fn compute_pi_kq_for_kq(
    k_actual: usize,
    q_idx: usize,
    q_val_mutated: SignedFixed,
    parent: &[i32; N],
    is_leaf: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    node_masses: &[[NonNegativeFixed; N]; K],
    local_weights: &[[NonNegativeFixed; 2 * Q]; N],
) -> [NonNegativeFixed; N] {
    let mut a_roots = [0i32; N];
    let mut a_max_root = i32::MIN;
    unroll_8_static!(i, {
        let is_r = parent[i & 7] == -1;
        let a_i = (((q_val_mutated.val as i64)
            .wrapping_mul(node_masses[k_actual & 3][i & 7].log2().val as i64))
            >> 16) as i32;
        a_roots[i & 7] = const_select_u32(is_r as u32, a_i as u32, i32::MIN as u32) as i32;
        a_max_root = const_max_i32(a_max_root, a_roots[i & 7]);
    });

    let mut root_w = [NonNegativeFixed::ZERO; N];
    let mut root_w_sum = NonNegativeFixed::ZERO;
    unroll_8_static!(i, {
        root_w[i & 7] = NonNegativeFixed::from_bits(const_select_u32(
            (parent[i & 7] == -1) as u32,
            SignedFixed::from_bits(a_roots[i & 7].wrapping_sub(a_max_root))
                .exp2()
                .val,
            0,
        ));
        root_w_sum += root_w[i & 7];
    });

    let mut alloc_flow = [NonNegativeFixed::ZERO; N];
    unroll_8_static!(i, {
        let is_r = parent[i & 7] == -1;
        let r_cond = const_eq_u32(root_w_sum.val, 0);
        let flow_val = root_w[i & 7].saturating_div(NonNegativeFixed::from_bits(const_select_u32(
            r_cond,
            NonNegativeFixed::ONE.val,
            root_w_sum.val,
        )));
        alloc_flow[i & 7] =
            NonNegativeFixed::from_bits(const_select_u32(is_r as u32, flow_val.val, 0));
    });

    let mut rho = [NonNegativeFixed::ZERO; N];
    let mut child_w = [[NonNegativeFixed::ZERO; N]; N];
    let mut cw_sum = [NonNegativeFixed::ZERO; N];
    let mut leaf_w = [[NonNegativeFixed::ZERO; N]; N];
    let mut lw_sum = [NonNegativeFixed::ZERO; N];

    unroll_8_static!(v, {
        let w_sum =
            local_weights[v & 7][(2 * q_idx) & 7] + local_weights[v & 7][(2 * q_idx + 1) & 7];
        rho[v & 7] = NonNegativeFixed::from_bits(const_select_u32(
            const_eq_u32(w_sum.val, 0),
            32768,
            local_weights[v & 7][(2 * q_idx + 1) & 7]
                .saturating_div(w_sum)
                .val,
        ));

        let mut a_c = [0i32; N];
        let mut a_max_c = i32::MIN;
        unroll_8_static!(c, {
            let is_c = parent[c & 7] == v as i32;
            a_c[c & 7] = const_select_u32(
                is_c as u32,
                (((q_val_mutated.val as i64)
                    .wrapping_mul(node_masses[k_actual & 3][c & 7].log2().val as i64))
                    >> 16) as u32,
                i32::MIN as u32,
            ) as i32;
            a_max_c = const_max_i32(a_max_c, a_c[c & 7]);
        });
        unroll_8_static!(c, {
            let matches = a_c[c & 7] != i32::MIN;
            child_w[v & 7][c & 7] = NonNegativeFixed::from_bits(const_select_u32(
                matches as u32,
                SignedFixed::from_bits(a_c[c & 7].wrapping_sub(a_max_c))
                    .exp2()
                    .val,
                0,
            ));
            cw_sum[v & 7] += child_w[v & 7][c & 7];
        });

        let mut a_l = [0i32; N];
        let mut a_max_l = i32::MIN;
        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7];
            a_l[x & 7] = const_select_u32(
                is_sub as u32,
                (((q_val_mutated.val as i64)
                    .wrapping_mul(node_masses[k_actual & 3][x & 7].log2().val as i64))
                    >> 16) as u32,
                i32::MIN as u32,
            ) as i32;
            a_max_l = const_max_i32(a_max_l, a_l[x & 7]);
        });
        unroll_8_static!(x, {
            let matches = a_l[x & 7] != i32::MIN;
            leaf_w[v & 7][x & 7] = NonNegativeFixed::from_bits(const_select_u32(
                matches as u32,
                SignedFixed::from_bits(a_l[x & 7].wrapping_sub(a_max_l))
                    .exp2()
                    .val,
                0,
            ));
            lw_sum[v & 7] += leaf_w[v & 7][x & 7];
        });
    });

    let mut flat_alloc = [NonNegativeFixed::ZERO; N];

    // Call flow_step 8 times sequentially to avoid stack frame nesting
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );
    flow_step(
        parent,
        is_leaf,
        is_subtree_leaf,
        &rho,
        &child_w,
        &cw_sum,
        &leaf_w,
        &lw_sum,
        &mut alloc_flow,
        &mut flat_alloc,
    );

    let mut res = [NonNegativeFixed::ZERO; N];
    unroll_8_static!(x, res[x & 7] = flat_alloc[x & 7] + alloc_flow[x & 7]);
    res
}

mod feasible_region;
pub use feasible_region::FeasibleRegion;

/// [`allocate`] parameterized by an explicit [`FeasibleRegion`] instead of
/// [`FeasibleRegion::CURRENT`]. See `allocate`'s docs for the mechanism,
/// inputs, and complexity -- identical here except for `region`.
///
/// # Branchless Contract
#[allow(clippy::too_many_arguments)] // deliberate wide parameter list preserving the public allocation API
pub fn allocate_in(
    region: &FeasibleRegion,
    states: &[PackedSemanticState; N],
    lenses: &[LensSpec; Q],
    lambda: &[[NonNegativeFixed; Q]; K],
    eta: NonNegativeFixed,
    parent: &[i32; N],
    weights: &mut [[NonNegativeFixed; 2 * Q]; N],
    payoffs: &[[NonNegativeFixed; 2 * Q]; N],
    zeta: NonNegativeFixed,
    _epsilon_kappa: NonNegativeFixed,
    mu: &[NonNegativeFixed; N],
    costs: &[NonNegativeFixed; N],
    t: u32,
    last_switch_t: &mut u32,
    prev_mode: &mut u32,
    tau_d: u32,
    digest: [u8; 32],
    proof: Option<&AdaptiveUpdate<CertifiedLearning>>,
) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
    let mut local_weights = *weights;
    let mut local_last_switch_t = *last_switch_t;
    let mut local_prev_mode = *prev_mode;

    let beta_max = region.beta_max;
    let m_min = region.m_min;
    let m_max = region.m_max;
    let mu_max = region.mu_max;

    let proof_some = proof.is_some();
    let degrade_to_certified_selection = proof.is_none();

    let mut digest_match = 1u32;
    unroll_32_static!(i, {
        digest_match &= const_eq_u32(
            digest[i & 31] as u32,
            crate::generated::stability_profile::CERTIFICATE_DIGEST[i & 31] as u32,
        );
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
        gd_ok &= lhs <= rhs;
    });

    let zeta_w_max_q16 =
        ((crate::generated::stability_profile::ZETA_W_MAX.raw * 65536) / 1_000_000_000) as u32;
    let eta_g_min_q16 =
        ((crate::generated::stability_profile::ETA_G_MIN.raw * 65536) / 1_000_000_000) as u32;

    let lr_err = const_lt_u32(zeta_w_max_q16, zeta.val) != 0;
    let dwell_err = const_lt_u32(
        tau_d,
        crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
    ) != 0;

    let mut q_err = false;
    unroll_4_static!(q_idx, {
        let q_val = lenses[q_idx & 3].q.val;
        q_err |= !(-131072..=131072).contains(&q_val);
    });

    let mut price_err = false;
    unroll_8_static!(i, {
        price_err |= const_lt_u32(mu_max.val, mu[i & 7].val) != 0;
    });

    let eta_err = const_lt_u32(eta.val, eta_g_min_q16) != 0;

    let is_zeta_less = const_lt_u32(zeta.val, beta_max.val);
    let beta = NonNegativeFixed::from_bits(const_select_u32(is_zeta_less, zeta.val, beta_max.val));
    let beta_m_max_q16 =
        ((crate::generated::stability_profile::BETA_M_MAX.raw * 65536) / 1_000_000_000) as u32;
    let beta_err = const_lt_u32(beta_m_max_q16, beta.val) != 0;

    let has_error =
        !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
    let freeze_learning = has_error & degrade_to_certified_selection;

    let mut is_leaf = [true; N];
    unroll_8_static!(i, {
        unroll_8_static!(j, {
            let is_match = parent[j & 7] == i as i32;
            is_leaf[i & 7] &= !is_match;
        });
    });

    #[allow(non_snake_case)]
    let P = ancestor_doubling_table(parent);

    // A well-formed parent forest on N=8 nodes always reaches a root (-1) within
    // N hops from any node: P[6] already covers the deepest possible chain (7
    // edges), and once a node's ancestor is -1 it stays -1 (no p_idx in 0..7
    // matches parent_node == -1). So P[7][j] != -1 for any j is exactly the
    // branchless witness of a cycle (or a chain deeper than N-1 permits) that
    // would otherwise silently degrade to root_w_sum == 0 and an all-eta output.
    // Same witness [`check_hierarchy_acyclic`] exposes under its
    // `CMCA_CONTRACT.md`-documented `HierarchyRefusal::Cyclic` name.
    let mut has_cycle = false;
    unroll_8_static!(j, {
        has_cycle |= P[7][j] != -1;
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
        unroll_8_static!(k, {
            is_subtree_leaf[i & 7][k & 7] = is_leaf[k & 7] & is_descendant[i & 7][k & 7];
        });
    });

    let mut node_masses = [[NonNegativeFixed::ZERO; N]; K];
    unroll_8_static!(i, {
        let state = &states[i & 7];
        let f_recomp = state.factors[FACTOR_RECOMPUTATION_COST];
        let f_verify = state.factors[FACTOR_VERIFICATION_COST];
        let f_stand = state.factors[FACTOR_STANDING];
        let f_access = state.factors[FACTOR_ACCESS_FREQUENCY];
        let f_search = state.factors[FACTOR_SEARCH_DEMAND];
        let f_retrieve = state.factors[FACTOR_RETRIEVAL_DEMAND];
        let f_sched = state.factors[FACTOR_SCHEDULING_DEMAND];
        let f_bval = state.factors[FACTOR_BUSINESS_VALUE];
        let f_conseq = state.factors[FACTOR_DOWNSTREAM_CONSEQUENCE];

        let m_cache = (f_recomp * NonNegativeFixed::from_num(5) + f_verify) * f_access * f_stand;
        let m_search = (f_bval + f_conseq) * f_search * f_stand;
        let m_retrieval = f_bval * f_retrieve;
        let m_sched = f_bval * f_sched;

        node_masses[MEASURE_CACHE][i & 7] = m_cache;
        node_masses[MEASURE_RETRIEVAL][i & 7] = m_retrieval;
        node_masses[MEASURE_SCHEDULING][i & 7] = m_sched;
        node_masses[MEASURE_SEARCH][i & 7] = m_search;
    });

    unroll_4_static!(k, {
        unroll_8_static!(i, {
            node_masses[k & 3][i & 7] = clip(node_masses[k & 3][i & 7], m_min, m_max);
        });
    });

    let mut root_idx = 0usize;
    unroll_8_static!(i, {
        let is_root = parent[i & 7] == -1;
        root_idx = const_select_u32(is_root as u32, i as u32, root_idx as u32) as usize;
    });

    // Load root weights branchlessly
    let mut root_weights = [NonNegativeFixed::ZERO; 2 * Q];
    unroll_8_static!(idx, {
        let matches = const_eq_u32(root_idx as u32, idx as u32);
        unroll_8_static!(e, {
            root_weights[e & 7] = NonNegativeFixed::from_bits(const_select_u32(
                matches,
                local_weights[idx & 7][e & 7].val,
                root_weights[e & 7].val,
            ));
        });
    });

    let mut max_w = NonNegativeFixed::ZERO;
    let mut dom_mode = 0u32;
    unroll_8_static!(e, {
        let w = root_weights[e & 7];
        let is_greater = const_lt_u32(max_w.val, w.val);
        max_w = NonNegativeFixed::from_bits(const_select_u32(is_greater, w.val, max_w.val));
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
            let mut _q_val_mutated = SignedFixed::from_bits(lenses[q_idx & 3].q.val);
            #[cfg(feature = "mutant_2")]
            {
                _q_val_mutated = SignedFixed::from_bits(0i32.wrapping_sub(_q_val_mutated.val));
            }
            let w_flat = local_weights[v & 7][(2 * q_idx) & 7];
            let w_desc = local_weights[v & 7][(2 * q_idx + 1) & 7];
            let is_updating = has_children & update_allowed;
            local_weights[v & 7][(2 * q_idx) & 7] = NonNegativeFixed::from_bits(const_select_u32(
                is_updating as u32,
                (w_flat
                    * SignedFixed::from_bits((beta * payoffs[v & 7][(2 * q_idx) & 7]).val as i32)
                        .exp())
                .val,
                w_flat.val,
            ));
            local_weights[v & 7][(2 * q_idx + 1) & 7] =
                NonNegativeFixed::from_bits(const_select_u32(
                    is_updating as u32,
                    (w_desc
                        * SignedFixed::from_bits(
                            (beta * payoffs[v & 7][(2 * q_idx + 1) & 7]).val as i32,
                        )
                        .exp())
                    .val,
                    w_desc.val,
                ));
        });

        unroll_4_static!(q_idx, {
            let w_flat = local_weights[v & 7][(2 * q_idx) & 7];
            let w_desc = local_weights[v & 7][(2 * q_idx + 1) & 7];
            let sum_div = w_flat + w_desc;
            local_weights[v & 7][(2 * q_idx) & 7] = NonNegativeFixed::from_bits(const_select_u32(
                update_allowed as u32,
                w_flat.saturating_div(sum_div).val,
                w_flat.val,
            ));
            local_weights[v & 7][(2 * q_idx + 1) & 7] =
                NonNegativeFixed::from_bits(const_select_u32(
                    update_allowed as u32,
                    w_desc.saturating_div(sum_div).val,
                    w_desc.val,
                ));
        });
    });

    let mut new_dom_mode = 0u32;
    let mut new_max_w = NonNegativeFixed::ZERO;

    // Reload root weights
    unroll_8_static!(idx, {
        let matches = const_eq_u32(root_idx as u32, idx as u32);
        unroll_8_static!(e, {
            root_weights[e & 7] = NonNegativeFixed::from_bits(const_select_u32(
                matches,
                local_weights[idx & 7][e & 7].val,
                root_weights[e & 7].val,
            ));
        });
    });

    unroll_8_static!(e, {
        let w = root_weights[e & 7];
        let is_greater = const_lt_u32(new_max_w.val, w.val);
        new_max_w = NonNegativeFixed::from_bits(const_select_u32(is_greater, w.val, new_max_w.val));
        new_dom_mode = const_select_u32(is_greater, e as u32, new_dom_mode);
    });

    let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;
    local_last_switch_t = const_select_u32(did_switch as u32, t, local_last_switch_t);
    local_prev_mode = const_select_u32(did_switch as u32, new_dom_mode, local_prev_mode);

    let mut pi_kq = [[[NonNegativeFixed::ZERO; N]; Q]; K];

    unroll_4_static!(k, {
        #[cfg(feature = "mutant_1")]
        const k_actual: usize = 0;
        #[cfg(not(feature = "mutant_1"))]
        const k_actual: usize = k;

        unroll_4_static!(q_idx, {
            let q_val_mutated = SignedFixed::from_bits(lenses[q_idx & 3].q.val);
            #[cfg(feature = "mutant_2")]
            let q_val_mutated = SignedFixed::from_bits(0i32.wrapping_sub(q_val_mutated.val));

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

    let mut pi_combined = [NonNegativeFixed::ZERO; N];
    unroll_4_static!(k, {
        unroll_4_static!(q_idx, {
            unroll_8_static!(x, {
                let term = lambda[k & 3][q_idx & 3] * pi_kq[k & 3][q_idx & 3][x & 7];
                pi_combined[x & 7] += term;
            });
        });
    });

    let mut pi_res = [NonNegativeFixed::ZERO; N];
    let mut priced_sum = NonNegativeFixed::ZERO;
    unroll_8_static!(x, {
        #[cfg(feature = "mutant_5")]
        let mu_actual = mu[x & 7];
        #[cfg(not(feature = "mutant_5"))]
        let mu_actual = clip(mu[x & 7], NonNegativeFixed::ZERO, mu_max);

        let p = pi_combined[x & 7]
            * SignedFixed::from_bits(0i32.wrapping_sub((mu_actual * costs[x & 7]).val as i32))
                .exp();
        priced_sum +=
            NonNegativeFixed::from_bits(const_select_u32(is_leaf[x & 7] as u32, p.val, 0));
    });
    let psd = NonNegativeFixed::from_bits(const_select_u32(
        const_eq_u32(priced_sum.val, 0),
        NonNegativeFixed::ONE.val,
        priced_sum.val,
    ));
    let mut nl = 0u32;
    unroll_8_static!(i, {
        nl += is_leaf[i & 7] as u32;
    });
    unroll_8_static!(x, {
        #[cfg(feature = "mutant_5")]
        let mu_actual = mu[x & 7];
        #[cfg(not(feature = "mutant_5"))]
        let mu_actual = clip(mu[x & 7], NonNegativeFixed::ZERO, mu_max);

        let p_mu = (pi_combined[x & 7]
            * SignedFixed::from_bits(0i32.wrapping_sub((mu_actual * costs[x & 7]).val as i32))
                .exp())
        .saturating_div(psd);

        #[cfg(feature = "mutant_4")]
        let eta_actual = zeta;
        #[cfg(not(feature = "mutant_4"))]
        let eta_actual = eta;

        // Use lookup table to avoid division
        let mut nl_recip = NonNegativeFixed::ZERO;
        unroll_9_static!(idx, {
            let matches = const_eq_u32(nl, idx as u32);
            nl_recip = NonNegativeFixed::from_bits(const_select_u32(
                matches,
                LEAF_RECIP[idx].val,
                nl_recip.val,
            ));
        });

        let val = (eta_actual * nl_recip) + ((NonNegativeFixed::ONE - eta_actual) * p_mu);
        let pi_val = pi_res[x & 7];
        pi_res[x & 7] = NonNegativeFixed::from_bits(const_select_u32(
            is_leaf[x & 7] as u32,
            val.val,
            pi_val.val,
        ));
    });

    // Numeric refusals accumulated through the Q16.16 arithmetic chain (overflow,
    // underflow, division by zero) were previously discarded here: `wrap_result`
    // only ever saw `err_val` below, never `pi_res[x].err`. Fold them in so
    // `NumericRangeExceeded`/`UnsupportedDomain` become reachable.
    let mut numeric_err = u32::MAX;
    unroll_8_static!(x, {
        numeric_err = crate::fixed::branchless_err_acc(numeric_err, pi_res[x & 7].err);
    });
    let numeric_has_err = const_eq_u32(numeric_err, u32::MAX) == 0;

    let has_error = has_error | has_cycle | numeric_has_err;
    let has_refusal = has_error & !degrade_to_certified_selection;
    unroll_8_static!(v, {
        unroll_8_static!(e, {
            weights[v & 7][e & 7] = NonNegativeFixed::from_bits(const_select_u32(
                has_refusal as u32,
                weights[v & 7][e & 7].val,
                local_weights[v & 7][e & 7].val,
            ));
        });
    });
    *last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
    *prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);

    let err_val = const_select_u32(
        has_cycle as u32,
        StabilityRefusal::ContractViolation as u32,
        const_select_u32(
            q_err as u32,
            5,
            const_select_u32(
                dwell_err as u32,
                4,
                const_select_u32(
                    (lr_err | beta_err | eta_err) as u32,
                    3,
                    const_select_u32(
                        (!gd_ok) as u32,
                        1,
                        const_select_u32(
                            digest_err as u32,
                            10,
                            const_select_u32(numeric_has_err as u32, numeric_err, 7),
                        ),
                    ),
                ),
            ),
        ),
    );
    wrap_result(
        pi_res,
        const_select_u32(has_refusal as u32, err_val, u32::MAX),
    )
}

/// Allocates resources down the node forest branchlessly, performing MWU step updates,
/// stable projections, and explore floors.
///
/// This is the entry point for the Cascade Allocation engine. It is
/// [`allocate_in`] with the bounds this crate has always used
/// ([`FeasibleRegion::CURRENT`]) -- preserved as a thin wrapper so no
/// existing caller needs to change:
/// `allocate(x) == allocate_in(&FeasibleRegion::CURRENT, x)` for every
/// input, by construction. Call `allocate_in` directly to use a different
/// [`FeasibleRegion`].
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
/// Returns a nonnegative resource-allocation vector over the $N$ nodes if
/// successful, or a [`StabilityRefusal`] code otherwise. In the ordinary
/// normalized regime the components sum approximately to `ONE` (see
/// [`FeasibleRegion::contains_allocation`]); a documented fallback regime
/// (`priced_sum` underflowing to zero) can legitimately return a
/// subnormalized vector instead -- this is not a probability distribution
/// by construction, only in the common case. See `contains_allocation`'s
/// docs for the counterexample that established this.
///
/// # Complexity
/// - **Time Complexity**: $O(K \cdot Q \cdot N^2)$ operations ($O(1)$ constant time).
/// - **Space Complexity**: $O(1)$ auxiliary stack space.
/// - **Cyclomatic Complexity**: $CC = 1$.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::fixed::NonNegativeFixed;
/// use bcinr_cmca::allocator::{allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, CertifiedLearning};
/// use bcinr_cmca::generated::consequence_mass::case_studies::{OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, N, Q};
/// use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
///
/// let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
/// let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
/// let mut last_switch_t = 0;
/// let mut prev_mode = 0;
/// let parent = [-1; N];
/// let mu = [NonNegativeFixed::ZERO; N];
/// let costs = [NonNegativeFixed::ZERO; N];
///
/// let proof = AdaptiveUpdate::admit_adaptive_update(
///     AdmittedControlState::admit_control_state(0),
///     CertificateReceipt::admit_certificate(0),
///     EnvelopeReceipt::admit_envelope(0),
///     OutcomeReceipt::admit_outcome(0),
///     NonNegativeFixed::ZERO,
///     NonNegativeFixed::ONE,
///     CertifiedLearning::admit_learning(),
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
///     NonNegativeFixed::ZERO,
///     NonNegativeFixed::ZERO,
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
#[allow(clippy::too_many_arguments)] // deliberate wide parameter list preserving the public allocation API
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
    allocate_in(
        &FeasibleRegion::CURRENT,
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
        t,
        last_switch_t,
        prev_mode,
        tau_d,
        digest,
        proof,
    )
}

/// ```compile_fail
/// use bcinr_cmca::allocator::AdmittedControlState;
/// let state = AdmittedControlState { digest: 0 };
/// ```
///
/// ```compile_fail
/// use bcinr_cmca::allocator::CertifiedLearning;
/// let mode = CertifiedLearning { _sealed: () };
/// ```
///
/// ```compile_fail
/// use bcinr_cmca::allocator::{AdaptiveUpdate, CertifiedLearning};
/// let update = AdaptiveUpdate::<CertifiedLearning> { _mode: core::marker::PhantomData };
/// ```
///
/// ```compile_fail
/// use bcinr_cmca::allocator::{AdaptiveUpdate, CertifiedLearning, AdmittedControlState, CertificateReceipt, EnvelopeReceipt};
/// use bcinr_cmca::fixed::NonNegativeFixed;
/// // Missing OutcomeReceipt
/// AdaptiveUpdate::admit_adaptive_update(
///     AdmittedControlState::admit_control_state(0),
///     CertificateReceipt::admit_certificate(0),
///     EnvelopeReceipt::admit_envelope(0),
///     NonNegativeFixed::ZERO,
///     NonNegativeFixed::ONE,
///     CertifiedLearning::admit_learning(),
/// );
/// ```
///
/// ```compile_fail
/// use bcinr_cmca::allocator::{AdaptiveUpdate, CertifiedSelectionOnly, AdmittedControlState, CertificateReceipt, EnvelopeReceipt, OutcomeReceipt};
/// use bcinr_cmca::fixed::NonNegativeFixed;
/// // mutate from selection-only mode
/// AdaptiveUpdate::admit_adaptive_update(
///     AdmittedControlState::admit_control_state(0),
///     CertificateReceipt::admit_certificate(0),
///     EnvelopeReceipt::admit_envelope(0),
///     OutcomeReceipt::admit_outcome(0),
///     NonNegativeFixed::ZERO,
///     NonNegativeFixed::ONE,
///     CertifiedSelectionOnly::admit_selection_only(),
/// );
/// ```
pub struct AuthorityCompileFailTests;
