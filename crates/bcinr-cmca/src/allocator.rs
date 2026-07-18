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

use crate::fixed::{CanonicalMask, NonNegativeFixed, NumericFaultSet, SignedFixed};

/// Branchless select over two [`NonNegativeFixed`] alternatives that distributes over the
/// `(value, faults)` pair as a whole (numeric-hot-path.md Invariant 2): the selected
/// alternative's own fault set survives, the unselected alternative's fault set is
/// dropped, and neither is silently re-derived through a fresh fault-free constructor.
#[inline(always)]
fn select_nnf(condition: u32, a: NonNegativeFixed, b: NonNegativeFixed) -> NonNegativeFixed {
    let mask = CanonicalMask::from_lsb(condition);
    NonNegativeFixed::from_parts(
        mask.select_u32(a.value_bits(), b.value_bits()),
        mask.select_faults(a.faults(), b.faults()),
    )
}

/// [`select_nnf`], for [`SignedFixed`].
#[inline(always)]
fn select_sf(condition: u32, a: SignedFixed, b: SignedFixed) -> SignedFixed {
    let mask = CanonicalMask::from_lsb(condition);
    SignedFixed::from_parts(
        mask.select_i32(a.value_bits(), b.value_bits()),
        mask.select_faults(a.faults(), b.faults()),
    )
}
use crate::generated::case_studies::{
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

// NOTE: the historical rounded-reciprocal lookup table (`LEAF_RECIP`) has been removed.
// Per numeric-hot-path.md Invariant 4 (exact-budget conservation), the explore-floor
// term is now computed per-leaf via an exact base-q + residual-r scheme (see the
// `q_floor`/`r_floor`/`leaf_rank` computation in `allocate`) rather than a single
// rounded value shared by every leaf, which did not sum exactly to 65536 for values
// of `nl` that do not divide 65536 evenly. The mfw generated-artifact manifest at
// `generated-artifact/case-studies/cmca_generation_manifest.json` exposes only a
// `leaf_floor_n_max` dimension (no `leaf_floor_base`/`leaf_floor_remainder` tables),
// so the formula is computed directly in Rust below rather than sourced from the
// artifact.

/// Refusal reasons that can co-occur while producing an [`AllocationOutcome`] — a flag
/// set, never a lossy single-variant enum, per `authority-and-c3.md` Invariant 2.
///
/// The inner representation is private; the only publicly constructible values are
/// `EMPTY`, unions of named bits, and masked selections thereof.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct RefusalSet(u32);

impl RefusalSet {
    pub const EMPTY: Self = Self(0);

    /// Disposition: REACHABLE. Unioned unconditionally on `nl_is_zero` in [`allocate`] — a
    /// leafless candidate forest is a structural property of the input (nowhere to
    /// allocate flow to), not a control-plane/certificate check, so it is never suppressed
    /// by `has_refusal`'s gating. Same-object test:
    /// `tests/jtbd_refusal_invariance_regression.rs::no_leaves_only_refusal_leaves_full_state_invariant`
    /// (passes). Full disposition table: `REFUSAL_REALIZATION_REPORT.md`.
    pub const NO_LEAVES: Self = Self(1 << 0);

    /// Disposition: RESERVED_WITH_EXPLICIT_NONCLAIM. No code path anywhere in this crate
    /// constructs this bit — not even a masked-to-zero one. "A certificate was never
    /// presented at all" (as distinct from `DIGEST_MISMATCH`'s "a certificate/digest was
    /// presented but does not match") has no representable trigger given the current API
    /// surface: [`allocate`]'s own `digest: [u8; 32]` parameter is mandatory, never
    /// `Option<[u8; 32]>`, and `mode_switch::apply_mode_switch` likewise takes
    /// `certificate: CertificateReceipt` by value, never `Option`. Both are deliberate
    /// consequences of the branchless/fixed-shape-input mandate this module opens with
    /// ("no input-dependent...branches") and of `numeric-hot-path.md` Invariant 6
    /// (the authoritative root must stay total over a fixed-shape domain) — introducing an
    /// `Option` to distinguish "missing" from "mismatched" would require either a branch
    /// inside the hot path or a caller-side type change, both outside this bit's own
    /// declaration site. This bit is kept (not removed as vestigial) because "no
    /// certificate was ever obtained" is a real, meaningful domain condition under
    /// `authority-and-c3.md` Invariant 1's four-authority chain (a caller can legitimately
    /// never reach a sealed `CertificateReceipt` at all, e.g. when `seal_certificate`
    /// returned `Err` upstream), and it is already read meaningfully by
    /// [`RefusalSet::primary_reason`] — reserved for a future API shape that can actually
    /// distinguish the two cases at this boundary, not vestigial. Full disposition table:
    /// `REFUSAL_REALIZATION_REPORT.md`.
    pub const CERTIFICATE_MISSING: Self = Self(1 << 1);

    /// Disposition: OWNED_BY_DIFFERENT_COMPONENT. No code path in `allocate()` constructs
    /// this bit. "A previously-valid certificate is no longer current" is realized
    /// downstream, on two other modules' own typed return types — never via `RefusalSet`,
    /// per `authority-and-c3.md` Invariant 1's four-separate-authorities structure:
    /// `mode_switch::ModeSwitchRefusal::CertificateDigestMismatch` (the certificate
    /// presented to `apply_mode_switch` no longer equals the currently expected one —
    /// exactly what a superseded, once-valid certificate produces) and
    /// `certification::CertificationRefusal::RoundIdentityMismatch` (one of the eleven
    /// sealed bindings `seal_certificate` verifies; a certificate bound to an earlier round
    /// no longer matches the current one). Same-object tests:
    /// `mode_switch::tests::rejection_cause_certificate_mismatch_leaves_state_untouched`,
    /// `certification::tests::refuses_solo_mismatch_round_identity` (both pass). Full
    /// disposition table: `REFUSAL_REALIZATION_REPORT.md`.
    pub const CERTIFICATE_STALE: Self = Self(1 << 2);

    /// Disposition: OWNED_BY_DIFFERENT_COMPONENT. No code path in `allocate()` constructs
    /// this bit. Round-identity mismatch is realized upstream, on two modules' own typed
    /// return types: `proposal::ProposalRefusal::RoundIdentityMismatch` (`admit_proposal`
    /// refuses when the caller-supplied round does not match the round the proposal was
    /// made for) and `certification::CertificationRefusal::RoundIdentityMismatch` (one of
    /// the eleven sealed bindings `seal_certificate` verifies). Same-object tests:
    /// `proposal::tests::refuses_on_round_mismatch`,
    /// `certification::tests::refuses_solo_mismatch_round_identity` (both pass). Note the
    /// overlap with `CERTIFICATE_STALE`'s ownership above: certification.rs's single
    /// `RoundIdentityMismatch` variant is the closest realized analog for both
    /// higher-level `RefusalSet` bits — the finer-grained module distinguishes the
    /// binding, the coarser `RefusalSet` vocabulary does not. Full disposition table:
    /// `REFUSAL_REALIZATION_REPORT.md`.
    pub const ROUND_MISMATCH: Self = Self(1 << 3);

    /// Disposition: REACHABLE. Unioned on `digest_err` (the `digest` parameter mismatching
    /// the compiled `CERTIFICATE_DIGEST`), gated by `has_refusal`. Same-object test:
    /// `tests/jtbd_refusal_invariance_regression.rs::digest_mismatch_only_refusal_leaves_full_state_invariant`
    /// (passes). Full disposition table: `REFUSAL_REALIZATION_REPORT.md`.
    pub const DIGEST_MISMATCH: Self = Self(1 << 4);

    /// Disposition: UNREACHABLE_BY_PROOF. The union site
    /// (`.union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as
    /// u32))`) exists and runs on every call, but the surrounding `gated_refusals` bundle
    /// is masked again by `has_refusal = (has_error | (nl_is_zero != 0)) &
    /// !degrade_to_certified_selection`, which requires `degrade_to_certified_selection ==
    /// false` — the exact negation of this bit's own mask condition
    /// (`degrade_to_certified_selection == true`). For any boolean value `b`, `b &
    /// !b == false`, so this bit is masked to zero on every call, unconditionally: a
    /// proof from the two conjuncts' own definitions, not an empirically-observed absence.
    /// Confirmed by a targeted run in
    /// `tests/jtbd_refusal_invariance_regression.rs::authority_missing_is_never_actually_set_verified_by_targeted_run`
    /// (passes) against the exact scenario (`proof = None` plus a real control-plane
    /// error) a working `AUTHORITY_MISSING` would be expected to fire under. Full
    /// disposition table: `REFUSAL_REALIZATION_REPORT.md`.
    pub const AUTHORITY_MISSING: Self = Self(1 << 5);

    /// Disposition: REACHABLE. Unioned on `(!gd_ok) | lr_err | beta_err | eta_err | q_err |
    /// price_err`, gated by `has_refusal`. Same-object test:
    /// `tests/jtbd_refusal_invariance_regression.rs::proposal_rejected_only_refusal_leaves_full_state_invariant`
    /// (passes). Full disposition table: `REFUSAL_REALIZATION_REPORT.md`.
    pub const PROPOSAL_REJECTED: Self = Self(1 << 6);

    /// Disposition: REACHABLE. Unioned on `dwell_err` (`tau_d < MODE_DWELL_ROUNDS_MIN`),
    /// gated by `has_refusal`. Same-object test:
    /// `tests/jtbd_refusal_invariance_regression.rs::dwell_unsatisfied_only_refusal_leaves_full_state_invariant`
    /// (passes). Full disposition table: `REFUSAL_REALIZATION_REPORT.md`.
    pub const DWELL_UNSATISFIED: Self = Self(1 << 7);

    /// Branchless Contract: bitwise union — the only accumulation operator for
    /// refusal sets; `{P: true} union(a, b) {Q: result.bits() == a.bits() |
    /// b.bits()}`, total, no branch (numeric-hot-path.md Invariant 1).
    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Branchless Contract: zeroes `self` unless `condition` is `1` (mirrors
    /// [`CanonicalMask::select_faults`]'s masked-selection contract for the analogous
    /// `NumericFaultSet` type, but as a single-operand mask since the "off" branch of a
    /// refusal accumulation is always `EMPTY`). `{P: condition in {0,1}} masked(self,
    /// condition) {Q: result == self if condition == 1 else EMPTY}`, no branch.
    #[inline(always)]
    pub const fn masked(self, condition: u32) -> Self {
        Self(self.0 & 0u32.wrapping_sub(condition & 1))
    }

    /// Collapses the set down to a single legacy [`StabilityRefusal`], for callers that
    /// have not migrated off the historical single-enum shape. This projection is
    /// layered on top of the full set (via [`AllocationOutcome::into_result`]) — it is
    /// never the sole representation of a refusal (`authority-and-c3.md` Invariant 2).
    pub fn primary_reason(self) -> StabilityRefusal {
        if self.contains(Self::DIGEST_MISMATCH) || self.contains(Self::CERTIFICATE_STALE) {
            StabilityRefusal::CertificateDigestMismatch
        } else if self.contains(Self::CERTIFICATE_MISSING) || self.contains(Self::AUTHORITY_MISSING)
        {
            StabilityRefusal::CertificateMissing
        } else if self.contains(Self::DWELL_UNSATISFIED) {
            StabilityRefusal::ModeDwellTimeViolated
        } else if self.contains(Self::ROUND_MISMATCH) {
            StabilityRefusal::ContractViolation
        } else if self.contains(Self::PROPOSAL_REJECTED) {
            StabilityRefusal::ContractionMarginInsufficient
        } else {
            StabilityRefusal::ContractViolation
        }
    }
}

/// Total outcome of a cascade allocation attempt.
///
/// Unlike the legacy `Result`-returning shape, `AllocationOutcome` is always
/// constructible for any admitted input: the authoritative root ([`allocate`]) never
/// early-returns, panics, or uses `Result`-as-control-flow. Any anomaly is represented
/// in [`numeric_faults`](Self::numeric_faults) or [`refusals`](Self::refusals) rather
/// than in the absence of a return (numeric-hot-path.md Invariant 6).
///
/// Fields are private and only producible through [`AllocationOutcome::new_internal`],
/// which is the sole constructor and enforces the aggregation invariant: `numeric_faults`
/// always equals the union of every candidate element's own `.faults()` with whatever
/// additional numeric faults were observed along the local allocator's computation path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllocationOutcome {
    candidate: [NonNegativeFixed; N],
    numeric_faults: NumericFaultSet,
    refusals: RefusalSet,
}

impl AllocationOutcome {
    /// Sole constructor. Aggregates ALL candidate-contained numeric faults (the union of
    /// each candidate element's own [`NonNegativeFixed::faults`]) with `local_faults`
    /// (numeric faults observed along the allocator's own computation path that never
    /// made it into a candidate element, e.g. a zero-normalization substitution).
    #[inline(always)]
    pub(crate) fn new_internal(
        candidate: [NonNegativeFixed; N],
        local_faults: NumericFaultSet,
        refusals: RefusalSet,
    ) -> Self {
        let mut folded = local_faults;
        unroll_8_static!(x, {
            folded = folded.union(candidate[x & 7].faults());
        });
        Self {
            candidate,
            numeric_faults: folded,
            refusals,
        }
    }

    #[inline(always)]
    pub const fn candidate(&self) -> [NonNegativeFixed; N] {
        self.candidate
    }

    #[inline(always)]
    pub const fn numeric_faults(&self) -> NumericFaultSet {
        self.numeric_faults
    }

    #[inline(always)]
    pub const fn refusals(&self) -> RefusalSet {
        self.refusals
    }

    #[inline(always)]
    pub const fn is_refused(&self) -> bool {
        !self.refusals.is_empty()
    }

    /// Ergonomic `Result`-based adapter for callers OUTSIDE the audited authoritative
    /// root. This is not part of the totality guarantee itself — `allocate` remains
    /// total regardless of whether a caller chooses this adapter or inspects
    /// `refusals()`/`numeric_faults()` directly.
    pub fn into_result(self) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        if self.refusals.is_empty() {
            Ok(self.candidate)
        } else {
            Err(self.refusals.primary_reason())
        }
    }
}

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

        let temp_ok = (const_lt_u32(temp_ceil, temperature.value_bits()) == 0) as u32;
        let dist_ok = (const_lt_u32(distinguishability.value_bits(), dist_floor) == 0) as u32;

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
#[inline(always)]
pub(crate) fn power(base: NonNegativeFixed, exponent: SignedFixed) -> NonNegativeFixed {
    let base_is_zero = const_eq_u32(base.value_bits(), 0);
    let log_val = base.log2();
    let exp_signed = exponent.value_bits();
    let log_signed = log_val.value_bits();
    let product = (((exp_signed as i64).wrapping_mul(log_signed as i64)) >> 16) as i32;
    // `product` is derived from both `exponent` and `log_val` (itself derived from
    // `base`); carry both operands' faults forward through the raw-bits reinterpretation
    // rather than silently starting fault-free (numeric-hot-path.md Invariant 1).
    let signed_product =
        SignedFixed::from_parts(product, exponent.faults().union(log_val.faults()));
    let pow_val = signed_product.exp2();
    let exp_val = exponent.value_bits();
    let exp_gt_zero = (((0i32.wrapping_sub(exp_val)) >> 31) & 1) as u32;
    let exp_eq_zero = const_eq_u32(exponent.value_bits() as u32, 0);
    let zero_res_bits = const_select_u32(
        exp_eq_zero,
        NonNegativeFixed::ONE.value_bits(),
        const_select_u32(exp_gt_zero, 0, u32::MAX),
    );
    // The `exp == 0` / `exp > 0` branch depends only on `exponent`; the `pow_val`
    // branch already carries `base`+`exponent`+`log_val` faults via `signed_product`.
    let zero_branch = NonNegativeFixed::from_parts(zero_res_bits, exponent.faults());
    select_nnf(base_is_zero, zero_branch, pow_val)
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
    let lt_min = const_lt_u32(val.value_bits(), min_val.value_bits());
    let val_or_min = select_nnf(lt_min, min_val, val);
    let gt_max = const_lt_u32(max_val.value_bits(), val_or_min.value_bits());
    select_nnf(gt_max, max_val, val_or_min)
}

