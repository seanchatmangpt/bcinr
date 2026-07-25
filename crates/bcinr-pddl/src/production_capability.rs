//! Capability profile for production cognitive composition.
//!
//! Admission is deliberately narrower than parser reachability. The current
//! MFW search rail consumes the classical PDDL8 ground representation, so only
//! STRIPS and typing have end-to-end exact standing there. Every construct that
//! could be flattened by the legacy projection is refused before grounding.

use crate::capability::{CapabilityProfile, PddlFeature, SemanticSupport};

/// Exact end-to-end feature profile for the production MFW planner.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProductionCapabilityProfile;

impl CapabilityProfile for ProductionCapabilityProfile {
    fn support(&self, feature: PddlFeature) -> SemanticSupport {
        match feature {
            PddlFeature::Strips | PddlFeature::Typing => SemanticSupport::Exact,
            PddlFeature::NegativePreconditions
            | PddlFeature::Disjunction
            | PddlFeature::Equality
            | PddlFeature::ExistentialPreconditions
            | PddlFeature::UniversalPreconditions
            | PddlFeature::ConditionalEffects
            | PddlFeature::NumericFluents
            | PddlFeature::NumericEffects
            | PddlFeature::DurativeActions
            | PddlFeature::TimedInitialLiterals
            | PddlFeature::DerivedPredicates
            | PddlFeature::TrajectoryConstraints
            | PddlFeature::Preferences
            | PddlFeature::Metrics => SemanticSupport::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{admit_planning_task, domain31_from_pddl, problem31_from_pddl, PlannerOutcome};

    use super::*;

    #[test]
    fn admits_typed_strips() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :typing) (:types item) \
             (:predicates (done ?x - item)) \
             (:action finish :parameters (?x - item) :precondition () :effect (done ?x)))",
        )
        .unwrap();
        let problem = problem31_from_pddl(
            "(define (problem p) (:domain d) (:objects a - item) (:init) (:goal (done a)))",
        )
        .unwrap();
        assert!(matches!(
            admit_planning_task(&domain, &problem, &ProductionCapabilityProfile),
            PlannerOutcome::Found(_)
        ));
    }

    #[test]
    fn refuses_negative_precondition_before_legacy_grounding() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :negative-preconditions) \
             (:predicates (locked) (done)) \
             (:action finish :parameters () :precondition (not (locked)) :effect (done)))",
        )
        .unwrap();
        let problem =
            problem31_from_pddl("(define (problem p) (:domain d) (:init) (:goal (done)))").unwrap();
        assert!(matches!(
            admit_planning_task(&domain, &problem, &ProductionCapabilityProfile),
            PlannerOutcome::Unsupported(_)
        ));
    }

    #[test]
    fn refuses_conditional_effect_even_when_requirement_is_omitted() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips) (:predicates (p) (q)) \
             (:action a :parameters () :precondition () :effect (when (p) (q))))",
        )
        .unwrap();
        let problem =
            problem31_from_pddl("(define (problem p) (:domain d) (:init (p)) (:goal (q)))")
                .unwrap();
        assert!(matches!(
            admit_planning_task(&domain, &problem, &ProductionCapabilityProfile),
            PlannerOutcome::Unsupported(_)
        ));
    }
}
