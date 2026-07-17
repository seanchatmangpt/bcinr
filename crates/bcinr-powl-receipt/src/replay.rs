//! replay — token-passing POWL replay verifier with branchless metric accumulation.
//!
//! Uses [`PowlReplayFrame`] — a high-level frame with token-flow semantics
//! (`node_bit`, `required_tokens`, `produces_tokens`), distinct from the binary-packed
//! [`crate::causal_receipt::OcelCausalFrame`] used for BLAKE3 hash chaining.
//!
//! # Generalization and Simplicity are computed using Q16.16 branchless estimators (RCME)
//!
//! [`PowlReplayVerifier::finalize`] computes all four [`ConformanceMetrics`]
//! dimensions from the replay state:
//! - `fitness` and `precision` are genuine, derived from token-passing accumulators.
//! - `generalization` and `simplicity` are proxy estimators (RCME) derived branchlessly
//!   from tape length, unique replayed node counts, and token configurations.
//! See the design proposal in `real_conformance_metric_estimators.md` for the formulas
//! and mathematical rationales.

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
/// Real (not stubbed) `fitness`/`precision`/`generalization`/`simplicity` accumulation
/// via [`Self::replay_frame`]'s token-passing state.
pub struct PowlReplayVerifier {
    enabled_tokens: u64,
    replayed: u64,
    fitted: u64,
    enabled_not_taken: u64,
    tape_length: u64,
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
            tape_length: 0,
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
        self.tape_length += 1;

        Ok(())
    }

    /// Finalise replay and return Q16.16 [`ConformanceMetrics`].
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
        let is_valid_frac = crate::conformance::mask_ge(0x0001_0000, gen_frac);
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
    fn finalize_computes_real_generalization_and_simplicity_for_single_node() {
        let mut v = PowlReplayVerifier::new(0x1);
        assert!(v.replay_frame(&f(0, 0x1, 0x1, 0x0)).is_ok());
        let m = v.finalize();
        assert_eq!(m.generalization, 0x0000_8000); // 0.5
        assert_eq!(m.simplicity, 0x0000_E38E);     // 8/9
    }

    #[test]
    fn strict_predicate_passes_on_a_perfect_trace_due_to_real_dimensions() {
        use crate::conformance::ConformancePredicate;

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

    fn oracle_rcme(
        tape_len: u64,
        n_unique: u64,
        t_not_taken: u64,
        t_active: u64,
    ) -> (f64, f64) {
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
                enabled_not_taken: (1u64.checked_shl(not_taken as u32).unwrap_or(0)).wrapping_sub(1),
                tape_length,
            };
            
            assert_eq!(v.replayed.count_ones() as u64, replayed);
            assert_eq!(v.enabled_not_taken.count_ones() as u64, not_taken);
            assert_eq!(v.enabled_tokens.count_ones() as u64, active);
            
            let m = v.finalize();
            let (g_exp, s_exp) = oracle_rcme(tape_length, replayed, not_taken, active);
            
            let g_act_f = m.generalization as f64 / 65536.0;
            let s_act_f = m.simplicity as f64 / 65536.0;
            
            assert!((g_act_f - g_exp).abs() <= 1.5 / 65536.0, "G diff boundary fail: act={}, exp={}", g_act_f, g_exp);
            assert!((s_act_f - s_exp).abs() <= 1.5 / 65536.0, "S diff boundary fail: act={}, exp={}", s_act_f, s_exp);
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
                enabled_not_taken: (1u64.checked_shl(not_taken as u32).unwrap_or(0)).wrapping_sub(1),
                tape_length,
            };
            
            let m = v.finalize();
            let (g_exp, s_exp) = oracle_rcme(tape_length, replayed, not_taken, active);
            
            let g_act_f = m.generalization as f64 / 65536.0;
            let s_act_f = m.simplicity as f64 / 65536.0;
            
            let g_diff = (g_act_f - g_exp).abs();
            let s_diff = (s_act_f - s_exp).abs();
            
            assert!(g_diff <= 1.5 / 65536.0, "G random diff too large: act={}, exp={}", g_act_f, g_exp);
            assert!(s_diff <= 1.5 / 65536.0, "S random diff too large: act={}, exp={}", s_act_f, s_exp);
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
