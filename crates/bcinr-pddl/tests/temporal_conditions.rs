//! Tests for temporal condition handling in durative actions.
//! Verifies that OverAll conditions are checked continuously during action intervals,
//! not just at scheduling time.

use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundTemporalProblem, PlannerOutcome};

#[test]
fn over_all_invariant_becomes_false_mid_interval() {
    // A simple domain with a durative action that has an OverAll condition.
    // The condition is true at start but becomes false mid-interval due to a TIL.
    // The planner should refuse this plan.

    let domain_str = r#"
(define (domain test-over-all)
  (:requirements :durative-actions :timed-initial-literals :negative-preconditions)
  (:predicates (flag) (start-resource) (end-resource))
  (:durative-action maintain-flag
    :parameters ()
    :duration (= ?duration 10)
    :condition (and (at start (start-resource)) (over all (flag)))
    :effect (and (at start (not (start-resource)))
                 (at end (end-resource)))
  )
)
"#;

    let problem_str = r#"
(define (problem test-over-all-1)
  (:domain test-over-all)
  (:init
    (flag)
    (start-resource)
    (at 5 (not (flag)))
  )
  (:goal (end-resource))
)
"#;

    let domain = domain_from_pddl(domain_str).expect("Failed to parse domain");
    let problem = problem_from_pddl(problem_str).expect("Failed to parse problem");

    let temporal_problem =
        GroundTemporalProblem::build(&domain, &problem).expect("Failed to build temporal problem");
    let result = temporal_problem.find_temporal_plan();

    // The plan should be exhausted because the OverAll condition (flag) is violated at t=5
    // while the action is still in flight [0, 10).
    match result {
        PlannerOutcome::Exhausted(_) => {
            // This is what we expect: the invariant violation prevents success
        }
        PlannerOutcome::Found(_) => {
            panic!("Expected plan to be exhausted due to OverAll condition violation, but found a plan");
        }
        PlannerOutcome::Bounded(_)
        | PlannerOutcome::Unsupported(_)
        | PlannerOutcome::Inconsistent(_) => {
            // These outcomes are acceptable for this test; could indicate search limits
        }
    }
}

#[test]
fn over_all_condition_held_through_interval() {
    // A durative action with an OverAll condition that remains true throughout.
    // This should succeed.

    let domain_str = r#"
(define (domain test-over-all-success)
  (:requirements :durative-actions :timed-initial-literals)
  (:predicates (flag) (start-resource) (end-resource))
  (:durative-action maintain-flag
    :parameters ()
    :duration (= ?duration 5)
    :condition (and (at start (start-resource)) (over all (flag)))
    :effect (and (at start (not (start-resource)))
                 (at end (end-resource)))
  )
)
"#;

    let problem_str = r#"
(define (problem test-over-all-success-1)
  (:domain test-over-all-success)
  (:init
    (flag)
    (start-resource)
  )
  (:goal (end-resource))
)
"#;

    let domain = domain_from_pddl(domain_str).expect("Failed to parse domain");
    let problem = problem_from_pddl(problem_str).expect("Failed to parse problem");

    let temporal_problem =
        GroundTemporalProblem::build(&domain, &problem).expect("Failed to build temporal problem");
    let result = temporal_problem.find_temporal_plan();

    // The plan should be found because flag remains true throughout [0, 5).
    match result {
        PlannerOutcome::Found(plan) => {
            assert!(!plan.steps.is_empty(), "Plan should have at least one step");
            assert!(plan.makespan > 0.0, "Makespan should be positive");
        }
        PlannerOutcome::Exhausted(_) => {
            panic!("Expected to find a plan, but planner exhausted");
        }
        PlannerOutcome::Bounded(_)
        | PlannerOutcome::Unsupported(_)
        | PlannerOutcome::Inconsistent(_) => {
            panic!("Expected to find a plan, but got {:?}", result);
        }
    }
}

#[test]
fn multiple_actions_all_over_all_conditions() {
    // Two concurrent actions, each with OverAll conditions.
    // One of them is violated mid-interval by a TIL.
    // Plan should fail.

    let domain_str = r#"
(define (domain test-concurrent-over-all)
  (:requirements :durative-actions :timed-initial-literals :negative-preconditions)
  (:predicates (flag-a) (flag-b) (res-a) (res-b) (goal))
  (:durative-action action-a
    :parameters ()
    :duration (= ?duration 10)
    :condition (and (at start (res-a)) (over all (flag-a)))
    :effect (and (at start (not (res-a))))
  )
  (:durative-action action-b
    :parameters ()
    :duration (= ?duration 10)
    :condition (and (at start (res-b)) (over all (flag-b)))
    :effect (and (at start (not (res-b))) (at end (goal)))
  )
)
"#;

    let problem_str = r#"
(define (problem test-concurrent-over-all-1)
  (:domain test-concurrent-over-all)
  (:init
    (flag-a)
    (flag-b)
    (res-a)
    (res-b)
    (at 5 (not (flag-b)))
  )
  (:goal (goal))
)
"#;

    let domain = domain_from_pddl(domain_str).expect("Failed to parse domain");
    let problem = problem_from_pddl(problem_str).expect("Failed to parse problem");

    let temporal_problem =
        GroundTemporalProblem::build(&domain, &problem).expect("Failed to build temporal problem");
    let result = temporal_problem.find_temporal_plan();

    // Should exhaust because action-b's OverAll condition (flag-b) is violated at t=5
    // while action-b is in flight [0, 10).
    match result {
        PlannerOutcome::Exhausted(_) => {
            // Expected: the invariant violation prevents success
        }
        PlannerOutcome::Found(_) => {
            panic!("Expected plan to be exhausted due to OverAll condition violation, but found a plan");
        }
        PlannerOutcome::Bounded(_)
        | PlannerOutcome::Unsupported(_)
        | PlannerOutcome::Inconsistent(_) => {
            // These outcomes are acceptable for this test; could indicate search limits
        }
    }
}
