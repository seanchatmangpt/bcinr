#![cfg(feature = "mfw-planner")]

use bcinr_pddl::{
    execute_cognitive_pddl, CognitiveExecutionStanding, CognitivePddlExecution,
};

#[test]
fn external_consumer_gets_witnessed_parallel_strips_execution() {
    let execution = execute_cognitive_pddl(
        "(define (domain d) (:requirements :strips) \
         (:predicates (ready) (left) (right)) \
         (:action left :parameters () :precondition (ready) :effect (left)) \
         (:action right :parameters () :precondition (ready) :effect (right)))",
        "(define (problem p) (:domain d) (:init (ready)) \
         (:goal (and (left) (right))))",
    )
    .unwrap();

    assert_eq!(
        execution.standing(),
        CognitiveExecutionStanding::WitnessedConcurrentStrips
    );
    assert!(matches!(execution, CognitivePddlExecution::Concurrent(_)));
    assert!(execution
        .batches()
        .unwrap()
        .iter()
        .any(|batch| batch.actions.len() == 2));
    execution.verify().unwrap();
}

#[test]
fn external_consumer_gets_exact_sequential_adl_execution() {
    let execution = execute_cognitive_pddl(
        "(define (domain d) (:requirements :adl :typing) (:types item) \
         (:predicates (ready ?x - item) (done ?x - item)) \
         (:action finish-all :parameters () \
          :precondition (forall (?x - item) (ready ?x)) \
          :effect (forall (?x - item) (when (ready ?x) (done ?x)))))",
        "(define (problem p) (:domain d) (:objects a b - item) \
         (:init (ready a) (ready b)) (:goal (and (done a) (done b))))",
    )
    .unwrap();

    assert_eq!(
        execution.standing(),
        CognitiveExecutionStanding::ExactSequentialClassical
    );
    assert!(matches!(
        execution,
        CognitivePddlExecution::ExactSequential { .. }
    ));
    assert!(execution
        .batches()
        .unwrap()
        .iter()
        .any(|batch| batch.actions == vec!["finish-all".to_string()]));
    execution.verify().unwrap();
}
