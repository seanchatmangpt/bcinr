//! Token-passing POWL replay verifier with branchless metric accumulation.
//!
//! This module provides [`PowlReplayVerifier`] and [`PowlReplayFrame`] to simulate
//! token-flow semantics on a Partially Ordered Workflow Language (POWL) structure
//! and derive conformance metrics.
//!
//! # Replay Semantics
//!
//! Replay is modeled as a token-passing game over a Petri-net like graph. Each transition
//! (node) in the POWL process structure requires a specific set of tokens to be enabled,
//! and when fired, consumes those tokens and produces new successor tokens.
//!
//! - **Required Tokens**: The set of tokens that must be present in the verifier's marking
//!   for the node to fire.
//! - **Produced Tokens**: The set of tokens generated and added to the verifier's marking
//!   after the node fires.
//! - **Enabled not Taken**: Enabled transitions that were present but not chosen/fired
//!   during the trace.
//!
//! # Mathematical Definitions for Metrics
//!
//! Upon completion of the trace, [`PowlReplayVerifier::finalize`] calculates the final metrics:
//!
//! 1. **Fitness ($F$):**
//!    $$F = \frac{\text{Fitted Nodes}}{\text{Replayed Nodes}}$$
//!    Here, both variables are derived from the unique transitions successfully replayed. For a fully fitting trace,
//!    this is $1.0$ (`0x0001_0000`).
//!
//! 2. **Precision ($P$):**
//!    $$P = \frac{N_{\text{unique}}}{N_{\text{unique}} + T_{\text{not\_taken}}}$$
//!    where $N_{\text{unique}}$ is the count of unique replayed nodes (the population of unique transitions observed in the trace),
//!    and $T_{\text{not\_taken}}$ is the total number of options enabled but never taken throughout the replay trace.
//!
//! 3. **Generalization ($G$):**
//!    $$G = 1.0 - \text{clamp}\left(\frac{N_{\text{unique}}}{L + T_{\text{not\_taken}} + 1}, 0.0, 1.0\right)$$
//!    where $L$ is the tape length (the number of events processed). If the model only allows the exact observed sequence,
//!    generalization is low. If it allows flexible alternatives, generalization scales appropriately.
//!
//! 4. **Simplicity ($S$):**
//!    $$S = \frac{K}{N_{\text{unique}} + T_{\text{not\_taken}} + T_{\text{active}} + K}$$
//!    where $K = 8$ is a scaling constant, and $T_{\text{active}}$ is the count of active tokens remaining in the model marking at the end of execution.
//!
//! # Examples
//!
//! ```
//! use bcinr_powl::receipt::replay::{PowlReplayVerifier, PowlReplayFrame};
//!
//! // Define a simple replay frame representing the first event
//! let frame = PowlReplayFrame {
//!     node_id: 1,
//!     node_bit: 0x1,
//!     required_tokens: 0x1,
//!     produces_tokens: 0x2,
//!     activity: "Start".to_string(),
//!     ts_ns: 1000,
//!     object_ids: vec![],
//! };
//!
//! // Create a verifier initialized with entry token 0x1
//! let mut verifier = PowlReplayVerifier::new(0x1);
//!
//! // Replay the frame
//! assert!(verifier.replay_frame(&frame).is_ok());
//!
//! // Finalize the metrics
//! let metrics = verifier.finalize();
//! assert_eq!(metrics.fitness, 0x0001_0000); // 1.0
//! ```

use crate::receipt::conformance::ConformanceMetrics;

/// High-level replay descriptor for one POWL node firing.
///
/// This struct holds the token-flow semantics of a single event in the process log
/// and is used by [`PowlReplayVerifier`] to progress the replay. It is distinct from
/// [`crate::receipt::causal_receipt::OcelCausalFrame`], which is the binary-packed structure
/// used for cryptographic chaining.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowlReplayFrame {
    /// Unique node identifier (primarily for error reporting and debugging).
    pub node_id: u32,
    /// 1-hot bitmask position for this node (must have exactly one bit set).
    pub node_bit: u64,
    /// Bitmask of tokens that must all be present for this node to fire.
    pub required_tokens: u64,
    /// Tokens this firing produces (post-set); 0 for sink nodes.
    pub produces_tokens: u64,
    /// Activity label — becomes `ocel:type` in the JSON bridge.
    pub activity: String,
    /// Nanosecond timestamp of the event.
    pub ts_ns: u64,
    /// Object identifiers touched by this event (E2O links).
    pub object_ids: Vec<String>,
}

