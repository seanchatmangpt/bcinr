//! Canonical classical-planning conformance fixtures.
//!
//! These are compact, bounded instances of the domain families repeatedly
//! used by the International Planning Competition and planning literature:
//! Logistics, Blocks World, and Gripper. The tests pin parser, typing,
//! grounding, shortest-path search, execution, and receipt determinism.

use std::collections::BTreeSet;

use bcinr_pddl::{
    domain_from_pddl, execute_tape, problem_from_pddl, GroundProblem, Pddl8GroundAtom,
};

fn execute(
    domain_text: &str,
    problem_text: &str,
    case_id: &str,
) -> (bcinr_pddl::Pddl8Tape, bcinr_pddl::Pddl8ExecutionReceipt) {
    let domain = domain_from_pddl(domain_text).unwrap();
    let problem = problem_from_pddl(problem_text).unwrap();
    let grounded = GroundProblem::build(&domain, &problem, None).unwrap();
    let tape = grounded.find_plan().into_result().unwrap();
    let initial = problem
        .init
        .iter()
        .map(|atom| Pddl8GroundAtom {
            pred: atom.pred.clone(),
            args: atom.args.clone(),
        })
        .collect::<BTreeSet<_>>();
    let goal = problem
        .goal
        .iter()
        .map(|atom| Pddl8GroundAtom {
            pred: atom.pred.clone(),
            args: atom.args.clone(),
        })
        .collect::<Vec<_>>();
    let (_, receipt, _) = execute_tape(&tape, &initial, &goal, case_id, &[]).unwrap();
    (tape, receipt)
}

const LOGISTICS_DOMAIN: &str = r#"
(define (domain logistics)
  (:requirements :strips :typing)
  (:types package truck location)
  (:predicates
    (at ?x - object ?l - location)
    (in ?p - package ?t - truck)
    (connected ?from - location ?to - location))
  (:action load-truck
    :parameters (?p - package ?t - truck ?l - location)
    :precondition (and (at ?p ?l) (at ?t ?l))
    :effect (and (in ?p ?t) (not (at ?p ?l))))
  (:action drive-truck
    :parameters (?t - truck ?from - location ?to - location)
    :precondition (and (at ?t ?from) (connected ?from ?to))
    :effect (and (at ?t ?to) (not (at ?t ?from))))
  (:action unload-truck
    :parameters (?p - package ?t - truck ?l - location)
    :precondition (and (in ?p ?t) (at ?t ?l))
    :effect (and (at ?p ?l) (not (in ?p ?t)))))
"#;

const LOGISTICS_PROBLEM: &str = r#"
(define (problem logistics-one-package)
  (:domain logistics)
  (:objects pkg1 - package truck1 - truck a b - location)
  (:init (at pkg1 a) (at truck1 a) (connected a b))
  (:goal (at pkg1 b)))
"#;

#[test]
fn typed_logistics_requires_load_drive_unload_in_that_order() {
    let domain = domain_from_pddl(LOGISTICS_DOMAIN).unwrap();
    let problem = problem_from_pddl(LOGISTICS_PROBLEM).unwrap();
    let grounded = GroundProblem::build(&domain, &problem, None).unwrap();

    // 1 package × 1 truck × 2 locations for load and unload, plus
    // 1 truck × 2 × 2 locations for drive. Type filtering excludes packages
    // as trucks and locations as movable objects.
    assert_eq!(grounded.actions.len(), 8);

    let tape = grounded.find_plan().into_result().unwrap();
    let names = tape
        .ops
        .iter()
        .map(|op| op.action.schema_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["load-truck", "drive-truck", "unload-truck"]);
    assert_eq!(tape.ops[0].action.label, "load-truck(pkg1,truck1,a)");
    assert_eq!(tape.ops[1].action.label, "drive-truck(truck1,a,b)");
    assert_eq!(tape.ops[2].action.label, "unload-truck(pkg1,truck1,b)");
}

#[test]
fn logistics_execution_and_receipt_are_deterministic() {
    let (first_tape, first) = execute(LOGISTICS_DOMAIN, LOGISTICS_PROBLEM, "ipc-logistics-1");
    let (second_tape, second) = execute(LOGISTICS_DOMAIN, LOGISTICS_PROBLEM, "ipc-logistics-1");
    assert_eq!(first_tape.ops.len(), 3);
    assert!(first.goal_reached);
    assert_eq!(first.plan_root, second.plan_root);
    assert_eq!(first.state_root, second.state_root);
    assert_eq!(first.chain_hash, second.chain_hash);
}

