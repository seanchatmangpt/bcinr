//! Token-based replay over the sequential workflow net of a chain model.
//!
//! Each chain model is a *strictly sequential* workflow net: places `p0..pn` (`p0` the
//! source, `pn` the sink) and one transition `t_i` per activity that consumes `p_i` and
//! produces `p_{i+1}`. Replaying an observed trace over this net and counting the four
//! token classes — **produced**, **consumed**, **missing**, **remaining** — is van der
//! Aalst's token-replay; the counts feed the fitness measure in
//! [`crate::procmine::conformance`].
//!
//! The implementation is `no_std` and allocation-free: the marking is a fixed
//! `[u32; MAX_PLACES]` array (one token counter per place). It is deterministic and
//! WCET-bounded — the loops have compile-time-bounded length.

use super::MAX_STEPS;

/// One more than [`MAX_STEPS`]: the number of places in the largest sequential net.
pub const MAX_PLACES: usize = MAX_STEPS + 1;

/// The four token classes accumulated by a single [`replay`].
///
/// `produced` counts tokens added to places (including the initial source token);
/// `consumed` counts tokens removed (including the final sink token); `missing` counts
/// input tokens that had to be created because a transition fired while not enabled; and
/// `remaining` counts tokens left in non-sink places after the trace ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TokenCounts {
    /// Tokens produced (`p`), including the initial marking.
    pub produced: u32,
    /// Tokens consumed (`c`), including the final marking.
    pub consumed: u32,
    /// Missing tokens (`m`): inputs created to force a disabled transition.
    pub missing: u32,
    /// Remaining tokens (`r`): tokens left in non-sink places at termination.
    pub remaining: u32,
}

/// Replay `trace` over the sequential workflow net implied by `model` (the activity order),
/// returning the token-class counts.
///
/// A trace that exactly reproduces `model` fires every transition while enabled and leaves
/// only the sink marked, so `missing == 0` and `remaining == 0`. Any reordering forces at
/// least one disabled transition (raising `missing`) and strands at least one token
/// (raising `remaining`).
///
/// Only the first [`MAX_STEPS`] activities of `model` define transitions; an observed
/// activity not present in `model` is a log-only move (counted as one missing + one
/// consumed) with no production.
#[must_use = "returns the token counts to feed into fitness_bp; ignoring them discards the replay"]
pub fn replay(model: &[u16], trace: &[u16]) -> TokenCounts {
    let n = model.len().min(MAX_STEPS);
    let model = &model[..n];

    // marking[k] = token count in place k, for k in 0..=n.
    let mut marking = [0u32; MAX_PLACES];
    marking[0] = 1; // initial marking: one token in the source place.
    let mut produced: u32 = 1; // the initial token counts as produced.
    let mut consumed: u32 = 0;
    let mut missing: u32 = 0;

    for &activity in trace {
        if let Some(t) = model.iter().position(|&label| label == activity) {
            // Transition t fires: consume place t, produce place t+1.
            let enabled = marking[t] >= 1;
            // Standard "create missing token" rule when the input place is empty.
            marking[t] = marking[t].saturating_sub(1);
            missing += u32::from(!enabled);
            consumed += 1;
            marking[t + 1] += 1;
            produced += 1;
        } else {
            // Foreign activity: a log move the model cannot replay.
            missing += 1;
            consumed += 1;
        }
    }

    // Consume the expected final (sink) token; if absent it is a missing token.
    let sink_present = marking[n] >= 1;
    marking[n] = marking[n].saturating_sub(1);
    missing += u32::from(!sink_present);
    consumed += 1;

    // Everything still marked (the sink was just consumed) is remaining.
    let remaining: u32 = marking[..=n].iter().copied().sum();

    TokenCounts {
        produced,
        consumed,
        missing,
        remaining,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_replay_has_no_missing_or_remaining() {
        let model = [10u16, 20, 30, 40, 50, 60, 70, 80];
        let c = replay(&model, &model);
        // 8 transitions + initial token produced; 8 consumed + final token consumed.
        assert_eq!(c.produced, 9);
        assert_eq!(c.consumed, 9);
        assert_eq!(c.missing, 0);
        assert_eq!(c.remaining, 0);
    }

    #[test]
    fn adjacent_swap_strands_a_token_and_misses_one() {
        let model = [10u16, 20, 30, 40, 50, 60, 70, 80];
        let mut trace = model;
        trace.swap(0, 1); // [20,10,30,...]
        let c = replay(&model, &trace);
        assert!(c.missing >= 1, "a disabled transition must be forced");
        assert!(c.remaining >= 1, "a token must be stranded");
    }

    #[test]
    fn foreign_activity_counts_as_missing() {
        let model = [1u16, 2, 3];
        let trace = [1u16, 2, 3, 999]; // 999 is not in the model
        let c = replay(&model, &trace);
        assert!(c.missing >= 1);
    }

    #[test]
    fn replay_is_deterministic() {
        let model = [1u16, 2, 3, 4, 5, 6, 7, 8];
        let trace = [3u16, 1, 2, 4, 5, 6, 7, 8];
        assert_eq!(replay(&model, &trace), replay(&model, &trace));
    }
}
