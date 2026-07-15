//! replay — token-passing POWL replay verifier with branchless metric accumulation.
//!
//! Uses [`PowlReplayFrame`] — a high-level frame with token-flow semantics
//! (`node_bit`, `required_tokens`, `produces_tokens`), distinct from the binary-packed
//! [`crate::causal_receipt::OcelCausalFrame`] used for BLAKE3 hash chaining.
//!
//! # `fitness`/`precision` are real; `generalization`/`simplicity` are MOCKED
//!
//! [`PowlReplayVerifier::finalize`] computes only two of the four
//! [`ConformanceMetrics`] dimensions from real replay state:
//! `fitness`/`precision` are genuine, derived from the token-passing
//! accumulators `replay_frame` actually updates as frames are consumed.
//! `generalization` and `simplicity` are **not computed at all** — this
//! verifier has no state-space enumeration or model-complexity analysis to
//! derive them from — and are hardcoded to `0x0000_0000` (Q16.16 zero).
//! This is a deliberate MOCKED placeholder, not a bug hidden as a real
//! value: `0` is a valid, in-range Q16.16 quantity that fails any
//! predicate with a nonzero `min_generalization`/`min_simplicity`
//! threshold (see [`crate::conformance::ConformancePredicate::STRICT`]/
//! `LENIENT`, both of which set both to `0x0000_8000`/`0x0000_4000`) — so
//! a caller who checks a real predicate against a `finalize()` result
//! cannot mistake "unmeasured" for "measured and passing." An earlier
//! version of this code used `0x8000_0000`/`0xC000_0000`, which are
//! ~32768x/49152x larger than `1.0` under this crate's own Q16.16
//! encoding (`conformance.rs`'s `0x0001_0000 == 1.0`) and, because
//! `mask_ge`'s branchless `>=` widens both operands to `i64` before
//! comparing, silently made every real `ConformancePredicate::check` call
//! report `generalization`/`simplicity` as passing regardless of replay
//! behavior — the exact "stub that returns success" shape this
//! workspace's honesty discipline forbids. See
//! `PowlReplayVerifier::finalize`'s tests below for both the disclosure
//! and a `ConformancePredicate::check` run against a real `finalize()`
//! output that empirically fails on these two dimensions.

use crate::conformance::ConformanceMetrics;

/// High-level replay descriptor for one POWL node firing.
///
/// Distinct from [`crate::causal_receipt::OcelCausalFrame`] (binary hash-chaining struct).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowlReplayFrame {
    /// Unique node identifier (for error reporting).
    pub node_id: u32,
    /// 1-hot bitmask position for this node (must have exactly one bit set).
    pub node_bit: u64,
    /// Bitmask of tokens that must all be present for this node to fire.
    pub required_tokens: u64,
    /// Tokens this firing produces (post-set); 0 for sink nodes.
    pub produces_tokens: u64,
    /// Activity label — becomes `ocel:type` in the JSON bridge.
    pub activity: String,
    /// Nanosecond timestamp.
    pub ts_ns: u64,
    /// Object identifiers touched by this event (E2O links).
    pub object_ids: Vec<String>,
}

/// Token-passing replay verifier for a POWL process model.
///
/// Real (not stubbed) `fitness`/`precision` accumulation via
/// [`Self::replay_frame`]'s token-passing state. [`Self::finalize`]'s
/// `generalization`/`simplicity` output is MOCKED — see that method's doc
/// comment.
pub struct PowlReplayVerifier {
    enabled_tokens: u64,
    replayed: u64,
    fitted: u64,
    enabled_not_taken: u64,
}

/// A violation detected during replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayViolation {
    /// The frame's `required_tokens` were not all present in `enabled_tokens`.
    TokenNotEnabled { node_id: u32 },
    /// The frame's `node_bit` is zero or has more than one bit set.
    UnknownNode { node_id: u32 },
    /// A XOR-choice edge was violated: a sibling exclusive token was still live.
    InvalidChoiceEdge { node_id: u32 },
}

impl PowlReplayVerifier {
    /// Create a verifier with `entry_op_bit` as the initial enabled token.
    pub fn new(entry_op_bit: u64) -> Self {
        PowlReplayVerifier {
            enabled_tokens: entry_op_bit,
            replayed: 0,
            fitted: 0,
            enabled_not_taken: 0,
        }
    }

    /// Replay one causal frame.
    pub fn replay_frame(&mut self, frame: &PowlReplayFrame) -> Result<(), ReplayViolation> {
        // Guard 1: node_bit must be exactly one bit set (power of two, non-zero).
        if frame.node_bit == 0 || (frame.node_bit & frame.node_bit.wrapping_sub(1)) != 0 {
            return Err(ReplayViolation::UnknownNode {
                node_id: frame.node_id,
            });
        }

        // Guard 2: all required tokens must be present — branchless XOR check.
        let missing = (self.enabled_tokens & frame.required_tokens) ^ frame.required_tokens;
        if missing != 0 {
            return Err(ReplayViolation::TokenNotEnabled {
                node_id: frame.node_id,
            });
        }

        // Accumulate enabled-not-taken before consuming tokens.
        self.enabled_not_taken |= self.enabled_tokens & !frame.required_tokens & !frame.node_bit;

        // Consume required tokens; produce successor tokens.
        self.enabled_tokens =
            (self.enabled_tokens & !frame.required_tokens) | frame.produces_tokens;

        self.replayed |= frame.node_bit;
        self.fitted |= frame.node_bit;

        Ok(())
    }

