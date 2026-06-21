//! `procmine` — offline, branchless, `no_std` object-centric **conformance checking** for the
//! JTBD chains.
//!
//! This module makes the `wasm4pm` doctrine — *engines project worlds; `wasm4games` operates
//! patterns; `wasm4pm` admits evidence* — executable **offline**, in the tradition of
//! process mining (Wil van der Aalst et al.): it takes an observed activity trace, replays it
//! over the *de jure* process model declared in `chains.ttl`, and scores conformance with
//! **token-based replay fitness**, returning a verdict on the `wasm4pm`-compat surface.
//!
//! # Pipeline
//!
//! | Stage | Type / fn | Process-mining concept |
//! |---|---|---|
//! | normative model | [`model::CHAIN_MODELS`] | the *de jure* process (declared order) |
//! | observed log | [`ocel_log::events_for_chain`] → [`Trace`] | the OCEL event log of a run |
//! | replay | [`replay::replay`] → [`replay::TokenCounts`] | token-based replay (p/c/m/r) |
//! | score | [`conformance::fitness_bp`] | van der Aalst fitness, basis points |
//! | discover | [`dfg::Dfg`] | directly-follows graph + model comparison |
//! | verdict | [`conformance::to_verdict`] | admission decision ([`crate::compat::Verdict`]) |
//! | oracle | [`GOLDEN_CONFORMANCE_DIGEST`] | frozen drift detector (à la `corpus.rs`) |
//!
//! Everything here is `no_std`, allocation-free (fixed-capacity arrays), deterministic, and
//! WCET-bounded — conformance is computed with bounded loops and integer arithmetic only.
//!
//! # Examples
//!
//! ```
//! use wasm4games::procmine::{conformance, CHAIN_MODELS};
//! use wasm4games::compat::Verdict;
//!
//! // A chain replayed against its own declared model is perfectly conformant.
//! let model = &CHAIN_MODELS[2]; // combat_hit
//! let result = conformance::check_model_self(model);
//! assert_eq!(result.fitness_bp, 10_000);
//! assert!(matches!(conformance::to_verdict(&result), Verdict::Admitted));
//! ```

pub mod conformance;
pub mod dfg;
pub mod model;
pub mod ocel_log;
pub mod replay;

pub use conformance::{
    check_model_self, check_trace, fitness_bp, to_verdict, ConformanceResult, FITNESS_PERFECT_BP,
};
pub use dfg::{Dfg, DfgDivergence};
pub use model::{ChainModel, CHAIN_MODELS};
pub use replay::{replay, TokenCounts};

/// Maximum number of steps (activities) in a chain trace.
pub const MAX_STEPS: usize = 8;

/// A bounded, `Copy`, allocation-free observed trace: an ordered activity (pattern-id)
/// sequence of at most [`MAX_STEPS`] entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Trace {
    acts: [u16; MAX_STEPS],
    len: u8,
}

impl Trace {
    /// An empty trace.
    #[inline]
    #[must_use = "returns an empty Trace; push activities into it"]
    pub const fn new() -> Self {
        Self {
            acts: [0; MAX_STEPS],
            len: 0,
        }
    }

    /// Append one activity. Saturates silently at [`MAX_STEPS`].
    #[inline]
    pub fn push(&mut self, activity: u16) {
        let idx = self.len as usize;
        if idx < MAX_STEPS {
            self.acts[idx] = activity;
            self.len += 1;
        }
    }

    /// The activity sequence as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u16] {
        &self.acts[..self.len as usize]
    }

    /// Number of activities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Whether the trace is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Swap the activities at positions `i` and `j` (both must be `< len`).
    ///
    /// Primarily a test/perturbation helper for constructing non-conforming traces.
    #[inline]
    pub fn swap(&mut self, i: usize, j: usize) {
        let n = self.len as usize;
        if i < n && j < n {
            self.acts.swap(i, j);
        }
    }

    /// Build a trace from an activity slice (truncated to [`MAX_STEPS`]).
    #[must_use]
    pub fn from_activities(activities: &[u16]) -> Self {
        let mut t = Self::new();
        for &a in activities {
            t.push(a);
        }
        t
    }

    /// Build a trace from a chain model's declared activity order (self-replay).
    #[must_use]
    pub fn from_model(m: &ChainModel) -> Self {
        Self::from_activities(m.activities.as_slice())
    }
}

