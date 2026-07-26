use bcinr_pddl::{execute_temporal_pddl_to_powl, TemporalPowlRuntime};

const DOMAIN: &str = r#"
(define (domain benchmark-swarm)
  (:requirements :strips :typing :durative-actions :numeric-fluents)
  (:types worker job)
  (:predicates (ready ?j - job) (done ?j - job))
  (:functions (capacity))
  (:durative-action execute-job
    :parameters (?w - worker ?j - job)
    :duration (= ?duration 2)
    :condition (and
      (at start (ready ?j))
      (over all (ready ?j))
      (at start (>= (capacity) 1)))
    :effect (and
      (at start (decrease (capacity) 1))
      (at end (increase (capacity) 1))
      (at end (done ?j)))))
"#;

const PROBLEM: &str = r#"
(define (problem benchmark-parallel-delivery)
  (:domain benchmark-swarm)
  (:objects w1 w2 - worker j1 j2 - job)
  (:init (ready j1) (ready j2) (= (capacity) 2))
  (:goal (and (done j1) (done j2))))
"#;

#[divan::bench]
fn temporal_swarm_end_to_end() {
    let domain = divan::black_box(DOMAIN);
    let problem = divan::black_box(PROBLEM);
    let case_id = divan::black_box("temporal-swarm-benchmark");
    let result = execute_temporal_pddl_to_powl(domain, problem, case_id);
    divan::black_box(result).expect("benchmark fixture must execute");
}

#[divan::bench(args = [1_usize, 8, 64])]
fn temporal_swarm_replay_batch(rounds: usize) {
    let runtime = TemporalPowlRuntime;
    for _ in 0..divan::black_box(rounds) {
        let result = runtime.execute(
            divan::black_box(DOMAIN),
            divan::black_box(PROBLEM),
            divan::black_box("temporal-swarm-batch"),
        );
        divan::black_box(result).expect("benchmark fixture must execute");
    }
}

fn main() {
    divan::main();
}
