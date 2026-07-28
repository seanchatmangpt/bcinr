//! Integration tests for temporal policy closure: trajectory constraints + monitors.
//!
//! Tests the full pipeline: PDDL parsing → grounding → temporal planning with constraint monitoring.

use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundTemporalProblem, PlannerOutcome};

#[test]
fn test_always_constraint_simple() {
    /// Simple domain with one predicate that must always be true.
    const DOMAIN: &str = r#"
(define (domain always-test)
  (:requirements :durative-actions)
  (:predicates (locked) (done))
  (:durative-action work
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (locked))
    :effect (at end (done))))
"#;

    const PROBLEM: &str = r#"
(define (problem always-test-1)
  (:domain always-test)
  (:init (locked))
  (:goal (done))
  (:constraints (always (locked))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("must ground");
    let outcome = gtp.find_temporal_plan();

    // The plan should be found because `locked` is true throughout
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

#[test]
fn test_sometime_constraint() {
    /// Domain where a condition must become true at some point.
    const DOMAIN: &str = r#"
(define (domain sometime-test)
  (:requirements :durative-actions :timed-initial-literals)
  (:predicates (ready) (success) (done))
  (:durative-action trigger
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (ready))
    :effect (at end (success)))
  (:durative-action finish
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (success))
    :effect (at end (done))))
"#;

    const PROBLEM: &str = r#"
(define (problem sometime-test-1)
  (:domain sometime-test)
  (:init (ready))
  (:goal (and (success) (done)))
  (:constraints (sometime (success))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("must ground");
    let outcome = gtp.find_temporal_plan();

    // Plan should succeed: `success` is achieved by the trigger action
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

#[test]
fn test_at_most_once_constraint() {
    /// Condition that can hold at most once.
    const DOMAIN: &str = r#"
(define (domain at-most-once-test)
  (:requirements :durative-actions)
  (:predicates (flag) (start-ready))
  (:durative-action set-flag
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (start-ready))
    :effect (at end (flag)))
  (:durative-action clear-flag
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (flag))
    :effect (at end (not (flag)))))
"#;

    const PROBLEM: &str = r#"
(define (problem at-most-once-1)
  (:domain at-most-once-test)
  (:init (start-ready))
  (:goal (not (flag)))
  (:constraints (at-most-once (flag))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("must ground");
    let outcome = gtp.find_temporal_plan();

    // Plan should succeed: flag is set once and then cleared, satisfying at-most-once
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

#[test]
fn test_sometime_before_constraint() {
    /// Condition C1 must hold before condition C2.
    const DOMAIN: &str = r#"
(define (domain sometime-before-test)
  (:requirements :durative-actions)
  (:predicates (start-signal) (end-signal) (can-start))
  (:durative-action emit-start
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (can-start))
    :effect (at end (start-signal)))
  (:durative-action emit-end
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (start-signal))
    :effect (at end (end-signal))))
"#;

    const PROBLEM: &str = r#"
(define (problem sometime-before-1)
  (:domain sometime-before-test)
  (:init (can-start))
  (:goal (and (start-signal) (end-signal)))
  ;; `(sometime-before phi psi)` requires psi strictly before any phi, so the
  ;; FIRST argument is the later one. The domain achieves start-signal then
  ;; end-signal, so end-signal is the trigger and start-signal must precede it.
  (:constraints (sometime-before (end-signal) (start-signal))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("must ground");
    let outcome = gtp.find_temporal_plan();

    // Plan should succeed: start-signal is achieved before end-signal
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

#[test]
fn test_sometime_after_constraint() {
    /// Condition C2 must hold after condition C1.
    const DOMAIN: &str = r#"
(define (domain sometime-after-test)
  (:requirements :durative-actions)
  (:predicates (trigger) (response) (can-trigger))
  (:durative-action trigger-event
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (can-trigger))
    :effect (at end (trigger)))
  (:durative-action respond
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (trigger))
    :effect (at end (response))))
"#;

    const PROBLEM: &str = r#"
(define (problem sometime-after-1)
  (:domain sometime-after-test)
  (:init (can-trigger))
  (:goal (and (trigger) (response)))
  (:constraints (sometime-after (trigger) (response))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("must ground");
    let outcome = gtp.find_temporal_plan();

    // Plan should succeed: trigger happens, then response happens after
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

#[test]
fn test_multiple_constraints() {
    /// Multiple constraints that must all be satisfied.
    const DOMAIN: &str = r#"
(define (domain multi-constraint-test)
  (:requirements :durative-actions)
  (:predicates (safe) (active) (success))
  (:durative-action activate
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (safe))
    :effect (at end (active)))
  (:durative-action complete
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (active))
    :effect (at end (success))))
"#;

    const PROBLEM: &str = r#"
(define (problem multi-constraint-1)
  (:domain multi-constraint-test)
  (:init (safe))
  (:goal (success))
  (:constraints (and
    (always (safe))
    (sometime (active))
    ;; success is the trigger; active must hold strictly before it.
    (sometime-before (success) (active)))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let gtp = GroundTemporalProblem::build(&domain, &problem).expect("must ground");
    let outcome = gtp.find_temporal_plan();

    // All constraints should be satisfied:
    // - always(safe): safe is initially true and never changed
    // - sometime(active): active becomes true via activate action
    // - sometime-before(success, active): active must hold strictly before success
    assert!(matches!(outcome, PlannerOutcome::Found(_)));
}

#[test]
fn test_within_constraint() {
    /// `within` is refused, not silently mis-monitored: `ConstraintMonitor::step`
    /// has no access to the current time/tick, so a `WithinMonitor` could only
    /// ever compute a wrong answer that looks like a real one (see
    /// `MonitorFactory::create_monitor`'s doc comment). `build` must report
    /// `Pddl8Error::UnsupportedTrajectoryConstraint` rather than silently
    /// admitting a constraint it cannot actually check.
    const DOMAIN: &str = r#"
(define (domain within-test)
  (:requirements :durative-actions)
  (:predicates (started) (goal-reached))
  (:durative-action execute
    :parameters ()
    :duration (= ?duration 2)
    :condition (at start (started))
    :effect (at end (goal-reached))))
"#;

    const PROBLEM: &str = r#"
(define (problem within-1)
  (:domain within-test)
  (:init (started))
  (:goal (goal-reached))
  (:constraints (within 5 (goal-reached))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let result = GroundTemporalProblem::build(&domain, &problem);
    let err = result.err();
    assert!(
        matches!(err, Some(bcinr_pddl::Pddl8Error::UnsupportedTrajectoryConstraint(_))),
        "expected UnsupportedTrajectoryConstraint, got {err:?}"
    );
}

#[test]
fn test_always_within_constraint() {
    /// Same reasoning as `test_within_constraint`: `always-within` also needs
    /// a time/window the monitor's `step` signature cannot see, so it is
    /// refused rather than silently mis-monitored.
    const DOMAIN: &str = r#"
(define (domain always-within-test)
  (:requirements :durative-actions)
  (:predicates (monitoring) (safe) (can-monitor))
  (:durative-action monitor
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (can-monitor))
    :effect (at end (monitoring)))
  (:durative-action ensure-safety
    :parameters ()
    :duration (= ?duration 1)
    :condition (at start (monitoring))
    :effect (at end (safe))))
"#;

    const PROBLEM: &str = r#"
(define (problem always-within-1)
  (:domain always-within-test)
  (:init (can-monitor))
  (:goal (and (monitoring) (safe)))
  (:constraints (always-within 3 (monitoring) (safe))))
"#;

    let domain = domain_from_pddl(DOMAIN).expect("domain must parse");
    let problem = problem_from_pddl(PROBLEM).expect("problem must parse");

    let result = GroundTemporalProblem::build(&domain, &problem);
    let err = result.err();
    assert!(
        matches!(err, Some(bcinr_pddl::Pddl8Error::UnsupportedTrajectoryConstraint(_))),
        "expected UnsupportedTrajectoryConstraint, got {err:?}"
    );
}

#[test]
fn test_monitor_factory_all_types() {
    /// Verify `MonitorFactory` creates a real monitor for the 5 constraint
    /// types it can actually check, and honestly refuses (`None`) the 2 timed
    /// types (`within`/`always-within`) whose semantics `ConstraintMonitor::
    /// step`'s time-blind signature cannot support -- see
    /// `MonitorFactory::create_monitor`'s doc comment.
    use bcinr_pddl::ground::monitors::MonitorFactory;
    use wasm4pm_compat::pddl::{Pddl8Atom, PddlCondition, TrajectoryConstraint};

    let atom = Pddl8Atom {
        pred: "test".to_string(),
        args: vec![],
    };
    let cond = Box::new(PddlCondition::Atom(atom.clone()));
    let cond2 = Box::new(PddlCondition::Atom(Pddl8Atom {
        pred: "test2".to_string(),
        args: vec![],
    }));

    let monitored = vec![
        TrajectoryConstraint::Always(cond.clone()),
        TrajectoryConstraint::Sometime(cond.clone()),
        TrajectoryConstraint::AtMostOnce(cond.clone()),
        TrajectoryConstraint::SometimeBefore(cond.clone(), cond2.clone()),
        TrajectoryConstraint::SometimeAfter(cond.clone(), cond2.clone()),
    ];
    for constraint in &monitored {
        let monitor = MonitorFactory::create_monitor(constraint);
        assert!(monitor.is_some(), "Failed to create monitor for {:?}", constraint);
    }

    let refused = vec![
        TrajectoryConstraint::Within(5.0, cond.clone()),
        TrajectoryConstraint::AlwaysWithin(3.0, cond.clone(), cond2.clone()),
    ];
    for constraint in &refused {
        let monitor = MonitorFactory::create_monitor(constraint);
        assert!(monitor.is_none(), "Expected refusal (None) for {:?}", constraint);
    }
}
