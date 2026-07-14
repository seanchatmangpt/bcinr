use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundTemporalProblem};

fn main() {
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

    println!("Makespan 1: {}", plan1.makespan);
    println!("Makespan 2: {}", plan2.makespan);
}
