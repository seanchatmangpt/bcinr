//! Tests for deadline/time-window constraints on temporal plans.
//!
//! This test suite verifies that:
//! 1. Plans meeting deadlines are accepted
//! 2. Plans violating deadlines are refused
//! 3. The canonical LogicalTime type is used consistently

#[cfg(feature = "mfw-planner")]
mod deadline_tests {
    use bcinr_mfw_ir::PlannerOutcome;
    use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundTemporalProblem, LogicalTime};

    /// Simple one-action temporal domain with known duration.
    const DOMAIN_PDDL: &str = r#"
(define (domain single-action)
  (:requirements :durative-actions)
  (:predicates (started) (done))
  (:durative-action work
    :parameters ()
    :duration (= ?duration 2.0)
    :condition (and (at start (started)))
    :effect (and (at end (done)))))
"#;

    /// Initial state: action can be started
    const PROBLEM_PDDL: &str = r#"
(define (problem single-action-prob)
  (:domain single-action)
  (:objects)
  (:init (started))
  (:goal (done)))
"#;

    /// Two-action temporal domain.
    const DOMAIN_TWO_PDDL: &str = r#"
(define (domain two-actions)
  (:requirements :durative-actions)
  (:predicates (p0) (p1) (p2))
  (:durative-action a1
    :parameters ()
    :duration (= ?duration 1.0)
    :condition (and (at start (p0)))
    :effect (and (at end (p1))))
  (:durative-action a2
    :parameters ()
    :duration (= ?duration 1.5)
    :condition (and (at start (p1)))
    :effect (and (at end (p2)))))
"#;

    const PROBLEM_TWO_PDDL: &str = r#"
(define (problem two-actions-prob)
  (:domain two-actions)
  (:objects)
  (:init (p0))
  (:goal (p2)))
