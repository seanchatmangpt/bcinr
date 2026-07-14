#![allow(unsafe_code)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use playground::{
    petri::petri_fire_transition,
    powl::{powl64_execute_step, Powl64Op, Powl64OpKind, PowlState},
    tekg::{compile_snapshot_chain, Tekg64Node, TekgLabel},
    wasm::{wasm_powl_execute_step, wasm_yawl_execute_task, WasmBYawlState, WasmPowlState},
    yawl::{BYawlEngine, BYawlTask, JoinType, SplitType},
};

fn bench_granular_yawl(c: &mut Criterion) {
    let mut group = c.benchmark_group("YAWL Granular Micro-Benchmarks");

    let mut engine = BYawlEngine::new();
    engine.state_mask = 0b1011;

    group.bench_function("trigger_event_bitwise", |b| {
        b.iter(|| {
            engine.trigger_event(black_box(0b1000));
        })
    });

    group.bench_function("spawn_instances_128bit_simd", |b| {
        b.iter(|| {
            engine.spawn_instances(black_box(14), black_box(5));
        })
    });

    let task = BYawlTask {
        id: 1,
        join_type: JoinType::AND,
        split_type: SplitType::OR,
        min_instances: 0,
        max_instances: 0,
        threshold_instances: 0,
        join_state_bit: 0,
        flags: 0,
        consume_mask: 0b0011,
        produce_mask: 0b0100,
        cancellation_mask: 0,
        condition_mask: 0,
        reset_mask: 0,
        reachability_mask: 0,
        interleaved_lock_mask: 0,
    };

    group.bench_function("execute_task_branchless", |b| {
        b.iter(|| {
            engine.execute_task_branchless(black_box(&task));
        })
    });

    group.finish();
}

fn bench_granular_powl(c: &mut Criterion) {
    let mut group = c.benchmark_group("POWL Granular Micro-Benchmarks");

    let mut state = PowlState::new();
    state.completed_ops = 1 << 3;
    let op = Powl64Op {
        kind: Powl64OpKind::PartialOrderGate,
        lane: 0,
        activity: 0,
        scope: 0,
        branch: 0,
        loop_id: 0,
        pred_mask: 1 << 3,
        succ_mask: 1 << 4,
        ctrl_mask: 0,
        intensity: 0,
        _pad: [0; 7],
    };

    group.bench_function("execute_step_swar", |b| {
        b.iter(|| {
            powl64_execute_step(black_box(&mut state), black_box(&op), black_box(0), black_box(0));
        })
    });

    group.finish();
}

fn bench_wasm_ffi_boundaries(c: &mut Criterion) {
    let mut group = c.benchmark_group("WASM Zero-Latency FFI Boundaries");

    let mut powl_state = WasmPowlState {
        completed_ops: 0,
        completed_branches: 0,
        active_scopes: 0,
        scope_stack: [0; 16],
        stack_depth: 0,
        completed_loops: 0,
    };
    let op = Powl64Op {
        kind: Powl64OpKind::PartialOrderGate,
        lane: 0,
        activity: 0,
        scope: 0,
        branch: 0,
        loop_id: 0,
        pred_mask: 1 << 3,
        succ_mask: 1 << 4,
        ctrl_mask: 0,
        intensity: 0,
        _pad: [0; 7],
    };

    group.bench_function("wasm_powl_null_redirect", |b| {
        b.iter(|| {
            unsafe {
                wasm_powl_execute_step(
                    black_box(&mut powl_state as *mut WasmPowlState),
                    black_box(&op as *const Powl64Op),
                    black_box(0), // No overrides
                    black_box(0),
                )
            };
        })
    });

    let mut yawl_state = WasmBYawlState {
        state_mask: 0,
        active_triggers: 0,
        fired_joins_mask: 0,
        active_locks: 0,
        active_instances: [0; 64],
    };
    let task = BYawlTask {
        id: 1,
        join_type: JoinType::AND,
        split_type: SplitType::OR,
        min_instances: 0,
        max_instances: 0,
        threshold_instances: 0,
        join_state_bit: 0,
        flags: 0,
        consume_mask: 0b0011,
        produce_mask: 0b0100,
        cancellation_mask: 0,
        condition_mask: 0,
        reset_mask: 0,
        reachability_mask: 0,
        interleaved_lock_mask: 0,
    };

    group.bench_function("wasm_yawl_null_redirect", |b| {
        b.iter(|| {
            unsafe {
                wasm_yawl_execute_task(
                    black_box(&mut yawl_state as *mut WasmBYawlState),
                    black_box(&task as *const BYawlTask),
                )
            };
        })
    });

    group.finish();
}

fn bench_tekg_compiler(c: &mut Criterion) {
    let mut group = c.benchmark_group("TEKG Ontology Compiler");

    let mut out = [Tekg64Node {
        timestamp_ns: 0,
        rel_mask: 0,
        node_id: 0,
        parent_id: 0,
        prev_snapshot_id: 0,
        label: TekgLabel::Log,
        _pad: [0; 41],
    }; 100];

    // A synthetic update vector of 99 timestamps
    let timestamps = [100; 99];

    group.bench_function("compile_snapshot_chain_100_nodes", |b| {
        b.iter(|| {
            compile_snapshot_chain(black_box(1), black_box(&timestamps), black_box(&mut out))
                .unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_granular_yawl,
    bench_granular_powl,
    bench_wasm_ffi_boundaries,
    bench_tekg_compiler
);
criterion_main!(benches);
