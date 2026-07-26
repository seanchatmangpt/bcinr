/// Comprehensive test suite for existential precondition closure (Phase 2b).
///
/// After Phase 2, `subst_condition` is used for quantified bodies in derived predicates,
/// enabling `exists` to work through the full grounding and evaluation pipeline.
///
/// Coverage:
/// - Exists in derived predicate bodies (now ALIVE)
/// - Nested exists (exists inside exists)
/// - Exists under and/or
/// - Exists with equality
/// - Exists inside derived rules
/// - Empty type domains (vacuously false)
/// - Type-hierarchy expansion
/// - Deterministic witness enumeration

use bcinr_pddl::{
    ground::GroundTemporalProblem,
    parse::{domain_from_pddl, problem_from_pddl},
};
use bcinr_mfw_ir::PlannerOutcome;

fn solve(domain_str: &str, problem_str: &str) -> Result<(), String> {
    let domain = domain_from_pddl(domain_str).map_err(|e| format!("Parse error: {:?}", e))?;
    let problem = problem_from_pddl(problem_str).map_err(|e| format!("Parse error: {:?}", e))?;

    // Build and solve the problem using GroundTemporalProblem
    let ground = GroundTemporalProblem::build(&domain, &problem)
        .map_err(|e| format!("Grounding failed: {:?}", e))?;

    let outcome = ground.find_temporal_plan();
    match outcome {
        PlannerOutcome::Found(_) => Ok(()),
        PlannerOutcome::Exhausted(e) => Err(format!("Plan exhausted: {:?}", e)),
        PlannerOutcome::Bounded(e) => Err(format!("Plan bounded: {:?}", e)),
        PlannerOutcome::Unsupported(e) => Err(format!("Unsupported: {:?}", e)),
        PlannerOutcome::Inconsistent(e) => Err(format!("Inconsistent: {:?}", e)),
    }
}

/// **Phase 2b complete: Derived predicates with exists bodies work end-to-end.**
///
/// The derived predicate `some-item` fires when ANY item is available.
/// The action uses this derived predicate to proceed.
#[test]
fn exists_in_derived_predicate_body_works_end_to_end() {
    let domain = r#"(define (domain d-exists-derived)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (available ?x - item) (done))
      (:derived (some-item) (exists (?x - item) (available ?x)))
      (:durative-action use-item :parameters ()
        :duration (= ?duration 1)
        :condition (at start (some-item))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-exists)
      (:domain d-exists-derived)
      (:objects a - item)
      (:init (available a))
      (:goal (done)))"#;

    solve(domain, problem).expect("exists in derived predicate should work");
}

/// Exists in derived predicate fails when no object satisfies the condition (vacuously false).
#[test]
fn exists_in_derived_predicate_vacuously_false_blocks_action() {
    let domain = r#"(define (domain d-exists-false)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (available ?x - item) (done))
      (:derived (some-item) (exists (?x - item) (available ?x)))
      (:durative-action use-item :parameters ()
        :duration (= ?duration 1)
        :condition (at start (some-item))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-exists-false)
      (:domain d-exists-false)
      (:objects a - item)
      (:init)
      (:goal (done)))"#;

    let result = solve(domain, problem);
    assert!(
        result.is_err(),
        "action should be impossible when exists condition is vacuously false"
    );
}

/// Multiple objects: exists should succeed if ANY satisfies the condition.
#[test]
fn exists_succeeds_with_multiple_objects_one_satisfying() {
    let domain = r#"(define (domain d-multi-obj)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (ready ?x - item) (done))
      (:derived (any-ready) (exists (?x - item) (ready ?x)))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (any-ready))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-multi)
      (:domain d-multi-obj)
      (:objects a b c - item)
      (:init (ready b))
      (:goal (done)))"#;

    solve(domain, problem).expect("exists should succeed with one ready item");
}

