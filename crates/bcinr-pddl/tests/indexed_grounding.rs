use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundProblem};
use bcinr_pddl::ground::lazy::IndexedGroundProblem;

#[test]
fn test_differential_grounding() {
    let domain_pddl = r#"
        (define (domain logistics)
            (:requirements :typing)
            (:types truck location package city)
            (:predicates
                (at ?obj - object ?loc - location)
                (in ?pkg - package ?veh - truck)
            )
            (:action load
                :parameters (?pkg - package ?veh - truck ?loc - location)
                :precondition (and (at ?pkg ?loc) (at ?veh ?loc))
                :effect (and (not (at ?pkg ?loc)) (in ?pkg ?veh))
            )
        )
    "#;
    let problem_pddl = r#"
        (define (problem log1)
            (:domain logistics)
            (:objects
                t1 t2 - truck
                l1 l2 - location
                p1 p2 - package
            )
            (:init
                (at t1 l1)
                (at t2 l2)
                (at p1 l2)
                (at p2 l1)
            )
            (:goal (and (at p1 l1) (at p2 l2)))
        )
    "#;

    let domain = domain_from_pddl(domain_pddl).unwrap();
    let problem = problem_from_pddl(problem_pddl).unwrap();

    let naive = GroundProblem::build(&domain, &problem, None).unwrap();
    let lazy = IndexedGroundProblem::build(&domain, &problem, None).unwrap();

    let naive_plan = naive.find_plan().unwrap();
    let lazy_plan = match lazy.find_plan() {
        bcinr_pddl::PlannerOutcome::Found(p) => p,
        _ => panic!("Expected Found"),
    };

    assert_eq!(naive_plan.ops.len(), lazy_plan.ops.len(), "Plans should have same length");
}
