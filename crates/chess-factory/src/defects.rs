
//! Generated defect taxonomy — manufactured from `ontology/chess.ttl`.
//!
//! First-class source (GGEN-SRC law): edit the `cf:Defect` individuals in the
//! ontology and re-run `ggen sync`. Each defect is a named-law refusal class
//! consumed by the verifier's `Refusal` / `DiagnosticPayload` surface.

/// One named-law defect class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Defect {
    /// Stable integer defect code (ORDER BY anchor).
    pub code: u16,
    /// snake_case defect name.
    pub name: &'static str,
    /// Human-readable law violated.
    pub doc: &'static str,
}

/// The defect taxonomy, ordered by code.
pub static DEFECTS: &[Defect] = &[
    Defect {
        code: 100,
        name: "illegal_move",
        doc: "Chosen move is not legal in the position (POWL validate_chess_move_powl rejects it).",
    },
    Defect {
        code: 101,
        name: "timeout",
        doc: "Decision exceeded its deterministic node budget (node-bounded, not wall-clock).",
    },
    Defect {
        code: 102,
        name: "missing_receipt",
        doc: "A move was played without an emitted MoveReceipt linking into the chain.",
    },
    Defect {
        code: 103,
        name: "invalid_transition",
        doc: "Petri-net pipeline replay did not reach a lawful marking (fitness != 1.0).",
    },
    Defect {
        code: 104,
        name: "strategy_divergence",
        doc: "Replayed station evaluation diverged from the recorded receipt (recompute_and_match failed).",
    },
    Defect {
        code: 105,
        name: "benchmark_regression",
        doc: "Measured Elo-at-budget fell below the prior baseline beyond the regression threshold.",
    },
];

/// Defect code for `illegal_move`.
pub const DEFECT_ILLEGAL_MOVE: u16 = 100;
/// Defect code for `timeout`.
pub const DEFECT_TIMEOUT: u16 = 101;
/// Defect code for `missing_receipt`.
pub const DEFECT_MISSING_RECEIPT: u16 = 102;
/// Defect code for `invalid_transition`.
pub const DEFECT_INVALID_TRANSITION: u16 = 103;
/// Defect code for `strategy_divergence`.
pub const DEFECT_STRATEGY_DIVERGENCE: u16 = 104;
/// Defect code for `benchmark_regression`.
pub const DEFECT_BENCHMARK_REGRESSION: u16 = 105;


/// Number of defect classes.
pub const DEFECT_COUNT: usize = 6;

/// Look up a defect by code. Returns `None` if the code is unknown.
#[must_use]
pub fn defect_by_code(code: u16) -> Option<&'static Defect> {
    let mut i = 0usize;
    while i < DEFECTS.len() {
        if DEFECTS[i].code == code {
            return Some(&DEFECTS[i]);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_is_ordered_and_complete() {
        assert_eq!(DEFECTS.len(), DEFECT_COUNT);
        for w in DEFECTS.windows(2) {
            assert!(w[0].code < w[1].code, "DEFECTS must be ORDER BY code");
        }
    }

    #[test]
    fn lookup_round_trips() {
        for d in DEFECTS {
            assert_eq!(defect_by_code(d.code).map(|x| x.code), Some(d.code));
        }
        assert!(defect_by_code(u16::MAX).is_none());
    }
}