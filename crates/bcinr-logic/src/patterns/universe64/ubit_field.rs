//! # UBitField — The U_{1,16777216} operating field.
//!
//! 64 aligned `U_{1,262144}` planes (each 32 KiB = `UniverseBlock`) arranged as
//! the 2 MiB L2 operating field. Planes hold: law, capability, expected state,
//! reward, policy, value, drift, projections, scenarios, checkpoints, custom.
//!
//! Requires `feature = "alloc"` — allocates exactly `UBIT_FIELD_BYTES` at init.
//! No heap allocation in T0/T1 hot paths after init.
//!
//! ```
//! # #[cfg(feature = "alloc")] {
//! use bcinr_logic::patterns::universe64::ubit_field::{UBitField, ROLE_LAW};
//! let field = UBitField::new();
//! assert_eq!(field.plane(ROLE_LAW).state[0], 0);
//! # }
//! ```

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
use super::block::UniverseBlock;
use super::constants::SIZE_BYTES;
#[cfg(feature = "alloc")]
use super::constants::UNIVERSE_WORDS;
#[cfg(feature = "alloc")]
use super::law::word_violation;
#[cfg(feature = "alloc")]
use super::drift::word_drift_bits;

/// Number of planes in a UBitField.
pub const UBIT_FIELD_PLANE_COUNT: usize = 64;

/// Total bytes occupied by a UBitField (64 × 32 KiB = 2 MiB).
pub const UBIT_FIELD_BYTES: usize = UBIT_FIELD_PLANE_COUNT * SIZE_BYTES;

// ---------------------------------------------------------------------------
// Role constants — named plane assignments
// ---------------------------------------------------------------------------

/// Law plane: invariant allowed-bit masks.
pub const ROLE_LAW: u8 = 0;
/// Capability plane: authorized action bits.
pub const ROLE_CAPABILITY: u8 = 1;
/// Expected state plane: conformance reference.
pub const ROLE_EXPECTED: u8 = 2;
/// Good reward plane: positive-signal bits.
pub const ROLE_REWARD_GOOD: u8 = 3;
/// Bad reward plane: negative-signal bits.
pub const ROLE_REWARD_BAD: u8 = 4;
/// Policy plane: action selection distribution.
pub const ROLE_POLICY: u8 = 5;
/// Value plane: state value estimates.
pub const ROLE_VALUE: u8 = 6;
/// Drift plane: accumulated deviation tracking.
pub const ROLE_DRIFT: u8 = 7;
/// First projection plane (planes 8..=15).
pub const ROLE_PROJECTION_0: u8 = 8;
/// First scenario plane (planes 16..=31).
pub const ROLE_SCENARIO_0: u8 = 16;
/// First checkpoint plane (planes 48..=55).
pub const ROLE_CHECKPOINT_0: u8 = 48;
/// First custom plane (planes 56..=63).
pub const ROLE_CUSTOM_0: u8 = 56;

// ---------------------------------------------------------------------------
// UBitPlaneRole — typed index into UBitField planes
// ---------------------------------------------------------------------------

/// Typed index into UBitField planes.
///
/// ```
/// use bcinr_logic::patterns::universe64::ubit_field::{UBitPlaneRole, ROLE_LAW};
/// let r = UBitPlaneRole(ROLE_LAW);
/// assert_eq!(r.index(), 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UBitPlaneRole(pub u8);

impl UBitPlaneRole {
    /// Masked index (always 0..63).
    #[inline(always)]
    pub fn index(self) -> usize {
        (self.0 & 63) as usize
    }
}

// ---------------------------------------------------------------------------
// UBitField — heap-allocated 2 MiB operating field
// ---------------------------------------------------------------------------

/// 64-plane 2 MiB operating field: U_{1,16777216}.
///
/// Each plane is an independent `UniverseBlock` (32 KiB, 4096 words).
/// All T0/T1 paths are alloc-free after `UBitField::new()`.
#[cfg(feature = "alloc")]
pub struct UBitField {
    /// 64 × UniverseBlock planes allocated on the heap.
    pub planes: Box<[UniverseBlock; UBIT_FIELD_PLANE_COUNT]>,
}

#[cfg(feature = "alloc")]
impl UBitField {
    /// Allocate a zeroed UBitField. All planes initialized to 0.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use bcinr_logic::patterns::universe64::ubit_field::UBitField;
    /// let f = UBitField::new();
    /// assert_eq!(f.planes[0].state[0], 0);
    /// # }
    /// ```
    pub fn new() -> Self {
        // SAFETY: [UniverseBlock; 64] is valid when zero-initialized because
        // UniverseBlock is repr(transparent) over [u64; UNIVERSE_WORDS], and
        // zero is a valid bit pattern for u64.
        let planes = unsafe {
            let layout = core::alloc::Layout::new::<[UniverseBlock; UBIT_FIELD_PLANE_COUNT]>();
            let ptr = alloc::alloc::alloc_zeroed(layout)
                as *mut [UniverseBlock; UBIT_FIELD_PLANE_COUNT];
            Box::from_raw(ptr)
        };
        Self { planes }
    }

    /// Get a plane by role constant (masked to 0..63).
    #[inline(always)]
    pub fn plane(&self, role: u8) -> &UniverseBlock {
        &self.planes[(role & 63) as usize]
    }

    /// Get a mutable plane by role constant (masked to 0..63).
    #[inline(always)]
    pub fn plane_mut(&mut self, role: u8) -> &mut UniverseBlock {
        &mut self.planes[(role & 63) as usize]
    }

