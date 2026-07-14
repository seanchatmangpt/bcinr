use bcinr_pddl::{Pddl8Domain, Pddl8Problem, GroundProblem, GroundTemporalProblem, domain_from_pddl, problem_from_pddl};

#[test]
fn test_numeric_cost() {
    let domain = domain_from_pddl("(define (domain d) (:requirements :numeric-fluents) (:predicates (p)) (:functions (cost)) (:action a :parameters () :precondition (>= (cost) 10) :effect (p)))").unwrap();
    let problem1 = problem_from_pddl("(define (problem p1) (:domain d) (:init (= (cost) 5)) (:goal (p)))").unwrap();
    let problem2 = problem_from_pddl("(define (problem p2) (:domain d) (:init (= (cost) 15)) (:goal (p)))").unwrap();
    
    let gp1 = GroundProblem::build(&domain, &problem1, None).unwrap();
    assert!(gp1.find_plan().into_result().is_err(), "cost 5 should not find a plan");
    
    let gp2 = GroundProblem::build(&domain, &problem2, None).unwrap();
    assert!(gp2.find_plan().into_result().is_ok(), "cost 15 should find a plan");
}

#[test]
fn test_derived_predicates() {
    let domain1 = domain_from_pddl("(define (domain d) (:requirements :derived-predicates) (:predicates (has-a) (ready)) (:derived (ready) (has-a)) (:action a :parameters () :precondition () :effect (has-a)))").unwrap();
    let domain2 = domain_from_pddl("(define (domain d) (:requirements :derived-predicates) (:predicates (has-a) (ready)) (:action a :parameters () :precondition () :effect (has-a)))").unwrap();
    let problem = problem_from_pddl("(define (problem p1) (:domain d) (:init) (:goal (ready)))").unwrap();
    
    let gp1 = GroundProblem::build(&domain1, &problem, None).unwrap();
    assert!(gp1.find_plan().into_result().is_ok(), "with derivation rule, should find a plan");
    
    let gp2 = GroundProblem::build(&domain2, &problem, None).unwrap();
    assert!(gp2.find_plan().into_result().is_err(), "without derivation rule, should not find a plan");
}

#[test]
fn test_trajectory_constraints() {
    let domain = domain_from_pddl("(define (domain d) (:requirements :constraints) (:predicates (p) (q)) (:action do-p :parameters () :precondition () :effect (p)) (:action do-q :parameters () :precondition (p) :effect (q)))").unwrap();
    let problem1 = problem_from_pddl("(define (problem c1) (:domain d) (:init) (:goal (q)))").unwrap();
    let problem2 = problem_from_pddl("(define (problem c2) (:domain d) (:init) (:goal (q)) (:constraints (and (always (not (p))))))").unwrap();
    
    let gp1 = GroundProblem::build(&domain, &problem1, None).unwrap();
    assert!(gp1.find_plan().into_result().is_ok(), "without constraint, should find a plan");
    
    let gp2 = GroundProblem::build(&domain, &problem2, None).unwrap();
    assert!(gp2.find_plan().into_result().is_err(), "with always not p constraint, should fail");
}

#[test]
fn test_til_schedule() {
    use bcinr_pddl::GroundTemporalProblem;
    let domain_str = "(define (domain d)
        (:requirements :durative-actions :timed-initial-literals)
        (:predicates (permission) (done))
        (:durative-action do-it
            :parameters ()
            :duration (= ?duration 5)
            :condition (and (at start (permission)))
            :effect (and (at end (done)))
        )
    )";
    let p1_str = "(define (problem p1)
        (:domain d)
        (:init (at 10 (permission)))
        (:goal (done))
    )";
    let p2_str = "(define (problem p2)
        (:domain d)
        (:init (at 40 (permission)))
        (:goal (done))
    )";

    let domain = domain_from_pddl(domain_str).unwrap();
    let p1 = problem_from_pddl(p1_str).unwrap();
    let p2 = problem_from_pddl(p2_str).unwrap();

    let gp1 = GroundTemporalProblem::build(&domain, &p1).unwrap();
    let gp2 = GroundTemporalProblem::build(&domain, &p2).unwrap();

    let plan1 = gp1.find_temporal_plan().into_result().unwrap();
    let plan2 = gp2.find_temporal_plan().into_result().unwrap();

    assert_eq!(plan1.makespan, 15.0);
    assert_eq!(plan2.makespan, 45.0);
}
