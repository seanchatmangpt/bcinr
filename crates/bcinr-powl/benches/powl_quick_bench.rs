//! Quick POWL benchmark: `compile_powl` scaling + a legacy-vs-wired scheduler
//! comparison on a fixed workload.
//!
//! Uses `divan` rather than Criterion: divan's per-benchmark overhead (no HTML
//! report generation, no adaptive long-running sampling) is dramatically lower,
//! which is what actually gets the whole suite to fit inside a tight wall-clock
//! budget instead of just tuning Criterion's knobs and hoping.
//!
//! Replaces the old `scheduler_bench.rs` (752 lines, ~25 Criterion functions,
//! legacy/wired/wide comparison). Trimmed to legacy vs wired only — `wide_tick`
//! operates on a separate `PowlTapeLarge` representation built by hand-rolled
//! mask helpers, not `compile_powl`'s `PowlTape` output, so a fair "same
//! workload, three variants" comparison would need a second tape-construction
//! path; out of scope for a quick benchmark.

use bcinr_powl::{
    compiler::{compile_powl, PowlAstNode},
    scheduler::{scheduler_tick, PowlRunState},
    scheduler_wired::{petri_tick, PowlPetriState},
    tape::PowlTape,
};
use std::time::Instant;

fn linear_chain(n: usize) -> PowlTape {
    let nodes: Vec<_> = (0..n)
        .map(|i| PowlAstNode::Atom(Box::leak(format!("op{i}").into_boxed_str())))
        .collect();
    compile_powl(&PowlAstNode::Sequence(nodes)).expect("linear compile")
}

fn run_to_done_legacy(tape: &PowlTape) -> u32 {
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    let mut state = PowlRunState::new(tape);
    let mut fired = 0u32;
    while state.check_mask != 0 {
        fired += scheduler_tick(&ops, &mut state).0.count_ones();
    }
    fired
}

fn run_to_done_wired(tape: &PowlTape) -> u32 {
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    let mut state = PowlPetriState::new(tape.entry_mask);
    let mut fired = 0u32;
    while state.check.words[0] != 0 {
        fired += petri_tick(&ops, &mut state, None, None, 0)
            .fired_ops
            .count_ones();
    }
    fired
}

// PowlTape has a fixed 64-op capacity (`ops: [Powl64Op; 64]`), so the scale
// here is capped well under that.
#[divan::bench(args = [4, 16, 64], sample_count = 10, sample_size = 5)]
fn compile_powl_bench(n: usize) {
    let nodes: Vec<String> = (0..n).map(|i| format!("op{i}")).collect();
    let atoms: Vec<PowlAstNode> = nodes
        .iter()
        .map(|s| PowlAstNode::Atom(s.as_str()))
        .collect();
    divan::black_box(compile_powl(&PowlAstNode::Sequence(atoms)).ok());
}

#[divan::bench(sample_count = 10, sample_size = 10)]
fn scheduler_legacy_linear_16ops() {
    let tape = linear_chain(16);
    divan::black_box(run_to_done_legacy(&tape));
}

#[divan::bench(sample_count = 10, sample_size = 10)]
fn scheduler_wired_linear_16ops() {
    let tape = linear_chain(16);
    divan::black_box(run_to_done_wired(&tape));
}

fn main() {
    let start = Instant::now();
    divan::main();
    eprintln!("powl_quick_bench wall clock: {:?}", start.elapsed());
}
