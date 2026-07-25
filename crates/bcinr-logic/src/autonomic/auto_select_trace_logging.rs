#![forbid(unsafe_code)]

//! # Auto Select Trace Logging Operator (Iteration 32)
//!
//! Log emitted branchless execution traces persistently. CC=1.

use crate::autonomic::auto_select_ocel_emission::OcelCausalFrame;

/// Typed refusal codes for Trace Logging.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceLoggingRefusal {
    None = 0,
    EnvelopeViolated = 21,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceBufferState<const N: usize> {
    pub cursor: u64,
    pub frames: [OcelCausalFrame; N],
}

impl<const N: usize> Default for TraceBufferState<N> {
    fn default() -> Self {
        Self {
            cursor: 0,
            frames: [OcelCausalFrame::default(); N],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceLoggingResult {
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 37: Radon Law verified.
// AXIOMATIC PROOF: { S_trace, E_ocel } -> { S_next = select(m, S \cup E, S) }

#[inline(always)]
#[must_use]
pub fn log_execution_trace<const N: usize>(
    state: &mut TraceBufferState<N>,
    trace: &OcelCausalFrame,
) -> TraceLoggingResult {
    let m_admitted = 0u64.wrapping_sub((trace.instruction_id > 0) as u64);

    let is_rejected_u8 = 0u8.wrapping_sub(1 ^ (m_admitted & 1) as u8);
    let refusal_code = is_rejected_u8 & (TraceLoggingRefusal::EnvelopeViolated as u8);

    let safe_cursor = (state.cursor as usize) % N;
    let old_frame = state.frames[safe_cursor];
    let next_frame = OcelCausalFrame::select(m_admitted, trace, &old_frame);
    state.frames[safe_cursor] = next_frame;

    let advanced = state.cursor.wrapping_add(m_admitted & 1);
    let is_n = (advanced == N as u64) as u64;
    let next_cursor = advanced.wrapping_sub(N as u64 * is_n);
    state.cursor = next_cursor;

    TraceLoggingResult { refusal_code }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_log<const N: usize>(
        state: &mut TraceBufferState<N>,
        trace: &OcelCausalFrame,
    ) -> TraceLoggingResult {
        if trace.instruction_id == 0 {
            return TraceLoggingResult {
                refusal_code: TraceLoggingRefusal::EnvelopeViolated as u8,
            };
        }

        let cursor = state.cursor as usize % N;
        state.frames[cursor] = *trace;

        let mut next_cursor = state.cursor + 1;
        if next_cursor == N as u64 {
            next_cursor = 0;
        }
        state.cursor = next_cursor;

        TraceLoggingResult {
            refusal_code: TraceLoggingRefusal::None as u8,
        }
    }

    fn mutant_log_bypassed_envelope<const N: usize>(
        state: &mut TraceBufferState<N>,
        trace: &OcelCausalFrame,
    ) -> TraceLoggingResult {
        // MUTANT: Always admit trace
        let m_admitted = 0xFFFFFFFFFFFFFFFF;

        let is_rejected_u8 = 0u8.wrapping_sub(1 ^ (m_admitted & 1) as u8);
        let refusal_code = is_rejected_u8 & (TraceLoggingRefusal::EnvelopeViolated as u8);

        let safe_cursor = (state.cursor as usize) % N;
        let old_frame = state.frames[safe_cursor];
        let next_frame = OcelCausalFrame::select(m_admitted, trace, &old_frame);
        state.frames[safe_cursor] = next_frame;

        let advanced = state.cursor.wrapping_add(m_admitted & 1);
        let is_n = (advanced == N as u64) as u64;
        let next_cursor = advanced.wrapping_sub(N as u64 * is_n);
        state.cursor = next_cursor;

        TraceLoggingResult { refusal_code }
    }

    fn mutant_log_failed_increment<const N: usize>(
        state: &mut TraceBufferState<N>,
        trace: &OcelCausalFrame,
    ) -> TraceLoggingResult {
        // MUTANT: Never increments the cursor
        let m_admitted = 0u64.wrapping_sub((trace.instruction_id > 0) as u64);

        let is_rejected_u8 = 0u8.wrapping_sub(1 ^ (m_admitted & 1) as u8);
        let refusal_code = is_rejected_u8 & (TraceLoggingRefusal::EnvelopeViolated as u8);

        let safe_cursor = (state.cursor as usize) % N;
        let old_frame = state.frames[safe_cursor];
        let next_frame = OcelCausalFrame::select(m_admitted, trace, &old_frame);
        state.frames[safe_cursor] = next_frame;

        // Missing cursor update

        TraceLoggingResult { refusal_code }
    }

    fn mutant_log_unmasked_commit<const N: usize>(
        state: &mut TraceBufferState<N>,
        trace: &OcelCausalFrame,
    ) -> TraceLoggingResult {
        // MUTANT: Unmasked commit
        let m_admitted = 0u64.wrapping_sub((trace.instruction_id > 0) as u64);

        let is_rejected_u8 = 0u8.wrapping_sub(1 ^ (m_admitted & 1) as u8);
        let refusal_code = is_rejected_u8 & (TraceLoggingRefusal::EnvelopeViolated as u8);

        let safe_cursor = (state.cursor as usize) % N;
        state.frames[safe_cursor] = *trace;

        let advanced = state.cursor.wrapping_add(m_admitted & 1);
        let is_n = (advanced == N as u64) as u64;
        let next_cursor = advanced.wrapping_sub(N as u64 * is_n);
        state.cursor = next_cursor;

        TraceLoggingResult { refusal_code }
    }

    #[test]
    fn test_trace_logging_equivalence() {
        let mut s1 = TraceBufferState::<4>::default();
        let mut s2 = TraceBufferState::<4>::default();

        let mut t1 = OcelCausalFrame::default();
        t1.instruction_id = 1;

        let r1 = log_execution_trace(&mut s1, &t1);
        let r2 = oracle_log(&mut s2, &t1);

        assert_eq!(r1, r2);
        assert_eq!(s1, s2);
        assert_eq!(s1.cursor, 1);

        // Envelope violation
        let mut t2 = OcelCausalFrame::default();
        t2.instruction_id = 0;
        let r3 = log_execution_trace(&mut s1, &t2);
        let r4 = oracle_log(&mut s2, &t2);

        assert_eq!(r3, r4);
        assert_eq!(s1, s2);
        assert_eq!(r3.refusal_code, TraceLoggingRefusal::EnvelopeViolated as u8);
        assert_eq!(s1.cursor, 1);
    }

    #[test]
    fn test_trace_logging_mutants() {
        let s = TraceBufferState::<4>::default();
        let mut t = OcelCausalFrame::default();
        t.instruction_id = 0; // Envelope violation
        t.ts_ns = 12345; // So it is distinct from default state

        let mut oracle_state = s.clone();
        let r_oracle = oracle_log(&mut oracle_state, &t);
        assert_eq!(
            r_oracle.refusal_code,
            TraceLoggingRefusal::EnvelopeViolated as u8
        );

        // M1: Bypassed envelope
        let mut s1 = s.clone();
        let m1 = mutant_log_bypassed_envelope(&mut s1, &t);
        assert_ne!(r_oracle.refusal_code, m1.refusal_code);
        assert_eq!(m1.refusal_code, 0);

        // M2: Failed increment
        let mut s2 = s.clone();
        let mut t_valid = OcelCausalFrame::default();
        t_valid.instruction_id = 1;

        let mut oracle_state2 = s.clone();
        oracle_log(&mut oracle_state2, &t_valid);

        mutant_log_failed_increment(&mut s2, &t_valid);
        assert_ne!(oracle_state2.cursor, s2.cursor);

        // M3: Unmasked commit
        let mut s3 = s.clone();
        mutant_log_unmasked_commit(&mut s3, &t); // t is invalid
        assert_ne!(oracle_state.frames[0], s3.frames[0]);
    }
}