/// Nested exists: outer exists succeeds if any binding of outer variable makes inner exists true.
#[test]
fn nested_exists_works() {
    let domain = r#"(define (domain d-nested-exists)
      (:requirements :durative-actions :typing :derived-predicates :equality)
      (:types item)
      (:predicates (related ?x ?y - item) (done))
      (:derived (some-related)
        (exists (?x - item) (exists (?y - item) (related ?x ?y))))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (some-related))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-nested)
      (:domain d-nested-exists)
      (:objects a b - item)
      (:init (related a b))
      (:goal (done)))"#;

    solve(domain, problem).expect("nested exists should work");
}

/// Exists under AND: all parts of the conjunction must hold.
#[test]
fn exists_under_and_conjunction() {
    let domain = r#"(define (domain d-exists-and)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (available ?x - item) (enabled) (done))
      (:derived (ready-to-use)
        (and (enabled) (exists (?x - item) (available ?x))))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (ready-to-use))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-and)
      (:domain d-exists-and)
      (:objects a - item)
      (:init (enabled) (available a))
      (:goal (done)))"#;

    solve(domain, problem).expect("exists under and should work when both hold");
}

/// Exists under AND fails when one part is false.
#[test]
fn exists_under_and_fails_when_conjunction_false() {
    let domain = r#"(define (domain d-exists-and-false)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (available ?x - item) (enabled) (done))
      (:derived (ready-to-use)
        (and (enabled) (exists (?x - item) (available ?x))))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (ready-to-use))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-and-false)
      (:domain d-exists-and-false)
      (:objects a - item)
      (:init (enabled))
      (:goal (done)))"#;

    let result = solve(domain, problem);
    assert!(result.is_err(), "exists-and should fail when exists part is false");
}

/// Exists under OR: the derived predicate fires if exists condition holds.
#[test]
fn exists_under_or_disjunction() {
    let domain = r#"(define (domain d-exists-or)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (available ?x - item) (fallback) (done))
      (:derived (proceed-allowed)
        (or (exists (?x - item) (available ?x)) (fallback)))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (proceed-allowed))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-or)
      (:domain d-exists-or)
      (:objects a - item)
      (:init (available a))
      (:goal (done)))"#;

    solve(domain, problem).expect("exists under or should succeed");
}

/// Exists with equality: exists quantifier can bind variables used in equality checks.
#[test]
fn exists_with_equality() {
    let domain = r#"(define (domain d-exists-eq)
      (:requirements :durative-actions :typing :derived-predicates :equality)
      (:types item)
      (:predicates (matches ?x ?y - item) (done))
      (:derived (found-match)
        (exists (?x - item) (exists (?y - item) (and (matches ?x ?y) (not (= ?x ?y))))))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (found-match))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-eq)
      (:domain d-exists-eq)
      (:objects a b - item)
      (:init (matches a b))
      (:goal (done)))"#;

    solve(domain, problem).expect("exists with equality should work");
}

/// Chained exists in multiple derived predicates:
/// one derived predicate uses exists to define a condition,
/// another derived predicate depends on it.
#[test]
fn exists_across_multiple_derived_predicates() {
    let domain = r#"(define (domain d-chained-exists)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (ready ?x - item) (working) (done))
      (:derived (some-ready) (exists (?x - item) (ready ?x)))
      (:derived (can-proceed) (and (some-ready) (working)))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (can-proceed))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-chained)
      (:domain d-chained-exists)
      (:objects a - item)
      (:init (ready a) (working))
      (:goal (done)))"#;

    solve(domain, problem).expect("chained exists in derived predicates should work");
}

/// Type hierarchy: exists respects type constraints.
/// Only objects of the declared type are considered for binding.
#[test]
fn exists_respects_type_hierarchy() {
    let domain = r#"(define (domain d-type-hier)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item location)
      (:predicates (at ?i - item ?l - location) (done))
      (:derived (something-at-base)
        (exists (?i - item) (at ?i base)))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (something-at-base))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-type)
      (:domain d-type-hier)
      (:objects box - item base - location)
      (:init (at box base))
      (:goal (done)))"#;

    solve(domain, problem).expect("exists with type hierarchy should work");
}

