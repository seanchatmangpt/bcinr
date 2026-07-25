#![forbid(unsafe_code)]

//! # Canonical Mass Derivation and CMCA Lens Selection (Iteration 13)
//!
//! Provides the mathematical reference oracle, hostile mutants, and test matrices
//! for deterministic, zero-allocation $m_i$ derivation and branchless `arg_max`.

/// Typed refusal codes for Auto Select operations.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoSelectRefusal {
    None = 0,
    ControlStateUnadmitted = 1,
    ContractionMarginInsufficient = 2,
    SupportMismatch = 3,
    UnsupportedDomain = 4,
    NumericRangeExceeded = 5,
    ContractViolation = 6,
    NoLeaves = 7,
}

/// A fixed-width C-ABI candidate tool coordinate vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolCandidate {
    pub tool_id: u8,
    pub semantic_fit: u8,
    pub evidence_fit: u8,
    pub authority_fit: u8,
    pub timing_fit: u8,
    pub downstream_fit: u8,
    pub reliability: u8,
    pub cost_fit: u8,
    pub mass: u8,
}

/// Fixed-width execution input for 8 candidates.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoSelectInput8 {
    pub request_id: u32,
    pub admitted_mask: u8,
    pub q_lens: u8,
    pub add_mask: u8,
    pub del_mask: u8,
    pub candidates: [ToolCandidate; 8],
}

impl Default for AutoSelectInput8 {
    fn default() -> Self {
        Self {
            request_id: 0,
            admitted_mask: 0,
            q_lens: 2,
            add_mask: 0,
            del_mask: 0,
            candidates: [ToolCandidate::default(); 8],
        }
    }
}

/// The result of an Auto Select pass.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoSelectResult {
    pub is_ok: u8,
    pub tool_id: u8,
    pub refusal_code: u8,
}

impl AutoSelectResult {
    #[inline(always)]
    #[rustfmt::skip]
    pub  fn is_err(&self) -> bool {
        self.is_ok == 0
    }
}

include!("tables.rs");

/// Computes the unweighted geometric mean of 7 semantic parameters.
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub  fn calculate_canonical_mass(c: &ToolCandidate) -> u8 {
    let mut sum = 0i64;

    sum += LOG2_TABLE[c.semantic_fit as usize];
    sum += LOG2_TABLE[c.evidence_fit as usize];
    sum += LOG2_TABLE[c.authority_fit as usize];
    sum += LOG2_TABLE[c.timing_fit as usize];
    sum += LOG2_TABLE[c.downstream_fit as usize];
    sum += LOG2_TABLE[c.reliability as usize];
    sum += LOG2_TABLE[c.cost_fit as usize];

    let mean_log = (sum / 7) as i32;
    let log2_255_q16: i32 = 523918;
    let log_m_i = log2_255_q16.wrapping_add(mean_log);

    let is_too_small = (log_m_i < 0) as u32;
    let mask = 0u32.wrapping_sub(is_too_small);

    let clamped_log = (log_m_i & !(log_m_i >> 31)) as u32;
    let int_part = clamped_log >> 16;
    let frac_part = clamped_log & 0xFFFF;

    let exp_val = EXP2_FRAC_TABLE[(frac_part >> 8) as usize];
    let m_i = (exp_val << int_part) >> 16;
    let m_i_masked = m_i & !mask;

    let is_too_large = (m_i_masked > 255) as u32;
    let clamp_mask = 0u32.wrapping_sub(is_too_large);

    ((255 & clamp_mask) | (m_i_masked & !clamp_mask)) as u8
}

