#![allow(clippy::incompatible_msrv)]
//! MCP+ admission-key cache proof suite (cold/warm latency + concurrent
//! throughput) — see docs/ for the full 7-benchmark proposal and which
//! items are honestly buildable today vs. deferred (token displacement and
//! the 100-agent scenario require an LLM tool-routing loop that doesn't
//! exist in this codebase, and are NOT approximated here).
//!
//! Exercises `bcinr_mcp::cache::CapabilityCache` wrapped around the same
//! underlying library calls the `manufacture_world`/`pddl_plan` MCP tool
//! handlers use (`bcinr_pddl::manufacture_world`,
//! `bcinr_pddl::domain_from_pddl` + `GroundProblem::find_plan`) — this is
//! the cache primitive itself under real workload, not rmcp's tool-dispatch
//! transport machinery (which isn't part of what's being measured).
//!
//! Uses divan, matching the other bench files in this crate, for low
//! per-benchmark overhead.

use bcinr_mcp::cache::CapabilityCache;
use bcinr_pddl::{domain_from_pddl, manufacture_world, problem_from_pddl, GroundProblem};
use std::time::Instant;

const DOMAIN: &str = r#"
(define (domain logistics)
  (:requirements :strips :typing)
  (:types package truck location)
  (:predicates
    (at ?x ?y)
    (in ?x ?y))
  (:action load-truck
    :parameters (?pkg - package ?truck - truck ?loc - location)
    :precondition (and (at ?pkg ?loc) (at ?truck ?loc))
    :effect (and (in ?pkg ?truck) (not (at ?pkg ?loc))))
  (:action drive-truck
    :parameters (?truck - truck ?from - location ?to - location)
    :precondition (at ?truck ?from)
    :effect (and (at ?truck ?to) (not (at ?truck ?from))))
  (:action unload-truck
    :parameters (?pkg - package ?truck - truck ?loc - location)
    :precondition (and (in ?pkg ?truck) (at ?truck ?loc))
    :effect (and (at ?pkg ?loc) (not (in ?pkg ?truck))))
)
"#;

fn problem_with_n_packages(n: usize) -> String {
    let pkgs: Vec<String> = (0..n).map(|i| format!("pkg{i}")).collect();
    let objects = format!(
        "{} - package\n    truck1 - truck\n    loc_a loc_b - location",
        pkgs.join(" ")
    );
    let init: Vec<String> = pkgs
        .iter()
        .map(|p| format!("(at {p} loc_a)"))
        .chain(std::iter::once("(at truck1 loc_a)".to_string()))
        .collect();
    let goal: Vec<String> = pkgs.iter().map(|p| format!("(at {p} loc_b)")).collect();
    format!(
        "(define (problem get-pkgs-to-loc_b)\n  (:domain logistics)\n  (:objects {objects})\n  (:init {})\n  (:goal (and {})))",
        init.join(" "), goal.join(" ")
    )
}

/// Same shape as `manufacture_world`'s MCP handler's cache wrap: check cache
/// by (tool, blake3(input)), compute on miss, insert before returning.
async fn cached_manufacture_world(
    cache: &CapabilityCache,
    domain: &str,
    problem: &str,
    case_id: &str,
) -> String {
    #[derive(serde::Serialize)]
    struct Input<'a> {
        domain_text: &'a str,
        problem_text: &'a str,
        case_id: &'a str,
    }
    let canonical = serde_json::to_vec(&Input {
        domain_text: domain,
        problem_text: problem,
        case_id,
    })
    .unwrap_or_default();
    let key = CapabilityCache::key("manufacture_world", &canonical);
    if let Some(cached) = cache.get(&key).await {
        return cached;
    }
    let receipt = manufacture_world(domain, problem, case_id, &[]);
    let result = serde_json::json!({
        "admitted": receipt.admitted,
        "makespan": receipt.plan.makespan,
        "step_count": receipt.plan.steps.len(),
    })
    .to_string();
    cache.insert(key, result.clone()).await;
    result
}

