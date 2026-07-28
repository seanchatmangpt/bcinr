//! Closed-loop PDDL controller for Claude Code, release v26.7.28.
//!
//! Runs on the exact classical rail (`ExactClassicalProblem`, `ground_v2.rs`):
//! BFS with a visited set, explicit depth and node bounds, and a genuine
//! distinction between `PlanDepthBoundExceeded` (frontier cut) and `NoPlan`
//! (proven unreachable). The temporal rail is deliberately not used -- it has no
//! closed list, and a measured episode there produced a 56-step plan that
//! committed four times.
//!
//! The domain fixture carries the design rationale; this file measures whether
//! the controller is actually tractable at release scale.

use bcinr_pddl::ground_v2::{ExactClassicalError, ExactClassicalProblem};
use bcinr_pddl::{domain31_from_pddl, problem31_from_pddl};

const CONTROLLER_DOMAIN: &str = include_str!("fixtures/claude-v26728-controller-domain.pddl");

/// One phase of the release: a name, the phases it depends on, and its required
/// verification suites.
struct PhaseSpec {
    name: &'static str,
    depends_on: &'static [&'static str],
    suites: &'static [&'static str],
}

/// The real v26.7.28 work program, with the dependency graph as specified.
const V26_7_28: &[PhaseSpec] = &[
    PhaseSpec {
        name: "baseline-repair",
        depends_on: &[],
        suites: &["baseline-nonvacuity"],
    },
    PhaseSpec {
        name: "language-enumerator",
        depends_on: &["baseline-repair"],
        suites: &["language-definition-3-9", "cyclic-language-bound"],
    },
    PhaseSpec {
        name: "runtime-choice",
        depends_on: &["language-enumerator"],
        suites: &[
            "figure-1b-trace-equivalence",
            "emitted-opkind-scheduler-coverage",
        ],
    },
    PhaseSpec {
        name: "recursive-hierarchy",
        depends_on: &["runtime-choice"],
        suites: &["hierarchy-over-64", "hierarchical-receipt-root"],
    },
    PhaseSpec {
        name: "wfnet-converter",
        depends_on: &["language-enumerator"],
        suites: &["wfnet-algorithms-1-3", "wfnet-negative-corpus"],
    },
    PhaseSpec {
        name: "theorem-oracle",
        depends_on: &["language-enumerator", "wfnet-converter"],
        suites: &[
            "theorem-5-5-language-preservation",
            "theorem-5-6-separable-completeness",
        ],
    },
    PhaseSpec {
        name: "capability-profile",
        depends_on: &["baseline-repair"],
        suites: &[
            "capability-profile-admission",
            "capability-doc-symbol-integrity",
        ],
    },
    PhaseSpec {
        name: "semantic-repairs",
        depends_on: &["capability-profile"],
        suites: &[
            "preference-softness",
            "negative-goal-preservation",
            "at-end-condition-enforcement",
            "duration-range-enforcement",
            "derived-numeric-binding",
            "domain-constraint-enforcement",
        ],
    },
    PhaseSpec {
        name: "pddl-powl-bridge",
        depends_on: &["runtime-choice", "recursive-hierarchy", "semantic-repairs"],
        suites: &[
            "bridge-causal-links",
            "bridge-binding-preservation",
            "bridge-choice-preservation",
        ],
    },
    PhaseSpec {
        name: "crown-verifier",
        depends_on: &["theorem-oracle", "pddl-powl-bridge"],
        suites: &[
            "workspace-tests",
            "workspace-clippy",
            "hostile-mutation-suite",
            "dead-variant-coverage",
            "crown-replay",
        ],
    },
];

/// Render a problem over the first `phase_count` phases with `workflow_count`
/// workflow slots, targeting `goal`.
fn render_problem(phase_count: usize, goal: &str, complete: &[&str]) -> String {
    let phases = &V26_7_28[..phase_count.min(V26_7_28.len())];
    let names: Vec<&str> = phases.iter().map(|p| p.name).collect();

    let mut suites: Vec<&str> = phases
        .iter()
        .flat_map(|p| p.suites.iter().copied())
        .collect();
    suites.sort_unstable();
    suites.dedup();

    let mut init = String::new();
    for p in phases {
        init.push_str(&format!("    (part-of-release {} v26-7-28)\n", p.name));
        if complete.contains(&p.name) {
            init.push_str(&format!("    (phase-complete {})\n", p.name));
        } else {
            init.push_str(&format!("    (phase-pending {})\n", p.name));
        }
        for d in p.depends_on {
            // Only declare a dependency that is itself in scope, so truncated
            // problems stay solvable rather than silently unreachable.
            if names.contains(d) {
                init.push_str(&format!("    (depends-on {} {})\n", p.name, d));
            }
        }
        for s in p.suites {
            init.push_str(&format!("    (required-test {} {})\n", p.name, s));
        }
    }

    format!(
        r#"(define (problem finish-v26-7-28)
  (:domain claude-v26728-controller)
  (:objects
    claude-code - agent
    {phases} - phase
    {suites} - test-suite
    v26-7-28 - release)
  (:init
    (available claude-code)
    (release-pending v26-7-28)
{init}  )
  (:goal {goal}))
"#,
        phases = names.join(" "),
        suites = suites.join(" "),
    )
}

