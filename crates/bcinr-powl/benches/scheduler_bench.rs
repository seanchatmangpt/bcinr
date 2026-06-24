use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use bcinr_powl::{
    compiler::{compile_powl, PowlAstNode},
    scheduler::{scheduler_tick, PowlRunState},
    scheduler_wired::{petri_tick, PowlPetriState, FiberPool},
    tape::PowlTape,
};

// ---------------------------------------------------------------------------
// Helpers — shared tape builders
// ---------------------------------------------------------------------------

fn linear_chain(n: usize) -> PowlTape {
    let nodes: Vec<_> = (0..n).map(|i| {
        PowlAstNode::Atom(Box::leak(format!("op{i}").into_boxed_str()))
    }).collect();
    compile_powl(&PowlAstNode::Sequence(nodes)).expect("linear compile")
}

fn parallel_spo(n: usize) -> PowlTape {
    let children: Vec<_> = (0..n).map(|i| {
        PowlAstNode::Atom(Box::leak(format!("p{i}").into_boxed_str()))
    }).collect();
    compile_powl(&PowlAstNode::PartialOrder { children, edges: vec![] })
        .expect("parallel compile")
}

fn run_to_done_legacy(tape: &PowlTape) -> u32 {
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    let mut state = PowlRunState::new(tape);
    let mut fired = 0u32;
    while state.check_mask != 0 {
        let fs = scheduler_tick(&ops, &mut state);
        fired += fs.0.count_ones();
    }
    fired
}

fn run_to_done_wired(tape: &PowlTape) -> u32 {
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    let mut state = PowlPetriState::new(tape.entry_mask);
    let mut fired = 0u32;
    while state.check.words[0] != 0 {
        fired += petri_tick(&ops, &mut state, None).count_ones() as u32;
    }
    fired
}

// ---------------------------------------------------------------------------
// GROUP 1: Legacy scheduler — baseline (original branchless bit-scan)
// ---------------------------------------------------------------------------

fn bench_legacy_linear_scaling(c: &mut Criterion) {
    let mut g = c.benchmark_group("legacy/linear_chain");
    for n in [1usize, 2, 4, 8, 16, 32, 64] {
        let tape = linear_chain(n);
        let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(tape, ops), |b, (tape, ops)| {
            b.iter(|| {
                let mut state = PowlRunState::new(tape);
                let mut fired = 0u32;
                while state.check_mask != 0 {
                    fired += scheduler_tick(ops, &mut state).0.count_ones();
                }
                criterion::black_box(fired)
            })
        });
    }
    g.finish();
}

fn bench_legacy_parallel_scaling(c: &mut Criterion) {
    let mut g = c.benchmark_group("legacy/parallel_spo");
    for n in [2usize, 4, 8, 16, 32] {
        let tape = parallel_spo(n);
        let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
        g.throughput(Throughput::Elements((n + 1) as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(tape, ops), |b, (tape, ops)| {
            b.iter(|| {
                let mut state = PowlRunState::new(tape);
                let mut fired = 0u32;
                while state.check_mask != 0 {
                    fired += scheduler_tick(ops, &mut state).0.count_ones();
                }
                criterion::black_box(fired)
            })
        });
    }
    g.finish();
}

fn bench_legacy_single_tick(c: &mut Criterion) {
    let tape = linear_chain(1);
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    c.bench_function("legacy/single_tick_1op", |b| {
        b.iter(|| {
            let mut state = PowlRunState::new(&tape);
            criterion::black_box(scheduler_tick(&ops, &mut state).0)
        })
    });
}

fn bench_legacy_xor_choice(c: &mut Criterion) {
    let ast = PowlAstNode::XorChoice(vec![
        PowlAstNode::Atom("branch0"), PowlAstNode::Atom("branch1"),
        PowlAstNode::Atom("branch2"), PowlAstNode::Atom("branch3"),
    ]);
    let tape = compile_powl(&ast).expect("xor compile");
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    c.bench_function("legacy/xor_choice_4branches", |b| {
        b.iter(|| {
            let mut state = PowlRunState::new(&tape);
            let mut fired = 0u32;
            while state.check_mask != 0 {
                fired += scheduler_tick(&ops, &mut state).0.count_ones();
            }
            criterion::black_box(fired)
        })
    });
}

fn bench_legacy_throughput_10k(c: &mut Criterion) {
    let tape = linear_chain(10);
    c.bench_function("legacy/10k_instances_linear_10ops", |b| {
        b.iter(|| {
            let mut total = 0u32;
            for _ in 0..10_000 { total += run_to_done_legacy(&tape); }
            criterion::black_box(total)
        })
    });
}

// ---------------------------------------------------------------------------
// GROUP 2: Wired scheduler — PriorityPetriEngine + TimeWheel hot path
// ---------------------------------------------------------------------------

fn bench_wired_linear_scaling(c: &mut Criterion) {
    let mut g = c.benchmark_group("wired/linear_chain");
    for n in [1usize, 2, 4, 8, 16, 32, 64] {
        let tape = linear_chain(n);
        let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(tape, ops), |b, (tape, ops)| {
            b.iter(|| {
                let mut state = PowlPetriState::new(tape.entry_mask);
                let mut fired = 0u32;
                while state.check.words[0] != 0 {
                    fired += petri_tick(ops, &mut state, None).count_ones() as u32;
                }
                criterion::black_box(fired)
            })
        });
    }
    g.finish();
}

