//! Deterministic replay frames.
//!
//! A replay is the initial admitted state plus an ordered sequence of frames. Re-folding
//! the same frames must reproduce an identical digest; [`crate::verify`] uses this to check
//! determinism, which is what gives a replay evidentiary value.
//!
//! # Model
//!
//! Each [`ReplayFrame`] records:
//! * `tick` — the logical tick at which an input was applied.
//! * `input` — the admitted input word at that tick.
//! * `state_digest` — the FNV-1a (or equivalent) digest of the resulting state.
//!
//! A verifier re-runs the same sequence of `(tick, input)` pairs and checks that each
//! resulting `state_digest` matches. Divergence at any frame indicates non-determinism.
//!
//! # `no_std` compatibility
//!
//! [`ReplayFrame`] is `Copy`, `Hash`, and `Ord` (ordered by `tick`); no heap allocations.

use core::fmt;

/// One deterministic replay frame: an input applied at a tick and the resulting digest.
///
/// # Ordering
///
/// Frames derive [`Ord`] with lexicographic ordering by `(tick, input, state_digest)`.
/// In practice, replay sequences are ordered by `tick`; two frames at the same tick with
/// different inputs indicate a non-determinism fault.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::replay::ReplayFrame;
/// let frame = ReplayFrame::new(42, 0xFF, 0xDEAD_BEEF_CAFE_BABE);
/// assert_eq!(frame.tick, 42);
/// assert_eq!(frame.input, 0xFF);
/// assert_eq!(frame.state_digest, 0xDEAD_BEEF_CAFE_BABE);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ReplayFrame {
    /// Logical tick at which the input was applied.
    ///
    /// Ticks are monotonically increasing. A replay verifier iterates frames in tick order.
    pub tick: u64,
    /// Admitted input word applied at this tick.
    ///
    /// Only admitted inputs appear in a valid replay; refused inputs are not recorded.
    pub input: u64,
    /// Digest of the resulting state after applying `input` at `tick`.
    ///
    /// Computed by the same FNV-1a chain used by [`crate::evidence::receipt::ReceiptChain`].
    pub state_digest: u64,
}

impl ReplayFrame {
    /// Construct a replay frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::replay::ReplayFrame;
    /// let frame = ReplayFrame::new(10, 0xAB, 0x1234_5678_9ABC_DEF0);
    /// assert_eq!(frame.tick, 10);
    /// assert_eq!(frame.input, 0xAB);
    /// assert_eq!(frame.state_digest, 0x1234_5678_9ABC_DEF0);
    /// ```
    #[inline]
    #[must_use = "returns the new ReplayFrame; bind it to a variable"]
    pub const fn new(tick: u64, input: u64, state_digest: u64) -> Self {
        Self {
            tick,
            input,
            state_digest,
        }
    }

    /// Whether this frame's state digest matches the expected value.
    ///
    /// Use this inside a verifier to detect the first divergence point.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::replay::ReplayFrame;
    /// let frame = ReplayFrame::new(1, 0, 0xABC);
    /// assert!(frame.digest_matches(0xABC));
    /// assert!(!frame.digest_matches(0xDEF));
    /// ```
    #[inline]
    #[must_use = "returns true if the digest matches; check this to detect replay divergence"]
    pub const fn digest_matches(self, expected: u64) -> bool {
        self.state_digest == expected
    }
}

impl fmt::Display for ReplayFrame {
    /// Formats as `ReplayFrame(tick=N, input=0xN, digest=0xN)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::replay::ReplayFrame;
    /// let frame = ReplayFrame::new(7, 0x0F, 0xABCD);
    /// let s = format!("{}", frame);
    /// assert!(s.contains("tick=7"));
    /// assert!(s.contains("input="));
    /// assert!(s.contains("digest="));
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReplayFrame(tick={}, input={:#018x}, digest={:#018x})",
            self.tick, self.input, self.state_digest
        )
    }
}

/// Convert a `(tick, input, state_digest)` tuple into a [`ReplayFrame`].
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::replay::ReplayFrame;
/// let frame = ReplayFrame::from((5u64, 7u64, 9u64));
/// assert_eq!(frame.tick, 5);
/// assert_eq!(frame.input, 7);
/// assert_eq!(frame.state_digest, 9);
/// ```
impl From<(u64, u64, u64)> for ReplayFrame {
    #[inline]
    fn from((tick, input, state_digest): (u64, u64, u64)) -> Self {
        Self::new(tick, input, state_digest)
    }
}

/// Convert a [`ReplayFrame`] into its `(tick, input, state_digest)` tuple.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::replay::ReplayFrame;
/// let frame = ReplayFrame::new(1, 2, 3);
/// let (tick, input, digest): (u64, u64, u64) = frame.into();
/// assert_eq!((tick, input, digest), (1, 2, 3));
/// ```
impl From<ReplayFrame> for (u64, u64, u64) {
    #[inline]
    fn from(f: ReplayFrame) -> (u64, u64, u64) {
        (f.tick, f.input, f.state_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_frame_behavior() {
        // new / fields / From roundtrip
        let frame = ReplayFrame::new(42, 0xFF, 0xDEAD_BEEF_CAFE_BABE);
        assert_eq!(
            (frame.tick, frame.input, frame.state_digest),
            (42, 0xFF, 0xDEAD_BEEF_CAFE_BABE)
        );
        let orig = ReplayFrame::new(1, 2, 3);
        let tuple: (u64, u64, u64) = orig.into();
        assert_eq!(ReplayFrame::from(tuple), orig);

        // digest_matches
        let frame = ReplayFrame::new(0, 0, 0xABCD_EF01);
        assert!(frame.digest_matches(0xABCD_EF01));
        assert!(!frame.digest_matches(0xABCD_EF00));
        assert!(!frame.digest_matches(0));

        // Copy + Eq
        let a = ReplayFrame::new(1, 2, 3);
        assert_eq!(a, a);

        // ordering: lexicographic (tick, input, state_digest)
        assert!(ReplayFrame::new(1, 0xFF, 0) < ReplayFrame::new(2, 0x00, 0));
        assert!(ReplayFrame::new(5, 1, 0) < ReplayFrame::new(5, 2, 0));
        assert!(ReplayFrame::new(5, 1, 0) < ReplayFrame::new(5, 1, 1));

        // Hash is consistent
        use core::hash::{Hash, Hasher};
        struct SimpleHasher(u64);
        impl Hasher for SimpleHasher {
            fn finish(&self) -> u64 {
                self.0
            }
            fn write(&mut self, bytes: &[u8]) {
                for &b in bytes {
                    self.0 = self.0.wrapping_mul(31).wrapping_add(b as u64);
                }
            }
        }
        let frame = ReplayFrame::new(10, 20, 30);
        let mut h1 = SimpleHasher(0);
        let mut h2 = SimpleHasher(0);
        frame.hash(&mut h1);
        frame.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn replay_frame_display() {
        let s = alloc::format!("{}", ReplayFrame::new(7, 0x0F, 0xABCD));
        assert!(s.contains("tick=7") && s.contains("input=") && s.contains("digest="));
    }
}