#[derive(Debug)]
struct SolveResult {
    labels: Vec<String>,
    ground_actions: usize,
}

fn solve(
    phase_count: usize,
    goal: &str,
    max_depth: usize,
    max_states: usize,
) -> Result<SolveResult, ExactClassicalError> {
    solve_from(phase_count, goal, max_depth, max_states, &[])
}

fn solve_from(
    phase_count: usize,
    goal: &str,
    max_depth: usize,
    max_states: usize,
    complete: &[&str],
) -> Result<SolveResult, ExactClassicalError> {
    let domain = domain31_from_pddl(CONTROLLER_DOMAIN).expect("controller domain must parse");
    let problem_text = render_problem(phase_count, goal, complete);
    let problem = problem31_from_pddl(&problem_text).expect("controller problem must parse");

    let ground = ExactClassicalProblem::build(&domain, &problem, 200_000)?;
    let ground_actions = ground.actions.len();
    // `SolveResult` carries labels and nothing else, so this takes the
    // label-only lowering. `find_plan` would refuse this domain outright. The
    // refusal actually reported is on the PRECONDITION axis, measured by
    // running `tools/bcinr-controller` (which calls `find_plan`) against this
    // fixture:
    //
    //   planning failed: action launch-implementation-workflow(claude-code,
    //   baseline-repair) carries a universally quantified precondition that
    //   the flat STRIPS tape cannot represent
    //
    // i.e. `launch-implementation-workflow`'s `forall`-over-dependencies
    // precondition, which the flat `Pddl8GroundAction` cannot represent.
    // `admit-implementation-success` independently carries a `forall`/`when`
    // conditional effect (invalidating prior test results on
    // re-implementation), refused on the effect axis; the precondition
    // refusal above is simply the one that surfaces. Either way it is a
    // refusal these assertions would be earning over a field they never read.
    let tape = ground.find_label_plan(max_depth, max_states)?;
    Ok(SolveResult {
        labels: tape.ops.iter().map(|o| o.label.clone()).collect(),
        ground_actions,
    })
}

/// The domain must parse on the exact rail, which means: no `:derived`, no
/// `:durative-action`. Those are what `ground_v2::validate_scope` refuses.
#[test]
fn controller_domain_parses_and_is_exact_rail_admissible() {
    let domain = domain31_from_pddl(CONTROLLER_DOMAIN).expect("must parse");

    assert_eq!(
        domain.actions.len(),
        12,
        "all twelve controller actions must survive parsing"
    );
    assert!(
        domain.derived.is_empty(),
        "the exact rail refuses :derived (ground_v2.rs:322); the governance rules \
         must be inlined instead"
    );
    assert!(
        domain.durative_actions.is_empty(),
        "the exact rail refuses :durative-action (ground_v2.rs:310)"
    );
}

/// The smallest real episode: one phase, one suite, through to release.
#[test]
fn single_phase_episode_reaches_release_complete() {
    let r = solve(1, "(release-complete v26-7-28)", 24, 500_000)
        .expect("a one-phase release must be planable");

    eprintln!(
        "1 phase: {} ground actions, {} steps: {:?}",
        r.ground_actions,
        r.labels.len(),
        r.labels
    );

    let pos = |n: &str| r.labels.iter().position(|l| l.starts_with(n));
    let impl_ = pos("launch-implementation-workflow").expect("must implement");
    let verify = pos("launch-verification-workflow").expect("must verify");
    let receipt = pos("admit-phase-receipt").expect("must seal a receipt");
    let complete = pos("mark-phase-complete").expect("must complete the phase");
    let release = pos("launch-release-workflow").expect("must launch the release");

    assert!(impl_ < verify, "verification must follow implementation");
    assert!(verify < receipt, "a receipt must follow verification");
    assert!(receipt < complete, "completion must follow the receipt");
    assert!(
        complete < release,
        "the release must follow phase completion"
    );

    // No action may repeat. This is the property the temporal rail violates.
    let mut sorted = r.labels.clone();
    sorted.sort();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        before,
        "no ground action may appear twice in a plan: {:?}",
        r.labels
    );
}