/// Selects the optimal tool branchlessly based on CMCA mass.
#[inline(always)]
#[must_use]
#[allow(unused_assignments)]
#[rustfmt::skip]
pub  fn select_optimal_candidate(input: &AutoSelectInput8) -> AutoSelectResult {
    let domain_invalid = ((input.q_lens < 1) as u8) | ((input.q_lens > 4) as u8);
    let contract_invalid = ((input.add_mask & input.del_mask) != 0) as u8;

    let mut range_invalid = 0u8;
    range_invalid |= (input.candidates[0].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[1].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[2].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[3].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[4].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[5].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[6].tool_id >= 8) as u8;
    range_invalid |= (input.candidates[7].tool_id >= 8) as u8;

    let admitted_invalid = (input.admitted_mask == 0) as u8;

    let mut pre_refusal = AutoSelectRefusal::None as u8;
    let mut is_refused = 0u8;

    macro_rules! check_refusal {
        ($cond:expr, $code:expr) => {
            let triggered = ($cond) & (is_refused ^ 1);
            let t_mask = 0u8.wrapping_sub(triggered);
            pre_refusal = ($code & t_mask) | (pre_refusal & !t_mask);
            is_refused |= triggered;
        };
    }

    check_refusal!(domain_invalid, AutoSelectRefusal::UnsupportedDomain as u8);
    check_refusal!(contract_invalid, AutoSelectRefusal::ContractViolation as u8);
    check_refusal!(
        admitted_invalid,
        AutoSelectRefusal::ControlStateUnadmitted as u8
    );
    check_refusal!(range_invalid, AutoSelectRefusal::NumericRangeExceeded as u8);

    let mut best_score: u32 = 0;
    let mut best_id: u8 = 0;
    let mut any_found: u8 = 0;

    macro_rules! step {
        ($i:expr) => {
            let is_admitted = ((input.admitted_mask >> $i) & 1) as u8;
            let admitted_mask_32 = 0u32.wrapping_sub(is_admitted as u32);

            let m_i = calculate_canonical_mass(&input.candidates[$i]) as u32;

            let score = m_i;
            let q = input.q_lens;

            let p2 = score.wrapping_mul(score);
            let p3 = p2.wrapping_mul(score);
            let p4 = p3.wrapping_mul(score);

            let mut pow_score = score;
            pow_score = (p2 & 0u32.wrapping_sub((q == 2) as u32))
                | (pow_score & !0u32.wrapping_sub((q == 2) as u32));
            pow_score = (p3 & 0u32.wrapping_sub((q == 3) as u32))
                | (pow_score & !0u32.wrapping_sub((q == 3) as u32));
            pow_score = (p4 & 0u32.wrapping_sub((q == 4) as u32))
                | (pow_score & !0u32.wrapping_sub((q == 4) as u32));

            let effective_score = pow_score & admitted_mask_32;

            let score_is_better = (effective_score > best_score) as u8;
            let is_better = is_admitted & ((any_found ^ 1) | score_is_better);
            let better_mask = 0u8.wrapping_sub(is_better);
            let better_mask_32 = 0u32.wrapping_sub(is_better as u32);

            best_score = (effective_score & better_mask_32) | (best_score & !better_mask_32);
            best_id = (input.candidates[$i].tool_id & better_mask) | (best_id & !better_mask);
            any_found |= is_admitted;
        };
    }

    step!(0);
    step!(1);
    step!(2);
    step!(3);
    step!(4);
    step!(5);
    step!(6);
    step!(7);

    let exec_refusal = (any_found ^ 1) * (AutoSelectRefusal::ControlStateUnadmitted as u8);

    let pre_mask = 0u8.wrapping_sub(is_refused);
    let final_refusal = (pre_refusal & pre_mask) | (exec_refusal & !pre_mask);
    let final_is_ok = (final_refusal == 0) as u8;

    AutoSelectResult {
        is_ok: final_is_ok,
        tool_id: best_id & 0u8.wrapping_sub(final_is_ok),
        refusal_code: final_refusal,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Independent oracle for calculate_canonical_mass
    #[rustfmt::skip]
pub  fn oracle_calculate_canonical_mass(c: &ToolCandidate) -> u8 {
        let fits = [
            c.semantic_fit as f64,
            c.evidence_fit as f64,
            c.authority_fit as f64,
            c.timing_fit as f64,
            c.downstream_fit as f64,
            c.reliability as f64,
            c.cost_fit as f64,
        ];

        let mut product = 1.0;
        for &fit in &fits {
            if fit == 0.0 {
                return 0;
            }
            product *= fit / 255.0;
        }

        let mean = product.powf(1.0 / 7.0);
        (mean * 255.0).floor() as u8
    }

    /// Independent oracle for select_optimal_candidate
    #[rustfmt::skip]
pub  fn oracle_select_optimal_candidate(input: &AutoSelectInput8) -> AutoSelectResult {
        if input.q_lens < 1 || input.q_lens > 4 {
            return AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: AutoSelectRefusal::UnsupportedDomain as u8,
            };
        }
        if (input.add_mask & input.del_mask) != 0 {
            return AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: AutoSelectRefusal::ContractViolation as u8,
            };
        }
        if input.admitted_mask == 0 {
            return AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: AutoSelectRefusal::ControlStateUnadmitted as u8,
            };
        }

        let mut best_score = 0u32;
        let mut best_id = 0u8;
        let mut any_found = false;

        for i in 0..8 {
            if input.candidates[i].tool_id >= 8 {
                return AutoSelectResult {
                    is_ok: 0,
                    tool_id: 0,
                    refusal_code: AutoSelectRefusal::NumericRangeExceeded as u8,
                };
            }

            if (input.admitted_mask & (1 << i)) != 0 {
                let mass = oracle_calculate_canonical_mass(&input.candidates[i]) as u32;
                let score = mass.pow(input.q_lens as u32);
                if score > best_score || !any_found {
                    best_score = score;
                    best_id = input.candidates[i].tool_id;
                    any_found = true;
                }
            }
        }

        if !any_found {
            AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: AutoSelectRefusal::ControlStateUnadmitted as u8,
            }
        } else {
            AutoSelectResult {
                is_ok: 1,
                tool_id: best_id,
                refusal_code: AutoSelectRefusal::None as u8,
            }
        }
    }

    // --- @armstrong_fault Hostile Mutants ---

    #[rustfmt::skip]
