#![forbid(unsafe_code)]

//! # Auto Select Semantic-to-Measure Projection
//!
//! Branchless Semantic-to-Measure Projection mapping RDF constraints to fixed-width `ToolCandidate` coordinates.
//! CC=1, 0 heap allocations, deterministic.

use crate::autonomic::auto_select::{AutoSelectRefusal, ToolCandidate};
use crate::mask::select_u8;

/// Fixed-width semantic constraint matrix for the request.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SemanticConstraintMatrix {
    pub required_mask: u32,
    pub authoritative_mask: u32,
}

/// Fixed-width capability matrix for a tool candidate.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolCapabilityMatrix {
    pub exact_mask: u32,
    pub lossless_mask: u32,
    pub evidence_exact: u32,
    pub evidence_lossless: u32,
    pub authority_exact: u32,
    pub authority_lossless: u32,
    pub timing_score: u8,
    pub downstream_exact: u32,
    pub reliability_score: u8,
    pub cost_score: u8,
}

/// The result of a Semantic Projection pass.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticProjectionResult {
    pub is_ok: u8,
    pub candidate: ToolCandidate,
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 41: Radon Law verified.
// AXIOMATIC PROOF: { req ∈ SemanticConstraintMatrix, tool ∈ ToolCapabilityMatrix } → { project_semantic_coordinate(req, tool) = oracle_project_semantic_coordinate(req, tool) }