/// Performs a single straight-line flow propagation step down the node forest.
///
/// Divides the incoming flow into flat and descendant parts, distributing them
/// branchlessly according to normalized leaf and child weights.
///
/// # Complexity
/// $O(N^2)$ operations, which is $O(1)$ since $N=8$.
// Fixed-shape (N=8, Q, K bounded) internal step; each parameter is one distinct
// bounded array/scalar the branchless flow computation reads, not a design smell —
// splitting them into a struct would add an allocation-free-but-still-indirection
// layer with no invariant benefit. Documented per AGENTS.md's "no undocumented allow"
// rule.
#[allow(clippy::too_many_arguments)]
#[inline(never)]
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

        let flat_part = select_nnf(
            has_children as u32,
            (NonNegativeFixed::ONE - rho[v & 7]) * alloc_flow[v & 7],
            NonNegativeFixed::ZERO,
        );
        let desc_part = select_nnf(
            has_children as u32,
            rho[v & 7] * alloc_flow[v & 7],
            NonNegativeFixed::ZERO,
        );

        #[allow(unused_variables)]
        let l_cond = const_eq_u32(lw_sum[v & 7].value_bits(), 0);
        #[cfg(feature = "mutant_3")]
        let lw_denom = NonNegativeFixed::ONE;
        #[cfg(not(feature = "mutant_3"))]
        let lw_denom = select_nnf(l_cond, NonNegativeFixed::ONE, lw_sum[v & 7]);

        let c_cond = const_eq_u32(cw_sum[v & 7].value_bits(), 0);
        let cw_denom = select_nnf(c_cond, NonNegativeFixed::ONE, cw_sum[v & 7]);

        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7] & has_children;
            let flat_addition = flat_part * leaf_w[v & 7][x & 7].saturating_div(lw_denom);
            flat_alloc[x & 7] += select_nnf(is_sub as u32, flat_addition, NonNegativeFixed::ZERO);

            let is_child = (parent[x & 7] == v as i32) & has_children;
            let flow_addition = desc_part * child_w[v & 7][x & 7].saturating_div(cw_denom);
            alloc_flow[x & 7] += select_nnf(is_child as u32, flow_addition, NonNegativeFixed::ZERO);
        });

        alloc_flow[v & 7] = select_nnf(
            has_children as u32,
            NonNegativeFixed::ZERO,
            alloc_flow[v & 7],
        );
    });
}

