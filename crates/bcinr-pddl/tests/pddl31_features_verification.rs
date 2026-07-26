//! Final verification tests for PDDL 3.1 features
//! Tests that the three features parse, ground, and work correctly.

use bcinr_pddl::ground::GroundTemporalProblem;
use bcinr_pddl::{domain_from_pddl, problem_from_pddl};
use bcinr_mfw_ir::PlannerOutcome;

/// Phase 1: Equality in preconditions
/// Verify equality operators work in preconditions
#[test]
fn feature_equality_works() {
    let domain = domain_from_pddl(
        r#"(define (domain eq)
             (:requirements :durative-actions :typing)
             (:types obj)
             (:predicates (p ?x - obj) (q ?x - obj))
             (:durative-action a :parameters (?x - obj ?y - obj)
               :duration (= ?duration 1)
               :condition (and (at start (p ?x)) (at start (= ?x ?y)))
               :effect (at end (q ?x))))"#,
    )
    .unwrap();
    assert_eq!(domain.durative_actions.len(), 1);
    // Equality works - the durative action with equality precondition parsed successfully
}

/// Phase 2: Derived predicates with quantified bodies
/// Verify that forall and exists in derived predicate bodies are supported
#[test]
fn feature_derived_with_forall_body_works() {
    let domain = domain_from_pddl(
        r#"(define (domain d-forall)
             (:requirements :derived-predicates :typing)
             (:types obj)
             (:predicates (p ?x - obj) (all-p))
             (:derived (all-p) (forall (?x - obj) (p ?x))))"#,
    )
    .unwrap();

    assert_eq!(domain.derived.len(), 1);
    // Verify the body contains a forall
    match &domain.derived[0].body {
        bcinr_pddl::PddlCondition::Forall { vars, body } => {
            assert_eq!(vars.len(), 1);
        }
        _ => panic!("Expected forall in derived predicate body"),
    }
}

/// Phase 2b: Derived predicates with exists bodies
#[test]
fn feature_derived_with_exists_body_works() {
    let domain = domain_from_pddl(
        r#"(define (domain d-exists)
             (:requirements :derived-predicates :typing)
             (:types obj)
             (:predicates (p ?x - obj) (some-p))
             (:derived (some-p) (exists (?x - obj) (p ?x))))"#,
    )
    .unwrap();

    assert_eq!(domain.derived.len(), 1);
    // Verify the body contains an exists
    match &domain.derived[0].body {
        bcinr_pddl::PddlCondition::Exists { vars, body } => {
            assert_eq!(vars.len(), 1);
        }
        _ => panic!("Expected exists in derived predicate body"),
    }
}

/// Phase 3: Conditional effects
/// Verify that when (condition) effects are parsed and work
#[test]
fn feature_conditional_effects_work() {
    // Test 1: Basic conditional effect that fires
    let domain = domain_from_pddl(
        r#"(define (domain cond)
             (:requirements :durative-actions :conditional-effects)
             (:predicates (c) (d))
             (:durative-action a :parameters ()
               :duration (= ?duration 1)
               :condition (at start (c))
               :effect (at end (when (c) (d)))))"#,
    )
    .unwrap();

    assert_eq!(domain.durative_actions.len(), 1);

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain cond)
             (:init (c))
             (:goal (d)))"#,
    )
    .unwrap();

    let ground = GroundTemporalProblem::build(&domain, &problem).unwrap();
    let outcome = ground.find_temporal_plan();
    assert!(matches!(outcome, PlannerOutcome::Found(_)),
        "conditional effect that satisfies condition should enable plan");
}

/// Phase 3b: Conditional effects blocking
/// Verify that conditional effects don't fire when condition is false
#[test]
fn feature_conditional_effects_can_block() {
    let domain = domain_from_pddl(
        r#"(define (domain cond-block)
             (:requirements :durative-actions :conditional-effects)
             (:predicates (c) (d))
             (:durative-action a :parameters ()
               :duration (= ?duration 1)
               :condition (at start (true))
               :effect (at end (when (c) (d)))))"#,
    )
    .unwrap();

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain cond-block)
             (:init)
             (:goal (d)))"#,
    )
    .unwrap();

    let ground = GroundTemporalProblem::build(&domain, &problem).unwrap();
    let outcome = ground.find_temporal_plan();
    assert!(matches!(outcome, PlannerOutcome::Exhausted(_)),
        "conditional effect without satisfied condition should block plan");
}

/// Phase 3c: Conditional effects with forall
/// Verify forall with when effects work
#[test]
fn feature_conditional_effects_with_forall() {
    let domain = domain_from_pddl(
        r#"(define (domain cond-forall)
             (:requirements :durative-actions :typing :conditional-effects)
             (:types obj)
             (:predicates (p ?x - obj) (q ?x - obj) (ready))
             (:durative-action a :parameters ()
               :duration (= ?duration 1)
               :condition (at start (ready))
               :effect (at end (forall (?x - obj) (when (p ?x) (q ?x))))))"#,
    )
    .unwrap();

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain cond-forall)
             (:objects o1 o2 - obj)
             (:init (ready) (p o1) (p o2))
             (:goal (and (q o1) (q o2))))"#,
    )
    .unwrap();

    let ground = GroundTemporalProblem::build(&domain, &problem).unwrap();
    let outcome = ground.find_temporal_plan();
    assert!(matches!(outcome, PlannerOutcome::Found(_)),
        "forall with conditional effects should work");
}

/// Integration test: all three features together
#[test]
fn all_three_features_together() {
    let domain = domain_from_pddl(
        r#"(define (domain integration)
             (:requirements :durative-actions :typing :derived-predicates :conditional-effects)
             (:types obj)
             (:predicates (p ?x - obj) (q ?x - obj) (all-p) (done))
             (:derived (all-p) (forall (?x - obj) (p ?x)))
             (:durative-action a :parameters ()
               :duration (= ?duration 1)
               :condition (at start (all-p))
               :effect (at end (when (all-p) (done)))))"#,
    )
    .unwrap();

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain integration)
             (:objects o1 o2 - obj)
             (:init (p o1) (p o2))
             (:goal (done)))"#,
    )
    .unwrap();

    let ground = GroundTemporalProblem::build(&domain, &problem).unwrap();
    let outcome = ground.find_temporal_plan();
    assert!(matches!(outcome, PlannerOutcome::Found(_)),
        "all three PDDL 3.1 features should work together");
}
