//! The conformance core: token-replay fitness, the [`ConformanceResult`], and the mapping
//! onto the `wasm4pm`-compat [`Verdict`] surface.
//!
//! Fitness follows van der Aalst's token-replay measure
//! `fitness = ½(1 − m/c) + ½(1 − r/p)`, computed in **integer basis points** (`u32` in
//! `[0, 10_000]`, where `10_000` = perfect) so the whole pipeline is `no_std` and free of
//! floating point. A trace that reproduces the declared model order scores
//! [`FITNESS_PERFECT_BP`]; any reordering scores strictly less.

use super::model::ChainModel;
use super::replay::{replay, TokenCounts};
use super::Trace;
use crate::class::status;
use crate::compat::Verdict;

/// Perfect fitness in basis points (100.00%).
pub const FITNESS_PERFECT_BP: u32 = 10_000;

/// Fitness threshold (basis points) at or above which a trace is treated as admissible-ish
/// (`PARTIAL`) rather than refused.
pub const FITNESS_ADMIT_BAND_BP: u32 = 9_000;

/// The outcome of conformance-checking one trace against one chain model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConformanceResult {
    /// The model the trace was checked against ([`ChainModel::chain_id`]).
    pub chain_id: u16,
    /// Token-replay fitness in basis points, `0..=10_000`.
    pub fitness_bp: u32,
    /// The raw token counts the fitness was derived from.
    pub counts: TokenCounts,
    /// Whether the trace exactly reproduces the declared activity order.
    pub trace_fits_order: bool,
    /// The [`crate::class::status`] band the fitness falls into.
    pub status: u8,
}

/// Token-replay fitness in basis points from a set of [`TokenCounts`].
///
/// `½(1 − m/c)` and `½(1 − r/p)` are each scaled to `5_000` bp; divisors are guarded to at
/// least `1` so the function is total, and `m`/`r` are clamped to their denominators so each
/// half stays in `[0, 5_000]`. A perfect replay (`m = r = 0`) returns [`FITNESS_PERFECT_BP`].
#[must_use = "returns the fitness score in basis points"]
pub fn fitness_bp(c: &TokenCounts) -> u32 {
    let consumed = c.consumed.max(1);
    let produced = c.produced.max(1);
    let missing = c.missing.min(consumed);
    let remaining = c.remaining.min(produced);
    let half_missing = 5_000 - (5_000 * missing) / consumed; // 5000·(1 − m/c)
    let half_remaining = 5_000 - (5_000 * remaining) / produced; // 5000·(1 − r/p)
    (half_missing + half_remaining).min(FITNESS_PERFECT_BP)
}

/// The [`crate::class::status`] band for a fitness score: perfect → `ADMITTED`,
/// at/above the admit band → `PARTIAL`, below → `REFUSED`.
#[must_use]
pub fn status_for_fitness(fitness_bp: u32) -> u8 {
    if fitness_bp >= FITNESS_PERFECT_BP {
        status::ADMITTED
    } else if fitness_bp >= FITNESS_ADMIT_BAND_BP {
        status::PARTIAL
    } else {
        status::REFUSED
    }
}

/// Whether `trace` exactly reproduces `model`'s declared activity order.
#[must_use]
pub fn matches_declared_order(model: &ChainModel, trace: &Trace) -> bool {
    trace.as_slice() == model.activities.as_slice()
}

/// Conformance-check one observed `trace` against one chain `model`.
#[must_use = "returns the conformance result; inspect fitness_bp or to_verdict"]
pub fn check_trace(model: &ChainModel, trace: &Trace) -> ConformanceResult {
    let counts = replay(model.activities.as_slice(), trace.as_slice());
    let fitness_bp = fitness_bp(&counts);
    ConformanceResult {
        chain_id: model.chain_id,
        fitness_bp,
        counts,
        trace_fits_order: matches_declared_order(model, trace),
        status: status_for_fitness(fitness_bp),
    }
}

