//! Repository manufacturing as a planning domain.
//!
//! The agents here are the actual participants in this repository's development
//! process: a governor with approval authority, coding agents, and verifier
//! agents (test runner, static analyzer, CI). The objects are the real planning
//! objects: repository, commit, file, task, test suite, pull request, receipt.
//!
//! This is not an analogy domain. The initial facts in `problem_grounded_instance`
//! are admitted consequences of events that actually occurred in this repository's
//! development, and the goal is the disposition those events were working toward.
//!
//! Closed-world discipline: PDDL treats an absent fact as false. That is unsafe
//! for partial logs — a workflow that was launched but whose completion was never
//! observed must NOT ground to `(not (workflow-completed w))`. Every task
//! therefore carries an explicit disposition predicate (`status-unknown`,
//! `status-alive`, `status-blocked`, ...), and `status-unknown` is the admitted
//! initial state. Only an admitted completion receipt may produce `status-alive`.

use bcinr_pddl::ground::GroundTemporalProblem;
use bcinr_pddl::{domain_from_pddl, problem_from_pddl, TemporalPlan};
use bcinr_mfw_ir::PlannerOutcome;

/// The coding-swarm domain, expressed with durative actions.
///
/// Durative actions are used deliberately: the audit of this crate found that
/// the classical rail (`GroundProblem`) discards negation, disjunction,
/// quantifiers, and equality when flattening preconditions, while the temporal
/// rail preserves the full `PddlCondition`. This domain depends on all four.
pub fn swarm_domain() -> &'static str {
    r#"
