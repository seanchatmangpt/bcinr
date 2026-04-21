// Universe64 subsystem benchmarks — Wave A through Wave E
//
// Covers all 18 new subsystems implemented in patterns/universe64/:
//   Wave A: coord_to_word_bit, BitTransitionIndex, TransitionWordIndex, IndexPlane
//   Wave B: LawKernel, AdmissionEvaluator, BoundaryKernel, ConformanceKernel, DriftKernel
//   Wave C: DeltaTape, DeltaBus
//   Wave D: ProjectionUpdater, ReadyMask, ScopePlanner
//   Wave E: RLKernel, PopcountHistogram, FixedHistogram, ScenarioRunner, UniverseExecutor

use bcinr_logic::patterns::universe64::{
    // Wave A
    coord_to_word_bit, BitTransitionIndex, TransitionWordIndex, IndexPlane,
    MAX_TRANSITIONS,
    // Wave B
    word_violation, LawKernel, cell_admit, eval_cell,
    BoundarySide, BoundaryKernel,
    ConformanceKernel, ConformanceState,
    DriftKernel, ExpectedModel,
    // Wave C
    DeltaTape, DeltaBus, SubscriberChannel,
    // Wave D
    ProjectionCache, ProjectionOp, ProjectionRegistry, ProjectionUpdater,
    ReadyMask,
    ScopePlanner,
    // Wave E
    RLKernel, RLState, RewardSpec, RewardTable,
    PopcountHistogram, FixedHistogram,
    ScenarioConfig, ScenarioRunner,
    UniverseExecutor,
    // Core
    UCoord, UDelta, UScope, UniverseBlock, CellMask,
    UNIVERSE_WORDS,
};
use bcinr_logic::patterns::universe64::law::{CellLaw, UniverseLaw};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ---------------------------------------------------------------------------
// Wave A — Geometry index
// ---------------------------------------------------------------------------

fn bench_coord_to_word_bit(c: &mut Criterion) {
    let coord = UCoord::new_const(7, 13, 42);
    c.bench_function("universe64/coord_to_word_bit", |b| {
        b.iter(|| coord_to_word_bit(black_box(coord)))
    });
}

fn bench_bit_transition_index_lookup(c: &mut Criterion) {
    let mut idx = BitTransitionIndex::new();
    idx.register(100, 3, 5);
    idx.register(100, 3, 7);
    let mut out = [0u16; 4];
    c.bench_function("universe64/BitTransitionIndex_lookup", |b| {
        b.iter(|| idx.lookup(black_box(100), black_box(3), &mut out))
    });
}

fn bench_transition_word_index_lookup(c: &mut Criterion) {
    let mut idx = TransitionWordIndex::default();
    idx.register(42, 10);
    idx.register(42, 20);
    c.bench_function("universe64/TransitionWordIndex_lookup", |b| {
        b.iter(|| idx.lookup(black_box(42)))
    });
}

fn bench_index_plane_lookup_all(c: &mut Criterion) {
    let mut plane = IndexPlane::default();
    for w in 0..8 {
        for b in 0..8 {
            plane.bit_transition.register(100 + w, b, w as u16);
        }
        plane.transition_word.register(w as usize, w * 4);
    }
    let mut tid_out = [0u16; 4];
    c.bench_function("universe64/IndexPlane_bit_lookup", |b| {
        b.iter(|| plane.bit_transition.lookup(black_box(100), black_box(0), &mut tid_out))
    });
}

// ---------------------------------------------------------------------------
// Wave B — Law / Admission / Boundary / Conformance / Drift
// ---------------------------------------------------------------------------

fn bench_word_violation(c: &mut Criterion) {
    c.bench_function("universe64/word_violation", |b| {
        b.iter(|| word_violation(black_box(0xDEADBEEFCAFEBABE), black_box(0xFFFFFFFF00000000)))
    });
}