/// Replay a model on itself — the perfect-fitness oracle (always [`FITNESS_PERFECT_BP`]).
#[must_use = "returns the self-conformance result"]
pub fn check_model_self(model: &ChainModel) -> ConformanceResult {
    check_trace(model, &Trace::from_model(model))
}

/// Map a [`ConformanceResult`] onto the `wasm4pm`-compat [`Verdict`].
///
/// Perfect fitness with the declared order is [`Verdict::Admitted`]; clearly sub-threshold
/// fitness is [`Verdict::Refused`] carrying the status band; the middle band is
/// [`Verdict::Unknown`] (the offline proxy cannot decide — the authority is `wasm4pm`).
#[must_use]
pub fn to_verdict(r: &ConformanceResult) -> Verdict {
    if r.fitness_bp == FITNESS_PERFECT_BP && r.trace_fits_order {
        Verdict::Admitted
    } else if r.fitness_bp < FITNESS_ADMIT_BAND_BP {
        Verdict::Refused(r.status)
    } else {
        Verdict::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::CHAIN_MODELS;
    use super::*;

    #[test]
    fn perfect_counts_score_ten_thousand() {
        let c = TokenCounts {
            produced: 9,
            consumed: 9,
            missing: 0,
            remaining: 0,
        };
        assert_eq!(fitness_bp(&c), 10_000);
    }

    #[test]
    fn fitness_is_bounded_and_monotone_in_missing() {
        // A worst case stays within range.
        let worst = TokenCounts {
            produced: 9,
            consumed: 9,
            missing: 9,
            remaining: 9,
        };
        assert!(fitness_bp(&worst) <= FITNESS_PERFECT_BP);

        // Increasing missing (others fixed) never raises fitness.
        let mut prev = fitness_bp(&TokenCounts {
            produced: 9,
            consumed: 9,
            missing: 0,
            remaining: 0,
        });
        for m in 1..=9u32 {
            let f = fitness_bp(&TokenCounts {
                produced: 9,
                consumed: 9,
                missing: m,
                remaining: 0,
            });
            assert!(f <= prev, "fitness must not increase as missing grows");
            prev = f;
        }
    }

    #[test]
    fn every_chain_self_conforms_perfectly() {
        for m in CHAIN_MODELS {
            let r = check_model_self(m);
            assert_eq!(r.fitness_bp, 10_000, "chain {} must self-conform", m.name);
            assert_eq!(r.counts.missing, 0);
            assert_eq!(r.counts.remaining, 0);
            assert!(r.trace_fits_order);
            assert!(matches!(to_verdict(&r), Verdict::Admitted));
            assert_eq!(r.status, status::ADMITTED);
        }
    }

    #[test]
    fn reordered_trace_drops_below_perfect_and_is_not_admitted() {
        let model = &CHAIN_MODELS[2]; // combat_hit
        let mut trace = Trace::from_model(model);
        trace.swap(1, 5); // non-adjacent reorder
        let r = check_trace(model, &trace);
        assert!(
            r.fitness_bp < FITNESS_PERFECT_BP,
            "reorder must lower fitness"
        );
        assert!(r.counts.missing > 0);
        assert!(!r.trace_fits_order);
        assert!(!matches!(to_verdict(&r), Verdict::Admitted));
    }

    #[test]
    fn verdict_bands_map_correctly() {
        let mk = |f: u32| ConformanceResult {
            chain_id: 1,
            fitness_bp: f,
            counts: TokenCounts::default(),
            trace_fits_order: f == FITNESS_PERFECT_BP,
            status: status_for_fitness(f),
        };
        assert!(matches!(to_verdict(&mk(10_000)), Verdict::Admitted));
        assert!(matches!(to_verdict(&mk(9_500)), Verdict::Unknown));
        assert!(matches!(to_verdict(&mk(4_000)), Verdict::Refused(_)));
    }
}
