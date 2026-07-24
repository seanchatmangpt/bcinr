//! Production composition aliases.
//!
//! The historical `DefaultMfwPlanner` uses the execution-order causal view.
//! `ProductionMfwPlanner` uses `PddlCausalAnalyzerV2`, whose precedence order
//! contains only witnessed dependent pairs, so independent actions remain
//! concurrent when projected into POWL.

#![cfg(feature = "mfw-planner")]

use crate::causal_v2::PddlCausalAnalyzerV2;
use crate::concurrency::PddlConcurrencyAnalyzer;
use crate::consequence::GoalReachabilityHorizon;
use crate::mfw::planner::MfwPlanner;

/// Production PDDL → causal/concurrency → POWL composition rail.
pub type ProductionMfwPlanner = MfwPlanner<
    GoalReachabilityHorizon,
    PddlCausalAnalyzerV2,
    PddlConcurrencyAnalyzer,
    bcinr_powl::projection::PowlProjector,
>;

#[cfg(test)]
mod tests {
    use bcinr_mfw_ir::EpochBounds;

    use crate::capability::DefaultCapabilityProfile;
    use crate::consequence::GoalReachabilityHorizon;
    use crate::mfw::QValue;

    use super::*;

    #[test]
    fn production_planner_projects_independent_actions_without_serializing_them() {
        let mut planner = ProductionMfwPlanner::new(
            GoalReachabilityHorizon,
            bcinr_powl::projection::PowlProjector,
            EpochBounds {
                max_ground_actions: 64,
                max_plan_depth: 64,
                max_search_steps: 1_000,
                max_partition_boxes: 8,
            },
            QValue::new(1.0).unwrap(),
            2,
            128,
        );
        let workflow = planner
            .plan(
                "(define (domain d) (:requirements :strips) (:predicates (a) (b)) \
                 (:action make-a :parameters () :precondition () :effect (a)) \
                 (:action make-b :parameters () :precondition () :effect (b)))",
                "(define (problem p) (:domain d) (:init) (:goal (and (a) (b))))",
                &DefaultCapabilityProfile,
            )
            .unwrap();
        assert!(workflow.causal_plan.precedes.edges.is_empty());
        assert_eq!(workflow.causal_plan.independence.independent.len(), 1);
        assert!(workflow.powl_model.order.edges.is_empty());
    }
}
