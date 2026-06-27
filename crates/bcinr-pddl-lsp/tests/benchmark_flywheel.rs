//! Flywheel benchmark — all new capabilities, total wall clock ≤ 8s.
//!
//! Each operation is measured individually with std::time::Instant.
//! Prints a timing table. Final assertion: sum of all ops ≤ 8 000ms.
//!
//! This is not criterion — it is a falsifiable timing contract:
//! if the flywheel slows past 8s, this test fails.

use std::{fs, path::PathBuf, time::{Duration, Instant}};
use tempfile::TempDir;

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sean_education_mode")
        .canonicalize()
        .unwrap()
}

fn full_project(dir: &TempDir) {
    write_file(dir, "README.md", "# Project\n## Status: ADMITTED");
    write_file(dir, "docs/prd.md", "# PRD\n## Status: ADMITTED\nFull plan.");
    write_file(dir, "docs/ard.md", "# ARD\n## Status: ADMITTED\nArchitecture.");
    write_file(dir, "docs/adr/0001.md", "# ADR\n## Status: ADMITTED");
    write_file(dir, "docs/work-units.md", "# Work Units\n- T1\n- T2\n- T3");
    write_file(dir, "src/lib.rs", "pub fn main() {}");
    write_file(dir, ".bcinr/test-report.json", r#"{"status":"passed"}"#);
    write_file(dir, "docs/architecture.md", "# Docs");
    write_file(dir, ".bcinr/release.json", r#"{"ready":true}"#);
    write_file(dir, ".bcinr/ocel/latest.json", r#"{"events":[]}"#);
}

struct Sample {
    name: &'static str,
    elapsed: Duration,
    iterations: u32,
}

impl Sample {
    fn ns_per_iter(&self) -> u64 {
        (self.elapsed.as_nanos() / self.iterations as u128) as u64
    }
    fn label(&self) -> &str {
        match self.ns_per_iter() {
            0..=999 => "ns",
            1_000..=999_999 => "µs",
            _ => "ms",
        }
    }
    fn value(&self) -> f64 {
        let ns = self.ns_per_iter() as f64;
        match self.label() {
            "ns" => ns,
            "µs" => ns / 1_000.0,
            _ => ns / 1_000_000.0,
        }
    }
}

fn measure<F: FnMut()>(name: &'static str, iters: u32, mut f: F) -> Sample {
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    Sample { name, elapsed: start.elapsed(), iterations: iters }
}

#[test]
fn flywheel_benchmark() {
    use bcinr_pddl_lsp::{
        bounds, build_broker, education, lifecycle, planner_client, projection, publish_gate,
        virtual_docs,
    };
    use bcinr_pddl::domain_from_pddl;

    let mut samples: Vec<Sample> = Vec::new();
    let wall_start = Instant::now();

    // ── 1. Lifecycle scan (empty) ──────────────────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        samples.push(measure("lifecycle_scan_empty", 1000, || {
            let _ = lifecycle::scan(dir.path());
        }));
    }

    // ── 2. Lifecycle scan (full project) ──────────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        full_project(&dir);
        samples.push(measure("lifecycle_scan_full", 500, || {
            let _ = lifecycle::scan(dir.path());
        }));
    }

    // ── 3. Education scan (fixture) ───────────────────────────────────────
    {
        let root = fixture_root();
        samples.push(measure("education_scan_fixture", 50, || {
            let _ = education::scan(&root, "sean");
        }));
    }

    // ── 4. emit_education_domain ──────────────────────────────────────────
    samples.push(measure("emit_education_domain", 5000, || {
        let _ = education::emit_education_domain();
    }));

    // ── 5. emit_education_problem ─────────────────────────────────────────
    {
        let root = fixture_root();
        let ws = education::scan(&root, "sean");
        samples.push(measure("emit_education_problem", 5000, || {
            let _ = education::emit_education_problem(&ws);
        }));
    }

    // ── 6. education_diagnostics ──────────────────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        let ws = education::scan(dir.path(), "sean");
        samples.push(measure("education_diagnostics", 5000, || {
            let _ = education::education_diagnostics(&ws);
        }));
    }

    // ── 7. PDDL8 lifecycle domain emit ───────────────────────────────────
    samples.push(measure("emit_lifecycle_domain", 5000, || {
        let _ = projection::emit_domain();
    }));

    // ── 8. PDDL8 lifecycle problem emit ──────────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        full_project(&dir);
        let lc = lifecycle::scan(dir.path());
        samples.push(measure("emit_lifecycle_problem", 5000, || {
            let _ = projection::emit_problem(&lc);
        }));
    }

    // ── 9. domain_from_pddl parse ─────────────────────────────────────────
    {
        let domain_text = projection::emit_domain();
        samples.push(measure("domain_parse_lifecycle", 10, || {
            let _ = domain_from_pddl(&domain_text).unwrap();
        }));
    }

    // ── 10. domain_from_pddl parse (education) ───────────────────────────
    {
        let domain_text = education::emit_education_domain();
        samples.push(measure("domain_parse_education", 10, || {
            let _ = domain_from_pddl(&domain_text).unwrap();
        }));
    }

    // ── 11. BFS plan (full project → short/empty plan) ───────────────────
    {
        let dir = TempDir::new().unwrap();
        full_project(&dir);
        let lc = lifecycle::scan(dir.path());
        let proj = projection::project(&lc);
        samples.push(measure("bfs_plan_full", 10, || {
            let _ = planner_client::plan(&proj);
        }));
    }

    // ── 12. BFS plan (empty project → full plan) ─────────────────────────
    {
        let dir = TempDir::new().unwrap();
        let lc = lifecycle::scan(dir.path());
        let proj = projection::project(&lc);
        samples.push(measure("bfs_plan_empty", 10, || {
            let _ = planner_client::plan(&proj);
        }));
    }

    // ── 13. Bounds check ─────────────────────────────────────────────────
    samples.push(measure("bounds_check_work_unit", 50_000, || {
        let _ = bounds::check_work_unit("unit-a", 7);
        let _ = bounds::check_work_unit("unit-big", 9);
    }));

    // ── 14. Bounds report (lifecycle domain) — real check, parse included ─
    samples.push(measure("bounds_check_lifecycle_domain", 10, || {
        let _ = bounds::check_lifecycle_domain();
    }));

    // ── 15. Publish gate from_lifecycle ──────────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        full_project(&dir);
        let lc = lifecycle::scan(dir.path());
        samples.push(measure("publish_gate_from_lifecycle", 10_000, || {
            let _ = publish_gate::from_lifecycle(&lc);
        }));
    }

    // ── 16. Build broker request_slot / release ───────────────────────────
    samples.push(measure("build_broker_acquire_release", 50_000, || {
        let mut state = build_broker::BuildBrokerState::default();
        let _ = state.request_slot("cargo build");
        state.release_slot();
    }));

    // ── 17. Virtual doc render_agent_assignments ──────────────────────────
    {
        let dir = TempDir::new().unwrap();
        full_project(&dir);
        let lc = lifecycle::scan(dir.path());
        let gate = publish_gate::from_lifecycle(&lc);
        samples.push(measure("render_agent_assignments", 5000, || {
            let _ = virtual_docs::render_agent_assignments(&lc, &gate);
        }));
    }

    // ── 18. Education render_education_status ─────────────────────────────
    {
        let root = fixture_root();
        let ws = education::scan(&root, "sean");
        samples.push(measure("render_education_status", 5000, || {
            let _ = education::render_education_status(&ws);
        }));
    }

    // ── 19. Education render_education_gate ──────────────────────────────
    {
        let root = fixture_root();
        let ws = education::scan(&root, "sean");
        samples.push(measure("render_education_gate", 5000, || {
            let _ = education::render_education_gate(&ws);
        }));
    }

    // ── 20. Full flywheel: scan → project → plan (education) ─────────────
    {
        let root = fixture_root();
        samples.push(measure("flywheel_education_e2e", 5, || {
            let ws = education::scan(&root, "sean");
            let domain = education::emit_education_domain();
            let problem = education::emit_education_problem(&ws);
            let proj = projection::Pddl8Projection {
                domain_text: domain,
                problem_text: problem,
            };
            let _ = planner_client::plan(&proj);
        }));
    }

    let wall_elapsed = wall_start.elapsed();

    // ── Print table ───────────────────────────────────────────────────────
    println!("\n{:<40} {:>8} {:>10} {:>8}", "Operation", "iters", "per-iter", "total");
    println!("{}", "─".repeat(70));
    for s in &samples {
        println!("{:<40} {:>8} {:>9.2}{} {:>7.1}ms",
            s.name, s.iterations, s.value(), s.label(),
            s.elapsed.as_secs_f64() * 1000.0
        );
    }
    println!("{}", "─".repeat(70));
    println!("{:<40} {:>8} {:>10} {:>7.1}ms",
        "TOTAL WALL CLOCK", "", "", wall_elapsed.as_secs_f64() * 1000.0
    );

    // ── Falsifiable contract: total ≤ 8 000ms ─────────────────────────────
    assert!(
        wall_elapsed.as_millis() <= 8_000,
        "Flywheel exceeded 8s wall clock: {}ms. Operations that regressed must be profiled.",
        wall_elapsed.as_millis()
    );

    // ── Per-operation contracts ───────────────────────────────────────────
    for s in &samples {
        let ns = s.ns_per_iter();
        match s.name {
            // String generation ops: must be < 100µs per iter
            n if n.starts_with("emit_") => assert!(
                ns < 100_000,
                "{} too slow: {}ns/iter (limit 100µs)", n, ns
            ),
            // Scan ops: must be < 5ms per iter (file I/O)
            n if n.starts_with("lifecycle_scan") || n.starts_with("education_scan") => assert!(
                ns < 5_000_000,
                "{} too slow: {}ns/iter (limit 5ms)", n, ns
            ),
            // BFS planning: must be < 50ms per iter (full lifecycle domain BFS)
            n if n.starts_with("bfs_plan") => assert!(
                ns < 50_000_000,
                "{} too slow: {}ns/iter (limit 50ms)", n, ns
            ),
            // Build broker: pure in-memory, < 10µs
            n if n.starts_with("build_broker") => assert!(
                ns < 10_000,
                "{} too slow: {}ns/iter (limit 10µs)", n, ns
            ),
            // bounds_check_work_unit: O(1) comparison, < 10µs
            "bounds_check_work_unit" => assert!(
                ns < 10_000,
                "bounds_check_work_unit too slow: {}ns/iter (limit 10µs)", ns
            ),
            // bounds_check_lifecycle_domain: real parse + check, < 20ms
            "bounds_check_lifecycle_domain" => assert!(
                ns < 20_000_000,
                "bounds_check_lifecycle_domain too slow: {}ms/iter (limit 20ms)", ns / 1_000_000
            ),
            // Render ops: must be < 100µs
            n if n.starts_with("render_") => assert!(
                ns < 100_000,
                "{} too slow: {}ns/iter (limit 100µs)", n, ns
            ),
            _ => {}
        }
    }

    println!("\nAll {} operations within per-operation contracts.", samples.len());
    println!("Flywheel is GREEN: {}ms total wall clock.", wall_elapsed.as_millis());
}