(define (domain repo-manufacturing)
  (:requirements
    :typing
    :durative-actions
    :equality
    :negative-preconditions
    :disjunctive-preconditions
    :existential-preconditions
    :universal-preconditions
    :conditional-effects
    :derived-predicates)

  (:types
    agent repository commit file task test-suite pull-request receipt - object
    coding-agent verifier-agent governor-agent - agent)

  (:predicates
    (has-access ?a - agent ?r - repository)
    (assigned ?a - coding-agent ?t - task)
    (approved ?t - task ?g - governor-agent)

    (base-resolved ?r - repository ?c - commit)
    (tree-materialized ?a - coding-agent ?r - repository ?c - commit)
    (doctrine-read ?a - coding-agent ?r - repository)
    (head-of ?r - repository ?c - commit)

    (touches ?t - task ?f - file)
    (generated-file ?f - file)
    (file-leased-to ?f - file ?a - coding-agent)
    (lease-held ?f - file)

    (inspected ?a - coding-agent ?f - file)
    (patch-applied ?a - coding-agent ?t - task)
    (hand-edited ?f - file)

    (test-required ?t - task ?s - test-suite)
    (test-passed ?t - task ?s - test-suite)
    (test-failed ?t - task ?s - test-suite)

    (committed ?t - task ?c - commit)
    (pr-opened ?t - task ?p - pull-request)
    (ci-passed ?p - pull-request)
    (receipt-sealed ?t - task ?r - receipt)

    ; Explicit dispositions. Absence of `status-alive` must never be read as
    ; failure; `status-unknown` is a distinct admitted state.
    (status-unknown ?t - task)
    (status-alive ?t - task)
    (status-blocked ?t - task))

  ; An agent may modify the repository only with approval, an exact resolved
  ; base, a materialized tree at that base, doctrine read, access, and a lease
  ; on every file the task touches.
  (:derived (ready-to-modify ?a - coding-agent ?t - task ?r - repository)
    (and
      (has-access ?a ?r)
      (doctrine-read ?a ?r)
      (exists (?g - governor-agent) (approved ?t ?g))
      (exists (?c - commit)
        (and (base-resolved ?r ?c)
             (tree-materialized ?a ?r ?c)
             (head-of ?r ?c)))
      (forall (?f - file)
        (imply (touches ?t ?f) (file-leased-to ?f ?a)))))

  ; Every required suite has passed.
  (:derived (locally-verified ?t - task)
    (forall (?s - test-suite)
      (imply (test-required ?t ?s) (test-passed ?t ?s))))

  ; A change is lawful when approved, locally verified, and no generated file
  ; was hand edited.
  (:derived (lawful-change ?t - task)
    (and
      (exists (?g - governor-agent) (approved ?t ?g))
      (locally-verified ?t)
      (forall (?f - file)
        (imply (and (touches ?t ?f) (generated-file ?f))
               (not (hand-edited ?f))))))

  (:durative-action resolve-base
    :parameters (?a - coding-agent ?r - repository ?c - commit)
    :duration (= ?duration 1)
    :condition (and (at start (has-access ?a ?r)) (at start (head-of ?r ?c)))
    :effect (at end (base-resolved ?r ?c)))

  (:durative-action materialize-tree
    :parameters (?a - coding-agent ?r - repository ?c - commit)
    :duration (= ?duration 1)
    :condition (and (at start (base-resolved ?r ?c)) (at start (has-access ?a ?r)))
    :effect (at end (tree-materialized ?a ?r ?c)))

  (:durative-action read-doctrine
    :parameters (?a - coding-agent ?r - repository)
    :duration (= ?duration 1)
    :condition (at start (has-access ?a ?r))
    :effect (at end (doctrine-read ?a ?r)))

  ; Inspection is non-exclusive: any number of coding agents may read the same
  ; file concurrently. Only the write lease is exclusive.
  (:durative-action inspect
    :parameters (?a - coding-agent ?r - repository ?f - file)
    :duration (= ?duration 1)
    :condition (at start (has-access ?a ?r))
    :effect (at end (inspected ?a ?f)))

  ; Exactly one coding agent may hold the write lease on a file. The
  ; negative precondition on `lease-held` is what enforces single-writer.
  (:durative-action claim-lease
    :parameters (?a - coding-agent ?f - file)
    :duration (= ?duration 1)
    :condition (at start (not (lease-held ?f)))
    :effect (and (at end (file-leased-to ?f ?a)) (at start (lease-held ?f))))

  ; The agent must have inspected the file it is about to modify. Expressed
  ; universally over the files the task touches, not against a hardcoded name.
  (:durative-action apply-patch
    :parameters (?a - coding-agent ?t - task ?r - repository)
    :duration (= ?duration 2)
    :condition (and
      (at start (assigned ?a ?t))
      (at start (ready-to-modify ?a ?t ?r))
      (at start (forall (?f - file) (imply (touches ?t ?f) (inspected ?a ?f)))))
    :effect (at end (patch-applied ?a ?t)))

  ; A verifier turns an applied patch into a pass or a fail. The conditional
  ; effect is what makes the outcome depend on observed state rather than being
  ; assumed.
  (:durative-action run-verifier
    :parameters (?v - verifier-agent ?t - task ?s - test-suite ?a - coding-agent)
    :duration (= ?duration 3)
    :condition (and
      (at start (patch-applied ?a ?t))
      (at start (test-required ?t ?s)))
    :effect (and
      (at end (when (not (test-failed ?t ?s)) (test-passed ?t ?s)))))

  ; Commit is gated on the derived lawful-change predicate, which folds in
  ; approval, universal test coverage, and the generated-file prohibition.
  (:durative-action commit-change
    :parameters (?a - coding-agent ?t - task ?c - commit)
    :duration (= ?duration 1)
    :condition (and
      (at start (lawful-change ?t))
      (at start (patch-applied ?a ?t)))
    :effect (at end (committed ?t ?c)))

  (:durative-action open-draft-pr
    :parameters (?a - coding-agent ?t - task ?c - commit ?p - pull-request)
    :duration (= ?duration 1)
    :condition (and
      (at start (committed ?t ?c))
      (at start (lawful-change ?t)))
    :effect (at end (pr-opened ?t ?p)))

  (:durative-action seal-receipt
    :parameters (?t - task ?p - pull-request ?rc - receipt)
    :duration (= ?duration 1)
    :condition (and
      (at start (pr-opened ?t ?p))
      (at start (lawful-change ?t)))
    :effect (and
      (at end (receipt-sealed ?t ?rc))
      (at end (status-alive ?t))
      (at end (not (status-unknown ?t)))))
)
"#
}

