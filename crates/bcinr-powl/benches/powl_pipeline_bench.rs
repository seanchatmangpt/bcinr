//! Divan benchmarks for the complete POWL production pipeline: compile ->
//! schedule -> OCEL log -> seal receipt -> conformance check.
//!
//! `powl_quick_bench.rs` (this crate's other bench target) measures
//! `compile_powl` scaling and raw scheduler-tick throughput in isolation.
//! Neither it nor any other benchmark in the repo previously timed
//! `bcinr_powl::ocel::OcelLog` (`record_op_fired`, `record_run_sealed`,
//! `seal_receipt`, `validate_against_tape`) — the layer this repo's
//! Chicago-TDD correctness tests already exercise
//! (`crates/bcinr-powl/tests/usecase_compliance_audit.rs`,
//! `usecase_consensus_termination.rs`). This file closes that benchmark gap,
//! mirroring `bcinr-bench/benches/cmca_execution_bench.rs`'s module layout
//! (`compile` / `schedule` / `ocel` / `end_to_end`) and its discipline of
//! using only the real public API with every input passed through
//! `divan::black_box`.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::{ConformanceResult, OcelLog};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

fn main() {
    divan::main();
}

const RUN_ID: u64 = 1;

fn sequence_workload() -> PowlAstNode<'static> {
    PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("intake"),
        PowlAstNode::Atom("review"),
        PowlAstNode::Atom("approve"),
        PowlAstNode::Atom("release"),
    ])
}

fn partial_order_workload() -> PowlAstNode<'static> {
    PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("validator_a"),
            PowlAstNode::Atom("validator_b"),
            PowlAstNode::Atom("validator_c"),
            PowlAstNode::Atom("validator_d"),
        ],
        edges: vec![],
    }
}

fn mixed_workload() -> PowlAstNode<'static> {
    PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("load"),
            PowlAstNode::Atom("compute"),
            PowlAstNode::Atom("store"),
            PowlAstNode::Atom("independent"),
        ],
        edges: vec![(0, 1), (1, 2)],
    }
}

/// Runs a compiled tape to completion, logging every fired op — the
/// realistic production shape (scheduler + OCEL logging interleaved), not
/// scheduler-only throughput.
fn run_and_log(tape: &PowlTape) -> OcelLog {
    let mut state = PowlRunState::new(tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;
    let mut tick: u32 = 0;

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            let _ = log.record_op_fired(RUN_ID, op_idx as u32, tick, 1);
            op_trace |= 1u64 << op_idx;
        }
        tick += 1;
    }
    let _ = log.record_run_sealed(RUN_ID, op_trace, tick);
    log
}

mod compile {
    use super::*;

    #[divan::bench]
    fn sequence() -> PowlTape {
        compile_powl(divan::black_box(&sequence_workload()))
            .expect("sequence workload must compile")
    }

    #[divan::bench]
    fn partial_order() -> PowlTape {
        compile_powl(divan::black_box(&partial_order_workload()))
            .expect("partial-order workload must compile")
    }

    #[divan::bench]
    fn mixed() -> PowlTape {
        compile_powl(divan::black_box(&mixed_workload())).expect("mixed workload must compile")
    }
}

mod schedule {
    use super::*;

    #[divan::bench]
    fn sequence_to_completion() -> u32 {
        let tape = compile_powl(&sequence_workload()).expect("must compile");
        let mut state = PowlRunState::new(&tape);
        let mut ticks = 0u32;
        while state.check_mask != 0 {
            let _ = divan::black_box(scheduler_tick(&tape.ops[..tape.len as usize], &mut state));
            ticks += 1;
        }
        ticks
    }

    #[divan::bench]
    fn partial_order_to_completion() -> u32 {
        let tape = compile_powl(&partial_order_workload()).expect("must compile");
        let mut state = PowlRunState::new(&tape);
        let mut ticks = 0u32;
        while state.check_mask != 0 {
            let _ = divan::black_box(scheduler_tick(&tape.ops[..tape.len as usize], &mut state));
            ticks += 1;
        }
        ticks
    }
}

mod ocel {
    use super::*;

    #[divan::bench]
    fn record_and_seal_sequence() -> OcelLog {
        let tape = compile_powl(&sequence_workload()).expect("must compile");
        run_and_log(divan::black_box(&tape))
    }

    #[divan::bench]
    fn seal_receipt() -> [u8; 32] {
        let tape = compile_powl(&sequence_workload()).expect("must compile");
        let log = run_and_log(&tape);
        divan::black_box(&log).seal_receipt().digest()
    }

    #[divan::bench]
    fn validate_against_tape() -> ConformanceResult {
        let tape = compile_powl(&sequence_workload()).expect("must compile");
        let log = run_and_log(&tape);
        divan::black_box(&log).validate_against_tape(divan::black_box(&tape))
    }
}

mod end_to_end {
    use super::*;

    /// Full production pipeline in one call: compile -> schedule -> log ->
    /// seal receipt -> validate conformance. Mirrors
    /// `cmca_execution_bench.rs::end_to_end`'s discipline of timing the
    /// whole chain together, not just its parts summed.
    #[divan::bench]
    fn compile_schedule_log_seal_validate() -> ConformanceResult {
        let tape =
            compile_powl(divan::black_box(&sequence_workload())).expect("workload must compile");
        let log = run_and_log(&tape);
        let receipt = log.seal_receipt();
        let _ = divan::black_box(receipt.digest());
        let result = log.validate_against_tape(&tape);
        assert_eq!(
            result,
            ConformanceResult::Conforms,
            "a faithfully recorded production run must conform to its own compiled tape"
        );
        result
    }
}
