#![cfg(feature = "mfw-planner")]

use bcinr_pddl::{execute_pddl_to_powl, PddlPowlConfig, PddlPowlRuntime};

const DOMAIN: &str = "(define (domain cognitive-compose)
  (:requirements :strips :typing)
  (:types signal)
  (:predicates (available ?s - signal) (classified ?s - signal) (routed ?s - signal))
  (:action classify
    :parameters (?s - signal)
    :precondition (available ?s)
    :effect (classified ?s))
  (:action route
    :parameters (?s - signal)
    :precondition (classified ?s)
    :effect (routed ?s)))";

const PROBLEM: &str = "(define (problem compose-one)
  (:domain cognitive-compose)
  (:objects input - signal)
  (:init (available input))
  (:goal (routed input)))";

#[test]
fn public_one_shot_api_plans_executes_and_replays() {
    let execution = execute_pddl_to_powl(DOMAIN, PROBLEM).unwrap();

    assert_eq!(
        execution.execution_batches().unwrap(),
        vec![
            vec!["classify(input)".to_string()],
            vec!["route(input)".to_string()]
        ]
    );
    assert!(execution.contains_fact("routed", &["input"]));
    execution.verify().unwrap();
}

#[test]
fn reusable_runtime_preserves_configuration_and_emits_independent_receipts() {
    let config = PddlPowlConfig {
        max_execution_ticks: 8,
        ..PddlPowlConfig::default()
    };
    let mut runtime = PddlPowlRuntime::new(config);

    let first = runtime.execute(DOMAIN, PROBLEM).unwrap();
    let second = runtime.execute(DOMAIN, PROBLEM).unwrap();

    first.verify().unwrap();
    second.verify().unwrap();
    assert_eq!(first.state_receipt.chain_root, second.state_receipt.chain_root);
    assert!(second.workflow.cache_hit);
}