/// The grounded instance: objects and admitted facts drawn from this
/// repository's actual development, with `scheduler-refusal` as the task.
///
/// `coding_agents` is parameterized because the number of coding agents turns
/// out to decide reachability -- see `second_coding_agent_exhausts_plan_depth`.
pub fn swarm_problem_with(coding_agents: &str, goal: &str) -> String {
    let access: String = coding_agents
        .split_whitespace()
        .map(|a| format!("    (has-access {a} bcinr)\n"))
        .collect();
    format!(
        r#"
(define (problem scheduler-refusal-episode)
  (:domain repo-manufacturing)
  (:objects
    sean - governor-agent
    {coding_agents} - coding-agent
    cargo-test - verifier-agent
    bcinr - repository
    commit-a96e62c8 - commit
    fscheduler - file
    scheduler-refusal - task
    powl-suite - test-suite
    pr-18 - pull-request
    release-receipt - receipt)

  (:init
{access}    (has-access sean bcinr)
    (head-of bcinr commit-a96e62c8)
    (approved scheduler-refusal sean)
    (assigned claude-code-1 scheduler-refusal)
    (touches scheduler-refusal fscheduler)
    (test-required scheduler-refusal powl-suite)
    (status-unknown scheduler-refusal))

  (:goal {goal}))
"#
    )
}

/// The full episode as it actually stood: both coding agents present.
pub fn swarm_problem() -> String {
    swarm_problem_with(
        "claude-code-1 codex-1",
        "(and (receipt-sealed scheduler-refusal release-receipt) (status-alive scheduler-refusal))",
    )
}

/// Probe 1: does the domain parse at all?
///
/// This asserts structure, not merely `is_ok()` — a parse that silently dropped
/// the derived predicates or the durative actions would pass an `is_ok()` check
/// while destroying the entire point of the domain.
#[test]
fn swarm_domain_parses_with_all_structure_intact() {
    let domain = domain_from_pddl(swarm_domain()).expect("swarm domain must parse");

    assert_eq!(
        domain.durative_actions.len(),
        10,
        "all ten repository-manufacturing actions must survive parsing"
    );
    assert_eq!(
        domain.derived.len(),
        3,
        "ready-to-modify, locally-verified, lawful-change must all survive parsing"
    );

    let names: Vec<&str> = domain
        .durative_actions
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    for expected in [
        "resolve-base",
        "materialize-tree",
        "read-doctrine",
        "inspect",
        "claim-lease",
        "apply-patch",
        "run-verifier",
        "commit-change",
        "open-draft-pr",
        "seal-receipt",
    ] {
        assert!(
            names.contains(&expected),
            "action {expected} missing after parse; got {names:?}"
        );
    }
}

/// Probe 2: does the problem instance parse, and are the admitted facts intact?
#[test]
fn swarm_problem_parses_with_admitted_facts_intact() {
    let problem = problem_from_pddl(&swarm_problem()).expect("swarm problem must parse");

    assert_eq!(problem.objects.len(), 11, "all 11 objects must be admitted");

    // The disposition fact is the closed-world guard. If it is dropped, an
    // unobserved task silently reads as "not alive" rather than "unknown".
    let has_unknown = problem
        .init
        .iter()
        .any(|atom| atom.pred == "status-unknown");
    assert!(
        has_unknown,
        "status-unknown must be an admitted initial fact, not an absence"
    );
}

