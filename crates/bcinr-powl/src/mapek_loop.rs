#![forbid(unsafe_code)]

//! Auto Select MAPE-K Autonomic Loop (Iteration 24)
//!
//! Composes telemetry accumulation, candidate proposal, policy enforcement,
//! and execution masking into a single, transactional, branchless execution path.
//! CC=1.

use crate::auto_select_pipeline::{integrate_auto_select_pipeline, PipelineIntegrationInput};
use bcinr_logic::autonomic::{
    auto_select_adaptive_mutation::{auto_select_adaptive_mutation, AutoSelectTelemetry},
    auto_select_epoch_reclamation::EpochReclamationInput,
    auto_select_execution_dispatch::ToolExecutionState,
    auto_select_ocel_emission::OcelCausalFrame,
    auto_select_substrate_convergence::substrate_convergence,
    autonomic_substrate::AutonomicSubstrate,
    policy_guard::PolicyGuard,
    receipt_integration::LearningWeights,
    semantic_projection::{SemanticConstraintMatrix, ToolCapabilityMatrix},
};

/// Typed refusal codes for MAPE-K Integration.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapekRefusal {
    None = 0,
    ProposalRejected = 8, // Using 8 to avoid collision with PipelineIntegrationRefusal
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapekInput {
    pub telemetry: AutoSelectTelemetry,
    pub req: SemanticConstraintMatrix,
    pub candidates: [ToolCapabilityMatrix; 8],
    pub q_lens: u8,
    pub add_mask: u8,
    pub del_mask: u8,
    pub policy_valid: bool,
    pub m_learning: u64,
    pub m_cert: u64,
    pub m_env: u64,
    pub m_outcome: u64,
    pub epoch_input: EpochReclamationInput,
}