/// Computes the allocation vector $\pi_{k, q}$ for a specific model `k` and lens spec `q`.
///
/// Traverses down the hierarchy by initializing roots and propagating flow via repeated
/// straight-line iterations of [`flow_step`].
///
/// # Complexity
/// $O(N^2)$ operations, which is $O(1)$ since $N=8$.
// Same rationale as `flow_step` above: fixed-shape bounded parameters, no hidden
// control flow. Documented allow per AGENTS.md's "no undocumented allow" rule.
#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn compute_pi_kq_for_kq(
    k_actual: usize,
    q_idx: usize,
    q_val_mutated: SignedFixed,
    parent: &[i32; N],
    is_leaf: &[bool; N],
    is_subtree_leaf: &[[bool; N]; N],
    node_masses: &[[NonNegativeFixed; N]; K],
    local_weights: &[[NonNegativeFixed; 2 * Q]; N],
) -> ([NonNegativeFixed; N], NumericFaultSet) {
    // Faults observed strictly along the raw-i32 exponent/log-domain arithmetic below:
    // `log2()` (and the `q_val_mutated` operand feeding every product) can carry
    // `DIVIDE_BY_ZERO`/`INVALID_DOMAIN`/`RANGE_VIOLATION` bits that would otherwise be
    // silently dropped by extracting `.value_bits()` into plain `i32`/`i64` arithmetic.
    // Threading them through explicitly (rather than losing them) satisfies
    // numeric-hot-path.md Invariant 1 for this path.
    let mut path_faults = NumericFaultSet::EMPTY;

    let mut a_roots = [0i32; N];
    let mut a_max_root = i32::MIN;
    unroll_8_static!(i, {
        let is_r = parent[i & 7] == -1;
        let mass_log = node_masses[k_actual & 3][i & 7].log2();
        path_faults = path_faults.union(mass_log.faults());
        let a_i = (((q_val_mutated.value_bits() as i64).wrapping_mul(mass_log.value_bits() as i64))
            >> 16) as i32;
        a_roots[i & 7] = const_select_u32(is_r as u32, a_i as u32, i32::MIN as u32) as i32;
        a_max_root = const_max_i32(a_max_root, a_roots[i & 7]);
    });
    path_faults = path_faults.union(q_val_mutated.faults());

    let mut root_w = [NonNegativeFixed::ZERO; N];
    let mut root_w_sum = NonNegativeFixed::ZERO;
    unroll_8_static!(i, {
        let root_exp = SignedFixed::from_value_bits(a_roots[i & 7].wrapping_sub(a_max_root)).exp2();
        root_w[i & 7] = select_nnf(
            (parent[i & 7] == -1) as u32,
            root_exp,
            NonNegativeFixed::ZERO,
        );
        root_w_sum += root_w[i & 7];
    });

    let mut alloc_flow = [NonNegativeFixed::ZERO; N];
    unroll_8_static!(i, {
        let is_r = parent[i & 7] == -1;
        let r_cond = const_eq_u32(root_w_sum.value_bits(), 0);
        let root_w_sum_safe = select_nnf(r_cond, NonNegativeFixed::ONE, root_w_sum);
        let flow_val = root_w[i & 7].saturating_div(root_w_sum_safe);
        alloc_flow[i & 7] = select_nnf(is_r as u32, flow_val, NonNegativeFixed::ZERO);
    });

    let mut rho = [NonNegativeFixed::ZERO; N];
    let mut child_w = [[NonNegativeFixed::ZERO; N]; N];
    let mut cw_sum = [NonNegativeFixed::ZERO; N];
    let mut leaf_w = [[NonNegativeFixed::ZERO; N]; N];
    let mut lw_sum = [NonNegativeFixed::ZERO; N];

    unroll_8_static!(v, {
        let w_sum =
            local_weights[v & 7][(2 * q_idx) & 7] + local_weights[v & 7][(2 * q_idx + 1) & 7];
        let rho_default = NonNegativeFixed::from_parts(32768, w_sum.faults());
        let rho_ratio = local_weights[v & 7][(2 * q_idx + 1) & 7].saturating_div(w_sum);
        rho[v & 7] = select_nnf(const_eq_u32(w_sum.value_bits(), 0), rho_default, rho_ratio);

        let mut a_c = [0i32; N];
        let mut a_max_c = i32::MIN;
        unroll_8_static!(c, {
            let is_c = parent[c & 7] == v as i32;
            let mass_log_c = node_masses[k_actual & 3][c & 7].log2();
            path_faults = path_faults.union(mass_log_c.faults());
            a_c[c & 7] = const_select_u32(
                is_c as u32,
                (((q_val_mutated.value_bits() as i64).wrapping_mul(mass_log_c.value_bits() as i64))
                    >> 16) as u32,
                i32::MIN as u32,
            ) as i32;
            a_max_c = const_max_i32(a_max_c, a_c[c & 7]);
        });
        unroll_8_static!(c, {
            let matches = a_c[c & 7] != i32::MIN;
            let child_exp = SignedFixed::from_value_bits(a_c[c & 7].wrapping_sub(a_max_c)).exp2();
            child_w[v & 7][c & 7] = select_nnf(matches as u32, child_exp, NonNegativeFixed::ZERO);
            cw_sum[v & 7] += child_w[v & 7][c & 7];
        });

        let mut a_l = [0i32; N];
        let mut a_max_l = i32::MIN;
        unroll_8_static!(x, {
            let is_sub = is_subtree_leaf[v & 7][x & 7];
            let mass_log_x = node_masses[k_actual & 3][x & 7].log2();
            path_faults = path_faults.union(mass_log_x.faults());
            a_l[x & 7] = const_select_u32(
                is_sub as u32,
                (((q_val_mutated.value_bits() as i64).wrapping_mul(mass_log_x.value_bits() as i64))
                    >> 16) as u32,
                i32::MIN as u32,
            ) as i32;
            a_max_l = const_max_i32(a_max_l, a_l[x & 7]);
        });
        unroll_8_static!(x, {
            let matches = a_l[x & 7] != i32::MIN;
            let leaf_exp = SignedFixed::from_value_bits(a_l[x & 7].wrapping_sub(a_max_l)).exp2();
            leaf_w[v & 7][x & 7] = select_nnf(matches as u32, leaf_exp, NonNegativeFixed::ZERO);
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
    (res, path_faults)
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
/// use bcinr_cmca::fixed::NonNegativeFixed;
/// use bcinr_cmca::allocator::{allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, CertifiedLearning};
/// use bcinr_cmca::generated::case_studies::{OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, N, Q};
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
/// assert!(!result.is_refused());
/// ```
///
/// # Branchless Contract
// This is the authoritative root: every parameter is a distinct fixed-shape input the
// admitted-input -> deterministic-output contract requires as its own named binding
// (per AGENTS.md Invariant 3, a seal/root must bind every enumerated identity
// explicitly, not through a bundling struct that would obscure which are checked).
// Documented allow per AGENTS.md's "no undocumented allow" rule.
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
    _epsilon_kappa: NonNegativeFixed,
    mu: &[NonNegativeFixed; N],
    costs: &[NonNegativeFixed; N],
    t: u32,
    last_switch_t: &mut u32,
    prev_mode: &mut u32,
    tau_d: u32,
    digest: [u8; 32],
    proof: Option<&AdaptiveUpdate<CertifiedLearning>>,
) -> AllocationOutcome {
    let mut local_weights = *weights;
    let mut local_last_switch_t = *last_switch_t;
    let mut local_prev_mode = *prev_mode;

    let beta_max = NonNegativeFixed::from_value_bits(6553);
    let m_min = NonNegativeFixed::from_value_bits(6);
    let m_max = NonNegativeFixed::from_value_bits(65536000);
    let mu_max = NonNegativeFixed::from_value_bits(6553600);

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

    let lr_err = const_lt_u32(zeta_w_max_q16, zeta.value_bits()) != 0;
    let dwell_err = const_lt_u32(
        tau_d,
        crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
    ) != 0;

    let mut q_err = false;
    unroll_4_static!(q_idx, {
        let q_val = lenses[q_idx & 3].q.value_bits();
        q_err |= !(-131072..=131072).contains(&q_val);
    });

    let mut price_err = false;
    unroll_8_static!(i, {
        price_err |= const_lt_u32(mu_max.value_bits(), mu[i & 7].value_bits()) != 0;
    });

    let eta_err = const_lt_u32(eta.value_bits(), eta_g_min_q16) != 0;

    let is_zeta_less = const_lt_u32(zeta.value_bits(), beta_max.value_bits());
    let beta = select_nnf(is_zeta_less, zeta, beta_max);
    let beta_m_max_q16 =
        ((crate::generated::stability_profile::BETA_M_MAX.raw * 65536) / 1_000_000_000) as u32;
    let beta_err = const_lt_u32(beta_m_max_q16, beta.value_bits()) != 0;

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
            root_weights[e & 7] =
                select_nnf(matches, local_weights[idx & 7][e & 7], root_weights[e & 7]);
        });
    });

    let mut max_w = NonNegativeFixed::ZERO;
    let mut dom_mode = 0u32;
    unroll_8_static!(e, {
        let w = root_weights[e & 7];
        let is_greater = const_lt_u32(max_w.value_bits(), w.value_bits());
        max_w = select_nnf(is_greater, w, max_w);
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
            let mut _q_val_mutated = SignedFixed::from_parts(
                lenses[q_idx & 3].q.value_bits(),
                lenses[q_idx & 3].q.faults(),
            );
            #[cfg(feature = "mutant_2")]
            {
                _q_val_mutated = SignedFixed::from_parts(
                    0i32.wrapping_sub(_q_val_mutated.value_bits()),
                    _q_val_mutated.faults(),
                );
            }
            let w_flat = local_weights[v & 7][(2 * q_idx) & 7];
            let w_desc = local_weights[v & 7][(2 * q_idx + 1) & 7];
            let is_updating = has_children & update_allowed;
            let flat_payoff = beta * payoffs[v & 7][(2 * q_idx) & 7];
            let flat_signed =
                SignedFixed::from_parts(flat_payoff.value_bits() as i32, flat_payoff.faults());
            let desc_payoff = beta * payoffs[v & 7][(2 * q_idx + 1) & 7];
            let desc_signed =
                SignedFixed::from_parts(desc_payoff.value_bits() as i32, desc_payoff.faults());
            local_weights[v & 7][(2 * q_idx) & 7] =
                select_nnf(is_updating as u32, w_flat * flat_signed.exp(), w_flat);
            local_weights[v & 7][(2 * q_idx + 1) & 7] =
                select_nnf(is_updating as u32, w_desc * desc_signed.exp(), w_desc);
        });

        unroll_4_static!(q_idx, {
            let w_flat = local_weights[v & 7][(2 * q_idx) & 7];
            let w_desc = local_weights[v & 7][(2 * q_idx + 1) & 7];
            let sum_div = w_flat + w_desc;
            local_weights[v & 7][(2 * q_idx) & 7] = select_nnf(
                update_allowed as u32,
                w_flat.saturating_div(sum_div),
                w_flat,
            );
            local_weights[v & 7][(2 * q_idx + 1) & 7] = select_nnf(
                update_allowed as u32,
                w_desc.saturating_div(sum_div),
                w_desc,
            );
        });
    });

    let mut new_dom_mode = 0u32;
    let mut new_max_w = NonNegativeFixed::ZERO;

    // Reload root weights
    unroll_8_static!(idx, {
        let matches = const_eq_u32(root_idx as u32, idx as u32);
        unroll_8_static!(e, {
            root_weights[e & 7] =
                select_nnf(matches, local_weights[idx & 7][e & 7], root_weights[e & 7]);
        });
    });

    unroll_8_static!(e, {
        let w = root_weights[e & 7];
        let is_greater = const_lt_u32(new_max_w.value_bits(), w.value_bits());
        new_max_w = select_nnf(is_greater, w, new_max_w);
        new_dom_mode = const_select_u32(is_greater, e as u32, new_dom_mode);
    });

    let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;
    local_last_switch_t = const_select_u32(did_switch as u32, t, local_last_switch_t);
    local_prev_mode = const_select_u32(did_switch as u32, new_dom_mode, local_prev_mode);

    let mut pi_kq = [[[NonNegativeFixed::ZERO; N]; Q]; K];
    // Numeric faults observed along the exponent/log-domain path of every (k, q) cell,
    // accumulated as a join-semilattice union (numeric-hot-path.md Invariant 1) rather
    // than kept per-cell only — the outcome's `numeric_faults` reports the union across
    // the whole candidate, not just what survives inside individual `pi_res` elements.
    let mut kq_path_faults = NumericFaultSet::EMPTY;

    unroll_4_static!(k, {
        #[cfg(feature = "mutant_1")]
        const k_actual: usize = 0;
        #[cfg(not(feature = "mutant_1"))]
        const k_actual: usize = k;

        unroll_4_static!(q_idx, {
            let q_val_mutated = SignedFixed::from_parts(
                lenses[q_idx & 3].q.value_bits(),
                lenses[q_idx & 3].q.faults(),
            );
            #[cfg(feature = "mutant_2")]
            let q_val_mutated = SignedFixed::from_parts(
                0i32.wrapping_sub(q_val_mutated.value_bits()),
                q_val_mutated.faults(),
            );

            let (res_kq, cell_faults) = compute_pi_kq_for_kq(
                k_actual,
                q_idx,
                q_val_mutated,
                parent,
                &is_leaf,
                &is_subtree_leaf,
                &node_masses,
                &local_weights,
            );
            kq_path_faults = kq_path_faults.union(cell_faults);
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

        let mu_cost = mu_actual * costs[x & 7];
        let neg_mu_cost = SignedFixed::from_parts(
            0i32.wrapping_sub(mu_cost.value_bits() as i32),
            mu_cost.faults(),
        );
        let p = pi_combined[x & 7] * neg_mu_cost.exp();
        priced_sum += select_nnf(is_leaf[x & 7] as u32, p, NonNegativeFixed::ZERO);
    });

    // Invariant: priced_sum == 0 accumulates INVALID_NORMALIZATION and the computation
    // continues with the existing safe-denominator substitution (ONE in place of ZERO)
    // rather than early-returning — the authoritative root stays total (Invariant 6).
    let priced_sum_is_zero = const_eq_u32(priced_sum.value_bits(), 0);
    let mut local_numeric_faults = NumericFaultSet::EMPTY.union(kq_path_faults);
    local_numeric_faults =
        local_numeric_faults.union(CanonicalMask::from_lsb(priced_sum_is_zero).select_faults(
            NumericFaultSet::INVALID_NORMALIZATION,
            NumericFaultSet::EMPTY,
        ));
    let psd = select_nnf(priced_sum_is_zero, NonNegativeFixed::ONE, priced_sum);

    let mut nl = 0u32;
    unroll_8_static!(i, {
        nl += is_leaf[i & 7] as u32;
    });
    // nl == 0 is reported as a refusal (`RefusalSet::NO_LEAVES`, see `final_refusals`
    // below), not a numeric fault: it is a structural property of the candidate forest
    // (no leaves to allocate to), not an arithmetic anomaly encountered while computing
    // a value.
    let nl_is_zero = const_eq_u32(nl, 0);

    // Exact Q16.16 explore-floor share per leaf (numeric-hot-path.md Invariant 4): rather
    // than every leaf sharing one rounded reciprocal (whose sum over `nl` leaves does not
    // in general equal 65536 exactly), split the unit budget as `q = 65536 / nl` plus a
    // remainder `r = 65536 - q * nl` distributed one extra unit each to the first `r`
    // leaves under canonical (index) rank. `nl_safe` avoids a divide-by-zero when
    // `nl == 0`; the divided-out value is unused in that case since no `x` has
    // `is_leaf[x]` true, and the defensive zeroing below makes that explicit rather than
    // incidental. The mfw generated-artifact manifest exposes no
    // `leaf_floor_base`/`leaf_floor_remainder` tables (only a `leaf_floor_n_max`
    // dimension), so the formula is computed directly here.
    let nl_safe = const_select_u32(nl_is_zero, 1, nl);
    let q_floor = 65536u32 / nl_safe;
    let r_floor = 65536u32 - q_floor * nl_safe;
    let mut leaf_rank = [0u32; N];
    let mut running_rank = 0u32;
    unroll_8_static!(x, {
        leaf_rank[x & 7] = running_rank;
        running_rank += is_leaf[x & 7] as u32;
    });

    unroll_8_static!(x, {
        #[cfg(feature = "mutant_5")]
        let mu_actual = mu[x & 7];
        #[cfg(not(feature = "mutant_5"))]
        let mu_actual = clip(mu[x & 7], NonNegativeFixed::ZERO, mu_max);

        let mu_cost = mu_actual * costs[x & 7];
        let neg_mu_cost = SignedFixed::from_parts(
            0i32.wrapping_sub(mu_cost.value_bits() as i32),
            mu_cost.faults(),
        );
        let p_mu = (pi_combined[x & 7] * neg_mu_cost.exp()).saturating_div(psd);

        #[cfg(feature = "mutant_4")]
        let eta_actual = zeta;
        #[cfg(not(feature = "mutant_4"))]
        let eta_actual = eta;

        let gets_extra = const_lt_u32(leaf_rank[x & 7], r_floor);
        let nl_recip = NonNegativeFixed::from_value_bits(q_floor + gets_extra);

        let val = (eta_actual * nl_recip) + ((NonNegativeFixed::ONE - eta_actual) * p_mu);
        let pi_val = pi_res[x & 7];
        pi_res[x & 7] = select_nnf(is_leaf[x & 7] as u32, val, pi_val);
    });

    // Root cause of the N-way conservation defect (numeric-hot-path.md Invariant 4):
    // `nl_recip` above is exactly conserved via the `q_floor`/`r_floor` base+residual
    // scheme, but the price-normalized term `p_mu` (a per-leaf `saturating_div`, which
    // truncates towards zero) is not — it is a second, independent partition of the same
    // unit budget across leaves, and nothing redistributes its own truncation remainder.
    // The subsequent `eta_actual * nl_recip + (1 - eta_actual) * p_mu` mix then truncates
    // *again* per leaf (`saturating_mul` also floors). `floor_conservation_tests` below
    // only checks the `q_floor`/`r_floor` formula in isolation and never observes this,
    // because it never exercises the live `eta < ONE` mixed path. The result: whenever
    // `eta_actual < NonNegativeFixed::ONE` (any non-degenerate mixing weight, e.g. the
    // real case-studies registry's `ETA = 0.5`), the returned `pi_res` shares under-count
    // the exact unit budget by the accumulated per-leaf truncation loss.
    //
    // Fix: apply the same explicit base-q + residual-r remainder-distribution technique
    // Invariant 4 mandates, once more, to the *actual returned* per-leaf values — the
    // only place a caller-visible conservation guarantee is meaningful — rather than only
    // to the intermediate floor sub-term. This makes `sum(pi_res[x] for is_leaf[x])`
    // exactly `NonNegativeFixed::ONE.value_bits()` regardless of `eta`, `mu`, `costs`, or
    // the price distribution, without a magic constant: the correction amount is derived
    // from the actual observed shortfall (or, symmetrically, any surplus) against the
    // fixed target, using the same canonical-rank ordering (`leaf_rank`) already computed
    // above for the floor term.
    let mut leaf_sum = 0u32;
    unroll_8_static!(x, {
        leaf_sum = leaf_sum.wrapping_add(
            select_nnf(is_leaf[x & 7] as u32, pi_res[x & 7], NonNegativeFixed::ZERO).value_bits(),
        );
    });
    let target_bits = NonNegativeFixed::ONE.value_bits();
    let is_deficit = const_lt_u32(leaf_sum, target_bits);
    let is_excess = const_lt_u32(target_bits, leaf_sum);
    let gap = const_select_u32(
        is_deficit,
        target_bits.wrapping_sub(leaf_sum),
        leaf_sum.wrapping_sub(target_bits),
    );
    let gap_safe = const_select_u32(nl_is_zero, 0, gap);
    let gap_q = gap_safe / nl_safe;
    let gap_r = gap_safe - gap_q * nl_safe;
    unroll_8_static!(x, {
        let gets_extra_unit = const_lt_u32(leaf_rank[x & 7], gap_r);
        let bump = gap_q + gets_extra_unit;
        // `from_parts` (not `from_value_bits`) is required here: the latter would
        // silently erase whatever fault set `pi_res[x]` already accumulated (e.g.
        // `SATURATION`/`RANGE_VIOLATION` from the mixing step above), violating
        // numeric-hot-path.md Invariant 2 ("silent erasure" — the selected value's own
        // fault set must survive a masked select/rewrite, not be re-derived fault-free).
        let bumped_up = NonNegativeFixed::from_parts(
            pi_res[x & 7].value_bits().wrapping_add(bump),
            pi_res[x & 7].faults(),
        );
        let bumped_down = NonNegativeFixed::from_parts(
            pi_res[x & 7].value_bits().wrapping_sub(bump),
            pi_res[x & 7].faults(),
        );
        let corrected = select_nnf(
            is_deficit,
            bumped_up,
            select_nnf(is_excess, bumped_down, pi_res[x & 7]),
        );
        pi_res[x & 7] = select_nnf(is_leaf[x & 7] as u32, corrected, pi_res[x & 7]);
    });

    // Defensive, explicit zeroing of the commit mask when there are no leaves at all
    // (item 4): the per-leaf assignment above already only ever touches `pi_res[x]` when
    // `is_leaf[x]` holds, so this is a no-op whenever `nl == 0`, but making it explicit
    // keeps the invariant testable rather than incidental.
    unroll_8_static!(x, {
        pi_res[x & 7] = select_nnf(nl_is_zero, NonNegativeFixed::ZERO, pi_res[x & 7]);
    });

    // `has_refusal` is the state-commit gate: numeric-hot-path.md Invariant 5 requires that
    // EVERY cause folded into `final_refusals` below (not only the certificate/proposal/
    // dwell/price/eta/beta/lr control-plane checks `has_error` was originally built from)
    // leave `weights`/`*last_switch_t`/`*prev_mode` byte-for-byte at their pre-call values.
    // `nl_is_zero` (the source of `RefusalSet::NO_LEAVES`, unioned into `final_refusals`
    // independently below) was omitted here until this fix: a NO_LEAVES-only refusal
    // (structurally empty leaf set, but no control-plane error) left `has_refusal` false,
    // so the write-back below committed `local_weights` even though the call was refused —
    // a real Invariant 5 violation, found and documented in
    // `tests/jtbd_boundary_adversarial_inputs.rs`. `nl_is_zero` is folded into the same
    // `!degrade_to_certified_selection` gate as `has_error` because the write-back this
    // variable guards is itself only ever a no-op vs. a real commit along that same axis
    // (`update_allowed`/`did_switch` both require `proof_some`, i.e. `!degrade_to_certified_
    // selection`) — when `degrade_to_certified_selection` is true, `local_weights` already
    // equals `*weights` byte-for-byte regardless of `nl_is_zero`, so gating on the
    // conjunction changes behavior only in the one case that was actually broken (a real
    // proof was supplied, and the leaf set was structurally empty).
    let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;
    // State surface this gate covers, in full, per numeric-hot-path.md Invariant 5: the
    // `weights` matrix, `*last_switch_t`, and `*prev_mode` — the entire mutable state
    // `allocate()` can persist back to its caller.
    unroll_8_static!(v, {
        unroll_8_static!(e, {
            weights[v & 7][e & 7] = select_nnf(
                has_refusal as u32,
                weights[v & 7][e & 7],
                local_weights[v & 7][e & 7],
            );
        });
    });
    *last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
    *prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);

    let gated_refusals = RefusalSet::EMPTY
        .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32))
        .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
        .union(
            RefusalSet::PROPOSAL_REJECTED
                .masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32),
        )
        .union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
        .masked(has_refusal as u32);
    // `NO_LEAVES` is reported independently of the certified/degraded-mode gate above: an
    // empty leaf set is a structural property of the candidate, not a stability-envelope
    // violation, so it is never suppressed by `has_refusal`. (It IS, as of the fix above,
    // now folded into `has_refusal` itself for the purpose of gating the state write-back —
    // the independence here is about `final_refusals` reporting, not about state commit.)
    let final_refusals = gated_refusals.union(RefusalSet::NO_LEAVES.masked(nl_is_zero));

    AllocationOutcome::new_internal(pi_res, local_numeric_faults, final_refusals)
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

