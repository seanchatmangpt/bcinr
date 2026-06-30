//! Capacity/concurrency integration test: durative actions gated by a numeric
//! fluent (`available-workers`) exercise both the new numeric-comparison
//! precondition support and the "schedule all applicable durative actions
//! per tick" concurrency behavior in `GroundTemporalProblem::find_temporal_plan`.
//!
//! Domain: assigning a worker consumes one unit of `available-workers` at
//! start and releases it at end. With capacity 1, only one worker can be
//! assigned at a time, forcing the two `assign-worker` steps to run
//! sequentially. With capacity 2, both workers can be assigned concurrently.

use bcinr_pddl::{analyze_schedule, domain_from_pddl, problem_from_pddl, GroundTemporalProblem};

const DOMAIN: &str = r#"
(define (domain capacity-demo)
  (:requirements :durative-actions :numeric-fluents :typing)
  (:types worker)
  (:predicates (idle ?w - worker) (busy ?w - worker) (done ?w - worker))
  (:functions (available-workers))
  (:durative-action assign-worker
    :parameters (?w - worker)
    :duration (= ?duration 5)
    :condition (and (at start (idle ?w)) (at start (>= (available-workers) 1)))
    :effect (and
      (at start (decrease (available-workers) 1))
      (at start (not (idle ?w))) (at start (busy ?w))
      (at end (increase (available-workers) 1))
      (at end (not (busy ?w))) (at end (done ?w)))))
"#;

fn problem_with_capacity(capacity: u32) -> String {
    format!(
        r#"
(define (problem assign-two-workers)
  (:domain capacity-demo)
  (:objects w1 w2 - worker)
  (:init
    (idle w1) (idle w2)
    (= (available-workers) {capacity}))
  (:goal (and (done w1) (done w2)))
)
"#
    )
}

/// Returns the [start_time, start_time + duration) intervals of every
/// `assign-worker` step in the plan, in plan order.
fn assign_worker_intervals(plan: &bcinr_pddl::TemporalPlan) -> Vec<(f64, f64)> {
    plan.steps
        .iter()
        .filter(|s| s.action_name == "assign-worker")
        .map(|s| (s.start_time, s.start_time + s.duration))
        .collect()
}

fn intervals_overlap(a: (f64, f64), b: (f64, f64)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

#[test]
fn capacity_one_forces_sequential() {
    let domain = domain_from_pddl(DOMAIN).expect("domain parse");
    let problem = problem_from_pddl(&problem_with_capacity(1)).expect("problem parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("grounding");
    let plan = gtp.find_temporal_plan().expect("temporal plan found");

    let intervals = assign_worker_intervals(&plan);
    assert_eq!(intervals.len(), 2, "expected two assign-worker steps, got {:?}", intervals);

    assert!(
        !intervals_overlap(intervals[0], intervals[1]),
        "with capacity 1, assign-worker intervals must not overlap: {:?}",
        intervals
    );
}

#[test]
fn capacity_two_allows_concurrent() {
    let domain = domain_from_pddl(DOMAIN).expect("domain parse");
    let problem = problem_from_pddl(&problem_with_capacity(2)).expect("problem parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("grounding");
    let plan = gtp.find_temporal_plan().expect("temporal plan found");

    let intervals = assign_worker_intervals(&plan);
    assert_eq!(intervals.len(), 2, "expected two assign-worker steps, got {:?}", intervals);

    assert!(
        intervals_overlap(intervals[0], intervals[1]),
        "with capacity 2, assign-worker intervals must overlap (concurrent start): {:?}",
        intervals
    );
}

#[test]
fn schedule_analysis_capacity_one_is_binding_and_sequential() {
    let domain = domain_from_pddl(DOMAIN).expect("domain parse");
    let problem = problem_from_pddl(&problem_with_capacity(1)).expect("problem parse");
    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("grounding");

    let resource_keys = vec!["available-workers".to_string()];
    let analysis = analyze_schedule(&gtp, &resource_keys).expect("schedule analysis");

    assert_eq!(analysis.op_count, 2);
    assert_eq!(
        analysis.max_parallelism, 1,
        "capacity 1 forces fully sequential execution, no overlap"
    );
    assert_eq!(
        analysis.critical_path_mask,
        (1u64 << analysis.op_count) - 1,
        "with no slack anywhere in a sequential 2-step plan, both ops are on the critical path"
    );
    assert_eq!(
        analysis.binding_resource_mask & 1,
        1,
        "available-workers must be flagged as binding at capacity 1"
    );
    let delta = analysis.capacity_delta.expect("capacity_delta for resource 0");
    assert_eq!(delta.baseline_makespan, analysis.makespan);
    assert!(
        delta.plus_one_makespan.unwrap_or(f64::INFINITY) < delta.baseline_makespan,
        "adding one unit of capacity should shorten makespan by enabling concurrency: {:?}",
        delta
    );
}

#[test]
fn schedule_analysis_capacity_two_allows_full_parallelism() {
    let domain = domain_from_pddl(DOMAIN).expect("domain parse");
    let problem = problem_from_pddl(&problem_with_capacity(2)).expect("problem parse");
    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("grounding");

    let resource_keys = vec!["available-workers".to_string()];
    let analysis = analyze_schedule(&gtp, &resource_keys).expect("schedule analysis");

    assert_eq!(analysis.op_count, 2);
    assert_eq!(
        analysis.max_parallelism, 2,
        "capacity 2 allows both assign-worker steps to run concurrently"
    );
    let delta = analysis.capacity_delta.expect("capacity_delta for resource 0");
    assert!(
        delta.plus_one_makespan.unwrap_or(f64::INFINITY) >= delta.baseline_makespan - 1e-9,
        "capacity 3 has no further benefit once both workers already run concurrently: {:?}",
        delta
    );
}