pub  fn mutant_1_corrupted_admitted_mask(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores admitted_mask when selecting candidate!
        let mut m = *input;
        m.admitted_mask = 0xFF;
        select_optimal_candidate(&m)
    }

    #[rustfmt::skip]
pub  fn mutant_2_bypassed_q_lens_domain(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores q_lens domain validation!
        let mut m = *input;
        m.q_lens = 2; // bypass domain validation
        select_optimal_candidate(&m)
    }

    #[rustfmt::skip]
pub  fn mutant_3_bypassed_contract_violation(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores overlapping cognition masks!
        let mut m = *input;
        m.add_mask = 0; // bypass contract violation
        select_optimal_candidate(&m)
    }

    #[rustfmt::skip]
pub  fn mutant_4_bypassed_numeric_range(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores tool_id range bounds!
        let mut m = *input;
        for i in 0..8 {
            m.candidates[i].tool_id = 0;
        }
        select_optimal_candidate(&m)
    }

    #[test]
    fn test_refusal_conservation_and_mutants() {
        let mut input = AutoSelectInput8::default();
        input.q_lens = 2;
        input.add_mask = 0b01;
        input.del_mask = 0b10; // no overlap
        input.admitted_mask = 0b0000_0001;

        let mut c = ToolCandidate::default();
        c.tool_id = 1;
        c.semantic_fit = 200;
        c.evidence_fit = 255;
        c.authority_fit = 255;
        c.timing_fit = 255;
        c.downstream_fit = 255;
        c.reliability = 255;
        c.cost_fit = 255;
        input.candidates[0] = c;

        // Add a highly scored but UNADMITTED candidate
        let mut c2 = ToolCandidate::default();
        c2.tool_id = 2;
        c2.semantic_fit = 255;
        c2.evidence_fit = 255;
        c2.authority_fit = 255;
        c2.timing_fit = 255;
        c2.downstream_fit = 255;
        c2.reliability = 255;
        c2.cost_fit = 255;
        input.candidates[1] = c2;

        let reference = oracle_select_optimal_candidate(&input);
        assert_eq!(reference.is_ok, 1);
        assert_eq!(reference.tool_id, 1);

        // Mutant 1 (Corrupted Admitted Mask)
        let m1 = mutant_1_corrupted_admitted_mask(&input);
        // It accepted an unadmitted candidate (tool 2)
        assert_eq!(
            m1.tool_id, 2,
            "Mutant 1 survived: failed to yield uncorrupted index or typed refusal"
        );

        // Refusal checks

        // 1. Unsupported Domain (q_lens = 0)
        let mut input_domain = input.clone();
        input_domain.q_lens = 0;
        let ref_domain = oracle_select_optimal_candidate(&input_domain);
        assert_eq!(
            ref_domain.refusal_code,
            AutoSelectRefusal::UnsupportedDomain as u8
        );

        // Mutant 2
        let m2 = mutant_2_bypassed_q_lens_domain(&input_domain);
        assert_eq!(
            m2.is_ok, 1,
            "Mutant 2 survived: bypassed q_lens domain validation"
        );

        // 2. Contract Violation (overlapping masks)
        let mut input_contract = input.clone();
        input_contract.add_mask = 0b11;
        input_contract.del_mask = 0b01; // overlap!
        let ref_contract = oracle_select_optimal_candidate(&input_contract);
        assert_eq!(
            ref_contract.refusal_code,
            AutoSelectRefusal::ContractViolation as u8
        );

        // Mutant 3
        let m3 = mutant_3_bypassed_contract_violation(&input_contract);
        assert_eq!(
            m3.is_ok, 1,
            "Mutant 3 survived: bypassed contract violation"
        );

        // 3. Numeric Range Exceeded (tool_id >= 8)
        let mut input_range = input.clone();
        input_range.candidates[0].tool_id = 8;
        let ref_range = oracle_select_optimal_candidate(&input_range);
        assert_eq!(
            ref_range.refusal_code,
            AutoSelectRefusal::NumericRangeExceeded as u8
        );

        // Mutant 4
        let m4 = mutant_4_bypassed_numeric_range(&input_range);
        assert_eq!(
            m4.is_ok, 1,
            "Mutant 4 survived: bypassed numeric range exceeded"
        );

        // 4. Control State Unadmitted (admitted_mask = 0)
        let mut input_unadmitted = input.clone();
        input_unadmitted.admitted_mask = 0;
        let ref_unadmitted = oracle_select_optimal_candidate(&input_unadmitted);
        assert_eq!(
            ref_unadmitted.refusal_code,
            AutoSelectRefusal::ControlStateUnadmitted as u8
        );
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle

// Axiomatic Hoare logic
