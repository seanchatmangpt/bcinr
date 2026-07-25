//! Production causal projection for PDDL plans.
//!
//! The legacy analyzer records the observed linear execution order in
//! `CausalPlan::precedes`, even for pairs it separately proves independent.
//! That is useful history but wrong process geometry. This analyzer reuses the
//! legacy pairwise witnesses and reduces precedence to dependent pairs only,
//! oriented by their occurrence position in the validated input plan.

use std::collections::{BTreeMap, BTreeSet};

use bcinr_mfw_ir::{
    ActionOccurrence, ActionOccurrenceId, CausalAnalyzer, CausalPlan, Digest, PrecedenceEdge,
    StrictPartialOrder,
};

use crate::capability::GroundedPlanningEpoch;
use crate::causal::{CausalAnalysisError, PddlCausalAnalyzer};

/// Causal analyzer whose `precedes` field denotes necessary ordering rather
/// than the source plan's arbitrary serialization.
#[derive(Debug, Clone, Copy, Default)]
pub struct PddlCausalAnalyzerV2;

impl CausalAnalyzer for PddlCausalAnalyzerV2 {
    type Epoch = GroundedPlanningEpoch;
    type Error = CausalAnalysisError;

    fn analyze(
        &self,
        epoch: &GroundedPlanningEpoch,
        occurrences: &[ActionOccurrence],
    ) -> Result<CausalPlan, CausalAnalysisError> {
        let mut plan = PddlCausalAnalyzer.analyze(epoch, occurrences)?;
        let position = occurrences
            .iter()
            .enumerate()
            .map(|(index, occurrence)| (occurrence.id, index))
            .collect::<BTreeMap<ActionOccurrenceId, usize>>();

        let mut edges = BTreeSet::new();
        for pair in plan.independence.dependent.keys() {
            let left_position = position[&pair.left];
            let right_position = position[&pair.right];
            let (before, after) = if left_position < right_position {
                (pair.left, pair.right)
            } else {
                (pair.right, pair.left)
            };
            edges.insert(PrecedenceEdge { before, after });
        }
        plan.precedes = StrictPartialOrder { edges };

        let mut digest = Vec::new();
        digest.extend_from_slice(epoch.theory_digest.as_bytes());
        digest.extend_from_slice(&(occurrences.len() as u64).to_le_bytes());
        digest.extend_from_slice(&(plan.precedes.edges.len() as u64).to_le_bytes());
        digest.extend_from_slice(&(plan.independence.independent.len() as u64).to_le_bytes());
        digest.extend_from_slice(&(plan.independence.dependent.len() as u64).to_le_bytes());
        for edge in &plan.precedes.edges {
            digest.extend_from_slice(&edge.before.0.to_le_bytes());
            digest.extend_from_slice(&edge.after.0.to_le_bytes());
        }
        plan.digest = Digest::hash(&digest);
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use bcinr_mfw_ir::{ActionOccurrence, CausalAnalyzer, PlanningEpochId};

    use crate::{domain_from_pddl, problem_from_pddl, GroundProblem};

    use super::*;

    fn epoch(domain_text: &str, problem_text: &str) -> GroundedPlanningEpoch {
        let domain = domain_from_pddl(domain_text).unwrap();
        let problem = problem_from_pddl(problem_text).unwrap();
        let grounded = GroundProblem::build(&domain, &problem, None).unwrap();
        let mut epoch = GroundedPlanningEpoch::from_ground_problem(
            &grounded,
            Digest::hash(b"causal-v2-test"),
            bcinr_mfw_ir::EpochBounds {
                max_ground_actions: 64,
                max_plan_depth: 64,
                max_search_steps: 1_000,
                max_partition_boxes: 8,
            },
        );
        epoch.id = PlanningEpochId(1);
        epoch
    }

    #[test]
    fn independent_actions_do_not_acquire_vector_order_precedence() {
        let epoch = epoch(
            "(define (domain d) (:predicates (a) (b)) \
             (:action make-a :parameters () :precondition () :effect (a)) \
             (:action make-b :parameters () :precondition () :effect (b)))",
            "(define (problem p) (:domain d) (:init) (:goal (and (a) (b))))",
        );
        let occurrences = vec![
            ActionOccurrence {
                id: ActionOccurrenceId(100),
                action: 0,
            },
            ActionOccurrence {
                id: ActionOccurrenceId(7),
                action: 1,
            },
        ];
        let plan = PddlCausalAnalyzerV2.analyze(&epoch, &occurrences).unwrap();
        assert_eq!(plan.independence.independent.len(), 1);
        assert!(plan.precedes.edges.is_empty());
    }

    #[test]
    fn dependent_actions_preserve_observed_direction_not_numeric_id_order() {
        let epoch = epoch(
            "(define (domain d) (:predicates (a) (b)) \
             (:action make-a :parameters () :precondition () :effect (a)) \
             (:action make-b :parameters () :precondition (a) :effect (b)))",
            "(define (problem p) (:domain d) (:init) (:goal (b)))",
        );
        let occurrences = vec![
            ActionOccurrence {
                id: ActionOccurrenceId(100),
                action: 0,
            },
            ActionOccurrence {
                id: ActionOccurrenceId(7),
                action: 1,
            },
        ];
        let plan = PddlCausalAnalyzerV2.analyze(&epoch, &occurrences).unwrap();
        assert_eq!(plan.precedes.edges.len(), 1);
        assert!(plan.precedes.edges.contains(&PrecedenceEdge {
            before: ActionOccurrenceId(100),
            after: ActionOccurrenceId(7),
        }));
    }
}