#[test]
fn logistics_refuses_unreachable_reverse_route() {
    let problem = r#"
    (define (problem unreachable)
      (:domain logistics)
      (:objects pkg1 - package truck1 - truck a b - location)
      (:init (at pkg1 a) (at truck1 b) (connected a b))
      (:goal (at pkg1 b)))
    "#;
    let domain = domain_from_pddl(LOGISTICS_DOMAIN).unwrap();
    let problem = problem_from_pddl(problem).unwrap();
    let grounded = GroundProblem::build(&domain, &problem, None).unwrap();
    assert!(grounded.find_plan().into_result().is_err());
}

const BLOCKS_DOMAIN: &str = r#"
(define (domain blocks)
  (:requirements :strips :typing)
  (:types block)
  (:predicates (on ?x - block ?y - block) (ontable ?x - block)
               (clear ?x - block) (holding ?x - block) (handempty))
  (:action pick-up
    :parameters (?x - block)
    :precondition (and (clear ?x) (ontable ?x) (handempty))
    :effect (and (holding ?x) (not (ontable ?x))
                 (not (clear ?x)) (not (handempty))))
  (:action put-down
    :parameters (?x - block)
    :precondition (holding ?x)
    :effect (and (ontable ?x) (clear ?x) (handempty) (not (holding ?x))))
  (:action stack
    :parameters (?x - block ?y - block)
    :precondition (and (holding ?x) (clear ?y))
    :effect (and (on ?x ?y) (clear ?x) (handempty)
                 (not (holding ?x)) (not (clear ?y))))
  (:action unstack
    :parameters (?x - block ?y - block)
    :precondition (and (on ?x ?y) (clear ?x) (handempty))
    :effect (and (holding ?x) (clear ?y) (not (on ?x ?y))
                 (not (clear ?x)) (not (handempty)))))
"#;

#[test]
fn blocks_world_two_block_instance_has_the_canonical_two_step_plan() {
    let problem = r#"
    (define (problem stack-a-on-b)
      (:domain blocks)
      (:objects a b - block)
      (:init (ontable a) (ontable b) (clear a) (clear b) (handempty))
      (:goal (on a b)))
    "#;
    let (tape, receipt) = execute(BLOCKS_DOMAIN, problem, "ipc-blocks-1");
    let names = tape
        .ops
        .iter()
        .map(|op| op.action.schema_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["pick-up", "stack"]);
    assert!(receipt.goal_reached);
}

const GRIPPER_DOMAIN: &str = r#"
(define (domain gripper)
  (:requirements :strips :typing)
  (:types ball room gripper)
  (:predicates (at-robby ?r - room) (at ?b - ball ?r - room)
               (free ?g - gripper) (carry ?b - ball ?g - gripper))
  (:action move
    :parameters (?from - room ?to - room)
    :precondition (at-robby ?from)
    :effect (and (at-robby ?to) (not (at-robby ?from))))
  (:action pick
    :parameters (?b - ball ?r - room ?g - gripper)
    :precondition (and (at ?b ?r) (at-robby ?r) (free ?g))
    :effect (and (carry ?b ?g) (not (at ?b ?r)) (not (free ?g))))
  (:action drop
    :parameters (?b - ball ?r - room ?g - gripper)
    :precondition (and (carry ?b ?g) (at-robby ?r))
    :effect (and (at ?b ?r) (free ?g) (not (carry ?b ?g)))))
"#;

#[test]
fn gripper_one_ball_instance_has_pick_move_drop_plan() {
    let problem = r#"
    (define (problem move-one-ball)
      (:domain gripper)
      (:objects ball1 - ball rooma roomb - room left - gripper)
      (:init (at-robby rooma) (at ball1 rooma) (free left))
      (:goal (at ball1 roomb)))
    "#;
    let (tape, receipt) = execute(GRIPPER_DOMAIN, problem, "ipc-gripper-1");
    let names = tape
        .ops
        .iter()
        .map(|op| op.action.schema_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["pick", "move", "drop"]);
    assert!(receipt.goal_reached);
}
