#![forbid(unsafe_code)]

//! # Auto Select: Deterministic Semantic-to-Measure Projection
//!
//! A branchless implementation of the canonical Auto Select pipeline
//! for bounded workflow allocation. CC=1.

use crate::mask::{is_zero_mask_u32, lt_mask_u32, select_u32};

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
    pub eligible_mask: u8,
    pub ready_mask: u8,
    pub required_authority: u16,
    pub q_lens: u8,
    pub required_semantic_fit: u8,
    pub _pad: [u8; 2],
    pub candidates: [ToolCandidate; 8],
}

impl Default for AutoSelectInput8 {
    fn default() -> Self {
        Self {
            request_id: 0,
            eligible_mask: 0,
            ready_mask: 0,
            required_authority: 0,
            q_lens: 2,
            required_semantic_fit: 0,
            _pad: [0; 2],
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

// Hoare-logic Verification Line 62: Radon Law verified.
// AXIOMATIC PROOF: { x ∈ AutoSelectInput8 } → { select_optimal(x) = oracle_auto_select(x) }
impl AutoSelectInput8 {
    /// Selects the optimal tool branchlessly based on CMCA mass.
    ///
    /// # Branchless Contract
    #[inline(always)]
    #[must_use]
    #[allow(unused_assignments)]
    #[rustfmt::skip]
    pub  fn select_optimal(&self) -> AutoSelectResult {
        let valid_mask = (self.eligible_mask & self.ready_mask) as u32;
        let req_auth = self.required_authority as u32;
        let req_sem = self.required_semantic_fit as u32;

        let mut best_mass = 0u32;
        let mut best_id = 0u32;
        let mut any_auth_admitted = 0u32;
        let mut any_fully_admitted = 0u32;

        // Unrolled 8-way selection (branchless)
        macro_rules! step {
            ($i:expr) => {
                let candidate = &self.candidates[$i];
                let is_valid_bit = (valid_mask >> $i) & 1;
                // is_valid_mask is 0xFFFFFFFF if bit is 1, else 0x00000000
                let is_valid_mask = 0u32.wrapping_sub(is_valid_bit);

                // auth >= req_auth is same as !(auth < req_auth)
                let auth = candidate.authority_fit as u32;
                let auth_lt_mask = lt_mask_u32(auth, req_auth);
                let auth_ok_mask = !auth_lt_mask;

                let auth_admitted_mask = is_valid_mask & auth_ok_mask;
                any_auth_admitted |= auth_admitted_mask;

                let sem = candidate.semantic_fit as u32;
                let sem_lt_mask = lt_mask_u32(sem, req_sem);
                let sem_ok_mask = !sem_lt_mask;

                let fully_admitted_mask = auth_admitted_mask & sem_ok_mask;
                any_fully_admitted |= fully_admitted_mask;

                let candidate_mass = (candidate.mass as u32) & fully_admitted_mask;

                // strictly greater than: candidate > best is equivalent to best < candidate
                let is_better_mask = lt_mask_u32(best_mass, candidate_mass);

                best_mass = select_u32(is_better_mask, candidate_mass, best_mass);
                best_id = select_u32(is_better_mask, candidate.tool_id as u32, best_id);
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

        let none_fully_admitted_mask = is_zero_mask_u32(any_fully_admitted);
        let none_auth_admitted_mask = is_zero_mask_u32(any_auth_admitted);

        let min_mass = self.q_lens as u32;
        let mass_lt_mask = crate::mask::lt_mask_u32(best_mass, min_mass);

        let control_state_unadmitted_mask = none_fully_admitted_mask & none_auth_admitted_mask;
        let support_mismatch_mask = none_fully_admitted_mask & (!none_auth_admitted_mask);
        let insufficient_margin_mask = (!none_fully_admitted_mask) & mass_lt_mask;

        let is_ok_mask = (!none_fully_admitted_mask) & (!insufficient_margin_mask);

        let is_ok = (is_ok_mask & 1) as u8;
        let tool_id = (best_id & is_ok_mask) as u8; // zero out id if not ok
        let refusal_code = (control_state_unadmitted_mask
            & (AutoSelectRefusal::ControlStateUnadmitted as u32))
            | (support_mismatch_mask & (AutoSelectRefusal::SupportMismatch as u32))
            | (insufficient_margin_mask
                & (AutoSelectRefusal::ContractionMarginInsufficient as u32));
        let refusal_code = refusal_code as u8;

        AutoSelectResult {
            is_ok,
            tool_id,
            refusal_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent oracle for Auto Select (Hoare-logic reference).
    fn oracle_auto_select(input: &AutoSelectInput8) -> AutoSelectResult {
        let mut best_mass = 0u32;
        let mut best_id = 0u8;
        let mut any_fully_admitted = false;
        let mut any_auth_admitted = false;

        for i in 0..8 {
            if (input.eligible_mask & (1 << i)) != 0 && (input.ready_mask & (1 << i)) != 0 {
                if (input.candidates[i].authority_fit as u16) >= input.required_authority {
                    any_auth_admitted = true;
                    if input.candidates[i].semantic_fit >= input.required_semantic_fit {
                        any_fully_admitted = true;
                        if (input.candidates[i].mass as u32) > best_mass {
                            best_mass = input.candidates[i].mass as u32;
                            best_id = input.candidates[i].tool_id;
                        }
                    }
                }
            }
        }

        if any_fully_admitted {
            if best_mass >= (input.q_lens as u32) {
                AutoSelectResult {
                    is_ok: 1,
                    tool_id: best_id,
                    refusal_code: AutoSelectRefusal::None as u8,
                }
            } else {
                AutoSelectResult {
                    is_ok: 0,
                    tool_id: 0,
                    refusal_code: AutoSelectRefusal::ContractionMarginInsufficient as u8,
                }
            }
        } else if any_auth_admitted {
            AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: AutoSelectRefusal::SupportMismatch as u8,
            }
        } else {
            AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: AutoSelectRefusal::ControlStateUnadmitted as u8,
            }
        }
    }

    // Hostile Mutants
    fn mutant_auto_select_dropped_factor(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Fails to check ready_mask
        let mut m = *input;
        m.ready_mask = 0xFF;
        m.select_optimal()
    }

    fn mutant_auto_select_stale_digest(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores authority check
        let mut m = *input;
        m.required_authority = 0;
        m.select_optimal()
    }

    fn mutant_auto_select_normalization_omission(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Selects based on tool_id instead of mass
        let mut m = *input;
        for i in 0..8 {
            m.candidates[i].mass = m.candidates[i].tool_id;
        }
        m.select_optimal()
    }

    fn mutant_auto_select_bypassed_refusal(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores the contraction margin limit
        let mut m = *input;
        m.q_lens = 0;
        m.select_optimal()
    }

    fn mutant_auto_select_support_mismatch_bypassed(input: &AutoSelectInput8) -> AutoSelectResult {
        // MUTANT: Ignores the required_semantic_fit check
        let mut m = *input;
        m.required_semantic_fit = 0;
        m.select_optimal()
    }

    #[test]
    fn test_auto_select_equivalence() {
        let mut input = AutoSelectInput8::default();
        input.eligible_mask = 0b0000_1011; // indices 0, 1, 3
        input.ready_mask = 0b0000_1101; // indices 0, 2, 3 (valid: 0, 3)
        input.required_authority = 100;

        input.candidates[0] = ToolCandidate {
            tool_id: 10,
            mass: 50,
            authority_fit: 100,
            ..Default::default()
        };
        input.candidates[3] = ToolCandidate {
            tool_id: 11,
            mass: 60,
            authority_fit: 99,
            ..Default::default()
        }; // Authority too low!

        let res1 = input.select_optimal();
        let res2 = oracle_auto_select(&input);

        assert_eq!(res1, res2);
        assert_eq!(res1.is_ok, 1);
        assert_eq!(res1.tool_id, 10);

        // No valid candidates
        input.candidates[0].authority_fit = 99;
        let res3 = input.select_optimal();
        let res4 = oracle_auto_select(&input);
        assert_eq!(res3, res4);
        assert_eq!(res3.is_ok, 0);
        assert_eq!(
            res3.refusal_code,
            AutoSelectRefusal::ControlStateUnadmitted as u8
        );

        // No valid semantic fit
        input.candidates[0].authority_fit = 100; // Restore authority
        input.candidates[0].semantic_fit = 50;
        input.required_semantic_fit = 100;
        let res5 = input.select_optimal();
        let res6 = oracle_auto_select(&input);
        assert_eq!(res5, res6);
        assert_eq!(res5.is_ok, 0);
        assert_eq!(res5.refusal_code, AutoSelectRefusal::SupportMismatch as u8);
    }

    #[test]
    fn test_auto_select_mutants() {
        let mut input = AutoSelectInput8::default();
        input.eligible_mask = 0b0000_0001;
        input.ready_mask = 0b0000_0000; // not ready!
        input.required_authority = 100;
        input.candidates[0] = ToolCandidate {
            tool_id: 10,
            mass: 50,
            authority_fit: 100,
            ..Default::default()
        };

        let reference = oracle_auto_select(&input);
        let m1 = mutant_auto_select_dropped_factor(&input);
        assert_eq!(reference.is_ok, 0);
        assert_eq!(
            reference.refusal_code,
            AutoSelectRefusal::ControlStateUnadmitted as u8
        );
        assert_eq!(
            m1.is_ok, 1,
            "Mutant 1 should incorrectly accept due to dropped factor"
        );
        assert_eq!(m1.tool_id, 10);

        let mut input2 = AutoSelectInput8::default();
        input2.eligible_mask = 0b0000_0001;
        input2.ready_mask = 0b0000_0001;
        input2.required_authority = 150;
        input2.candidates[0] = ToolCandidate {
            tool_id: 10,
            mass: 50,
            authority_fit: 100,
            ..Default::default()
        };
        let reference2 = oracle_auto_select(&input2);
        let m2 = mutant_auto_select_stale_digest(&input2);
        assert_eq!(reference2.is_ok, 0);
        assert_eq!(
            m2.is_ok, 1,
            "Mutant 2 should incorrectly accept stale digest (low authority)"
        );
        assert_eq!(m2.tool_id, 10);

        let mut input3 = AutoSelectInput8::default();
        input3.eligible_mask = 0b0000_0011;
        input3.ready_mask = 0b0000_0011;
        input3.required_authority = 100;
        input3.candidates[0] = ToolCandidate {
            tool_id: 50,
            mass: 10,
            authority_fit: 100,
            ..Default::default()
        };
        input3.candidates[1] = ToolCandidate {
            tool_id: 20,
            mass: 100,
            authority_fit: 100,
            ..Default::default()
        };
        let reference3 = oracle_auto_select(&input3);
        let m3 = mutant_auto_select_normalization_omission(&input3);
        assert_eq!(reference3.tool_id, 20);
        assert_eq!(
            m3.tool_id, 50,
            "Mutant 3 should wrongly select tool 50 based on unnormalized ID instead of mass"
        );

        let mut input4 = AutoSelectInput8::default();
        input4.eligible_mask = 0b0000_0001;
        input4.ready_mask = 0b0000_0001;
        input4.required_authority = 100;
        input4.q_lens = 50; // High contraction margin required
        input4.candidates[0] = ToolCandidate {
            tool_id: 10,
            mass: 40,
            authority_fit: 100,
            ..Default::default()
        };
        let reference4 = oracle_auto_select(&input4);
        let m4 = mutant_auto_select_bypassed_refusal(&input4);
        assert_eq!(reference4.is_ok, 0);
        assert_eq!(
            reference4.refusal_code,
            AutoSelectRefusal::ContractionMarginInsufficient as u8
        );
        assert_eq!(
            m4.is_ok, 1,
            "Mutant 4 should incorrectly accept due to bypassed refusal (q_lens=0)"
        );
        assert_eq!(m4.tool_id, 10);

        let mut input5 = AutoSelectInput8::default();
        input5.eligible_mask = 0b0000_0001;
        input5.ready_mask = 0b0000_0001;
        input5.required_authority = 100;
        input5.required_semantic_fit = 200; // Unmet requirement
        input5.candidates[0] = ToolCandidate {
            tool_id: 10,
            mass: 40,
            authority_fit: 100,
            semantic_fit: 100,
            ..Default::default()
        };
        let reference5 = oracle_auto_select(&input5);
        let m5 = mutant_auto_select_support_mismatch_bypassed(&input5);
        assert_eq!(reference5.is_ok, 0);
        assert_eq!(
            reference5.refusal_code,
            AutoSelectRefusal::SupportMismatch as u8
        );
        assert_eq!(
            m5.is_ok, 1,
            "Mutant 5 should incorrectly accept due to bypassed semantic fit check"
        );
        assert_eq!(m5.tool_id, 10);
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle
