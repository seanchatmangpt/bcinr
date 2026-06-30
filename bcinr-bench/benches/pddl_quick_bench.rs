//! Quick PDDL benchmark: `manufacture_world` end-to-end + isolated `find_plan`,
//! at [1, 3, 6] logistics-package scale.
//!
//! Uses `divan` rather than Criterion: divan's per-benchmark overhead (no HTML
//! report generation, no adaptive long-running sampling) is dramatically lower,
//! which is what actually gets the whole suite to fit inside a tight wall-clock
//! budget instead of just tuning Criterion's knobs and hoping.
//!
//! The domain declares real per-parameter types (`package`/`truck`/`location`),
//! not just the `:typing` requirement flag — this matters: the grounder only
//! restricts bindings using parameter type annotations it can actually see.
//! An earlier version of this benchmark declared `:typing` but left action
//! parameters untyped (`?pkg` with no `- type` suffix), which silently fell
//! back to "matches everything" and measured 146ms at n=3 even after the
//! grounder fix landed — that was the untyped domain's correct behavior, not
//! a bug, but it meant the benchmark wasn't exercising the fix at all. With
//! real types declared, n=6 is back in budget (was previously cut for
//! exceeding it under the untyped cross-product).

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
        init.join(" "),
        goal.join(" ")
    )
}

#[divan::bench(args = [1, 3, 6], sample_count = 5, sample_size = 2)]
fn manufacture_world_bench(n: usize) {
    let problem = problem_with_n_packages(n);
    divan::black_box(manufacture_world(DOMAIN, &problem, "bench-case", &[]));
}

#[divan::bench(args = [1, 3, 6], sample_count = 5, sample_size = 2)]
fn find_plan_bench(n: usize) {
    let domain = domain_from_pddl(DOMAIN).expect("domain parse");
    let problem = problem_from_pddl(&problem_with_n_packages(n)).expect("problem parse");
    let ground = GroundProblem::build(&domain, &problem, None).expect("grounding");
    divan::black_box(ground.find_plan().ok());
}

fn main() {
    let start = Instant::now();
    divan::main();
    eprintln!("pddl_quick_bench wall clock: {:?}", start.elapsed());
}