/// Probe 3: does the domain ground, and do the derived governance rules survive?
///
/// This is the probe that the parameterized-derived-predicate defect blocked.
/// Before the `parse_derived` fix, `(ready-to-modify ?a - coding-agent ...)` was
/// stored as a 9-argument atom whose arguments included the `-` separators and
/// the type names, so it exceeded the arity bound outright; a two-parameter head
/// stayed under the bound but grounded to a wrong-arity atom that could never
/// match its use site.
#[test]
fn swarm_domain_grounds_with_derived_governance_rules() {
    let domain = domain_from_pddl(swarm_domain()).expect("domain parses");
    let problem = problem_from_pddl(&swarm_problem()).expect("problem parses");

    let ground = GroundTemporalProblem::build(&domain, &problem).expect("swarm domain grounds");

    assert!(
        !ground.derived_predicates.is_empty(),
        "the three governance rules must produce ground instances"
    );

    // Every ground derived head must have the arity its declaration promises.
    // `ready-to-modify` takes three parameters, so a ground instance must carry
    // exactly three arguments -- not nine, and not a token stream containing "-".
    let ready: Vec<_> = ground
        .derived_predicates
        .iter()
        .filter(|d| d.head.pred == "ready-to-modify")
        .collect();
    assert!(
        !ready.is_empty(),
        "ready-to-modify must ground; got preds {:?}",
        ground
            .derived_predicates
            .iter()
            .map(|d| d.head.pred.as_str())
            .collect::<Vec<_>>()
    );
    for d in &ready {
        assert_eq!(
            d.head.args.len(),
            3,
            "ready-to-modify must ground to arity 3, got {:?}",
            d.head.args
        );
        assert!(
            !d.head.args.iter().any(|a| a == "-"),
            "a type separator must never appear as a ground argument: {:?}",
            d.head.args
        );
    }

    let lawful: Vec<_> = ground
        .derived_predicates
        .iter()
        .filter(|d| d.head.pred == "lawful-change")
        .collect();
    for d in &lawful {
        assert_eq!(
            d.head.args.len(),
            1,
            "lawful-change must ground to arity 1, got {:?}",
            d.head.args
        );
    }
}

fn plan_for(coding_agents: &str, goal: &str) -> PlannerOutcome<TemporalPlan> {
    let domain = domain_from_pddl(swarm_domain()).expect("domain parses");
    let problem =
        problem_from_pddl(&swarm_problem_with(coding_agents, goal)).expect("problem parses");
    let ground = GroundTemporalProblem::build(&domain, &problem).expect("grounds");
    ground.find_temporal_plan()
}

const FULL_GOAL: &str =
    "(and (receipt-sealed scheduler-refusal release-receipt) (status-alive scheduler-refusal))";

/// Probe 4: a single coding agent manufactures the change, and the plan respects
/// the governance ordering.
///
/// The ordering assertions are the point. A plan that reached the receipt while
/// committing before verification would satisfy the goal and still be unlawful.
#[test]
fn single_agent_reaches_a_sealed_receipt_in_governance_order() {
    let outcome = plan_for("claude-code-1", FULL_GOAL);

    let PlannerOutcome::Found(plan) = &outcome else {
        panic!("a single-agent episode must reach a sealed receipt; got {outcome:?}");
    };

    let labels: Vec<&str> = plan.steps.iter().map(|s| s.action_name.as_str()).collect();
    let first = |needle: &str| labels.iter().position(|l| l.contains(needle));

    let lease = first("claim-lease").expect("the lease must be acquired");
    let patch = first("apply-patch").expect("the patch must be applied");
    let verify = first("run-verifier").expect("the verifier must run");
    let commit = first("commit-change").expect("the change must be committed");
    let seal = first("seal-receipt").expect("the receipt must be sealed");

    assert!(
        lease < patch,
        "a patch must never precede its file lease: {labels:?}"
    );
    assert!(
        patch < verify,
        "verification must never precede the patch it verifies: {labels:?}"
    );
    assert!(
        verify < commit,
        "a commit must never precede verification: {labels:?}"
    );
    assert!(
        commit < seal,
        "a receipt must never precede the commit it covers: {labels:?}"
    );
}