"#;

    #[test]
    fn plan_meets_generous_deadline() {
        let domain = domain_from_pddl(DOMAIN_PDDL).expect("domain should parse");
        let problem = problem_from_pddl(PROBLEM_PDDL).expect("problem should parse");

        let mut gtp =
            GroundTemporalProblem::build(&domain, &problem).expect("problem should ground");

        // Set a generous deadline: 5 seconds
        // The plan should take 2 seconds (1 action, 2.0s duration)
        gtp.set_deadline(LogicalTime::from_seconds_f64(5.0));

        let outcome = gtp.find_temporal_plan();
        match outcome {
            PlannerOutcome::Found(plan) => {
                // Verify the plan's makespan is within the deadline
                assert!(
                    plan.makespan <= 2.1,
                    "expected makespan ~2.0s, got {}",
                    plan.makespan
                );
                assert_eq!(plan.steps.len(), 1, "expected 1 action in plan");
                // Verify step duration
                assert!((plan.steps[0].duration - 2.0).abs() < 0.01);
            }
            other => panic!("expected Found, got {:?}", other),
        }
    }

    #[test]
    fn plan_violates_tight_deadline() {
        let domain = domain_from_pddl(DOMAIN_TWO_PDDL).expect("domain should parse");
        let problem = problem_from_pddl(PROBLEM_TWO_PDDL).expect("problem should parse");

        let mut gtp =
            GroundTemporalProblem::build(&domain, &problem).expect("problem should ground");

        // Set a tight deadline: 1.0 seconds
        // The plan requires 2.5 seconds (1.0s + 1.5s), so it violates the deadline
        gtp.set_deadline(LogicalTime::from_seconds_f64(1.0));

        let outcome = gtp.find_temporal_plan();
        match outcome {
            PlannerOutcome::Exhausted(_) => {
                // Expected: the planner exhausted the search without finding a plan
                // that meets the deadline
            }
            PlannerOutcome::Found(_) => {
                panic!("should not find a plan that meets a 1.0s deadline when minimum is 2.5s")
            }
            other => panic!("unexpected outcome: {:?}", other),
        }
    }

    #[test]
    fn deadline_at_exact_makespan_boundary() {
        let domain = domain_from_pddl(DOMAIN_TWO_PDDL).expect("domain should parse");
        let problem = problem_from_pddl(PROBLEM_TWO_PDDL).expect("problem should parse");

        let mut gtp =
            GroundTemporalProblem::build(&domain, &problem).expect("problem should ground");

        // First, find the plan without deadline to know the actual makespan
        let baseline_plan = match gtp.find_temporal_plan() {
            PlannerOutcome::Found(plan) => plan,
            other => panic!("baseline plan should be found, got {:?}", other),
        };

        // Now set deadline at the baseline makespan (should accept)
        gtp.set_deadline(LogicalTime::from_seconds_f64(baseline_plan.makespan + 0.01));

        let outcome = gtp.find_temporal_plan();
        match outcome {
            PlannerOutcome::Found(plan) => {
                assert!(
                    plan.makespan <= baseline_plan.makespan + 0.02,
                    "expected makespan <= baseline, got {}",
                    plan.makespan
                );
            }
            other => panic!("expected Found when deadline >= makespan, got {:?}", other),
        }
    }

    #[test]
    fn plan_without_deadline() {
        let domain = domain_from_pddl(DOMAIN_TWO_PDDL).expect("domain should parse");
        let problem = problem_from_pddl(PROBLEM_TWO_PDDL).expect("problem should parse");

        let gtp = GroundTemporalProblem::build(&domain, &problem).expect("problem should ground");

        // No deadline is set; the plan should be found normally
        let outcome = gtp.find_temporal_plan();
        match outcome {
            PlannerOutcome::Found(plan) => {
                assert!(!plan.steps.is_empty(), "expected at least 1 action in plan");
                assert!(plan.makespan > 0.0, "makespan should be positive");
            }
            other => panic!("expected Found, got {:?}", other),
        }
    }

    #[test]
    fn clear_deadline() {
        let domain = domain_from_pddl(DOMAIN_TWO_PDDL).expect("domain should parse");
        let problem = problem_from_pddl(PROBLEM_TWO_PDDL).expect("problem should parse");

        let mut gtp =
            GroundTemporalProblem::build(&domain, &problem).expect("problem should ground");

        // Set a very tight deadline that no plan can meet
        gtp.set_deadline(LogicalTime::from_seconds_f64(0.1));

        // Verify plan is not found with tight deadline
        let outcome_with_deadline = gtp.find_temporal_plan();
        assert!(
            matches!(outcome_with_deadline, PlannerOutcome::Exhausted(_)),
            "expected Exhausted with tight deadline, got {:?}",
            outcome_with_deadline
        );

        // Clear the deadline
        gtp.clear_deadline();

        // After clearing, the plan should be found
        let outcome = gtp.find_temporal_plan();
        match outcome {
            PlannerOutcome::Found(plan) => {
                assert!(!plan.steps.is_empty(), "expected at least 1 action in plan");
            }
            other => panic!("expected Found after clearing deadline, got {:?}", other),
        }
    }

    #[test]
    fn logical_time_conversions() {
        // Test that LogicalTime conversions are consistent
        let t_ms = LogicalTime::from_millis(2500);
        assert_eq!(t_ms.as_millis(), 2500);
        assert!((t_ms.as_seconds_f64() - 2.5).abs() < 0.001);

        let t_s = LogicalTime::from_seconds_f64(2.5);
        assert_eq!(t_s.as_millis(), 2500);
        assert!((t_s.as_seconds_f64() - 2.5).abs() < 0.001);

        // Test zero
        let t_zero = LogicalTime::zero();
        assert_eq!(t_zero.as_millis(), 0);
        assert_eq!(t_zero.as_seconds_f64(), 0.0);

        // Test ordering
        let t1 = LogicalTime::from_millis(100);
        let t2 = LogicalTime::from_millis(200);
        assert!(t1 < t2);
        assert!(t1 <= t1);
        assert!(t2 > t1);
    }
}

#[cfg(not(feature = "mfw-planner"))]
mod placeholder {
    #[test]
    fn feature_not_enabled() {
        // Placeholder test when mfw-planner feature is not enabled
    }
}