#[cfg(test)]
mod floor_conservation_tests {
    // The crate is `no_std` unless the `std` feature is enabled (see `lib.rs`), and
    // that feature-gated `extern crate std;` does not cover plain `cargo test` runs
    // built without `--features std`. `cfg(test)` builds always link std via the
    // test harness, so re-declaring it here (scoped to this test module only) is
    // sound and does not affect the no_std authoritative build.
    extern crate std;
    use std::vec::Vec;

    /// Independent oracle for the exact base-q + residual-r floor scheme used in
    /// `allocate` (see the `q_floor`/`r_floor`/`leaf_rank` computation above): computes
    /// each leaf's share directly from the definition, not by re-deriving the
    /// implementation's own control flow, and checks the sum against the whole unit
    /// (numeric-hot-path.md Invariant 4 / Required Evidence Class).
    fn oracle_shares(nl: u32) -> Vec<u32> {
        let q = 65536u32 / nl;
        let r = 65536u32 - q * nl;
        (0..nl)
            .map(|rank| q + if rank < r { 1 } else { 0 })
            .collect()
    }

    #[test]
    fn floor_shares_sum_exactly_to_65536_for_every_admitted_leaf_count() {
        // The allocator's forest is bounded at N = 8 nodes, so 1..=8 is the full
        // admitted domain for `nl` (0 is handled separately as the NO_LEAVES refusal
        // path and produces no leaf shares at all).
        for nl in 1..=8u32 {
            let shares = oracle_shares(nl);
            let sum: u32 = shares.iter().sum();
            assert_eq!(
                sum, 65536,
                "nl={nl}: floor shares {shares:?} summed to {sum}, not 65536 (Invariant 4 violation)"
            );
            // Every share must be exactly q or q+1 (no share more than 1 unit off any
            // other), which is the "first r leaves get q+1, rest get q" shape from item 5.
            let q = 65536u32 / nl;
            for &s in &shares {
                assert!(
                    s == q || s == q + 1,
                    "nl={nl}: share {s} outside {{{q}, {}}}",
                    q + 1
                );
            }
        }
    }

    #[test]
    fn floor_shares_match_allocator_rank_assignment_construction() {
        // Mirrors the exact per-leaf construction used inside `allocate`
        // (`leaf_rank[x]` = count of leaves before `x`; `gets_extra` = rank < r_floor)
        // against the independent oracle above, for every admitted leaf count and every
        // rank within it.
        for nl in 1..=8u32 {
            let q_floor = 65536u32 / nl;
            let r_floor = 65536u32 - q_floor * nl;
            for rank in 0..nl {
                let gets_extra = (rank < r_floor) as u32;
                let allocator_share = q_floor + gets_extra;
                let oracle_share = oracle_shares(nl)[rank as usize];
                assert_eq!(allocator_share, oracle_share, "nl={nl}, rank={rank}");
            }
        }
    }
}