    /// Get role for a plane index (masked).
    #[inline(always)]
    pub fn role(&self, index: u8) -> UBitPlaneRole {
        UBitPlaneRole(index & 63)
    }

    // ---------------------------------------------------------------------------
    // T2 kernels — reuse word-level primitives
    // ---------------------------------------------------------------------------

    /// Count law violations across all UNIVERSE_WORDS words.
    ///
    /// `live_role` is the current state plane, `law_role` is the law plane.
    /// Returns the total number of bits set in violation words.
    pub fn law_violation(&self, live_role: u8, law_role: u8) -> u32 {
        let live = self.plane(live_role);
        let law = self.plane(law_role);
        let mut total: u32 = 0;
        for i in 0..UNIVERSE_WORDS {
            total = total
                .saturating_add(word_violation(live.state[i], law.state[i]).count_ones());
        }
        total
    }

    /// Hamming distance between live and expected planes.
    ///
    /// Returns the total number of bits that differ (XOR popcount).
    pub fn conformance_vs(&self, live_role: u8, expected_role: u8) -> u32 {
        let live = self.plane(live_role);
        let expected = self.plane(expected_role);
        let mut dist: u32 = 0;
        for i in 0..UNIVERSE_WORDS {
            dist = dist.saturating_add(word_drift_bits(live.state[i], expected.state[i]));
        }
        dist
    }

    /// Reward score: sum over all words of (popcount(good & delta) - popcount(bad & delta)).
    ///
    /// `delta_word` is applied uniformly to every word index.
    pub fn reward_score(&self, delta_word: u64, good_role: u8, bad_role: u8) -> i32 {
        let good = self.plane(good_role);
        let bad = self.plane(bad_role);
        let mut score: i32 = 0;
        for i in 0..UNIVERSE_WORDS {
            let g = (good.state[i] & delta_word).count_ones() as i32;
            let b = (bad.state[i] & delta_word).count_ones() as i32;
            score = score.saturating_add(g - b);
        }
        score
    }

    /// Capability gap: bits set in live that are NOT covered by capability plane.
    ///
    /// Returns the total popcount of uncovered bits across all words.
    pub fn capability_gap(&self, live_role: u8, capability_role: u8) -> u32 {
        let live = self.plane(live_role);
        let cap = self.plane(capability_role);
        let mut gap: u32 = 0;
        for i in 0..UNIVERSE_WORDS {
            // bits set in live but not in capability
            let uncovered = live.state[i] & !cap.state[i];
            gap = gap.saturating_add(uncovered.count_ones());
        }
        gap
    }

    /// Drift vs baseline: total bits that differ between live and baseline planes.
    ///
    /// Alias of `conformance_vs` for semantic clarity.
    pub fn drift_vs_baseline(&self, live_role: u8, baseline_role: u8) -> u32 {
        self.conformance_vs(live_role, baseline_role)
    }
}

#[cfg(feature = "alloc")]
impl Default for UBitField {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    use super::*;

    #[test]
    fn ubit_field_allocates_exact_2_mib() {
        let f = UBitField::new();
        assert_eq!(f.planes.len(), UBIT_FIELD_PLANE_COUNT);
        // Each plane is SIZE_BYTES, total is UBIT_FIELD_BYTES
        assert_eq!(core::mem::size_of_val(&*f.planes), UBIT_FIELD_BYTES);
    }

    #[test]
    fn ubit_field_plane_roles_mask_to_sixty_four() {
        let f = UBitField::new();
        // role 64 wraps to 0
        let r = f.role(64);
        assert_eq!(r.index(), 0);
        // role 65 wraps to 1
        let r2 = f.role(65);
        assert_eq!(r2.index(), 1);
    }

    #[test]
    fn ubit_field_law_violation_detects_forbidden_bits() {
        let mut f = UBitField::new();
        // Law plane: word 0 law = 0xF (only bottom 4 bits allowed)
        f.plane_mut(ROLE_LAW).state[0] = 0xF;
        // Live plane: word 0 has bit 7 set (violates law)
        f.plane_mut(ROLE_EXPECTED).state[0] = 0x80;
        let violations = f.law_violation(ROLE_EXPECTED, ROLE_LAW);
        assert!(violations > 0, "should detect law violation");
    }

    #[test]
    fn ubit_field_conformance_counts_drift() {
        let mut f = UBitField::new();
        f.plane_mut(ROLE_EXPECTED).state[0] = 0xFF;
        // Live (ROLE_LAW plane used as "live" for this test) has 0x00 — all 8 bits differ
        let dist = f.conformance_vs(ROLE_LAW, ROLE_EXPECTED);
        assert_eq!(dist, 8);
    }

    #[test]
    fn ubit_field_reward_scores_good_minus_bad_delta() {
        let mut f = UBitField::new();
        // Good plane: bits 0,1 set in word 0
        f.plane_mut(ROLE_REWARD_GOOD).state[0] = 0b11;
        // Bad plane: bit 2 set in word 0
        f.plane_mut(ROLE_REWARD_BAD).state[0] = 0b100;
        // delta covers bits 0,1,2
        let delta_word = 0b111u64;
        let score = f.reward_score(delta_word, ROLE_REWARD_GOOD, ROLE_REWARD_BAD);
        // For word 0: (popcount(0b11 & 0b111) - popcount(0b100 & 0b111)) = 2 - 1 = 1
        // For all other words: 0
        // Total score should be positive (≥1)
        assert!(score > 0, "expected positive reward score, got {}", score);
    }
}
