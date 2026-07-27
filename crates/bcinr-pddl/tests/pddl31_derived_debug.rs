// Debug derived predicates

use bcinr_pddl::ground::GroundTemporalProblem;
use bcinr_pddl::{domain_from_pddl, problem_from_pddl};

#[test]
fn derived_simple() {
    let domain = domain_from_pddl(
        r#"(define (domain d)
             (:requirements :durative-actions :derived-predicates)
             (:predicates (p) (q))
             (:derived (q) (p))
             (:durative-action a
               :parameters ()
               :duration (= ?duration 1)
               :condition (at start (p))
               :effect (at end (q))))"#,
    )
    .expect("failed to parse");

    println!("Domain derived count: {}", domain.derived.len());

    let problem = problem_from_pddl(
        r#"(define (problem p)
             (:domain d)
             (:init (p))
             (:goal (q)))"#,
    )
    .expect("failed to parse");

    let ground = GroundTemporalProblem::build(&domain, &problem)
        .expect("failed to ground");

    println!("Ground derived predicates: {}", ground.derived_predicates.len());
    for (i, dp) in ground.derived_predicates.iter().enumerate() {
        println!("  [{}] {} => {:?}", i, dp.head.label(), dp.condition);
    }

    assert!(!ground.derived_predicates.is_empty(), "derived predicates should be grounded");
}