async fn cached_pddl_plan(cache: &CapabilityCache, domain: &str, problem: &str) -> String {
    #[derive(serde::Serialize)]
    struct Input<'a> {
        domain_text: &'a str,
        problem_text: &'a str,
    }
    let canonical = serde_json::to_vec(&Input {
        domain_text: domain,
        problem_text: problem,
    })
    .unwrap_or_default();
    let key = CapabilityCache::key("pddl_plan", &canonical);
    if let Some(cached) = cache.get(&key).await {
        return cached;
    }
    let result = (|| -> Result<String, bcinr_pddl::Pddl8Error> {
        let d = domain_from_pddl(domain)?;
        let p = problem_from_pddl(problem)?;
        let ground = GroundProblem::build(&d, &p, None)?;
        let tape = ground.find_plan().into_result()?;
        Ok(serde_json::json!({"ok": true, "step_count": tape.ops.len()}).to_string())
    })()
    .unwrap_or_else(|e| serde_json::json!({"ok": false, "error": e.to_string()}).to_string());
    cache.insert(key, result.clone()).await;
    result
}

#[divan::bench(sample_count = 5, sample_size = 3)]
fn cold_vs_warm_manufacture_world(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    bencher.bench_local(|| {
        rt.block_on(async {
            let cache = CapabilityCache::new();
            let problem = problem_with_n_packages(3);

            let t_cold = Instant::now();
            let _ = divan::black_box(
                cached_manufacture_world(&cache, DOMAIN, &problem, "bench-cold").await,
            );
            let cold_ns = t_cold.elapsed().as_nanos();

            let t_warm = Instant::now();
            let _ = divan::black_box(
                cached_manufacture_world(&cache, DOMAIN, &problem, "bench-cold").await,
            );
            let warm_ns = t_warm.elapsed().as_nanos();

            eprintln!(
                "manufacture_world cold_ns={cold_ns} warm_ns={warm_ns} speedup={:.1}x",
                cold_ns as f64 / warm_ns.max(1) as f64
            );
        });
    });
}

#[divan::bench(sample_count = 5, sample_size = 3)]
fn cold_vs_warm_pddl_plan(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    bencher.bench_local(|| {
        rt.block_on(async {
            let cache = CapabilityCache::new();
            let problem = problem_with_n_packages(3);

            let t_cold = Instant::now();
            let _ = divan::black_box(cached_pddl_plan(&cache, DOMAIN, &problem).await);
            let cold_ns = t_cold.elapsed().as_nanos();

            let t_warm = Instant::now();
            let _ = divan::black_box(cached_pddl_plan(&cache, DOMAIN, &problem).await);
            let warm_ns = t_warm.elapsed().as_nanos();

            eprintln!(
                "pddl_plan cold_ns={cold_ns} warm_ns={warm_ns} speedup={:.1}x",
                cold_ns as f64 / warm_ns.max(1) as f64
            );
        });
    });
}

/// Concurrent throughput: N tasks against a shared cache, mix of
/// cache-hit-inducing (identical input) and cache-miss-inducing (distinct
/// package counts) requests.
#[divan::bench(args = [1, 8, 32, 128], sample_count = 3, sample_size = 1)]
fn concurrent_throughput(n: usize) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let cache = std::sync::Arc::new(CapabilityCache::new());
        let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let start = Instant::now();

        let mut handles = Vec::with_capacity(n);
        for i in 0..n {
            let cache = cache.clone();
            let hits = hits.clone();
            // Half the requests share one input (cache-hit-inducing after
            // the first), half use a distinct package count (cache-miss-inducing).
            let n_packages = if i % 2 == 0 { 2 } else { 2 + (i % 5) };
            handles.push(tokio::spawn(async move {
                let problem = problem_with_n_packages(n_packages);
                #[derive(serde::Serialize)]
                struct Input<'a> {
                    domain_text: &'a str,
                    problem_text: &'a str,
                    case_id: &'a str,
                }
                let canonical = serde_json::to_vec(&Input {
                    domain_text: DOMAIN,
                    problem_text: &problem,
                    case_id: "throughput",
                })
                .unwrap_or_default();
                let key = CapabilityCache::key("manufacture_world", &canonical);
                if cache.get(&key).await.is_some() {
                    hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                let _ = cached_manufacture_world(&cache, DOMAIN, &problem, "throughput").await;
            }));
        }
        for h in handles {
            let _ = h.await;
        }

        let elapsed = start.elapsed();
        let req_per_sec = n as f64 / elapsed.as_secs_f64();
        let hit_count = hits.load(std::sync::atomic::Ordering::Relaxed);
        eprintln!(
            "concurrent_throughput n={n} elapsed={:?} req_per_sec={:.1} cache_hits={hit_count}/{n}",
            elapsed, req_per_sec
        );
    });
}

fn main() {
    let start = Instant::now();
    divan::main();
    eprintln!("mcp_cache_bench wall clock: {:?}", start.elapsed());
}