fn bench_wired_parallel_scaling(c: &mut Criterion) {
    let mut g = c.benchmark_group("wired/parallel_spo");
    for n in [2usize, 4, 8, 16, 32] {
        let tape = parallel_spo(n);
        let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
        g.throughput(Throughput::Elements((n + 1) as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(tape, ops), |b, (tape, ops)| {
            b.iter(|| {
                let mut state = PowlPetriState::new(tape.entry_mask);
                let mut fired = 0u32;
                while state.check.words[0] != 0 {
                    fired += petri_tick(ops, &mut state, None).count_ones() as u32;
                }
                criterion::black_box(fired)
            })
        });
    }
    g.finish();
}

fn bench_wired_single_tick(c: &mut Criterion) {
    let tape = linear_chain(1);
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    c.bench_function("wired/single_tick_1op", |b| {
        b.iter(|| {
            let mut state = PowlPetriState::new(tape.entry_mask);
            criterion::black_box(petri_tick(&ops, &mut state, None))
        })
    });
}

fn bench_wired_xor_choice(c: &mut Criterion) {
    let ast = PowlAstNode::XorChoice(vec![
        PowlAstNode::Atom("b0"), PowlAstNode::Atom("b1"),
        PowlAstNode::Atom("b2"), PowlAstNode::Atom("b3"),
    ]);
    let tape = compile_powl(&ast).expect("xor compile");
    let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
    c.bench_function("wired/xor_choice_4branches", |b| {
        b.iter(|| {
            let mut state = PowlPetriState::new(tape.entry_mask);
            let mut fired = 0u32;
            while state.check.words[0] != 0 {
                fired += petri_tick(&ops, &mut state, None).count_ones() as u32;
            }
            criterion::black_box(fired)
        })
    });
}

fn bench_wired_throughput_10k(c: &mut Criterion) {
    let tape = linear_chain(10);
    c.bench_function("wired/10k_instances_linear_10ops", |b| {
        b.iter(|| {
            let mut total = 0u32;
            for _ in 0..10_000 { total += run_to_done_wired(&tape); }
            criterion::black_box(total)
        })
    });
}

// ---------------------------------------------------------------------------
// GROUP 3: TimeWheel<256> — isolated SLA deadline scheduling
// ---------------------------------------------------------------------------

fn bench_time_wheel_tick(c: &mut Criterion) {
    use bcinr_logic::patterns::time_wheel::TimeWheel;
    let mut g = c.benchmark_group("primitives/time_wheel");

    // Tick overhead with no scheduled events
    g.bench_function("tick_empty_wheel", |b| {
        let mut wheel = TimeWheel::<256>::new();
        b.iter(|| criterion::black_box(wheel.tick()))
    });

    // Tick overhead with 64 events scheduled at varying offsets
    g.bench_function("tick_64_events_scheduled", |b| {
        b.iter_batched(
            || {
                let mut wheel = TimeWheel::<256>::new();
                for i in 0u32..64 {
                    wheel.schedule((i as usize % 250) + 1, i);
                }
                wheel
            },
            |mut wheel| {
                // Advance 256 ticks to drain all slots
                let mut acc = 0u64;
                for _ in 0..256 { acc |= wheel.tick(); }
                criterion::black_box(acc)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // schedule() call overhead
    g.bench_function("schedule_single_event", |b| {
        let mut wheel = TimeWheel::<256>::new();
        let mut t = 1usize;
        b.iter(|| {
            wheel.schedule(t % 255 + 1, 0u32);
            t += 1;
            criterion::black_box(wheel.tick())
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// GROUP 4: LockFreeMpmcRing — push/pop CAS throughput
// ---------------------------------------------------------------------------

fn bench_mpmc_ring(c: &mut Criterion) {
    use bcinr_logic::patterns::deterministic_mpmc::LockFreeMpmcRing;
    use bcinr_powl::scheduler_wired::WorkItem;

    let mut g = c.benchmark_group("primitives/mpmc_ring");

    g.bench_function("push_pop_roundtrip", |b| {
        let ring = LockFreeMpmcRing::<WorkItem, 64>::new_checked().unwrap();
        let item = WorkItem { op_idx: 7, succ_mask: 0xFF };
        b.iter(|| {
            ring.push_t1(item);
            criterion::black_box(ring.pop_t1())
        })
    });

    // Saturate ring: 64 pushes then 64 pops
    g.throughput(Throughput::Elements(64));
    g.bench_function("push64_pop64_throughput", |b| {
        let ring = LockFreeMpmcRing::<WorkItem, 64>::new_checked().unwrap();
        let item = WorkItem { op_idx: 0, succ_mask: 0 };
        b.iter(|| {
            for i in 0u32..64 { ring.push_t1(WorkItem { op_idx: i, succ_mask: 0 }); }
            let mut acc = 0u32;
            for _ in 0u32..64 {
                if let (Some(w), _) = ring.pop_t1() { acc += w.op_idx; }
            }
            criterion::black_box(acc)
        })
    });

    // Petri tick WITH ring dispatch vs without
    g.bench_function("petri_tick_with_ring_parallel_8", |b| {
        let tape = parallel_spo(8);
        let ops: Vec<_> = tape.ops[..tape.len as usize].to_vec();
        let ring = LockFreeMpmcRing::<WorkItem, 64>::new_checked().unwrap();
        b.iter(|| {
            let mut state = PowlPetriState::new(tape.entry_mask);
            let mut fired = 0u32;
            while state.check.words[0] != 0 {
                fired += petri_tick(&ops, &mut state, Some(&ring)).count_ones() as u32;
            }
            criterion::black_box(fired)
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// GROUP 5: WcetFiber — pool claim/advance/release
// ---------------------------------------------------------------------------

fn bench_fiber_pool(c: &mut Criterion) {
    let mut g = c.benchmark_group("primitives/fiber_pool");

    // Single fiber advance (TICKS=8)
    g.bench_function("fiber_pool_4slots_advance_8ticks", |b| {
        let events = [1u32; 8];
        b.iter_batched(
            || {
                let mut pool = FiberPool::<4, 8>::new();
                pool.claim(0).unwrap();
                pool.claim(1).unwrap();
                pool
            },
            |mut pool| criterion::black_box(pool.advance_all(&events)),
            criterion::BatchSize::SmallInput,
        )
    });

    // Claim + advance + release cycle
    g.bench_function("claim_advance_release_cycle", |b| {
        let events = [1u32; 8];
        let mut pool = FiberPool::<8, 8>::new();
        b.iter(|| {
            if let Some(slot) = pool.claim(42) {
                pool.advance_all(&events);
                criterion::black_box(pool.release(slot));
            }
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// GROUP 6: PriorityPetriEngine — isolated primitive throughput
// ---------------------------------------------------------------------------

fn bench_petri_engine(c: &mut Criterion) {
    use bcinr_logic::{patterns::swar_petri::PriorityPetriEngine, models::petri::KBitSet};
    let mut g = c.benchmark_group("primitives/petri_engine");

    // 8-transition engine (typical SPO fan-out)
    g.bench_function("step_8_transitions", |b| {
        let initial = KBitSet::<1> { words: [0b1111_1111] };
        let inputs: [KBitSet<1>; 8] = std::array::from_fn(|i| KBitSet { words: [1u64 << i] });
        let outputs: [KBitSet<1>; 8] = std::array::from_fn(|i| KBitSet { words: [(1u64 << i) | (1u64 << ((i + 1) % 8))] });
        b.iter(|| {
            let mut engine = PriorityPetriEngine::<1, 8>::new_checked(initial, inputs, outputs).unwrap();
            criterion::black_box(engine.step())
        })
    });

    // 64-transition max capacity
    g.throughput(Throughput::Elements(64));
    g.bench_function("step_64_transitions_max", |b| {
        let initial = KBitSet::<1> { words: [!0u64] };
        let inputs: [KBitSet<1>; 64] = std::array::from_fn(|i| KBitSet { words: [1u64.wrapping_shl(i as u32)] });
        let outputs: [KBitSet<1>; 64] = std::array::from_fn(|i| KBitSet { words: [1u64.wrapping_shl(i as u32)] });
        b.iter(|| {
            let mut engine = PriorityPetriEngine::<1, 64>::new_checked(initial, inputs, outputs).unwrap();
            criterion::black_box(engine.step())
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// GROUP 7: scan/bitset primitives — bulk check_mask propagation
// ---------------------------------------------------------------------------

fn bench_bulk_propagation(c: &mut Criterion) {
    use bcinr_logic::{scan::prefix_xor_u64x8, bitset::union_u64_slices};
    let mut g = c.benchmark_group("primitives/bulk_propagation");

    g.bench_function("prefix_xor_u64x8", |b| {
        let words = [0xDEAD_BEEF_u64; 8];
        b.iter(|| criterion::black_box(prefix_xor_u64x8(words)))
    });

    g.throughput(Throughput::Elements(8));
    g.bench_function("union_u64_slices_8words", |b| {
        let mut a = [0x5555_5555_u64; 8];
        let b_arr = [0xAAAA_AAAA_u64; 8];
        b.iter(|| {
            union_u64_slices(&mut a, &b_arr);
            criterion::black_box(a[0])
        })
    });

    // propagate_check_mask_large end-to-end
    g.bench_function("propagate_check_mask_large_8fired", |b| {
        use bcinr_powl::scheduler_wired::propagate_check_mask_large;
        let succ_table: Vec<[u64; 8]> = (0..64).map(|i: usize| {
            let mut w = [0u64; 8];
            w[0] = 1u64 << ((i + 1) % 64);
            w
        }).collect();
        let fired_words = [0xFF_u64, 0, 0, 0, 0, 0, 0, 0]; // ops 0-7 fired
        let done = [0xFF_u64, 0u64, 0, 0, 0, 0, 0, 0];
        b.iter(|| {
            let mut check = [0u64; 8];
            propagate_check_mask_large(fired_words, &succ_table, &mut check, &done);
            criterion::black_box(check[0])
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// GROUP 8: Max tape + loop stress
// ---------------------------------------------------------------------------

fn bench_stress(c: &mut Criterion) {
    let mut g = c.benchmark_group("stress");

    // 64-op linear — legacy
    let tape64 = linear_chain(64);
    g.bench_function("legacy/linear_64ops", |b| {
        b.iter(|| criterion::black_box(run_to_done_legacy(&tape64)))
    });

    // 64-op linear — wired
    g.bench_function("wired/linear_64ops", |b| {
        b.iter(|| criterion::black_box(run_to_done_wired(&tape64)))
    });

    // Loop 3 iters — legacy
    let loop_ast = PowlAstNode::Loop {
        body: Box::new(PowlAstNode::Atom("work")),
        redo: Box::new(PowlAstNode::Atom("check")),
    };
    let loop_tape = compile_powl(&loop_ast).unwrap();
    let loop_ops: Vec<_> = loop_tape.ops[..loop_tape.len as usize].to_vec();
    g.bench_function("legacy/loop_3iter", |b| {
        b.iter(|| {
            let mut state = PowlRunState::new(&loop_tape);
            let mut fired = 0u32;
            let mut body = 0u32;
            while state.check_mask != 0 && body < 3 {
                let fs = scheduler_tick(&loop_ops, &mut state);
                if fs.0 & 1 != 0 { body += 1; }
                fired += fs.0.count_ones();
                if body >= 3 { break; }
            }
            criterion::black_box(fired)
        })
    });

    g.finish();
}

criterion_group!(
    benches,
    // Legacy baseline
    bench_legacy_linear_scaling,
    bench_legacy_parallel_scaling,
    bench_legacy_single_tick,
    bench_legacy_xor_choice,
    bench_legacy_throughput_10k,
    // Wired scheduler
    bench_wired_linear_scaling,
    bench_wired_parallel_scaling,
    bench_wired_single_tick,
    bench_wired_xor_choice,
    bench_wired_throughput_10k,
    // Isolated primitives
    bench_time_wheel_tick,
    bench_mpmc_ring,
    bench_fiber_pool,
    bench_petri_engine,
    bench_bulk_propagation,
    // Stress
    bench_stress,
);
criterion_main!(benches);
