#![cfg(feature = "mfw-planner")]

use bcinr_pddl::{execute_cognitive_pddl, execute_pddl_to_powl, CognitiveExecutionStanding};

#[test]
fn undeclared_negative_precondition_cannot_enter_the_concurrent_strips_rail() {
    let domain = "(define (domain d) (:requirements :strips) \
        (:predicates (locked) (done)) \
        (:action finish :parameters () :precondition (not (locked)) :effect (done)))";
    let problem = "(define (problem p) (:domain d) (:init) (:goal (done)))";

    assert!(execute_pddl_to_powl(domain, problem).is_err());

    let execution = execute_cognitive_pddl(domain, problem).unwrap();
    assert_eq!(
        execution.standing(),
        CognitiveExecutionStanding::ExactSequentialClassical
    );
    execution.verify().unwrap();
}

#[test]
fn undeclared_numeric_condition_cannot_be_flattened_by_the_concurrent_rail() {
    let domain = "(define (domain d) (:requirements :strips) \
        (:predicates (done)) (:functions (fuel)) \
        (:action finish :parameters () :precondition (>= (fuel) 1) \
          :effect (and (decrease (fuel) 1) (done))))";
    let problem = "(define (problem p) (:domain d) (:init (= (fuel) 1)) (:goal (done)))";

    assert!(execute_pddl_to_powl(domain, problem).is_err());

    let execution = execute_cognitive_pddl(domain, problem).unwrap();
    assert_eq!(
        execution.standing(),
        CognitiveExecutionStanding::ExactSequentialClassical
    );
    execution.verify().unwrap();
}

#[test]
fn undeclared_boolean_quantifier_and_equality_surface_routes_exactly() {
    let domain = "(define (domain d) (:requirements :strips :typing) (:types item) \
        (:predicates (ready ?x - item) (done)) \
        (:action finish :parameters (?x - item ?y - item) \
          :precondition (or (not (= ?x ?y)) (exists (?z - item) (ready ?z))) \
          :effect (done)))";
    let problem = "(define (problem p) (:domain d) (:objects a b - item) \
        (:init (ready a)) (:goal (done)))";

    assert!(execute_pddl_to_powl(domain, problem).is_err());

    let execution = execute_cognitive_pddl(domain, problem).unwrap();
    assert_eq!(
        execution.standing(),
        CognitiveExecutionStanding::ExactSequentialClassical
    );
    execution.verify().unwrap();
}

#[test]
fn pddl_plus_processes_are_typed_refusals_for_every_current_rail() {
    let domain = "(define (domain d) (:requirements :strips) \
        (:predicates (done)) (:functions (level)) \
        (:process drift :parameters () :precondition (>= (level) 0) \
          :effect (increase (level) 1)))";
    let problem = "(define (problem p) (:domain d) (:init (= (level) 0)) (:goal (done)))";

    assert!(execute_pddl_to_powl(domain, problem).is_err());
    assert!(execute_cognitive_pddl(domain, problem).is_err());
}