/// Token-passing replay verifier for a POWL process model.
///
/// Accumulates fitness, precision, generalization, and simplicity metrics during trace replay.
/// It maintains the active marking (`enabled_tokens`) and counts unique visited nodes,
/// tape length, and enabled options that were not taken.
pub struct PowlReplayVerifier {
    /// Current marking (bitmask of active/enabled tokens).
    enabled_tokens: u64,
    /// Bitmask of all unique replayed transitions.
    replayed: u64,
    /// Bitmask of transitions that successfully fit the replay semantics.
    fitted: u64,
    /// Bitmask of all options enabled during the replay but never chosen/fired.
    enabled_not_taken: u64,
    /// Total number of events (replay frames) replayed.
    tape_length: u64,
}

/// A violation detected during replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayViolation {
    /// The frame's `required_tokens` were not all present in `enabled_tokens`.
    TokenNotEnabled {
        /// The identifier of the node that failed to fire.
        node_id: u32,
    },
    /// The frame's `node_bit` is zero or has more than one bit set.
    UnknownNode {
        /// The identifier of the node with the invalid bitmask.
        node_id: u32,
    },
    /// A XOR-choice edge was violated: a sibling exclusive token was still live.
    InvalidChoiceEdge {
        /// The identifier of the node associated with the choice violation.
        node_id: u32,
    },
}

impl PowlReplayVerifier {
    /// Create a new verifier with `entry_op_bit` as the initial enabled token.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::receipt::replay::PowlReplayVerifier;
    ///
    /// let verifier = PowlReplayVerifier::new(0x1);
    /// ```
    pub fn new(entry_op_bit: u64) -> Self {
        PowlReplayVerifier {
            enabled_tokens: entry_op_bit,
            replayed: 0,
            fitted: 0,
            enabled_not_taken: 0,
            tape_length: 0,
        }
    }

    /// Replay one causal frame.
    ///
    /// Updates the active token marking and metric accumulators.
    /// Returns `Ok(())` if the transition can be fired legally under the current marking.
    /// Otherwise, returns a [`ReplayViolation`].
    ///
    /// # Errors
    ///
    /// - [`ReplayViolation::UnknownNode`]: If `frame.node_bit` is not a power of 2.
    /// - [`ReplayViolation::TokenNotEnabled`]: If not all `frame.required_tokens` are present.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::receipt::replay::{PowlReplayVerifier, PowlReplayFrame};
    ///
    /// let mut verifier = PowlReplayVerifier::new(0x1);
    /// let frame = PowlReplayFrame {
    ///     node_id: 0,
    ///     node_bit: 0x1,
    ///     required_tokens: 0x1,
    ///     produces_tokens: 0x2,
    ///     activity: "A".to_string(),
    ///     ts_ns: 0,
    ///     object_ids: vec![],
    /// };
    /// assert!(verifier.replay_frame(&frame).is_ok());
    /// ```
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
        self.tape_length += 1;

        Ok(())
    }

    /// Finalise replay and return Q16.16 [`ConformanceMetrics`].
    ///
    /// Computes the final fitness, precision, generalization, and simplicity values.
    /// Computations are branchless, utilizing Q16.16 division and bitwise clamping masks.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::receipt::replay::PowlReplayVerifier;
    ///
    /// let verifier = PowlReplayVerifier::new(0x1);
    /// let metrics = verifier.finalize();
    /// // Fitness and precision default to 1.0 when no frames are replayed.
    /// assert_eq!(metrics.fitness, 0x0001_0000);
    /// assert_eq!(metrics.precision, 0x0001_0000);
    /// ```
    pub fn finalize(self) -> ConformanceMetrics {
        let replayed = self.replayed.count_ones() as u64;
        let fitted = self.fitted.count_ones() as u64;
        let not_taken = self.enabled_not_taken.count_ones() as u64;
        let active = self.enabled_tokens.count_ones() as u64;

        let fitness = fixed_div(fitted, replayed);
        let precision = fixed_div(replayed, replayed + not_taken);

        // --- RCME Calculations ---

        // G = 1.0 - (N_unique / (L + T_not_taken + 1))
        let gen_num = replayed;
        let gen_den = self.tape_length + not_taken + 1;
        let gen_frac = fixed_div(gen_num, gen_den);

        // Branchless clamp to [0, 1.0] using mask_ge
        let is_valid_frac = crate::receipt::conformance::mask_ge(0x0001_0000, gen_frac);
        let gen_frac_clamped = (is_valid_frac & gen_frac) | (!is_valid_frac & 0x0001_0000);
        let generalization = 0x0001_0000 - gen_frac_clamped;

        // S = K / (N_unique + T_not_taken + T_active + K)
        let k = 8u64;
        let simplicity = fixed_div(k, replayed + not_taken + active + k);

        ConformanceMetrics {
            fitness,
            precision,
            generalization,
            simplicity,
        }
    }
}

