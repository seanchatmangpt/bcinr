//! bcinr-controller: parse -> exact plan -> independent validate -> print.
//!
//! A solver-produced plan is never trusted without an independent replay
//! check (see `bcinr_pddl::validate::validate_plan`'s module doc). This
//! binary exists to enforce that discipline at the CLI boundary: plan, then
//! check the plan against the domain/problem's own semantics before
//! anything is printed as a "next mandate" for a human or downstream
//! process to act on.

use std::path::Path;
use std::process::exit;

use bcinr_pddl::{
    domain31_from_pddl, problem31_from_pddl, validate_plan, ExactClassicalError,
    ExactClassicalProblem, EXACT_MAX_GROUND_ACTIONS, EXACT_MAX_PLAN_DEPTH, EXACT_MAX_SEARCH_STATES,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        let program = args
            .first()
            .map(String::as_str)
            .unwrap_or("bcinr-controller");
        eprintln!("usage: {program} <domain.pddl> <problem.pddl>");
        exit(2);
    }
    let domain_path = Path::new(&args[1]);
    let problem_path = Path::new(&args[2]);

    let domain_text = match std::fs::read_to_string(domain_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "failed to read domain file {}: {err}",
                domain_path.display()
            );
            exit(1);
        }
    };
    let problem_text = match std::fs::read_to_string(problem_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "failed to read problem file {}: {err}",
                problem_path.display()
            );
            exit(1);
        }
    };

    let domain = match domain31_from_pddl(&domain_text) {
        Ok(domain) => domain,
        Err(err) => {
            eprintln!("domain parse failed: {err}");
            exit(1);
        }
    };
    let problem = match problem31_from_pddl(&problem_text) {
        Ok(problem) => problem,
        Err(err) => {
            eprintln!("problem parse failed: {err}");
            exit(1);
        }
    };

    let grounded = match ExactClassicalProblem::build(&domain, &problem, EXACT_MAX_GROUND_ACTIONS) {
        Ok(grounded) => grounded,
        Err(err) => {
            eprintln!("grounding failed: {err}");
            exit(1);
        }
    };

    // `find_plan`'s three bounded-search outcomes are reported distinctly on
    // purpose: a genuinely-exhausted search space (`NoPlan`) is a different
    // fact than a search that was merely cut off by a depth or state bound
    // (`PlanDepthBoundExceeded` / `SearchStateBoundExceeded`) -- the latter
    // two are inconclusive, not proof that no plan exists.
    let tape = match grounded.find_plan(EXACT_MAX_PLAN_DEPTH, EXACT_MAX_SEARCH_STATES) {
        Ok(tape) => tape,
        Err(ExactClassicalError::NoPlan) => {
            eprintln!(
                "planning failed: reachable state space exhausted -- no plan exists within the admitted classical semantics"
            );
            exit(1);
        }
        Err(ExactClassicalError::PlanDepthBoundExceeded { limit }) => {
            eprintln!(
                "planning failed: plan depth bound ({limit}) exceeded before a goal state was reached -- inconclusive, not proof no plan exists"
            );
            exit(1);
        }
        Err(ExactClassicalError::SearchStateBoundExceeded { limit }) => {
            eprintln!(
                "planning failed: search state bound ({limit}) exceeded before a goal state was reached -- inconclusive, not proof no plan exists"
            );
            exit(1);
        }
        Err(err) => {
            eprintln!("planning failed: {err}");
            exit(1);
        }
    };

    // Never trust a plan the solver claims is valid: replay it against an
    // independent implementation before treating it as fact.
    if let Err(violation) = validate_plan(&domain, &problem, &tape) {
        eprintln!("solver-produced plan failed independent validation: {violation}");
        exit(1);
    }

    for op in &tape.ops {
        println!("{}", op.label);
    }
}