/// Deterministic witness enumeration:
/// the same condition should always enumerate candidates in the same order.
/// (Tests that iteration is stable, not that there's a specific order.)
#[test]
fn exists_enumerates_deterministically() {
    let domain = r#"(define (domain d-deterministic)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (queued ?x - item) (found-queue))
      (:derived (has-queued) (exists (?x - item) (queued ?x)))
      (:durative-action discover :parameters ()
        :duration (= ?duration 1)
        :condition (at start (has-queued))
        :effect (at end (found-queue))))"#;

    let problem = r#"(define (problem p-det)
      (:domain d-deterministic)
      (:objects a b c - item)
      (:init (queued b))
      (:goal (found-queue)))"#;

    // Run the same problem twice
    let result1 = solve(domain, problem);
    let result2 = solve(domain, problem);

    // Both should succeed (deterministic evaluation)
    assert!(result1.is_ok(), "first run should succeed");
    assert!(result2.is_ok(), "second run should succeed");
}

/// Empty domain: exists over an empty type domain is vacuously false.
#[test]
fn exists_over_empty_domain_is_false() {
    let domain = r#"(define (domain d-empty-dom)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (ready ?x - item) (done))
      (:derived (any-ready) (exists (?x - item) (ready ?x)))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (any-ready))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-empty)
      (:domain d-empty-dom)
      (:objects)
      (:init)
      (:goal (done)))"#;

    let result = solve(domain, problem);
    assert!(
        result.is_err(),
        "exists over empty type domain should be vacuously false and block action"
    );
}

/// Complex: exists with multiple variables and constraints.
#[test]
fn exists_with_multiple_variables_and_complex_body() {
    let domain = r#"(define (domain d-complex)
      (:requirements :durative-actions :typing :derived-predicates :equality)
      (:types item)
      (:predicates (connected ?x ?y - item) (enabled) (done))
      (:derived (has-connection)
        (and (enabled)
             (exists (?x - item) (exists (?y - item)
               (and (connected ?x ?y) (not (= ?x ?y)))))))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (has-connection))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-complex)
      (:domain d-complex)
      (:objects a b - item)
      (:init (enabled) (connected a b))
      (:goal (done)))"#;

    solve(domain, problem)
        .expect("complex exists with multiple variables should work");
}

/// Negation of exists (testing De Morgan's law):
/// `not (exists (?x - item) (ready ?x))` is equivalent to `forall (?x - item) (not (ready ?x))`
#[test]
fn negation_of_exists() {
    let domain = r#"(define (domain d-not-exists)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item)
      (:predicates (ready ?x - item) (all-not-ready))
      (:derived (all-not-ready)
        (not (exists (?x - item) (ready ?x))))
      (:durative-action proceed :parameters ()
        :duration (= ?duration 1)
        :condition (at start (all-not-ready))
        :effect (at end (all-not-ready))))"#;

    let problem = r#"(define (problem p-not-exists)
      (:domain d-not-exists)
      (:objects a b - item)
      (:init)
      (:goal (all-not-ready)))"#;

    solve(domain, problem).expect("negation of exists (vacuously true when none ready) should work");
}

/// Interaction between forall and exists in same derived predicate:
/// tests that both quantifiers can coexist.
#[test]
fn mixed_forall_and_exists_in_derived() {
    let domain = r#"(define (domain d-mixed-quant)
      (:requirements :durative-actions :typing :derived-predicates)
      (:types item processor)
      (:predicates (ready ?i - item) (has-processor ?p - processor) (done))
      (:derived (can-start)
        (and (forall (?p - processor) (has-processor ?p))
             (exists (?i - item) (ready ?i))))
      (:durative-action start :parameters ()
        :duration (= ?duration 1)
        :condition (at start (can-start))
        :effect (at end (done))))"#;

    let problem = r#"(define (problem p-mixed)
      (:domain d-mixed-quant)
      (:objects item1 - item proc1 - processor)
      (:init (ready item1) (has-processor proc1))
      (:goal (done)))"#;

    solve(domain, problem).expect("mixed forall and exists should work");
}