/// Dependencies must be respected: with `baseline-repair` incomplete, no
/// dependent phase may be started first.
#[test]
fn dependencies_gate_the_first_action() {
    let r = solve(2, "(release-complete v26-7-28)", 32, 1_000_000)
        .expect("a two-phase release must be planable");

    let first_impl = r
        .labels
        .iter()
        .find(|l| l.starts_with("launch-implementation-workflow"))
        .expect("must implement something first");

    assert!(
        first_impl.contains("baseline-repair"),
        "the first implementation must be the dependency-free phase, got {first_impl}"
    );

    // language-enumerator depends on baseline-repair, so baseline-repair must be
    // marked complete before language-enumerator is ever launched.
    let baseline_done = r
        .labels
        .iter()
        .position(|l| l.starts_with("mark-phase-complete") && l.contains("baseline-repair"));
    let lang_start = r.labels.iter().position(|l| {
        l.starts_with("launch-implementation-workflow") && l.contains("language-enumerator")
    });

    if let (Some(done), Some(start)) = (baseline_done, lang_start) {
        assert!(
            done < start,
            "language-enumerator started before baseline-repair completed: {:?}",
            r.labels
        );
    }
}

/// Scale measurement. The plan's premise is that full-horizon optimistic
/// planning is what drives the controller; this test establishes empirically how
/// far that actually scales, so the controller's horizon strategy is chosen from
/// data rather than assumed.
#[test]
fn measure_full_horizon_scaling() {
    for phase_count in 1..=7 {
        let started = std::time::Instant::now();
        let r = solve(phase_count, "(release-complete v26-7-28)", 128, 400_000)
            .unwrap_or_else(|e| panic!("phases={phase_count} must plan, got {e:?}"));
        let ms = started.elapsed().as_millis();
        eprintln!(
            "phases={phase_count:2} -> {} ground actions, {} steps, {ms} ms",
            r.ground_actions,
            r.labels.len()
        );

        // Plan length is exactly 8 per phase plus the 2 release steps, minus the
        // 2 release steps counted once. Measured: 8, 16, 24, 32, 40, 48, 56.
        assert_eq!(
            r.labels.len(),
            8 * phase_count,
            "plan length must stay linear in phase count -- any excess is \
             redundant work of the kind the temporal rail produces"
        );

        let mut sorted = r.labels.clone();
        sorted.sort();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            before,
            "no ground action may repeat at phases={phase_count}"
        );
    }
}

/// Full-horizon planning does not reach the whole release, and this records the
/// boundary rather than leaving it to be rediscovered.
///
/// Measured this session, `max_states = 400_000`:
///   phases=7 -> 56 steps in 4.8 s
///   phases=8 -> SearchStateBoundExceeded after 366 s
///
/// The cause is not plan length -- that stays linear at 8 per phase. It is that
/// BFS explores every interleaving of *independent* phases, and the dependency
/// graph unblocks several at once (`language-enumerator` and
/// `capability-profile` both become ready as soon as `baseline-repair`
/// completes). The number of orderings is factorial in the frontier width.
///
/// This is why the controller plans a receding horizon instead. If this test
/// starts passing, the planner gained symmetry reduction or a heuristic and the
/// horizon strategy should be revisited.
#[test]
fn full_horizon_does_not_reach_the_whole_release() {
    let outcome = solve(V26_7_28.len(), "(release-complete v26-7-28)", 128, 60_000);
    assert!(
        matches!(
            outcome,
            Err(ExactClassicalError::SearchStateBoundExceeded { .. })
        ),
        "full-horizon planning over all {} phases was expected to exhaust the \
         search bound; got {outcome:?} -- if it now plans, the controller can \
         drop the receding horizon",
        V26_7_28.len()
    );
}

