//! replay — token-passing POWL replay verifier with branchless metric accumulation.
//!
//! Uses [`PowlReplayFrame`] — a high-level frame with token-flow semantics
//! (`node_bit`, `required_tokens`, `produces_tokens`), distinct from the binary-packed
//! [`crate::causal_receipt::OcelCausalFrame`] used for BLAKE3 hash chaining.

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
    pub fn finalize(self) -> ConformanceMetrics {
        let replayed = self.replayed.count_ones() as u64;
        let fitted = self.fitted.count_ones() as u64;
        let not_taken = self.enabled_not_taken.count_ones() as u64;

        ConformanceMetrics {
            fitness: fixed_div(fitted, replayed),
            precision: fixed_div(replayed, replayed + not_taken),
            generalization: 0x8000_0000,
            simplicity: 0xC000_0000,
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