/// Q16.16 fixed-point division.
///
/// Returns `0x0001_0000` (1.0) when denominator == 0.
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
    fn finalize_computes_real_generalization_and_simplicity_for_single_node() {
        let mut v = PowlReplayVerifier::new(0x1);
        assert!(v.replay_frame(&f(0, 0x1, 0x1, 0x0)).is_ok());
        let m = v.finalize();
        assert_eq!(m.generalization, 0x0000_8000); // 0.5
        assert_eq!(m.simplicity, 0x0000_E38E); // 8/9
    }

    #[test]
    fn strict_predicate_passes_on_a_perfect_trace_due_to_real_dimensions() {
        use crate::receipt::conformance::ConformancePredicate;

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
        assert!(ConformancePredicate::STRICT.check(&m).is_ok());
    }

    fn oracle_rcme(tape_len: u64, n_unique: u64, t_not_taken: u64, t_active: u64) -> (f64, f64) {
        let g = 1.0 - (n_unique as f64 / (tape_len + t_not_taken + 1) as f64);
        let g_clamped = g.clamp(0.0, 1.0);

        let k = 8.0;
        let s = k / (n_unique + t_not_taken + t_active + 8) as f64;

        (g_clamped, s)
    }

    struct Lcg {
        state: u64,
    }
    impl Lcg {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }
        fn next_u64(&mut self) -> u64 {
            self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
            self.state
        }
        fn next_range(&mut self, min: u64, max: u64) -> u64 {
            if min >= max {
                return min;
            }
            min + (self.next_u64() % (max - min + 1))
        }
    }

    #[test]
    fn test_rcme_differential_oracle() {
        let mut lcg = Lcg::new(42);

        // 1. Boundary cases
        let boundaries = [
            // L, N_unique, T_not_taken, T_active
            (0, 0, 0, 0),
            (0, 0, 64, 64),
            (100, 0, 0, 0),
            (100, 64, 0, 0),
            (100, 50, 64, 64),
            (10000, 64, 64, 64),
        ];

        for &(tape_length, replayed, not_taken, active) in &boundaries {
            let v = PowlReplayVerifier {
                enabled_tokens: (1u64.checked_shl(active as u32).unwrap_or(0)).wrapping_sub(1),
                replayed: (1u64.checked_shl(replayed as u32).unwrap_or(0)).wrapping_sub(1),
                fitted: (1u64.checked_shl(replayed as u32).unwrap_or(0)).wrapping_sub(1),
                enabled_not_taken: (1u64.checked_shl(not_taken as u32).unwrap_or(0))
                    .wrapping_sub(1),
                tape_length,
            };

            assert_eq!(v.replayed.count_ones() as u64, replayed);
            assert_eq!(v.enabled_not_taken.count_ones() as u64, not_taken);
            assert_eq!(v.enabled_tokens.count_ones() as u64, active);

            let m = v.finalize();
            let (g_exp, s_exp) = oracle_rcme(tape_length, replayed, not_taken, active);

            let g_act_f = m.generalization as f64 / 65536.0;
            let s_act_f = m.simplicity as f64 / 65536.0;

            assert!(
                (g_act_f - g_exp).abs() <= 1.5 / 65536.0,
                "G diff boundary fail: act={}, exp={}",
                g_act_f,
                g_exp
            );
            assert!(
                (s_act_f - s_exp).abs() <= 1.5 / 65536.0,
                "S diff boundary fail: act={}, exp={}",
                s_act_f,
                s_exp
            );
        }

        // 2. 50,000 random runs
        for _ in 0..50000 {
            let tape_length = lcg.next_range(0, 10000);
            let replayed = lcg.next_range(0, tape_length.min(64));
            let not_taken = lcg.next_range(0, 64);
            let active = lcg.next_range(0, 64);

            let v = PowlReplayVerifier {
                enabled_tokens: (1u64.checked_shl(active as u32).unwrap_or(0)).wrapping_sub(1),
                replayed: (1u64.checked_shl(replayed as u32).unwrap_or(0)).wrapping_sub(1),
                fitted: (1u64.checked_shl(replayed as u32).unwrap_or(0)).wrapping_sub(1),
                enabled_not_taken: (1u64.checked_shl(not_taken as u32).unwrap_or(0))
                    .wrapping_sub(1),
                tape_length,
            };

            let m = v.finalize();
            let (g_exp, s_exp) = oracle_rcme(tape_length, replayed, not_taken, active);

            let g_act_f = m.generalization as f64 / 65536.0;
            let s_act_f = m.simplicity as f64 / 65536.0;

            let g_diff = (g_act_f - g_exp).abs();
            let s_diff = (s_act_f - s_exp).abs();

            assert!(
                g_diff <= 1.5 / 65536.0,
                "G random diff too large: act={}, exp={}",
                g_act_f,
                g_exp
            );
            assert!(
                s_diff <= 1.5 / 65536.0,
                "S random diff too large: act={}, exp={}",
                s_act_f,
                s_exp
            );
        }
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