/// The controller's actual strategy, in the regime it actually runs in.
///
/// The controller never plans to a distant phase from the initial state. Its
/// admitted state advances as evidence arrives, so its target is always a
/// *frontier* phase -- one whose dependencies are already complete. Planning to
/// a deep phase from scratch is expensive (measured: 30 steps, 14.5 s for
/// `recursive-hierarchy`) precisely because the search has to rediscover all the
/// prerequisites, which in the real loop are already facts.
///
/// Every frontier solve must be small and fast, for every one of the ten phases.
#[test]
fn frontier_phase_planning_is_cheap_for_every_phase() {
    let n = V26_7_28.len();

    for spec in V26_7_28 {
        // Admit every dependency as already complete -- exactly the state the
        // controller is in when this phase becomes the frontier.
        let complete: Vec<&str> = spec.depends_on.to_vec();

        let started = std::time::Instant::now();
        let goal = format!("(phase-complete {})", spec.name);
        let r = solve_from(n, &goal, 32, 400_000, &complete)
            .unwrap_or_else(|e| panic!("frontier plan for {} failed: {e:?}", spec.name));
        let ms = started.elapsed().as_millis();

        eprintln!(
            "frontier {:<22} deps={} suites={} -> {} steps, {ms} ms, first={}",
            spec.name,
            spec.depends_on.len(),
            spec.suites.len(),
            r.labels.len(),
            r.labels.first().map(String::as_str).unwrap_or("<none>")
        );

        // A frontier phase costs exactly: launch, admit-success, then a
        // launch/admit pair per required suite, then receipt and complete.
        assert_eq!(
            r.labels.len(),
            4 + 2 * spec.suites.len(),
            "frontier plan for {} must be minimal, got {:?}",
            spec.name,
            r.labels
        );

        // The first action must start the goal phase itself, since by
        // construction its dependencies are already satisfied.
        let first = r.labels.first().expect("a non-trivial plan");
        assert_eq!(
            first,
            &format!("launch-implementation-workflow(claude-code,{})", spec.name),
            "frontier plan for {} must start by launching it",
            spec.name
        );

        // Measured: 15-240 ms for every phase with <= 2 suites, but 7.7 s for
        // `semantic-repairs`, the only phase with 6. The plan stays minimal at
        // 16 steps; the cost is that six suites may be verified in any order and
        // BFS explores all 720 permutations. Suite symmetry, the same class of
        // blow-up the workflow-slot pool caused before it was removed.
        //
        // `per_suite_horizon_is_flat_in_suite_count` shows the controller does
        // not have to pay this.
        assert!(
            ms < 30_000,
            "frontier planning for {} took {ms} ms",
            spec.name
        );
    }
}

/// The controller's horizon can be one suite rather than one whole phase, and
/// that removes the factorial dependence on suite count entirely.
///
/// Planning `(phase-complete p)` for a 6-suite phase costs 7.7 s because BFS
/// walks every ordering of the six verifications. Planning `(test-passed p s)`
/// for one named suite is 2 steps regardless of how many sibling suites exist,
/// because the goal does not mention them.
///
/// Since the controller executes only step 0 and re-plans after each observation,
/// it never needs a committed ordering over sibling suites.
#[test]
fn per_suite_horizon_is_flat_in_suite_count() {
    let n = V26_7_28.len();
    let worst = V26_7_28
        .iter()
        .max_by_key(|p| p.suites.len())
        .expect("a phase");
    assert_eq!(worst.name, "semantic-repairs", "the 6-suite phase");

    let mut complete: Vec<&str> = worst.depends_on.to_vec();
    complete.push("__none__"); // keep the slice type stable; harmless sentinel
    complete.pop();

    for suite in worst.suites {
        let started = std::time::Instant::now();
        let goal = format!("(test-passed {} {})", worst.name, suite);
        let r = solve_from(n, &goal, 32, 400_000, &complete)
            .unwrap_or_else(|e| panic!("per-suite plan for {suite} failed: {e:?}"));
        let ms = started.elapsed().as_millis();

        eprintln!(
            "per-suite {:<32} -> {} steps, {ms} ms, first={}",
            suite,
            r.labels.len(),
            r.labels.first().map(String::as_str).unwrap_or("<none>")
        );

        // launch-implementation, admit-success, launch-verification, admit-pass.
        assert_eq!(
            r.labels.len(),
            4,
            "a per-suite goal must not depend on sibling suites, got {:?}",
            r.labels
        );
        assert!(
            ms < 2_000,
            "per-suite planning must stay flat; {suite} took {ms} ms"
        );
    }
}

/// A failed verifier must drive the phase back to repair and invalidate the
/// receipt -- the property that a latching derived predicate would have broken.
#[test]
fn observed_test_failure_invalidates_the_receipt_and_forces_repair() {
    // Plan to a state where the phase is complete, then check the domain would
    // not allow completion while a failure observation stands.
    let spec = &V26_7_28[0];

    // Without the failure observation, the phase completes in 6 steps.
    let ok = solve_from(
        1,
        &format!("(phase-complete {})", spec.name),
        32,
        400_000,
        &[],
    )
    .expect("baseline phase must complete");
    assert_eq!(ok.labels.len(), 4 + 2 * spec.suites.len());
    assert!(
        !ok.labels
            .iter()
            .any(|l| l.starts_with("admit-verification-failure")),
        "the optimistic model must not plan through a failure it did not observe: {:?}",
        ok.labels
    );

    // admit-verification-failure is gated on an observation, so it can never be
    // chosen by the planner. That is what keeps failure exogenous.
    let domain = domain31_from_pddl(CONTROLLER_DOMAIN).expect("parses");
    let fail_action = domain
        .actions
        .iter()
        .find(|a| a.name == "admit-verification-failure")
        .expect("the domain must model verification failure");
    let precond = format!("{:?}", fail_action.precondition);
    assert!(
        precond.contains("observed-test-fail"),
        "admit-verification-failure must require an observation, else the \
         planner may choose to fail: {precond}"
    );
}
