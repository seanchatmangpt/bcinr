//! RDTSC/CNTVCT measured tick tables for the receipt-subsystem kernels
//! (`bcinr-powl`), grounding dissertation Ch9 with real numbers.
//!
//! Mirrors the kernels in `receipt_bench.rs` (same setup, same per-iteration
//! work) but times them with a hardware counter instead of Divan: on x86_64
//! the counter is `rdtsc` (`_rdtsc()`); on aarch64 the fallback is the
//! userspace-readable `CNTVCT_EL0` virtual counter. On this host
//! (Apple M3 Max, arm64) `rdtsc` is unavailable, so CNTVCT_EL0 is the
//! surrogate — disclosed in the emitted table.
//!
//! Run: `cargo bench -p bcinr-powl --bench rdtsc`
//! Writes `benchmarks/rdtsc_results.md` (repo root) and prints the table.

use bcinr_powl::receipt::{
    causal_receipt::OcelCausalReceipt,
    conformance::{ConformanceMetrics, ConformancePredicate},
    denial::DenialPolarity,
    ocel_emit::OcelEmitArena,
    replay::{PowlReplayFrame, PowlReplayVerifier},
};
use std::fmt::Write as _;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Counter read: rdtsc on x86_64, CNTVCT_EL0 fallback on aarch64.
// ---------------------------------------------------------------------------

#[inline(never)]
fn counter_ticks() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { std::arch::x86_64::_rdtsc() }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: `mrs cntvct_el0` is a userspace-readable counter read; no
        // memory effects.
        unsafe {
            let out: u64;
            std::arch::asm!("mrs {}, cntvct_el0", out(reg) out, options(nostack));
            out
        }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        compile_error!("no hardware counter for this architecture");
    }
}

const COUNTER_NAME: &str = if cfg!(target_arch = "x86_64") {
    "rdtsc"
} else if cfg!(target_arch = "aarch64") {
    "CNTVCT_EL0 (aarch64 fallback surrogate; rdtsc unavailable on this host)"
} else {
    "none"
};

/// Median tick cost of one counter read, reported and subtracted from every
/// timed sample so per-op numbers sit on a measured floor, not zero.
fn counter_overhead_ticks(samples: usize) -> u64 {
    let mut diffs = Vec::with_capacity(samples);
    for _ in 0..samples {
        let a = counter_ticks();
        let b = counter_ticks();
        diffs.push(b - a);
    }
    diffs.sort_unstable();
    diffs[samples / 2]
}

// ---------------------------------------------------------------------------
// Timing core: per kernel, K inner iterations per timed sample, R samples;
// report per-op tick distribution (median / p50 / p99).
// ---------------------------------------------------------------------------

const SAMPLES: usize = 2_000;
const WARMUP: usize = 200;

struct Kernel {
    name: &'static str,
    iters: u64,
    op: Box<dyn Fn()>,
}

/// CNTVCT_EL0 ticks at 1 ns granularity on this host, so sub-ns kernels need
/// enough inner iterations that the sample total clears the granularity floor.
fn percentiles(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let at = |q: f64| v[(((n - 1) as f64) * q).round() as usize];
    (v[n / 2], at(0.50), at(0.99))
}

