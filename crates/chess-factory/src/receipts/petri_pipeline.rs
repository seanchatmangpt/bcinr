//! Branchless 5-place decision-pipeline Petri net (lifted from the playground
//! conformance demo into the factory as a shared, receipt-grade module).
//!
//! Every lawful move traverses the declared decision process:
//!
//! ```text
//! READY -> Perceive -> Evaluate -> Search -> Select -> Commit -> READY
//! ```
//!
//! Each stage corresponds to a Petri transition that consumes its input place's
//! token and produces its output place's token. Replaying a recorded stage log
//! through this net yields a conformance [`ReplayResult`]; a lawful run is
//! `fitness() == 1.0` with no missing/remaining tokens. A skipped or reordered
//! stage strands tokens, dropping fitness below 1.0 — the conformance teeth that
//! make a receipt provably lawful (or provably not).
//!
//! The firing kernel and fitness reduction are branchless (CC=1), built on the
//! `bcinr-logic` mask/int substrate.

use bcinr_logic::int::popcount_u64;
use bcinr_logic::mask::{is_zero_mask_u32, select_u32, select_u64};

/// One bit per place in the `u64` marking.
pub const READY: u64 = 1 << 0;
/// Position has been perceived (legal action set enumerated).
pub const PERCEIVED: u64 = 1 << 1;
/// Position has been evaluated.
pub const EVALUATED: u64 = 1 << 2;
/// Candidate children have been searched.
pub const SEARCHED: u64 = 1 << 3;
/// A move has been selected.
pub const SELECTED: u64 = 1 << 4;

/// Lawful transitions in firing order: `(name, in_place, out_place)`.
pub const TRANSITIONS: &[(&str, u64, u64)] = &[
    ("Perceive", READY, PERCEIVED),
    ("Evaluate", PERCEIVED, EVALUATED),
    ("Search", EVALUATED, SEARCHED),
    ("Select", SEARCHED, SELECTED),
    ("Commit", SELECTED, READY),
];

/// The canonical, ordered stage names a lawful move receipt must record.
pub const STAGE_NAMES: [&str; 5] = ["Perceive", "Evaluate", "Search", "Select", "Commit"];

/// Result of replaying a stage log through the decision Petri net.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ReplayResult {
    /// Tokens that had to be supplied on demand (a stage fired without its
    /// precondition — i.e. an out-of-order or skipped transition).
    pub missing: u32,
    /// Tokens left stranded outside the idle `READY` place at completion.
    pub remaining: u32,
    /// Tokens produced across all firings.
    pub produced: u32,
    /// Tokens consumed across all firings.
    pub consumed: u32,
}

impl ReplayResult {
    /// Construct a result from raw token counters.
    #[must_use]
    pub const fn new(missing: u32, remaining: u32, produced: u32, consumed: u32) -> Self {
        Self {
            missing,
            remaining,
            produced,
            consumed,
        }
    }

    /// Branchless conformance fitness in `[0, 1]`:
    /// `1.0 - (missing + remaining) / (consumed + missing + produced)`,
    /// or `1.0` when the denominator is zero.
    #[must_use]
    pub fn fitness(&self) -> f64 {
        let denom_u32 = self
            .consumed
            .wrapping_add(self.missing)
            .wrapping_add(self.produced);
        let is_zero = is_zero_mask_u32(denom_u32);
        let safe_denom = select_u32(is_zero, 1, denom_u32);

        let sum_num = self.missing.wrapping_add(self.remaining);
        let raw_fitness = 1.0 - (f64::from(sum_num) / f64::from(safe_denom));

        let is_zero_64 = (u64::from(is_zero)) | (u64::from(is_zero) << 32);
        let fit_bits = select_u64(is_zero_64, (1.0f64).to_bits(), raw_fitness.to_bits());
        f64::from_bits(fit_bits)
    }

    /// True iff no tokens were missing and none stranded.
    #[must_use]
    pub fn is_perfect(&self) -> bool {
        (self.missing | self.remaining) == 0
    }
}

/// Constant-time Petri transition firing step (CC = 1). Missing tokens are
/// supplied on demand so the counters can detect out-of-order execution.
#[inline(always)]
pub fn petri_fire_transition(
    marking: &mut u64,
    in_mask: u64,
    out_mask: u64,
    missing: &mut u32,
    consumed: &mut u32,
    produced: &mut u32,
) {
    let need = in_mask & !(*marking);
    *missing = (*missing).wrapping_add(popcount_u64(need) as u32);
    *marking |= need;

    *marking = (*marking & !in_mask) | out_mask;
    *consumed = (*consumed).wrapping_add(popcount_u64(in_mask) as u32);
    *produced = (*produced).wrapping_add(popcount_u64(out_mask) as u32);
}

/// Replay a sequence of stage names through the decision Petri net.
///
/// Unknown stage names are ignored (they cannot fire a lawful transition);
/// known stages fire their `(in, out)` transition. The starting marking is one
/// idle `READY` token, matching the engine's between-moves resting state.
#[must_use]
pub fn replay(stages: &[&str]) -> ReplayResult {
    let mut marking: u64 = READY;
    let (mut missing, mut consumed, mut produced) = (0u32, 0u32, 0u32);
    for stage in stages {
        if let Some(&(_, in_place, out_place)) =
            TRANSITIONS.iter().find(|(name, _, _)| name == stage)
        {
            petri_fire_transition(
                &mut marking,
                in_place,
                out_place,
                &mut missing,
                &mut consumed,
                &mut produced,
            );
        }
    }
    let remaining = (marking & !READY).count_ones();
    ReplayResult::new(missing, remaining, produced, consumed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lawful_run_is_perfect() {
        let r = replay(&STAGE_NAMES);
        assert!(r.is_perfect());
        assert!((r.fitness() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn skipped_stage_is_rejected() {
        // Drop "Evaluate": Search fires without its EVALUATED precondition.
        let r = replay(&["Perceive", "Search", "Select", "Commit"]);
        assert!(!r.is_perfect());
        assert!(r.fitness() < 1.0);
    }

    #[test]
    fn empty_log_strands_nothing() {
        let r = replay(&[]);
        assert!(r.is_perfect());
    }
}
