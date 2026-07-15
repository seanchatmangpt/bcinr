use bcinr_pddl::{
    domain_from_pddl, powl_bridge::temporal_plan_to_powl_tape, problem_from_pddl,
    GroundTemporalProblem,
};
use divan::{bench, black_box, Bencher};

fn main() {
    divan::main();
}

fn generate_fixture(n: usize) -> (String, String) {
    let domain = r#"(define (domain deploy-services)
    (:requirements :durative-actions :typing)
    (:types service)
    (:predicates (deployed ?s - service))
    (:durative-action deploy
        :parameters (?s - service)
        :duration (= ?duration 10)
        :condition ()
        :effect (and (at end (deployed ?s)))
    )
)"#
    .to_string();

    let mut objects = String::new();
    let mut goals = String::new();
    for i in 1..=n {
        objects.push_str(&format!("s{} ", i));
        goals.push_str(&format!("(deployed s{}) ", i));
    }

    let problem = format!(
        r#"(define (problem deploy-n)
    (:domain deploy-services)
    (:objects {} - service)
    (:init)
    (:goal (and {}))
)"#,
        objects.trim(),
        goals.trim()
    );

    (domain, problem)
}

const ARGS: &[usize] = &[8, 16, 32, 64, 128, 256, 512];

#[bench(args = ARGS)]
fn bench_ir(bencher: Bencher, n: usize) {
    let (domain_pddl, problem_pddl) = generate_fixture(n);
    bencher.bench_local(|| {
        let domain = domain_from_pddl(black_box(&domain_pddl)).unwrap();
        let problem = problem_from_pddl(black_box(&problem_pddl)).unwrap();
        black_box((domain, problem));
    });
}

#[bench(args = ARGS)]
fn bench_ground(bencher: Bencher, n: usize) {
    let (domain_pddl, problem_pddl) = generate_fixture(n);
    let domain = domain_from_pddl(&domain_pddl).unwrap();
    let problem = problem_from_pddl(&problem_pddl).unwrap();

    bencher.bench_local(|| {
        let gp = GroundTemporalProblem::build(black_box(&domain), black_box(&problem)).unwrap();
        black_box(gp);
    });
}

#[bench(args = ARGS)]
fn bench_solve(bencher: Bencher, n: usize) {
    let (domain_pddl, problem_pddl) = generate_fixture(n);
    let domain = domain_from_pddl(&domain_pddl).unwrap();
    let problem = problem_from_pddl(&problem_pddl).unwrap();
    let gp = GroundTemporalProblem::build(&domain, &problem).unwrap();

    bencher.bench_local(|| {
        let plan = gp.find_temporal_plan().into_result().unwrap();
        black_box(plan);
    });
}

#[bench(args = ARGS)]
fn bench_powl(bencher: Bencher, n: usize) {
    let (domain_pddl, problem_pddl) = generate_fixture(n);
    let domain = domain_from_pddl(&domain_pddl).unwrap();
    let problem = problem_from_pddl(&problem_pddl).unwrap();
    let gp = GroundTemporalProblem::build(&domain, &problem).unwrap();
    let plan = gp.find_temporal_plan().into_result().unwrap();

    bencher.bench_local(|| {
        let tape = temporal_plan_to_powl_tape(black_box(&plan));
        black_box(tape);
    });
}

#[bench(args = ARGS)]
fn bench_e2e(bencher: Bencher, n: usize) {
    let (domain_pddl, problem_pddl) = generate_fixture(n);
    bencher.bench_local(|| {
        let domain = domain_from_pddl(black_box(&domain_pddl)).unwrap();
        let problem = problem_from_pddl(black_box(&problem_pddl)).unwrap();
        let gp = GroundTemporalProblem::build(&domain, &problem).unwrap();
        let plan = gp.find_temporal_plan().into_result().unwrap();
        let tape = temporal_plan_to_powl_tape(&plan);
        black_box(tape);
    });
}