fn bench_law_check_word(c: &mut Criterion) {
    let law = CellLaw { allowed: 0xFFFFFFFF00000000 };
    c.bench_function("universe64/CellLaw_check_word", |b| {
        b.iter(|| law.violation(black_box(0xDEADBEEFCAFEBABE)))
    });
}

fn bench_law_check_universe(c: &mut Criterion) {
    let block = UniverseBlock::new();
    let law = UniverseLaw::new_permissive();
    c.bench_function("universe64/LawKernel_check_universe", |b| {
        b.iter(|| LawKernel::check_universe(black_box(&block), black_box(&law)))
    });
}

fn bench_cell_admit(c: &mut Criterion) {
    c.bench_function("universe64/cell_admit", |b| {
        b.iter(|| cell_admit(black_box(0xFFFF000000000000), black_box(0xFFFF000000000000)))
    });
}

fn bench_eval_cell(c: &mut Criterion) {
    let block = UniverseBlock::new();
    c.bench_function("universe64/eval_cell", |b| {
        b.iter(|| eval_cell(black_box(&block), black_box(0), CellMask(black_box(0))))
    });
}

fn bench_boundary_execute(c: &mut Criterion) {
    let mut block = UniverseBlock::new();
    block.state[0] = 0xFF;
    let mut src = BoundarySide::empty();
    src.push(0, 0xFF);
    let mut dst = BoundarySide::empty();
    dst.push(1, 0);
    let blank = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 0, fired_mask: 0 };
    let mut out = [blank; 64];
    c.bench_function("universe64/BoundaryKernel_execute", |b| {
        b.iter(|| {
            BoundaryKernel::execute(
                black_box(&mut block),
                black_box(&src),
                black_box(&dst),
                black_box(!0u64),
                black_box(0u32),
                &mut out,
            )
        })
    });
}

fn bench_conformance_update_delta(c: &mut Criterion) {
    let law = [u64::MAX; UNIVERSE_WORDS];
    let mut state = ConformanceState::zero();
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 0b1111, fired_mask: !0 };
    c.bench_function("universe64/ConformanceKernel_update_delta", |b| {
        b.iter(|| ConformanceKernel::update_delta(black_box(&mut state), black_box(&delta), black_box(law[0])))
    });
}

fn bench_drift_scan_universe(c: &mut Criterion) {
    let block = UniverseBlock::new();
    let model = ExpectedModel::all_zero();
    c.bench_function("universe64/DriftKernel_scan_universe", |b| {
        b.iter(|| DriftKernel::scan_universe(black_box(&block), black_box(&model)))
    });
}

// ---------------------------------------------------------------------------
// Wave C — DeltaTape / DeltaBus
// ---------------------------------------------------------------------------

fn bench_delta_tape_append(c: &mut Criterion) {
    let mut tape = DeltaTape::new();
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 1, fired_mask: !0 };
    c.bench_function("universe64/DeltaTape_append", |b| {
        b.iter(|| tape.append(black_box(delta)))
    });
}

fn bench_delta_tape_replay(c: &mut Criterion) {
    let mut tape = DeltaTape::new();
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 1, fired_mask: !0 };
    for _ in 0..64 {
        tape.append(delta);
    }
    let start = tape.tape_position();
    c.bench_function("universe64/DeltaTape_replay_64", |b| {
        b.iter(|| {
            let mut n = 0u32;
            tape.replay_from(start.saturating_sub(64), |_d| n = n.wrapping_add(1));
            black_box(n)
        })
    });
}

fn bench_delta_bus_publish(c: &mut Criterion) {
    let mut bus = DeltaBus::new();
    bus.subscribe(SubscriberChannel::ReadyMask, |_| {});
    bus.subscribe(SubscriberChannel::RLKernel, |_| {});
    bus.subscribe(SubscriberChannel::ReceiptChain, |_| {});
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 1, fired_mask: !0 };
    c.bench_function("universe64/DeltaBus_publish_3subs", |b| {
        b.iter(|| bus.publish(black_box(&delta)))
    });
}