/// Probe 5: the measured coordination defect, now fixed.
///
/// This test originally characterized a real coordination defect: adding a
/// second coding agent (`codex-1`, never assigned the task, never competing
/// for the lease) made the same goal unreachable, because
/// `find_temporal_plan_with_fn_overrides` had no guard against re-scheduling
/// a grounded durative-action instance that had already completed once --
/// `codex-1`'s own always-applicable actions kept re-firing every outer-loop
/// iteration alongside `claude-code-1`'s real work, so incidental work scaled
/// with agent count and exhausted the 64-step depth bound before the goal was
/// reached (measured then: 1 agent 56 steps, 2 agents `Bounded(PlanDepth)`).
///
/// Fixed this session (see `GroundTemporalProblem::find_temporal_plan_with_fn_overrides`'s
/// `completed` set): once a grounded instance runs to completion, it is never
/// rescheduled in the same trajectory, so an irrelevant second agent's
/// one-time-each actions no longer drown the search. Re-measured after the
/// fix: 1 agent 10 steps, 2 agents 15 steps -- both `Found`.
#[test]
fn second_coding_agent_still_reaches_goal() {
    let one = plan_for("claude-code-1", FULL_GOAL);
    let two = plan_for("claude-code-1 codex-1", FULL_GOAL);

    let PlannerOutcome::Found(one_plan) = one else {
        panic!("baseline: one agent must reach the goal; got {one:?}");
    };
    let PlannerOutcome::Found(two_plan) = two else {
        panic!(
            "the coordination defect this test used to characterize has been fixed \
             (see the doc comment) -- a second, irrelevant agent must not prevent \
             the goal from being reached; got {two:?}"
        );
    };

    eprintln!(
        "1 agent: {} steps; 2 agents: {} steps",
        one_plan.steps.len(),
        two_plan.steps.len()
    );

    // A second agent can add its own one-time-each actions to the trajectory
    // but must never blow up the plan length the way unbounded re-firing did.
    assert!(
        two_plan.steps.len() < 30,
        "two-agent plan length ({}) is far larger than the single-agent \
         baseline ({}) -- redundant re-firing may have regressed",
        two_plan.steps.len(),
        one_plan.steps.len()
    );
}

/// Probe 6: effect-idempotence closes the redundant-work mechanism.
///
/// The causal core of the episode is roughly: resolve-base, materialize-tree,
/// read-doctrine, inspect, claim-lease, apply-patch, run-verifier, commit,
/// open-pr, seal -- ten actions. This test used to characterize every action
/// without a self-negating precondition re-firing on every outer-loop
/// iteration (see `second_coding_agent_still_reaches_goal`'s doc comment for
/// the same underlying defect and its fix); with the `completed`-set guard in
/// place, the planner now produces exactly the minimal, causal-core plan.
#[test]
fn plan_length_matches_the_causal_core() {
    let PlannerOutcome::Found(plan) = plan_for("claude-code-1", FULL_GOAL) else {
        panic!("single-agent baseline must plan");
    };

    let causal_core = 10;
    let steps = plan.steps.len();

    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for s in &plan.steps {
        *counts.entry(s.action_name.as_str()).or_default() += 1;
    }
    eprintln!("plan length {steps} vs causal core {causal_core}; action counts: {counts:?}");

    assert_eq!(
        steps, causal_core,
        "the minimal plan should be exactly the causal core -- if this grows, \
         redundant re-firing may have regressed"
    );

    // Every action in the causal core fires exactly once: the `completed` set
    // now enforces this for every action, not just the two (`claim-lease`,
    // `seal-receipt`) whose own effects happened to negate their precondition.
    for (action, count) in &counts {
        assert_eq!(*count, 1, "{action} should fire exactly once, fired {count}x");
    }
    assert_eq!(
        counts.get("commit-change"),
        Some(&1),
        "commit-change must fire exactly once -- a real repository would take \
         duplicate side effects from a second commit"
    );
}