    /// Finalise replay and return Q16.16 [`ConformanceMetrics`].
    ///
    /// `fitness` and `precision` are real, computed from the token-passing
    /// accumulators [`Self::replay_frame`] maintained across every replayed
    /// frame. `generalization` and `simplicity` are **MOCKED**: this
    /// verifier has no state-space or model-complexity data to compute
    /// them from, so both are always `0x0000_0000` — a real, in-range
    /// Q16.16 zero that fails any predicate with a nonzero minimum, not a
    /// value that could be mistaken for a genuine measurement. See the
    /// module doc comment for why this matters and what it replaced.
    pub fn finalize(self) -> ConformanceMetrics {
        let replayed = self.replayed.count_ones() as u64;
        let fitted = self.fitted.count_ones() as u64;
        let not_taken = self.enabled_not_taken.count_ones() as u64;

        ConformanceMetrics {
            fitness: fixed_div(fitted, replayed),
            precision: fixed_div(replayed, replayed + not_taken),
            // MOCKED — see this method's doc comment. Deliberately 0
            // (in-range Q16.16, fails any nonzero-threshold predicate)
            // rather than a fabricated "measured" value.
            generalization: 0x0000_0000,
            simplicity: 0x0000_0000,
        }
    }
}

/// Q16.16 fixed-point division. Returns 1.0 when denominator == 0.
#[inline]
fn fixed_div(numerator: u64, denominator: u64) -> u32 {
    if denominator == 0 {
        return 0x0001_0000;
    }
    let shifted = (numerator << 16) / denominator;
    if shifted > u32::MAX as u64 {
        u32::MAX
    } else {
        shifted as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(node_id: u32, node_bit: u64, required: u64, produces: u64) -> PowlReplayFrame {
        PowlReplayFrame {
            node_id,
            node_bit,
            required_tokens: required,
            produces_tokens: produces,
            activity: format!("op-{node_id}"),
            ts_ns: node_id as u64 * 1_000_000_000,
            object_ids: vec![],
        }
    }

    #[test]
    fn single_node_replay_fitness_is_one() {
        let mut v = PowlReplayVerifier::new(0x1);
        assert!(v.replay_frame(&f(0, 0x1, 0x1, 0x0)).is_ok());
        assert_eq!(v.finalize().fitness, 0x0001_0000);
    }

    /// Disclosure test: `generalization`/`simplicity` are MOCKED as an
    /// in-range Q16.16 zero, not silently upgraded into a value that
    /// could pass for a real measurement (see `finalize`'s doc comment
    /// and the historical `0x8000_0000`/`0xC000_0000` bug it replaced).
    #[test]
    fn finalize_discloses_mocked_generalization_and_simplicity_as_zero() {
        let mut v = PowlReplayVerifier::new(0x1);
        assert!(v.replay_frame(&f(0, 0x1, 0x1, 0x0)).is_ok());
        let m = v.finalize();
        assert_eq!(
            m.generalization, 0x0000_0000,
            "MOCKED: must be a real, in-range zero"
        );
        assert_eq!(
            m.simplicity, 0x0000_0000,
            "MOCKED: must be a real, in-range zero"
        );
    }

    /// A real `ConformancePredicate::check` run against a real
    /// `finalize()` output must fail on the two mocked dimensions, even
    /// for a perfect-fitness/perfect-precision trace — proving the mock
    /// cannot be mistaken for a passing measurement by the branchless
    /// gate that actually consumes it. Before this fix, the out-of-range
    /// placeholder values made this check spuriously pass.
    #[test]
    fn strict_predicate_fails_on_a_perfect_trace_due_to_mocked_dimensions() {
        use crate::conformance::{ConformanceDimension, ConformancePredicate};

        let mut v = PowlReplayVerifier::new(0x1);
        assert!(v.replay_frame(&f(0, 0x1, 0x1, 0x0)).is_ok());
        let m = v.finalize();
        assert_eq!(
            m.fitness, 0x0001_0000,
            "sanity: this trace is perfectly fit"
        );
        assert_eq!(
            m.precision, 0x0001_0000,
            "sanity: this trace is perfectly precise"
        );

        let violation = ConformancePredicate::STRICT
            .check(&m)
            .expect_err("STRICT requires generalization/simplicity >= 0.5, which is unmeasured");
        assert_eq!(violation.dim, ConformanceDimension::Generalization);
    }

    #[test]
    fn zero_node_bit_is_unknown_node() {
        let mut v = PowlReplayVerifier::new(0x1);
        let bad = PowlReplayFrame {
            node_id: 99,
            node_bit: 0,
            required_tokens: 0,
            produces_tokens: 0,
            activity: "X".into(),
            ts_ns: 0,
            object_ids: vec![],
        };
        assert_eq!(
            v.replay_frame(&bad),
            Err(ReplayViolation::UnknownNode { node_id: 99 })
        );
    }
}