fn bench_delta_bus_publish_batch(c: &mut Criterion) {
    let mut bus = DeltaBus::new();
    bus.subscribe(SubscriberChannel::ReadyMask, |_| {});
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 1, fired_mask: !0 };
    let batch = [delta; 8];
    c.bench_function("universe64/DeltaBus_publish_batch_8", |b| {
        b.iter(|| bus.publish_batch(black_box(&batch)))
    });
}

// ---------------------------------------------------------------------------
// Wave D — Projection / ReadyMask / ScopePlanner
// ---------------------------------------------------------------------------

fn bench_projection_mark_dirty(c: &mut Criterion) {
    let mut cache = ProjectionCache::new();
    c.bench_function("universe64/ProjectionCache_mark_dirty", |b| {
        b.iter(|| cache.mark_dirty(black_box(7)))
    });
}

fn bench_projection_rebuild_dirty(c: &mut Criterion) {
    let mut block = UniverseBlock::new();
    for i in 0..UNIVERSE_WORDS { block.state[i] = i as u64; }
    let mut reg = ProjectionRegistry::new();
    reg.register(ProjectionOp::Or, &[0u16, 1, 2, 3]);
    let mut cache = ProjectionCache::new();
    c.bench_function("universe64/ProjectionUpdater_rebuild_dirty", |b| {
        b.iter(|| {
            cache.dirty = u64::MAX;
            ProjectionUpdater::rebuild_dirty(black_box(&block), black_box(&reg), black_box(&mut cache))
        })
    });
}

fn bench_ready_mask_set_clear(c: &mut Criterion) {
    let mut rm = ReadyMask::new();
    c.bench_function("universe64/ReadyMask_set_clear", |b| {
        b.iter(|| {
            rm.set_ready(black_box(42));
            rm.clear_ready(black_box(42));
        })
    });
}

fn bench_ready_mask_collect(c: &mut Criterion) {
    let mut rm = ReadyMask::new();
    for i in 0..MAX_TRANSITIONS { rm.set_ready(i); }
    let mut out = [0u16; 256];
    c.bench_function("universe64/ReadyMask_collect_ready", |b| {
        b.iter(|| rm.collect_ready(&mut out))
    });
}

fn bench_scope_planner_resolve(c: &mut Criterion) {
    let mut idx = TransitionWordIndex::default();
    idx.register(7, 10);
    idx.register(7, 20);
    idx.register(7, 30);
    let mut set = bcinr_logic::patterns::universe64::scratch::ActiveWordSet::new();
    c.bench_function("universe64/ScopePlanner_resolve", |b| {
        b.iter(|| ScopePlanner::resolve(black_box(7), black_box(&idx), &mut set))
    });
}

fn bench_scope_planner_classify(c: &mut Criterion) {
    c.bench_function("universe64/ScopePlanner_classify_scope", |b| {
        b.iter(|| ScopePlanner::classify_scope(black_box(4)))
    });
}

// ---------------------------------------------------------------------------
// Wave E — RL / Histogram / Scenario / Executor
// ---------------------------------------------------------------------------

fn bench_rl_update(c: &mut Criterion) {
    let mut table = RewardTable::new_neutral();
    table.set_word(0, RewardSpec { good_bits: 0xFF, bad_bits: 0 });
    let mut state = RLState::zero();
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 0xFF, fired_mask: !0 };
    c.bench_function("universe64/RLKernel_update", |b| {
        b.iter(|| RLKernel::update(black_box(&mut state), black_box(&delta), black_box(&table)))
    });
}

fn bench_rl_update_batch(c: &mut Criterion) {
    let mut table = RewardTable::new_neutral();
    table.set_word(0, RewardSpec { good_bits: 0xFF, bad_bits: 0 });
    let mut state = RLState::zero();
    let delta = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 0xFF, fired_mask: !0 };
    let batch = [delta; 64];
    c.bench_function("universe64/RLKernel_update_batch_64", |b| {
        b.iter(|| RLKernel::update_batch(black_box(&mut state), black_box(&batch), black_box(&table)))
    });
}

