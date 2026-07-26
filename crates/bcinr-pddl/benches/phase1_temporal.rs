//! Phase 1 (Temporal) and Phase 2 (Resource Ledger) benchmarks using Divan.
//!
//! Benchmarks temporal planning with deadlines and resource conflict detection:
//!
//! - Phase 1: `find_temporal_plan` on action with deadline (tight) vs. without deadline
//! - Phase 1: `find_temporal_plan` with maintenance windows (OverAll conditions)
//! - Phase 2: `ResourceLedger::request_lease` on overlapping vs. non-overlapping intervals

use bcinr_pddl::{
    domain_from_pddl, problem_from_pddl, GroundTemporalProblem, LogicalTime, ResourceLedger,
    Resource, ResourceMode,
};

fn main() {
    divan::main();
}

mod phase1_temporal_planning {
    use super::*;

    /// PDDL domain with a single durative action, no deadline constraint.
    const DOMAIN_NO_DEADLINE: &str = "(define (domain temporal-no-deadline) \
        (:requirements :durative-actions) \
        (:predicates (ready) (done)) \
        (:durative-action work \
            :parameters () \
            :duration (= ?duration 10) \
            :condition (at start (ready)) \
            :effect (at end (done))))";

    const PROBLEM_NO_DEADLINE: &str = "(define (problem p1) \
        (:domain temporal-no-deadline) \
        (:init (ready)) \
        (:goal (done)))";

    /// PDDL domain with OverAll condition: maintenance window must remain open.
    const DOMAIN_MAINTENANCE_WINDOW: &str = "(define (domain temporal-maintenance-window) \
        (:requirements :durative-actions :timed-initial-literals) \
        (:predicates (window-open) (done)) \
        (:durative-action do-work \
            :parameters () \
            :duration (= ?duration 5) \
            :condition (over all (window-open)) \
            :effect (at end (done))))";

    const PROBLEM_MAINTENANCE_WINDOW: &str = "(define (problem p3) \
        (:domain temporal-maintenance-window) \
        (:init (at 10 (window-open)) (at 20 (not (window-open)))) \
        (:goal (done)))";

    /// Benchmark: find temporal plan without deadline constraint.
    /// Tests baseline temporal planning performance.
    #[divan::bench]
    fn find_plan_no_deadline() -> Option<f64> {
        let domain = domain_from_pddl(divan::black_box(DOMAIN_NO_DEADLINE))
            .expect("domain must parse");
        let problem = problem_from_pddl(divan::black_box(PROBLEM_NO_DEADLINE))
            .expect("problem must parse");

        let ground = GroundTemporalProblem::build(&domain, &problem)
            .expect("must ground");
        ground
            .find_temporal_plan()
            .into_result()
            .ok()
            .map(|plan| plan.makespan)
    }

    /// Benchmark: find temporal plan with tight deadline constraint.
    /// Deadline is just enough to accommodate the plan (tight constraint).
    #[divan::bench]
    fn find_plan_with_tight_deadline() -> Option<f64> {
        let domain = domain_from_pddl(divan::black_box(DOMAIN_NO_DEADLINE))
            .expect("domain must parse");
        let problem = problem_from_pddl(divan::black_box(PROBLEM_NO_DEADLINE))
            .expect("problem must parse");

        let mut ground = GroundTemporalProblem::build(&domain, &problem)
            .expect("must ground");
        ground.set_deadline(LogicalTime::from_seconds_f64(12.0)); // Just enough for duration 10
        ground
            .find_temporal_plan()
            .into_result()
            .ok()
            .map(|plan| plan.makespan)
    }

    /// Benchmark: find temporal plan with loose deadline constraint.
    /// Deadline is generous, imposing minimal constraint.
    #[divan::bench]
    fn find_plan_with_loose_deadline() -> Option<f64> {
        let domain = domain_from_pddl(divan::black_box(DOMAIN_NO_DEADLINE))
            .expect("domain must parse");
        let problem = problem_from_pddl(divan::black_box(PROBLEM_NO_DEADLINE))
            .expect("problem must parse");

        let mut ground = GroundTemporalProblem::build(&domain, &problem)
            .expect("must ground");
        ground.set_deadline(LogicalTime::from_seconds_f64(1000.0)); // Very loose
        ground
            .find_temporal_plan()
            .into_result()
            .ok()
            .map(|plan| plan.makespan)
    }