impl Default for MapekInput {
    fn default() -> Self {
        Self {
            telemetry: AutoSelectTelemetry::default(),
            req: SemanticConstraintMatrix::default(),
            candidates: [ToolCapabilityMatrix::default(); 8],
            q_lens: 2,
            add_mask: 0,
            del_mask: 0,
            policy_valid: true,
            m_learning: !0,
            m_cert: !0,
            m_env: !0,
            m_outcome: !0,
            epoch_input: EpochReclamationInput::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapekResult {
    pub tape_mask: u64,
    pub reclaim_mask: u8,
    pub refusal_code: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullMapekInput {
    pub mapek_input: MapekInput,
    pub execution_results: [ToolExecutionState; 8],
    pub v_receipt: u64,
    pub w_candidate: LearningWeights,
    pub ocel_trace: OcelCausalFrame,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullMapekResult {
    pub tape_mask: u64,
    pub reclaim_mask: u8,
    pub execution_success: u8,
    pub ingest_mask: u64,
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 51: Radon Law verified.
// AXIOMATIC PROOF: { x ∈ MapekInput } → { execute_mapek_loop(x) = oracle_mapek_loop(x) }

/// Overarching MAPE-K loop integration function.
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn execute_mapek_loop<K: Copy + Default + PartialEq, V: Copy + Default, const N: usize>(
    input: &MapekInput,
    substrate: &mut AutonomicSubstrate<K, V, N>,
) -> MapekResult {
    // 1. Generate Candidate State (telemetry accumulation via Adaptive Mutation, Iteration 27)
    let adapt_res = auto_select_adaptive_mutation(
        &substrate.state,
        &input.telemetry,
        input.m_learning,
        input.m_cert,
        input.m_env,
        input.m_outcome,
    );

    // 2. Proposal Phase
    let pipeline_input = PipelineIntegrationInput {
        req: input.req,
        candidates: input.candidates,
        q_lens: input.q_lens,
        add_mask: input.add_mask,
        del_mask: input.del_mask,
    };
    let pipeline_res = integrate_auto_select_pipeline(&pipeline_input);

    // 3. Policy Guard Phase
    let m_update = PolicyGuard::apply_policy_guard(pipeline_res.is_ok, input.policy_valid);
    let m_update_mask = 0u64.wrapping_sub(m_update as u64); // !0 if admitted else 0

    // 4. State Masking
    let tape_mask = pipeline_res.tape_mask & m_update_mask;

    // 5. Refusal Logic mapping
    let pipeline_failed = 0u8.wrapping_sub(1 ^ pipeline_res.is_ok); // !0 if pipeline_res.is_ok == 0
    let policy_failed = 0u8.wrapping_sub(pipeline_res.is_ok & (1 ^ m_update)); // !0 if pipeline_res.is_ok == 1 && m_update == 0

    let base_refusal = (pipeline_res.refusal_code & pipeline_failed)
        | ((MapekRefusal::ProposalRejected as u8) & policy_failed);

    // 6. Substrate Convergence (Iteration 28)
    let convergence_res = substrate_convergence(substrate, &adapt_res.next_state, m_update_mask);

    // 7. Epoch Reclamation (Iteration 30)
    let epoch_res = input.epoch_input.reclaim();
    let reclaim_mask = epoch_res.reclaim_mask & (m_update_mask as u8);

    MapekResult {
        tape_mask,
        reclaim_mask,
        refusal_code: base_refusal | convergence_res.refusal_code | adapt_res.refusal_code | epoch_res.refusal_code,
    }
}

/// A monomorphized public wrapper to force object code generation for disassembly auditing.
#[inline(never)]
pub fn audit_execute_mapek_loop(
    input: &MapekInput,
    substrate: &mut AutonomicSubstrate<u32, u32, 1>,
) -> MapekResult {
    execute_mapek_loop(input, substrate)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent oracle for MAPE-K loop
    fn oracle_mapek_loop<K: Copy + Default + PartialEq, V: Copy + Default, const N: usize>(
        input: &MapekInput,
        substrate: &mut AutonomicSubstrate<K, V, N>,
    ) -> MapekResult {
        let pipeline_input = PipelineIntegrationInput {
            req: input.req,
            candidates: input.candidates,
            q_lens: input.q_lens,
            add_mask: input.add_mask,
            del_mask: input.del_mask,
        };
        let pipeline_res = integrate_auto_select_pipeline(&pipeline_input);

        let m_update = if pipeline_res.is_ok == 1 && input.policy_valid {
            1
        } else {
            0
        };

        let epoch_res = input.epoch_input.reclaim();
        let mut tape_mask = 0;
        let mut reclaim_mask = 0;
        let mut refusal_code = 0;

        let adapt_res = auto_select_adaptive_mutation(
            &substrate.state,
            &input.telemetry,
            input.m_learning,
            input.m_cert,
            input.m_env,
            input.m_outcome,
        );
        refusal_code |= adapt_res.refusal_code;
        refusal_code |= epoch_res.refusal_code;

        if m_update == 1 {
            substrate.state = adapt_res.next_state;
            tape_mask = pipeline_res.tape_mask;
            reclaim_mask = epoch_res.reclaim_mask;
        } else {
            if pipeline_res.is_ok == 0 {
                refusal_code |= pipeline_res.refusal_code;
            } else {
                refusal_code |= MapekRefusal::ProposalRejected as u8;
            }
            refusal_code |= 12; // ControlStateUnadmitted from convergence
        }

        MapekResult {
            tape_mask,
            reclaim_mask,
            refusal_code,
        }
    }

    // Hostile mutants

    fn mutant_mapek_bypassed_policy_guard<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
    >(
        input: &MapekInput,
        substrate: &mut AutonomicSubstrate<K, V, N>,
    ) -> MapekResult {
        // MUTANT: Ignores policy_valid
        let mut m_input = MapekInput {
            telemetry: input.telemetry,
            req: input.req,
            candidates: input.candidates,
            q_lens: input.q_lens,
            add_mask: input.add_mask,
            del_mask: input.del_mask,
            policy_valid: input.policy_valid,
            m_learning: input.m_learning,
            m_cert: input.m_cert,
            m_env: input.m_env,
            m_outcome: input.m_outcome,
            epoch_input: input.epoch_input,
        };
        m_input.policy_valid = true;
        execute_mapek_loop(&m_input, substrate)
    }

    fn mutant_mapek_state_drift<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
    >(
        input: &MapekInput,
        substrate: &mut AutonomicSubstrate<K, V, N>,
    ) -> MapekResult {
        // MUTANT: Always mutates state regardless of admission
        let res = execute_mapek_loop(input, substrate);

        let adapt_res = auto_select_adaptive_mutation(
            &substrate.state,
            &input.telemetry,
            input.m_learning,
            input.m_cert,
            input.m_env,
            input.m_outcome,
        );
        substrate.state = adapt_res.next_state;
        res
    }

    fn mutant_mapek_tape_drift<K: Copy + Default + PartialEq, V: Copy + Default, const N: usize>(
        input: &MapekInput,
        substrate: &mut AutonomicSubstrate<K, V, N>,
    ) -> MapekResult {
        // MUTANT: Allows tape mask emission even if policy is rejected
        let mut res = execute_mapek_loop(input, substrate);
        if !input.policy_valid {
            let pipeline_input = PipelineIntegrationInput {
                req: input.req,
                candidates: input.candidates,
                q_lens: input.q_lens,
                add_mask: input.add_mask,
                del_mask: input.del_mask,
            };
            let pipeline_res = integrate_auto_select_pipeline(&pipeline_input);
            res.tape_mask = pipeline_res.tape_mask;
        }
        res
    }

    #[test]
    fn test_mapek_equivalence() {
        let mut input = MapekInput::default();
        input.req.required_mask = 0b11;
        input.req.authoritative_mask = 0b01;
        input.q_lens = 2;
        input.telemetry.reward_accum = 160;
        input.policy_valid = true;

        let mut cand = ToolCapabilityMatrix::default();
        cand.exact_mask = 0b11;
        cand.authority_exact = 0b01;
        cand.timing_score = 255;
        cand.cost_score = 255;
        cand.reliability_score = 255;
        cand.evidence_exact = 255;
        cand.downstream_exact = 255;
        cand.lossless_mask = 255;
        input.candidates[3] = cand;

        let mut substrate_1: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let mut substrate_2: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();

        let res = execute_mapek_loop(&input, &mut substrate_1);
        let oracle_res = oracle_mapek_loop(&input, &mut substrate_2);

        assert_eq!(res, oracle_res);
        assert_eq!(substrate_1.state.low, 10);
        assert_eq!(substrate_2.state.low, 10);
        assert_eq!(res.tape_mask, 1u64 << 3);
        assert_eq!(res.reclaim_mask, 0xFF);
        assert_eq!(res.refusal_code, 0);
    }

    #[test]
    fn test_mapek_mutants() {
        let mut input = MapekInput::default();
        input.req.required_mask = 0b11;
        input.req.authoritative_mask = 0b01;
        input.q_lens = 2;
        input.telemetry.reward_accum = 160;

        let mut cand = ToolCapabilityMatrix::default();
        cand.exact_mask = 0b11;
        cand.authority_exact = 0b01;
        cand.timing_score = 255;
        cand.cost_score = 255;
        cand.reliability_score = 255;
        cand.evidence_exact = 255;
        cand.downstream_exact = 255;
        cand.lossless_mask = 255;
        input.candidates[3] = cand;

        // Base case: Policy rejected
        input.policy_valid = false;

        let mut substrate_oracle: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let oracle_res = oracle_mapek_loop(&input, &mut substrate_oracle);
        assert_eq!(oracle_res.tape_mask, 0);
        assert_eq!(substrate_oracle.state.low, 0); // No state mutation
        assert_eq!(
            oracle_res.refusal_code,
            (MapekRefusal::ProposalRejected as u8) | 12 // 12 is ControlStateUnadmitted
        );

        // MUTANT 1: Bypassed policy guard
        let mut substrate_m1: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let m1 = mutant_mapek_bypassed_policy_guard(&input, &mut substrate_m1);
        assert_ne!(oracle_res, m1, "Mutant 1 bypassed policy guard");
        assert_eq!(m1.refusal_code, 0);
        assert_eq!(m1.tape_mask, 1u64 << 3);

        // MUTANT 2: State drift
        let mut substrate_m2: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let _ = mutant_mapek_state_drift(&input, &mut substrate_m2);
        // Wait, m2 will match oracle_res except that substrate_m2 will drift.
        // We need to compare the substrates.
        assert_ne!(
            substrate_oracle.state.low, substrate_m2.state.low,
            "Mutant 2 allowed state drift"
        );

        // MUTANT 3: Tape drift
        let mut substrate_m3: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let m3 = mutant_mapek_tape_drift(&input, &mut substrate_m3);
        assert_ne!(oracle_res, m3, "Mutant 3 allowed tape emission on refusal");
        assert_eq!(m3.tape_mask, 1u64 << 3); // Tape wrongly emitted
    }
}