fn bench_popcount_histogram_observe(c: &mut Criterion) {
    let mut h = PopcountHistogram::new();
    c.bench_function("universe64/PopcountHistogram_observe", |b| {
        b.iter(|| h.observe(black_box(0xDEADBEEFCAFEBABE)))
    });
}

fn bench_popcount_histogram_domain(c: &mut Criterion) {
    let block = UniverseBlock::new();
    let mut h = PopcountHistogram::new();
    c.bench_function("universe64/PopcountHistogram_observe_domain", |b| {
        b.iter(|| h.observe_domain(black_box(&block), black_box(0)))
    });
}

fn bench_fixed_histogram_observe_block(c: &mut Criterion) {
    let block = UniverseBlock::new();
    let mut h = FixedHistogram::new(58);
    c.bench_function("universe64/FixedHistogram_observe_block", |b| {
        b.iter(|| h.observe_block(black_box(&block)))
    });
}

fn bench_fixed_histogram_mode(c: &mut Criterion) {
    let mut h = FixedHistogram::new(58);
    for i in 0..64u64 { for _ in 0..(i + 1) { h.observe(i << 58); } }
    c.bench_function("universe64/FixedHistogram_mode_bucket", |b| {
        b.iter(|| h.mode_bucket())
    });
}

fn bench_scenario_run(c: &mut Criterion) {
    let config = ScenarioConfig { seed: 0xABC, max_steps: 64, fired_mask: !0 };
    c.bench_function("universe64/ScenarioRunner_run_64steps", |b| {
        b.iter(|| ScenarioRunner::run_from_snapshot(black_box([0u64; UNIVERSE_WORDS]), black_box(&config)))
    });
}

fn bench_executor_execute_one(c: &mut Criterion) {
    let mut exec = UniverseExecutor::new();
    c.bench_function("universe64/UniverseExecutor_execute_one", |b| {
        b.iter(|| exec.execute_one(black_box(0), black_box(0), black_box(!0u64), black_box(0u32)))
    });
}

fn bench_executor_run_epoch(c: &mut Criterion) {
    let mut exec = UniverseExecutor::new();
    let instructions: [(usize, u32, u64, u32); 64] = core::array::from_fn(|i| (i % UNIVERSE_WORDS, (i % 64) as u32, !0u64, i as u32));
    c.bench_function("universe64/UniverseExecutor_run_epoch_64", |b| {
        b.iter(|| exec.run_epoch(black_box(&instructions)))
    });
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    wave_a,
    bench_coord_to_word_bit,
    bench_bit_transition_index_lookup,
    bench_transition_word_index_lookup,
    bench_index_plane_lookup_all,
);

criterion_group!(
    wave_b,
    bench_word_violation,
    bench_law_check_word,
    bench_law_check_universe,
    bench_cell_admit,
    bench_eval_cell,
    bench_boundary_execute,
    bench_conformance_update_delta,
    bench_drift_scan_universe,
);

criterion_group!(
    wave_c,
    bench_delta_tape_append,
    bench_delta_tape_replay,
    bench_delta_bus_publish,
    bench_delta_bus_publish_batch,
);

criterion_group!(
    wave_d,
    bench_projection_mark_dirty,
    bench_projection_rebuild_dirty,
    bench_ready_mask_set_clear,
    bench_ready_mask_collect,
    bench_scope_planner_resolve,
    bench_scope_planner_classify,
);

criterion_group!(
    wave_e,
    bench_rl_update,
    bench_rl_update_batch,
    bench_popcount_histogram_observe,
    bench_popcount_histogram_domain,
    bench_fixed_histogram_observe_block,
    bench_fixed_histogram_mode,
    bench_scenario_run,
    bench_executor_execute_one,
    bench_executor_run_epoch,
);

criterion_main!(wave_a, wave_b, wave_c, wave_d, wave_e);