    /// Benchmark: find temporal plan with OverAll conditions (maintenance window).
    /// Tests handling of continuous validity conditions throughout action duration.
    #[divan::bench]
    fn find_plan_with_maintenance_window() -> Option<f64> {
        let domain = domain_from_pddl(divan::black_box(DOMAIN_MAINTENANCE_WINDOW))
            .expect("domain must parse");
        let problem = problem_from_pddl(divan::black_box(PROBLEM_MAINTENANCE_WINDOW))
            .expect("problem must parse");

        let ground = GroundTemporalProblem::build(&domain, &problem)
            .expect("must ground");
        ground
            .find_temporal_plan()
            .into_result()
            .ok()
            .map(|plan| plan.makespan)
    }
}

mod phase2_resource_ledger {
    use super::*;

    /// Benchmark: request lease on overlapping interval (conflict detected).
    /// Tests resource conflict detection on exclusive resource.
    /// First lease: [0, 10), second lease: [5, 15) — should fail.
    #[divan::bench]
    fn request_lease_overlapping_exclusive() -> bool {
        let mut ledger = ResourceLedger::new();

        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        // First lease succeeds
        let _lease1 = ledger
            .request_lease(cpu.clone(), 0.0, 10.0)
            .expect("first lease must succeed");

        // Second lease should conflict
        let result = ledger.request_lease(divan::black_box(cpu), divan::black_box(5.0), divan::black_box(15.0));
        result.is_err()  // Returns true if conflict detected (expected)
    }

    /// Benchmark: request lease on non-overlapping interval (no conflict).
    /// Tests that non-overlapping intervals are allowed on exclusive resource.
    /// First lease: [0, 10), second lease: [10, 20) — should succeed.
    #[divan::bench]
    fn request_lease_nonoverlapping_exclusive() -> bool {
        let mut ledger = ResourceLedger::new();

        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        // First lease succeeds
        let _lease1 = ledger
            .request_lease(cpu.clone(), 0.0, 10.0)
            .expect("first lease must succeed");

        // Second lease should succeed (non-overlapping)
        let result = ledger.request_lease(divan::black_box(cpu), divan::black_box(10.0), divan::black_box(20.0));
        result.is_ok()  // Returns true if no conflict (expected)
    }

    /// Benchmark: request lease on shared resource within capacity.
    /// Tests shared resource capacity enforcement.
    /// Capacity: 2, Lease1: [0, 10), Lease2: [5, 15) → both succeed.
    #[divan::bench]
    fn request_lease_shared_within_capacity() -> bool {
        let mut ledger = ResourceLedger::new();

        let workers = Resource {
            name: "workers".to_string(),
            capacity: 2,
            mode: ResourceMode::Shared,
        };

        let _lease1 = ledger
            .request_lease(workers.clone(), 0.0, 10.0)
            .expect("lease 1 must succeed");

        // Overlaps with lease1 but within capacity (2)
        let result = ledger.request_lease(divan::black_box(workers), divan::black_box(5.0), divan::black_box(15.0));
        result.is_ok()  // Should succeed
    }

    /// Benchmark: request lease exceeding shared resource capacity.
    /// Tests capacity enforcement when capacity is exceeded.
    /// Capacity: 2, already have 2 concurrent leases, third should fail.
    #[divan::bench]
    fn request_lease_shared_exceeds_capacity() -> bool {
        let mut ledger = ResourceLedger::new();

        let workers = Resource {
            name: "workers".to_string(),
            capacity: 2,
            mode: ResourceMode::Shared,
        };

        let _lease1 = ledger
            .request_lease(workers.clone(), 0.0, 10.0)
            .expect("lease 1 must succeed");

        let _lease2 = ledger
            .request_lease(workers.clone(), 5.0, 15.0)
            .expect("lease 2 must succeed");

        // Third overlapping lease should fail (exceeds capacity)
        let result = ledger.request_lease(divan::black_box(workers), divan::black_box(7.0), divan::black_box(12.0));
        result.is_err()  // Should fail
    }

    /// Benchmark: sequential non-overlapping leases on exclusive resource.
    /// Tests repeated non-overlapping lease requests (stress test).
    #[divan::bench]
    fn sequential_nonoverlapping_leases() -> usize {
        let mut ledger = ResourceLedger::new();

        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        let mut count = 0;
        for i in 0..16 {
            let start = (i * 10) as f64;
            let end = start + 10.0;
            if let Ok(_lease) = ledger.request_lease(divan::black_box(cpu.clone()), divan::black_box(start), divan::black_box(end)) {
                count += 1;
            }
        }
        count
    }
}
