use bcinr_pddl::{
    domain_from_pddl, problem_from_pddl, validate_temporal_plan_shape, GroundTemporalProblem,
    TemporalPlan, TemporalPlanStep, TemporalPowlRuntime, TemporalProductionError,
};

const PARALLEL_DOMAIN: &str = r#"
(define (domain deterministic-swarm)
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

const PARALLEL_PROBLEM: &str = r#"
(define (problem parallel-delivery)
  (:domain deterministic-swarm)
  (:objects w1 w2 - worker j1 j2 - job)
  (:init (ready j1) (ready j2) (= (capacity) 2))
  (:goal (and (done j1) (done j2))))
"#;

const SUBSTITUTION_DOMAIN: &str = r#"
(define (domain deterministic-substitution)
  (:requirements :strips :durative-actions)
  (:predicates (primary-available) (fallback-available) (done))
  (:durative-action primary-worker
    :parameters ()
    :duration (= ?duration 1)
    :condition (and (at start (primary-available)))
    :effect (and (at end (done))))
  (:durative-action fallback-worker
    :parameters ()
    :duration (= ?duration 1)
    :condition (and (at start (fallback-available)))
    :effect (and (at end (done)))))
"#;

const SUBSTITUTION_PROBLEM: &str = r#"
(define (problem substitution)
  (:domain deterministic-substitution)
  (:init (fallback-available))
  (:goal (and (done))))
"#;

fn grounded_parallel() -> GroundTemporalProblem {
    let domain = domain_from_pddl(PARALLEL_DOMAIN).expect("parallel domain must parse");
    let problem = problem_from_pddl(PARALLEL_PROBLEM).expect("parallel problem must parse");
    GroundTemporalProblem::build(&domain, &problem).expect("parallel problem must ground")
}

#[test]
fn parallel_workers_execute_under_shared_capacity_and_replay() {
    let runtime = TemporalPowlRuntime;
    let first = runtime
        .execute(PARALLEL_DOMAIN, PARALLEL_PROBLEM, "parallel-delivery")
        .expect("parallel swarm must execute");
    let second = runtime
        .execute(PARALLEL_DOMAIN, PARALLEL_PROBLEM, "parallel-delivery")
        .expect("replay must execute");

    assert!(first.receipt.goal_reached);
    assert!(first.plan.steps.len() >= 2);
    assert!(first
        .plan
        .steps
        .windows(2)
        .any(|pair| pair[0].start_time == pair[1].start_time));
    assert_eq!(first.execution_root, second.execution_root);
    assert_eq!(first.receipt.chain_hash, second.receipt.chain_hash);
    first.verify().expect("first receipt must verify");
    second.verify().expect("second receipt must verify");
}

#[test]
fn unavailable_primary_worker_is_replaced_by_admitted_fallback() {
    let execution = TemporalPowlRuntime
        .execute(
            SUBSTITUTION_DOMAIN,
            SUBSTITUTION_PROBLEM,
            "worker-substitution",
        )
        .expect("fallback worker must execute");

    assert_eq!(execution.plan.steps.len(), 1);
    assert_eq!(execution.plan.steps[0].action_name, "fallback-worker");
    assert!(execution.receipt.goal_reached);
}

#[test]
fn overlapping_duplicate_worker_instance_is_typed_refused() {
    let grounded = grounded_parallel();
    let duplicated = TemporalPlan {
        steps: vec![
            TemporalPlanStep {
                action_name: "execute-job".to_string(),
                args: vec!["w1".to_string(), "j1".to_string()],
                start_time: 0.0,
                duration: 2.0,
            },
            TemporalPlanStep {
                action_name: "execute-job".to_string(),
                args: vec!["w1".to_string(), "j1".to_string()],
                start_time: 1.0,
                duration: 2.0,
            },
        ],
        makespan: 3.0,
        metric_value: None,
    };

    let error = validate_temporal_plan_shape(&grounded, &duplicated)
        .expect_err("one grounded worker instance cannot overlap itself");
    assert!(matches!(
        error,
        TemporalProductionError::OverlappingDuplicate { .. }
    ));
}

#[test]
fn duration_outside_the_grounded_contract_is_typed_refused() {
    let grounded = grounded_parallel();
    let invalid = TemporalPlan {
        steps: vec![TemporalPlanStep {
            action_name: "execute-job".to_string(),
            args: vec!["w1".to_string(), "j1".to_string()],
            start_time: 0.0,
            duration: 3.0,
        }],
        makespan: 3.0,
        metric_value: None,
    };

    let error = validate_temporal_plan_shape(&grounded, &invalid)
        .expect_err("duration drift must be refused");
    assert!(matches!(
        error,
        TemporalProductionError::DurationOutOfBounds { .. }
    ));
}

#[test]
fn malformed_time_is_refused_before_execution() {
    let grounded = grounded_parallel();
    let invalid = TemporalPlan {
        steps: vec![TemporalPlanStep {
            action_name: "execute-job".to_string(),
            args: vec!["w1".to_string(), "j1".to_string()],
            start_time: f64::NAN,
            duration: 2.0,
        }],
        makespan: 2.0,
        metric_value: None,
    };

    let error = validate_temporal_plan_shape(&grounded, &invalid)
        .expect_err("non-finite logical time must be refused");
    assert!(matches!(error, TemporalProductionError::InvalidTime { .. }));
}

#[test]
fn production_runtime_contains_no_llm_dependency() {
    let profile = bcinr_pddl::temporal_production::TEMPORAL_RUNTIME_PROFILE;
    assert_eq!(profile.llm_dependency, 0);
    assert_eq!(profile.deterministic, 1);
    assert_eq!(profile.panic_across_boundary, 0);
    assert_eq!(profile.ambient_authority, 0);
}