impl Default for Trace {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// FNV-1a fold of every chain's self-conformance result into one rolling digest.
///
/// Mirrors [`crate::corpus`]'s `GOLDEN_CORPUS_DIGEST` idiom: it binds the conformance math,
/// the model order, the token counts, and each chain's `ch:golden` into a single value, so
/// any drift in the conformance algorithm, the model, or the chain ontology changes it.
#[must_use = "returns the rolling conformance digest; compare it to the pinned golden"]
pub fn conformance_digest() -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut fold = |x: u64| {
        h = (h ^ x).wrapping_mul(0x0000_0100_0000_01b3);
    };
    for m in CHAIN_MODELS {
        let r = check_model_self(m);
        fold(u64::from(r.chain_id));
        fold(u64::from(r.fitness_bp));
        fold(u64::from(r.status));
        fold(u64::from(r.counts.produced));
        fold(u64::from(r.counts.consumed));
        fold(u64::from(r.counts.missing));
        fold(u64::from(r.counts.remaining));
        for &a in &m.activities {
            fold(u64::from(a));
        }
        fold(m.golden);
    }
    h
}

/// The pinned golden conformance digest. Changes only when conformance behavior is
/// intentionally altered (then re-pin via [`conformance_digest`]).
pub const GOLDEN_CONFORMANCE_DIGEST: u64 = 0x19cc_167a_7d96_3549;

/// Assert the live [`conformance_digest`] matches the pinned [`GOLDEN_CONFORMANCE_DIGEST`].
///
/// # Panics
///
/// Panics if conformance behavior has drifted from the frozen oracle.
pub fn assert_conformance_stable() {
    assert_eq!(
        conformance_digest(),
        GOLDEN_CONFORMANCE_DIGEST,
        "procmine conformance digest drifted from the pinned golden"
    );
}

/// Whether the live conformance digest matches the pinned golden (non-panicking form).
#[must_use]
pub fn verify_conformance() -> bool {
    conformance_digest() == GOLDEN_CONFORMANCE_DIGEST
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::PATTERN_REGISTRY;

    #[test]
    fn there_are_eight_chain_models_with_valid_activities() {
        assert_eq!(CHAIN_MODELS.len(), 8);
        for m in CHAIN_MODELS {
            assert_eq!(m.activities.len(), 8);
            for &a in &m.activities {
                assert!(
                    PATTERN_REGISTRY.iter().any(|s| s.id.raw() == a),
                    "chain {} references unknown pattern id {}",
                    m.name,
                    a
                );
            }
        }
    }

    #[test]
    fn chain_ids_are_one_based_and_ordered_by_name() {
        for (i, m) in CHAIN_MODELS.iter().enumerate() {
            assert_eq!(m.chain_id as usize, i + 1);
        }
        for w in CHAIN_MODELS.windows(2) {
            assert!(w[0].name < w[1].name, "CHAIN_MODELS must be name-ordered");
        }
    }

    #[test]
    fn trace_push_saturates_and_round_trips() {
        let mut t = Trace::new();
        assert!(t.is_empty());
        for i in 0..(MAX_STEPS as u16 + 3) {
            t.push(i);
        }
        assert_eq!(t.len(), MAX_STEPS);
        assert_eq!(t.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn golden_conformance_digest_is_stable() {
        assert_eq!(
            conformance_digest(),
            conformance_digest(),
            "must be deterministic"
        );
        assert_conformance_stable();
        assert!(verify_conformance());
    }
}
