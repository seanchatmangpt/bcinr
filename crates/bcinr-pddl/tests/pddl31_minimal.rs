// Minimal tests for PDDL 3.1 features - debugging version

use bcinr_mfw_ir::PlannerOutcome;
use bcinr_pddl::ground::GroundTemporalProblem;
use bcinr_pddl::{domain_from_pddl, problem_from_pddl};

/// Simplest possible durative action test - based on working capacity.rs pattern
#[test]
fn simplest_durative_action_test() {
    let domain = domain_from_pddl(
        r#"(define (domain simple)
             (:requirements :durative-actions)
             (:predicates (ready) (done))
             (:durative-action a
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (ready))
               :effect (at end (done))))"#,
    )
    .expect("failed to parse");

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain simple)
             (:init (ready))
             (:goal (done)))"#,
    )
    .expect("failed to parse");

    let ground = GroundTemporalProblem::build(&domain, &problem).expect("failed to ground");

    println!("Durative actions: {}", ground.durative_actions.len());
    let outcome = ground.find_temporal_plan();
    println!("Outcome: {:?}", outcome);
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

/// Test with multiple effects at end
#[test]
fn durative_with_multiple_effects() {
    let domain = domain_from_pddl(
        r#"(define (domain multi)
             (:requirements :durative-actions)
             (:predicates (ready) (done) (marker))
             (:durative-action a
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (ready))
               :effect (and (at end (done)) (at end (marker)))))"#,
    )
    .expect("failed to parse");

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain multi)
             (:init (ready))
             (:goal (and (done) (marker))))"#,
    )
    .expect("failed to parse");

    let ground = GroundTemporalProblem::build(&domain, &problem).expect("failed to ground");

    let outcome = ground.find_temporal_plan();
    println!("Multi effects outcome: {:?}", outcome);
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}