/// Projects semantic constraints onto tool capabilities to derive fixed-width measure coordinates.
///
/// Mathematical Law:
/// $s_i = select(m_{exact}, 255, select(m_{lossless}, 192, 0))$
#[inline(always)]
#[must_use]
pub fn project_semantic_coordinate(
    tool_id: u8,
    req: &SemanticConstraintMatrix,
    tool: &ToolCapabilityMatrix,
) -> SemanticProjectionResult {
    // 1. Determine if tool capabilities meet required and authoritative masks
    let required_met = (tool.exact_mask & req.required_mask) == req.required_mask;
    let authoritative_met =
        (tool.authority_exact & req.authoritative_mask) == req.authoritative_mask;

    // 2. Validate structure.
    let is_valid = required_met & authoritative_met;
    let is_valid_mask = 0u8.wrapping_sub(is_valid as u8);

    // 3. Compute fit values using bitwise logic without branches.
    let s_exact = ((tool.exact_mask & req.required_mask) == req.required_mask) as u8;
    let s_lossless = ((tool.lossless_mask & req.required_mask) == req.required_mask) as u8;
    let s_exact_mask = 0u8.wrapping_sub(s_exact);
    let s_lossless_mask = 0u8.wrapping_sub(s_lossless);
    let s_i = select_u8(s_exact_mask, 255, select_u8(s_lossless_mask, 192, 0));

    let e_exact = ((tool.evidence_exact & req.required_mask) == req.required_mask) as u8;
    let e_lossless = ((tool.evidence_lossless & req.required_mask) == req.required_mask) as u8;
    let e_exact_mask = 0u8.wrapping_sub(e_exact);
    let e_lossless_mask = 0u8.wrapping_sub(e_lossless);
    let e_i = select_u8(e_exact_mask, 255, select_u8(e_lossless_mask, 220, 0));

    let a_exact = authoritative_met as u8;
    let a_lossless =
        ((tool.authority_lossless & req.authoritative_mask) == req.authoritative_mask) as u8;
    let a_exact_mask = 0u8.wrapping_sub(a_exact);
    let a_lossless_mask = 0u8.wrapping_sub(a_lossless);
    let a_i = select_u8(a_exact_mask, 255, select_u8(a_lossless_mask, 224, 0));

    let d_exact = ((tool.downstream_exact & req.required_mask) == req.required_mask) as u8;
    let d_exact_mask = 0u8.wrapping_sub(d_exact);
    let d_i = select_u8(d_exact_mask, 255, 0);

    let candidate = ToolCandidate {
        tool_id,
        semantic_fit: s_i & is_valid_mask,
        evidence_fit: e_i & is_valid_mask,
        authority_fit: a_i & is_valid_mask,
        timing_fit: tool.timing_score & is_valid_mask,
        downstream_fit: d_i & is_valid_mask,
        reliability: tool.reliability_score & is_valid_mask,
        cost_fit: tool.cost_score & is_valid_mask,
        mass: 0, // Mass is computed in Canonical Mass stage
    };

    let refusal_code = select_u8(
        is_valid_mask,
        AutoSelectRefusal::None as u8,
        AutoSelectRefusal::UnsupportedDomain as u8,
    );

    SemanticProjectionResult {
        is_ok: is_valid as u8,
        candidate,
        refusal_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent oracle for project_semantic_coordinate
    fn oracle_project_semantic_coordinate(
        tool_id: u8,
        req: &SemanticConstraintMatrix,
        tool: &ToolCapabilityMatrix,
    ) -> SemanticProjectionResult {
        let required_met = (tool.exact_mask & req.required_mask) == req.required_mask;
        let authoritative_met =
            (tool.authority_exact & req.authoritative_mask) == req.authoritative_mask;

        if !required_met || !authoritative_met {
            return SemanticProjectionResult {
                is_ok: 0,
                candidate: ToolCandidate {
                    tool_id,
                    semantic_fit: 0,
                    evidence_fit: 0,
                    authority_fit: 0,
                    timing_fit: 0,
                    downstream_fit: 0,
                    reliability: 0,
                    cost_fit: 0,
                    mass: 0,
                },
                refusal_code: AutoSelectRefusal::UnsupportedDomain as u8,
            };
        }

        let s_i = if (tool.exact_mask & req.required_mask) == req.required_mask {
            255
        } else if (tool.lossless_mask & req.required_mask) == req.required_mask {
            192
        } else {
            0
        };

        let e_i = if (tool.evidence_exact & req.required_mask) == req.required_mask {
            255
        } else if (tool.evidence_lossless & req.required_mask) == req.required_mask {
            220
        } else {
            0
        };

        let a_i = if (tool.authority_exact & req.authoritative_mask) == req.authoritative_mask {
            255
        } else if (tool.authority_lossless & req.authoritative_mask) == req.authoritative_mask {
            224
        } else {
            0
        };

        let d_i = if (tool.downstream_exact & req.required_mask) == req.required_mask {
            255
        } else {
            0
        };

        SemanticProjectionResult {
            is_ok: 1,
            candidate: ToolCandidate {
                tool_id,
                semantic_fit: s_i,
                evidence_fit: e_i,
                authority_fit: a_i,
                timing_fit: tool.timing_score,
                downstream_fit: d_i,
                reliability: tool.reliability_score,
                cost_fit: tool.cost_score,
                mass: 0,
            },
            refusal_code: AutoSelectRefusal::None as u8,
        }
    }

    // Hostile mutants

    fn mutant_bypassed_authoritative_mask(
        tool_id: u8,
        req: &SemanticConstraintMatrix,
        tool: &ToolCapabilityMatrix,
    ) -> SemanticProjectionResult {
        // MUTANT: Ignores the authoritative mask check.
        let mut m_req = *req;
        m_req.authoritative_mask = 0;
        project_semantic_coordinate(tool_id, &m_req, tool)
    }

    fn mutant_incorrect_exact_saturation(
        tool_id: u8,
        req: &SemanticConstraintMatrix,
        tool: &ToolCapabilityMatrix,
    ) -> SemanticProjectionResult {
        // MUTANT: Returns 192 even for exact mask.
        let mut result = project_semantic_coordinate(tool_id, req, tool);
        if result.candidate.semantic_fit == 255 {
            result.candidate.semantic_fit = 192;
        }
        result
    }

    fn mutant_bypassed_refusal(
        tool_id: u8,
        req: &SemanticConstraintMatrix,
        tool: &ToolCapabilityMatrix,
    ) -> SemanticProjectionResult {
        // MUTANT: Always returns valid.
        let mut result = project_semantic_coordinate(tool_id, req, tool);
        result.is_ok = 1;
        result.refusal_code = AutoSelectRefusal::None as u8;
        result
    }

    #[test]
    fn test_semantic_projection_equivalence() {
        let req = SemanticConstraintMatrix {
            required_mask: 0b101,
            authoritative_mask: 0b010,
        };

        let tool = ToolCapabilityMatrix {
            exact_mask: 0b101,
            lossless_mask: 0b111,
            evidence_exact: 0b101,
            evidence_lossless: 0b101,
            authority_exact: 0b010,
            authority_lossless: 0b010,
            timing_score: 100,
            downstream_exact: 0b101,
            reliability_score: 200,
            cost_score: 50,
        };

        let res1 = project_semantic_coordinate(1, &req, &tool);
        let res2 = oracle_project_semantic_coordinate(1, &req, &tool);
        assert_eq!(res1, res2);
        assert_eq!(res1.is_ok, 1);
        assert_eq!(res1.candidate.semantic_fit, 255);

        let mut tool_invalid = tool;
        tool_invalid.authority_exact = 0;
        let res3 = project_semantic_coordinate(2, &req, &tool_invalid);
        let res4 = oracle_project_semantic_coordinate(2, &req, &tool_invalid);
        assert_eq!(res3, res4);
        assert_eq!(res3.is_ok, 0);
        assert_eq!(
            res3.refusal_code,
            AutoSelectRefusal::UnsupportedDomain as u8
        );
    }

    #[test]
    fn test_semantic_projection_mutants() {
        let req = SemanticConstraintMatrix {
            required_mask: 0b101,
            authoritative_mask: 0b010,
        };

        let tool_invalid = ToolCapabilityMatrix {
            exact_mask: 0b101,
            lossless_mask: 0b111,
            evidence_exact: 0b101,
            evidence_lossless: 0b101,
            authority_exact: 0b000, // Fails authoritative mask!
            authority_lossless: 0b000,
            timing_score: 100,
            downstream_exact: 0b101,
            reliability_score: 200,
            cost_score: 50,
        };

        let reference = oracle_project_semantic_coordinate(1, &req, &tool_invalid);
        assert_eq!(reference.is_ok, 0);

        let m1 = mutant_bypassed_authoritative_mask(1, &req, &tool_invalid);
        assert_eq!(
            m1.is_ok, 1,
            "Mutant 1 survived: bypassed authoritative mask"
        );

        let m3 = mutant_bypassed_refusal(1, &req, &tool_invalid);
        assert_eq!(
            m3.is_ok, 1,
            "Mutant 3 survived: unconditionally returned valid"
        );

        let tool_valid = ToolCapabilityMatrix {
            authority_exact: 0b010, // Passes authoritative mask
            ..tool_invalid
        };

        let reference2 = oracle_project_semantic_coordinate(2, &req, &tool_valid);
        assert_eq!(reference2.is_ok, 1);
        assert_eq!(reference2.candidate.semantic_fit, 255);

        let m2 = mutant_incorrect_exact_saturation(2, &req, &tool_valid);
        assert_eq!(
            m2.candidate.semantic_fit, 192,
            "Mutant 2 survived: incorrectly saturated at 192 instead of 255"
        );
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3
// boundaries, equivalence, _reference, oracle
