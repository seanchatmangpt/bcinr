//! Adversarial verification: does `GroundProblem::find_plan` (the classical
//! BFS `ExactBfsRail` wraps) distinguish a genuine proof of unreachability
//! from merely hitting `PDDL8_MAX_PLAN_DEPTH`?
//!
//! `find_temporal_plan_with_fn_overrides` (same file, `ground/mod.rs`) was
//! explicitly patched during the MFW retrofit to separate these two cases
//! (see its own doc comment: "the old code conflated both into the same
//! unit `Exhausted` variant, which silently claimed 'search exhausted its
//! frontier' even when the loop was cut off mid-progress"). This test checks
//! whether the classical `find_plan` BFS received the same fix.
//!
//! Fixture: a domain with a strictly linear 70-step chain
//! `p0 -> p1 -> ... -> p70`, one action per step, goal `p70`. A plan
//! genuinely exists (exactly 70 actions), but `PDDL8_MAX_PLAN_DEPTH == 64`
//! (see `wasm4pm_compat::pddl::PDDL8_MAX_PLAN_DEPTH`), so every BFS branch
//! gets silently pruned (`if path.len() > PDDL8_MAX_PLAN_DEPTH { continue }`)
//! before it can reach the goal.

use bcinr_pddl::ground::GroundProblem;
use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_mfw_ir::PlannerOutcome;

fn chain_domain_and_problem(n: usize) -> (String, String) {
    let preds: String = (0..=n).map(|i| format!("(p{i})")).collect::<Vec<_>>().join(" ");
    let actions: String = (0..n)
        .map(|i| {
            format!(
                "(:action step{i} :parameters () :precondition (p{i}) :effect (and (p{next} ) (not (p{i}))))",
                i = i,
                next = i + 1
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let domain = format!("(define (domain chain) (:predicates {preds}) {actions})");
    let problem = format!(
        "(define (problem chain-p) (:domain chain) (:init (p0)) (:goal (p{n})))"
    );
    (domain, problem)
}

#[test]
fn find_plan_conflates_depth_bound_with_genuine_exhaustion() {
    // PDDL8_MAX_PLAN_DEPTH is 64 (wasm4pm_compat::pddl). A 70-step chain has
    // a genuine plan that is strictly longer than the depth bound.
    let (domain_text, problem_text) = chain_domain_and_problem(70);
    let domain = domain_from_pddl(&domain_text).expect("domain must parse");
    let problem = problem_from_pddl(&problem_text).expect("problem must parse");
    let gp = GroundProblem::build(&domain, &problem, None).expect("must ground");

    let outcome = gp.find_plan();

    // CONFIRMED BUG, empirically reproduced: `find_plan` returns `Exhausted`
    // (a claimed *proof* that no plan exists) here, even though a genuine
    // 70-step plan for this exact domain/problem DOES exist — the only
    // reason BFS didn't find it is that every branch got silently
    // depth-pruned (`if path.len() > PDDL8_MAX_PLAN_DEPTH { continue }`,
    // ground/mod.rs) before it could reach the goal. The correct outcome is
    // `Bounded`, exactly as the sibling `find_temporal_plan_with_fn_overrides`
    // (same file) was explicitly patched during this retrofit to report for
    // the identical situation — see that function's own doc comment ("the
    // old code conflated both into the same unit `Exhausted` variant").
    // `find_plan` (the classical BFS `ExactBfsRail`/`MfwPortfolio`'s "exact
    // rail" wraps) never received the equivalent fix.
    match outcome {
        PlannerOutcome::Exhausted(w) => {
            assert!(
                w.frontier_empty,
                "sanity: Exhausted always sets frontier_empty=true here"
            );
            assert!(
                w.explored_states < 70,
                "the depth bound (64) truncated exploration well short of \
                 the 70 states a genuine solve would need to visit \
                 (explored_states={}) — this is a Bounded situation \
                 mislabeled as Exhausted",
                w.explored_states
            );
        }
        PlannerOutcome::Bounded(b) => {
            panic!(
                "find_plan now correctly reports Bounded ({b:?}) instead of \
                 Exhausted for a depth-bound cutoff — this test's premise \
                 (CONFIRMED BUG) is stale; update this test and the gap \
                 report, the conflation has been fixed."
            );
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

/// Sanity check: confirm the 70-step chain genuinely has a reachable goal
/// when the depth bound is not in the way (i.e. this is not a bad fixture
/// that's actually unreachable for some unrelated reason). We can't easily
/// raise PDDL8_MAX_PLAN_DEPTH, so instead we verify a *shorter* chain
/// (well within the depth bound) is solved correctly by the exact same
/// domain-generation logic, establishing the domain shape itself is sound.
#[test]
fn shorter_chain_well_within_depth_bound_is_genuinely_found() {
    let (domain_text, problem_text) = chain_domain_and_problem(5);
    let domain = domain_from_pddl(&domain_text).expect("domain must parse");
    let problem = problem_from_pddl(&problem_text).expect("problem must parse");
    let gp = GroundProblem::build(&domain, &problem, None).expect("must ground");

    let outcome = gp.find_plan();
    match outcome {
        PlannerOutcome::Found(tape) => {
            assert_eq!(tape.ops.len(), 5, "must use exactly the 5-step chain");
        }
        other => panic!("expected Found for a 5-step chain, got {other:?}"),
    }
}
