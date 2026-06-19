//! Deterministic replay frames.
//!
//! A replay is the initial admitted state plus an ordered sequence of frames. Re-folding
//! the same frames must reproduce an identical digest; [`crate::verify`] uses this to check
//! determinism, which is what gives a replay evidentiary value.

/// One deterministic replay frame: an input applied at a tick and the resulting digest.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReplayFrame {
    /// Logical tick.
    pub tick: u64,
    /// Admitted input word applied at this tick.
    pub input: u64,
    /// Digest of the resulting state.
    pub state_digest: u64,
}

impl ReplayFrame {
    /// Construct a replay frame.
    #[inline]
    #[must_use]
    pub const fn new(tick: u64, input: u64, state_digest: u64) -> Self {
        Self {
            tick,
            input,
            state_digest,
        }
    }
}
