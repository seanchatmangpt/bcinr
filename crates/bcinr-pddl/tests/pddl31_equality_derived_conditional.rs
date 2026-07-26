//! Test suite for PDDL 3.1 features: Equality + Derived Predicates + Conditional Effects
//!
//! This test suite validates the three core 80/20 PDDL 3.1 features:
//! - Phase 1: Equality in preconditions
//! - Phase 2: Derived predicates with quantified bodies (forall/exists)
//! - Phase 3: Conditional effects (when with condition evaluation)

use bcinr_pddl::ground::GroundTemporalProblem;
use bcinr_pddl::{domain_from_pddl, problem_from_pddl};
use bcinr_mfw_ir::PlannerOutcome;

/// Phase 1: Equality in preconditions
/// Uses durative actions so we can test with GroundTemporalProblem
#[test]
fn phase1_equality_in_durative_preconditions() {
    let domain = domain_from_pddl(
        r#"(define (domain equality-test)
             (:requirements :durative-actions :typing)
             (:types item)
             (:predicates (ready ?x - item) (done ?x - item))
             (:durative-action self-check
               :parameters (?x - item ?y - item)
               :duration (= ?duration 1)
               :condition (and (at start (ready ?x)) (at start (= ?x ?y)))
               :effect (at end (done ?x))))"#,
    )
    .expect("failed to parse domain");

    let problem = problem_from_pddl(
        r#"(define (problem eq-test)
             (:domain equality-test)
             (:objects a - item)
             (:init (ready a))
             (:goal (done a)))"#,
    )
    .expect("failed to parse problem");

    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground problem");

    let outcome = ground.find_temporal_plan();
    assert!(
        matches!(outcome, PlannerOutcome::Found(_)),
        "expected plan with equality in durative precondition"
    );
}

/// Phase 2: Derived predicates with forall bodies
/// Grounds successfully with forall in derived body
#[test]
fn phase2_derived_predicate_with_forall_body_parses() {
    let domain = domain_from_pddl(
        r#"(define (domain derived-forall-test)
             (:requirements :durative-actions :typing :derived-predicates)
             (:types item)
             (:predicates (ready ?x - item) (all-ready))
             (:derived (all-ready)
               (forall (?x - item) (ready ?x)))
             (:durative-action finish-all
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (all-ready))
               :effect (at end (all-ready))))"#,
    )
    .expect("failed to parse domain with forall derived predicate");

    let problem = problem_from_pddl(
        r#"(define (problem derived-forall-prob)
             (:domain derived-forall-test)
             (:objects a b c - item)
             (:init (ready a) (ready b) (ready c))
             (:goal (all-ready)))"#,
    )
    .expect("failed to parse problem");

    // Key test: grounding should not fail with forall in derived body
    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground problem with forall in derived body");

    // Verify derived predicates were grounded (should have entries for each parameter binding)
    assert!(
        !ground.derived_predicates.is_empty(),
        "expected derived predicates to be grounded"
    );
}

/// Phase 2 variant: Derived predicate with exists body
#[test]
fn phase2_derived_predicate_with_exists_body_parses() {
    let domain = domain_from_pddl(
        r#"(define (domain derived-exists-test)
             (:requirements :durative-actions :typing :derived-predicates)
             (:types item)
             (:predicates (ready ?x - item) (any-ready))
             (:derived (any-ready)
               (exists (?x - item) (ready ?x)))
             (:durative-action finish-any
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (any-ready))
               :effect (at end (any-ready))))"#,
    )
    .expect("failed to parse domain with exists derived predicate");

    let problem = problem_from_pddl(
        r#"(define (problem derived-exists-prob)
             (:domain derived-exists-test)
             (:objects a b - item)
             (:init (ready a))
             (:goal (any-ready)))"#,
    )
    .expect("failed to parse problem");

    // Key test: grounding should not fail with exists in derived body
    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground problem with exists in derived body");

    // Verify derived predicates were grounded
    assert!(
        !ground.derived_predicates.is_empty(),
        "expected derived predicates to be grounded"
    );
}

/// Phase 3: Conditional effects with when
/// Conditional effect that fires when condition holds
#[test]
fn phase3_conditional_effect_when_fires_on_condition() {
    let domain = domain_from_pddl(
        r#"(define (domain conditional-effect-test)
             (:requirements :durative-actions :conditional-effects)
             (:predicates (enabled) (done) (acted))
             (:durative-action act-if-enabled
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (enabled))
               :effect (and (at end (acted)) (at end (when (enabled) (done))))))"#,
    )
    .expect("failed to parse domain with conditional effects");

    let problem = problem_from_pddl(
        r#"(define (problem conditional-enabled)
             (:domain conditional-effect-test)
             (:init (enabled))
             (:goal (done)))"#,
    )
    .expect("failed to parse problem");

    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground problem");

    let outcome = ground.find_temporal_plan();
    assert!(
        matches!(outcome, PlannerOutcome::Found(_)),
        "expected plan when conditional effect's condition is true"
    );
}

/// Phase 3 variant: Conditional effect should not fire when condition is false
#[test]
fn phase3_conditional_effect_when_does_not_fire_without_condition() {
    let domain = domain_from_pddl(
        r#"(define (domain conditional-blocked-test)
             (:requirements :durative-actions :conditional-effects)
             (:predicates (enabled) (done) (acted))
             (:durative-action act-if-enabled
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (true))
               :effect (and (at end (acted)) (at end (when (enabled) (done))))))"#,
    )
    .expect("failed to parse domain");

    let problem = problem_from_pddl(
        r#"(define (problem conditional-disabled)
             (:domain conditional-blocked-test)
             (:init)
             (:goal (done)))"#,
    )
    .expect("failed to parse problem");

    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground problem");

    let outcome = ground.find_temporal_plan();
    assert!(
        matches!(outcome, PlannerOutcome::Exhausted(_)),
        "expected no plan when conditional effect's condition is false"
    );
}

/// Phase 3: Conditional effects with forall
/// An action with a forall effect containing conditional effects
#[test]
fn phase3_conditional_effect_with_forall() {
    let domain = domain_from_pddl(
        r#"(define (domain forall-conditional-test)
             (:requirements :durative-actions :typing :conditional-effects)
             (:types item)
             (:predicates (ready ?x - item) (done ?x - item))
             (:durative-action finish-all
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (forall (?x - item) (ready ?x)))
               :effect (at end (forall (?x - item) (when (ready ?x) (done ?x))))))"#,
    )
    .expect("failed to parse domain with forall conditional effects");

    let problem = problem_from_pddl(
        r#"(define (problem forall-cond-prob)
             (:domain forall-conditional-test)
             (:objects a b - item)
             (:init (ready a) (ready b))
             (:goal (and (done a) (done b))))"#,
    )
    .expect("failed to parse problem");

    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground problem");

    let outcome = ground.find_temporal_plan();
    assert!(
        matches!(outcome, PlannerOutcome::Found(_)),
        "expected plan with forall conditional effects"
    );
}
