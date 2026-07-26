//! Phase 3 (Scheduler) and Phase 4 (Validation) benchmarks using Divan.
//!
//! Benchmarks POWL scheduler operations and tape validation:
//!
//! - Phase 3: `scheduler_tick` on ops with complex dependencies vs. simple linear chain
//! - Phase 4: `OcelLog::validate_against_tape` conformance checking and receipt sealing

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::OcelLog;
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

fn main() {
    divan::main();
}

const RUN_ID: u64 = 1;

/// Helper: construct a simple linear sequence tape
fn linear_sequence() -> PowlAstNode<'static> {
    PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("step1"),
        PowlAstNode::Atom("step2"),
        PowlAstNode::Atom("step3"),
        PowlAstNode::Atom("step4"),
    ])
}

/// Helper: construct a partial order with many dependencies
fn complex_dependencies() -> PowlAstNode<'static> {
    PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("init"),
            PowlAstNode::Atom("validate_a"),
            PowlAstNode::Atom("validate_b"),
            PowlAstNode::Atom("validate_c"),
            PowlAstNode::Atom("parallel_x"),
            PowlAstNode::Atom("parallel_y"),
            PowlAstNode::Atom("parallel_z"),
            PowlAstNode::Atom("finalize"),
        ],
        edges: vec![
            (0, 1), (0, 2), (0, 3),  // init → all validators
            (1, 4), (2, 4), (3, 5),  // validators → parallel ops
            (4, 7), (5, 7), (6, 7),  // parallel ops → finalize
        ],
    }
}

/// Helper: run scheduler to completion and return tick count
fn run_to_completion(tape: &PowlTape) -> u32 {
    let mut state = PowlRunState::new(tape);
    let mut ticks = 0u32;
    for _ in 0..1000 {
        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }
        let _ = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        ticks += 1;
    }
    ticks
}

/// Helper: run scheduler and log every fired op
fn run_and_log(tape: &PowlTape) -> OcelLog {
    let mut state = PowlRunState::new(tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;
    let mut tick: u32 = 0;

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let fired_set = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        let mut bits = fired_set.0;
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

mod phase3_scheduler {
    use super::*;

    /// Benchmark: scheduler_tick on simple linear sequence.
    /// Represents the baseline: each tick fires one op, all dependencies are sequential.
    #[divan::bench]
    fn scheduler_tick_linear_sequence() -> u32 {
        let tape = compile_powl(divan::black_box(&linear_sequence()))
            .expect("linear sequence must compile");
        run_to_completion(divan::black_box(&tape))
    }

    /// Benchmark: scheduler_tick on complex partial order.
    /// Tests scheduler with multiple independent branches and re-convergence.
    /// Has 8 ops with 7 dependencies forming a DAG with parallelism.
    #[divan::bench]
    fn scheduler_tick_complex_dependencies() -> u32 {
        let tape = compile_powl(divan::black_box(&complex_dependencies()))
            .expect("complex dependencies must compile");
        run_to_completion(divan::black_box(&tape))
    }

    /// Benchmark: scheduler_tick on single op (minimal).
    /// Tests scheduler overhead for the simplest possible schedule.
    #[divan::bench]
    fn scheduler_tick_single_op() -> u32 {
        let tape = compile_powl(divan::black_box(&PowlAstNode::Atom("work")))
            .expect("single op must compile");
        run_to_completion(divan::black_box(&tape))
    }

    /// Benchmark: scheduler_tick on wide parallelism.
    /// Tests scheduler with many independent parallel ops.
    #[divan::bench]
    fn scheduler_tick_wide_parallelism() -> u32 {
        let wide = PowlAstNode::PartialOrder {
            children: vec![
                PowlAstNode::Atom("p0"),
                PowlAstNode::Atom("p1"),
                PowlAstNode::Atom("p2"),
                PowlAstNode::Atom("p3"),
                PowlAstNode::Atom("p4"),
                PowlAstNode::Atom("p5"),
                PowlAstNode::Atom("p6"),
                PowlAstNode::Atom("p7"),
            ],
            edges: vec![],  // No dependencies: all ops can run in parallel
        };
        let tape = compile_powl(divan::black_box(&wide))
            .expect("wide parallelism must compile");
        run_to_completion(divan::black_box(&tape))
    }
}

mod phase4_validation {
    use super::*;

    /// Benchmark: validate_against_tape on linear sequence.
    /// Tests conformance checking on a simple, structured execution.
    #[divan::bench]
    fn validate_linear_sequence() -> bool {
        let tape = compile_powl(&linear_sequence())
            .expect("linear sequence must compile");
        let log = run_and_log(divan::black_box(&tape));
        let _receipt = log.seal_receipt();
        let result = divan::black_box(&log).validate_against_tape(divan::black_box(&tape));
        result == bcinr_powl::ocel::ConformanceResult::Conforms
    }

    /// Benchmark: validate_against_tape on complex partial order.
    /// Tests conformance checking with dependencies and parallelism.
    #[divan::bench]
    fn validate_complex_dependencies() -> bool {
        let tape = compile_powl(&complex_dependencies())
            .expect("complex dependencies must compile");
        let log = run_and_log(divan::black_box(&tape));
        let _receipt = log.seal_receipt();
        let result = divan::black_box(&log).validate_against_tape(divan::black_box(&tape));
        result == bcinr_powl::ocel::ConformanceResult::Conforms
    }

    /// Benchmark: seal_receipt on OCEL log.
    /// Tests receipt generation (BLAKE3 hashing of execution trace).
    #[divan::bench]
    fn seal_receipt_linear() -> [u8; 32] {
        let tape = compile_powl(&linear_sequence())
            .expect("linear sequence must compile");
        let log = run_and_log(divan::black_box(&tape));
        divan::black_box(&log).seal_receipt().digest()
    }

    /// Benchmark: full pipeline — schedule, log, seal, validate.
    /// Represents the realistic end-to-end Phase 4 flow.
    #[divan::bench]
    fn full_pipeline_linear_sequence() -> bool {
        let tape = compile_powl(divan::black_box(&linear_sequence()))
            .expect("linear sequence must compile");
        let log = run_and_log(&tape);
        let receipt = log.seal_receipt();
        let _ = divan::black_box(receipt.digest());
        let result = log.validate_against_tape(&tape);
        result == bcinr_powl::ocel::ConformanceResult::Conforms
    }
}