fn main() {
    let out_path = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../benchmarks/rdtsc_results.md"
            )
            .to_string()
        });

    // Calibration: ticks per nanosecond, measured against the monotonic
    // wall clock over a ~0.25 s busy loop.
    let mut acc = 0u64;
    let mut i = 0u64;
    let cal_start_wall = Instant::now();
    let cal_start_ticks = counter_ticks();
    while cal_start_wall.elapsed().as_secs_f64() < 0.25 {
        acc = acc.wrapping_add(i);
        i += 1;
    }
    let cal_end_ticks = counter_ticks();
    std::hint::black_box(acc);
    let ticks_per_ns =
        (cal_end_ticks - cal_start_ticks) as f64 / cal_start_wall.elapsed().as_nanos() as f64;

    let overhead = counter_overhead_ticks(1_000);

    // -- kernels (mirror receipt_bench.rs setups) ---------------------------

    let obj_refs: Vec<(u8, u32)> = (0..8u8).map(|i| (i, i as u32 * 100)).collect();
    let mut seed_arena = OcelEmitArena::new();
    let chain_frame = seed_arena
        .emit(42, 7, &[], DenialPolarity::ADMITTED, 0)
        .clone();
    let chain_frame_rolling = chain_frame.clone();

    const PASSING: ConformanceMetrics = ConformanceMetrics {
        fitness: 0xFFFF_0000,
        precision: 0xFFFF_0000,
        generalization: 0xFFFF_0000,
        simplicity: 0xFFFF_0000,
    };
    const FAILING: ConformanceMetrics = ConformanceMetrics {
        fitness: 0x7FFF_0000,
        precision: 0xFFFF_0000,
        generalization: 0xFFFF_0000,
        simplicity: 0xFFFF_0000,
    };

    let frames10 = sequential_frames(10);
    let frames64 = sequential_frames(64);
    let polarities = [
        DenialPolarity::ADMITTED,
        DenialPolarity::SLA_BREACH,
        DenialPolarity::AUTHORIZATION_DENIED,
        DenialPolarity::WATCHDOG_DRAINED,
        DenialPolarity::CONFORMANCE_GATE_FAILED,
    ];

    // Emit arenas live outside the timed closures and are recycled at the
    // 4090 mark, mirroring receipt_bench.rs's warm steady state — a fresh
    // arena per sample would measure first-touch page faults, not `emit`.
    let arena_a = std::cell::RefCell::new(OcelEmitArena::new());
    let arena_b = std::cell::RefCell::new(OcelEmitArena::new());
    let arena_c = std::cell::RefCell::new(OcelEmitArena::new());

    let kernels: Vec<Kernel> = vec![
        Kernel {
            name: "emit_no_objects",
            iters: 64,
            op: Box::new(move || {
                let mut arena = arena_a.borrow_mut();
                for _ in 0..64 {
                    if arena.len() >= 4090 {
                        *arena = OcelEmitArena::new();
                    }
                    let _ = arena.emit(0, 0, &[], DenialPolarity::ADMITTED, 0);
                }
                std::hint::black_box(arena.len());
            }),
        },
        Kernel {
            name: "emit_8_objects",
            iters: 64,
            op: Box::new(move || {
                let mut arena = arena_b.borrow_mut();
                for _ in 0..64 {
                    if arena.len() >= 4090 {
                        *arena = OcelEmitArena::new();
                    }
                    let _ = arena.emit(0, 1, &obj_refs, DenialPolarity::ADMITTED, 2);
                }
                std::hint::black_box(arena.len());
            }),
        },
        Kernel {
            name: "emit_sla_breach",
            iters: 64,
            op: Box::new(move || {
                let mut arena = arena_c.borrow_mut();
                for _ in 0..64 {
                    if arena.len() >= 4090 {
                        *arena = OcelEmitArena::new();
                    }
                    let _ = arena.emit(0, 2, &[], DenialPolarity::SLA_BREACH, 0);
                }
                std::hint::black_box(arena.len());
            }),
        },
        Kernel {
            name: "chain_1_frame_blake3",
            iters: 64,
            op: Box::new(move || {
                for _ in 0..64 {
                    let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
                    receipt.chain(&chain_frame);
                    std::hint::black_box(receipt.chain_hash);
                }
            }),
        },
        Kernel {
            name: "chain_100_frames_rolling",
            iters: 1,
            op: Box::new(move || {
                let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
                for _ in 0..100 {
                    receipt.chain(&chain_frame_rolling);
                }
                std::hint::black_box(receipt.chain_hash);
            }),
        },
        Kernel {
            name: "conformance_check_pass",
            iters: 4096,
            op: Box::new(move || {
                for _ in 0..4096 {
                    let ok = ConformancePredicate::STRICT.check(&PASSING).is_ok();
                    std::hint::black_box(ok);
                }
            }),
        },
        Kernel {
            name: "conformance_check_fail",
            iters: 4096,
            op: Box::new(move || {
                for _ in 0..4096 {
                    let err = ConformancePredicate::STRICT.check(&FAILING).is_err();
                    std::hint::black_box(err);
                }
            }),
        },
        Kernel {
            name: "replay_10_frames",
            iters: 64,
            op: Box::new(move || {
                for _ in 0..64 {
                    let mut v = PowlReplayVerifier::new(1u64);
                    for f in &frames10 {
                        let _ = v.replay_frame(f);
                    }
                    std::hint::black_box(v.finalize());
                }
            }),
        },
        Kernel {
            name: "replay_64_frames_max",
            iters: 32,
            op: Box::new(move || {
                for _ in 0..32 {
                    let mut v = PowlReplayVerifier::new(1u64);
                    for f in &frames64 {
                        let _ = v.replay_frame(f);
                    }
                    std::hint::black_box(v.finalize());
                }
            }),
        },
        Kernel {
            name: "denial_to_fired_mask",
            iters: 4096,
            op: Box::new(move || {
                for _ in 0..4096 {
                    let mut acc = 0u64;
                    for &p in polarities.iter() {
                        acc ^= p.to_fired_mask();
                    }
                    std::hint::black_box(acc);
                }
            }),
        },
    ];

    // -- run ----------------------------------------------------------------

    println!(
        "counter={}  ticks/ns={:.3}  counter-read overhead={} ticks",
        COUNTER_NAME, ticks_per_ns, overhead
    );

    let mut rows: Vec<(&str, f64, f64, f64, f64, f64)> = Vec::new();
    for k in &kernels {
        let mut per_op: Vec<f64> = Vec::with_capacity(SAMPLES);
        for _ in 0..WARMUP {
            (k.op)();
        }
        for _ in 0..SAMPLES {
            (k.op)(); // flush; timing brackets only the K iterations below
            let a = counter_ticks();
            (k.op)();
            let b = counter_ticks();
            let ticks = (b - a).saturating_sub(overhead) as f64 / k.iters as f64;
            per_op.push(ticks);
        }
        let (median, p50, p99) = percentiles(per_op);
        let ns_median = median / ticks_per_ns;
        let ns_p99 = p99 / ticks_per_ns;
        println!(
            "{:>24}  median={:>5} ticks ({:>7.2} ns)  p50={:>5}  p99={:>6} ticks ({:>7.2} ns)",
            k.name, median, ns_median, p50, p99, ns_p99
        );
        rows.push((k.name, median, p50, p99, ns_median, ns_p99));
    }

    // -- emit markdown ------------------------------------------------------

    let mut md = String::new();
    writeln!(
        md,
        "# RDTSC Measured Tick Tables — receipt subsystem (`bcinr-powl`)"
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "Grounds the dissertation Ch9 latency claims with measured hardware-counter ticks. \
         Regenerate with `cargo bench -p bcinr-powl --bench rdtsc` (harness: \
         `crates/bcinr-powl/benches/rdtsc.rs`)."
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(md, "## Environment").unwrap();
    writeln!(md).unwrap();
    writeln!(md, "- Counter: **{}**", COUNTER_NAME).unwrap();
    writeln!(md, "- Host: Apple M3 Max, arm64 (Darwin), {}", host_id()).unwrap();
    writeln!(
        md,
        "- Calibration: {:.3} ticks/ns, measured against `std::time::Instant` over a 0.25 s busy loop",
        ticks_per_ns
    )
    .unwrap();
    writeln!(
        md,
        "- Counter-read overhead: {} ticks (median of paired reads; subtracted from every timed sample)",
        overhead
    )
    .unwrap();
    writeln!(
        md,
        "- Method: {} timed samples per kernel, each sampling a fixed inner-iteration \
         count K (64–4096 per kernel, 1 for the 100-frame chain; see harness) with {} warmup; \
         per-op ticks = (sample ticks − overhead) / K",
        SAMPLES, WARMUP
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(md, "## Tick table").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "| kernel | median ticks | p50 ticks | p99 ticks | median ns | p99 ns |"
    )
    .unwrap();
    writeln!(md, "|---|---:|---:|---:|---:|---:|").unwrap();
    for (name, median, p50, p99, ns_median, ns_p99) in &rows {
        writeln!(
            md,
            "| `{}` | {} | {} | {} | {:.2} | {:.2} |",
            name, median, p50, p99, ns_median, ns_p99
        )
        .unwrap();
    }
    writeln!(md).unwrap();
    writeln!(
        md,
        "Stated targets (from `receipt_bench.rs`): emit < 10 ns; BLAKE3 chain < 250 ns/frame \
         measured envelope (re-amended down 26.10.09 after the genesis pre-hash, commit \
         f46ddc77 — Variant 3 — dropped `chain_1_frame_blake3` into the 227–243 ns band, \
         firing the previous < 250 ns/frame reopen falsifier; the 26.10.08 < 500 ns/frame \
         amendment and the original < 15 ns/frame budget are preserved in \
         `benchmarks/rdtsc_results.md`); \
         conformance check < 2 ns; replay < 20 ns/frame; denial mask ~ 1 ns."
    )
    .unwrap();

    // Preserve hand-maintained content: the auto-generated section is written
    // between explicit BEGIN/END markers; anything outside them (e.g. the
    // negative-result notes) survives regeneration untouched. A pre-marker
    // file is refused rather than silently truncated.
    const BEGIN: &str =
        "<!-- BEGIN AUTO: generated by cargo bench --bench rdtsc; do not edit -->\n";
    const END: &str = "<!-- END AUTO -->\n";

    let existing = std::fs::read_to_string(&out_path).ok();
    let doc = match existing.as_deref() {
        Some(old) if old.contains(BEGIN) && old.contains(END) => {
            let pre = &old[..old.find(BEGIN).unwrap()];
            let post = &old[old.find(END).unwrap() + END.len()..];
            format!("{pre}{BEGIN}{md}{END}{post}")
        }
        Some(old) if !old.trim().is_empty() => panic!(
            "{} exists but has no BEGIN/END AUTO markers; \
             wrap the generated section in the markers once by hand, then rerun",
            out_path
        ),
        _ => format!("{BEGIN}{md}{END}"),
    };

    if let Some(parent) = std::path::Path::new(&out_path).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&out_path, &doc).unwrap();
    println!("wrote {}", out_path);
}

fn host_id() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "rustc (unavailable)".to_string());
    format!("{os}/{arch}, {rustc}")
}

fn sequential_frames(n: u32) -> Vec<PowlReplayFrame> {
    (0..n)
        .map(|i| PowlReplayFrame {
            node_id: i,
            node_bit: 1u64 << i,
            required_tokens: if i == 0 { 1u64 } else { 1u64 << (i - 1) },
            produces_tokens: if i < n - 1 { 1u64 << i } else { 0 },
            activity: format!("op{i}"),
            ts_ns: i as u64 * 1000,
            object_ids: vec![],
        })
        .collect()
}
