//! Content-derived PDDL capability detection.
//!
//! Declared `:requirements` are not a sufficient admission boundary: PDDL
//! files in the wild sometimes omit requirement declarations. Production
//! admission therefore derives every feature that is materially present in the
//! parsed condition/effect trees and checks those features against the selected
//! capability profile as well.

use std::collections::BTreeSet;

use wasm4pm_compat::pddl::{
    Pddl31Domain, Pddl31Problem, PddlCondition, PddlEffect, TrajectoryConstraint,
};

use crate::capability::PddlFeature;

/// Derive all capability-relevant features actually present in the parsed task.
#[allow(dead_code)]
pub(crate) fn content_features(
    domain: &Pddl31Domain,
    problem: &Pddl31Problem,
) -> BTreeSet<PddlFeature> {
    let mut features = BTreeSet::new();

    if !domain.types.is_empty()
        || domain
            .predicates
            .iter()
            .flat_map(|(_, params)| params)
            .any(|(_, typ)| typ != "object")
        || domain
            .actions
            .iter()
            .flat_map(|action| &action.params)
            .any(|(_, typ)| typ != "object")
        || domain
            .durative_actions
            .iter()
            .flat_map(|action| &action.params)
            .any(|(_, typ)| typ != "object")
        || problem.objects.iter().any(|(_, typ)| typ != "object")
    {
        features.insert(PddlFeature::Typing);
    }

    if !domain.functions.is_empty() || !problem.init_fn_values.is_empty() {
        features.insert(PddlFeature::NumericFluents);
    }

    for action in &domain.actions {
        collect_condition_features(&action.precondition, &mut features);
        for effect in &action.effect {
            collect_effect_features(effect, &mut features);
        }
    }

    if !domain.durative_actions.is_empty() {
        features.insert(PddlFeature::DurativeActions);
    }
    for action in &domain.durative_actions {
        for condition in &action.conditions {
            collect_condition_features(condition, &mut features);
        }
        for effect in &action.effects {
            collect_effect_features(effect, &mut features);
        }
    }

    if !domain.derived.is_empty() {
        features.insert(PddlFeature::DerivedPredicates);
    }
    for derived in &domain.derived {
        collect_condition_features(&derived.body, &mut features);
    }

    if !domain.constraints.is_empty() {
        features.insert(PddlFeature::TrajectoryConstraints);
    }
    for constraint in &domain.constraints {
        collect_trajectory_features(&constraint.constraint, &mut features);
    }

    if !problem.timed_inits.is_empty() {
        features.insert(PddlFeature::TimedInitialLiterals);
    }
    collect_condition_features(&problem.goal, &mut features);

    if !problem.preferences.is_empty() {
        features.insert(PddlFeature::Preferences);
    }
    for preference in &problem.preferences {
        collect_trajectory_features(&preference.constraint, &mut features);
    }

    if problem.metric.is_some() {
        features.insert(PddlFeature::Metrics);
    }

    features
}

#[allow(dead_code)]
fn collect_condition_features(condition: &PddlCondition, features: &mut BTreeSet<PddlFeature>) {
    match condition {
        PddlCondition::Atom(atom) => {
            if atom.pred == "=" {
                features.insert(PddlFeature::Equality);
            }
        }
        PddlCondition::Not(inner) => {
            features.insert(PddlFeature::NegativePreconditions);
            collect_condition_features(inner, features);
        }
        PddlCondition::And(parts) => {
            for part in parts {
                collect_condition_features(part, features);
            }
        }
        PddlCondition::Or(parts) => {
            features.insert(PddlFeature::Disjunction);
            for part in parts {
                collect_condition_features(part, features);
            }
        }
        PddlCondition::Forall { body, .. } => {
            features.insert(PddlFeature::UniversalPreconditions);
            collect_condition_features(body, features);
        }
        PddlCondition::Exists { body, .. } => {
            features.insert(PddlFeature::ExistentialPreconditions);
            collect_condition_features(body, features);
        }
        PddlCondition::Imply(left, right) => {
            // Implication requires the same full Boolean condition surface as
            // disjunction. The current feature vocabulary has no standalone
            // implication variant, so Disjunction is the conservative gate.
            features.insert(PddlFeature::Disjunction);
            collect_condition_features(left, features);
            collect_condition_features(right, features);
        }
        PddlCondition::Timed(_, inner) => {
            features.insert(PddlFeature::DurativeActions);
            collect_condition_features(inner, features);
        }
        PddlCondition::Compare(_, _, _) => {
            features.insert(PddlFeature::NumericFluents);
        }
    }
}

#[allow(dead_code)]
fn collect_effect_features(effect: &PddlEffect, features: &mut BTreeSet<PddlFeature>) {
    match effect {
        PddlEffect::Add(_) | PddlEffect::Del(_) => {}
        PddlEffect::Numeric(_) => {
            features.insert(PddlFeature::NumericFluents);
            features.insert(PddlFeature::NumericEffects);
        }
        PddlEffect::Timed(_, inner) => {
            features.insert(PddlFeature::DurativeActions);
            collect_effect_features(inner, features);
        }
        PddlEffect::Forall { effects, .. } => {
            features.insert(PddlFeature::ConditionalEffects);
            for nested in effects {
                collect_effect_features(nested, features);
            }
        }
        PddlEffect::When { condition, effects } => {
            features.insert(PddlFeature::ConditionalEffects);
            collect_condition_features(condition, features);
            for nested in effects {
                collect_effect_features(nested, features);
            }
        }
    }
}

#[allow(dead_code)]
fn collect_trajectory_features(
    constraint: &TrajectoryConstraint,
    features: &mut BTreeSet<PddlFeature>,
) {
    features.insert(PddlFeature::TrajectoryConstraints);
    match constraint {
        TrajectoryConstraint::Always(condition)
        | TrajectoryConstraint::Sometime(condition)
        | TrajectoryConstraint::Within(_, condition)
        | TrajectoryConstraint::AtMostOnce(condition)
        | TrajectoryConstraint::HoldDuring(_, _, condition)
        | TrajectoryConstraint::HoldAfter(_, condition) => {
            collect_condition_features(condition, features);
        }
        TrajectoryConstraint::SometimeBefore(left, right)
        | TrajectoryConstraint::SometimeAfter(left, right)
        | TrajectoryConstraint::AlwaysWithin(_, left, right) => {
            collect_condition_features(left, features);
            collect_condition_features(right, features);
        }
        TrajectoryConstraint::And(parts) => {
            for part in parts {
                collect_trajectory_features(part, features);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain31_from_pddl, problem31_from_pddl};

    use super::*;

    #[test]
    fn derives_undeclared_negative_quantified_numeric_and_conditional_features() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :typing) (:types item) \
             (:predicates (ready ?x - item) (blocked ?x - item) (done ?x - item)) \
             (:functions (fuel)) \
             (:action finish :parameters () \
               :precondition (and (forall (?x - item) (not (blocked ?x))) (>= (fuel) 1)) \
               :effect (forall (?x - item) (when (ready ?x) (done ?x)))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(
            "(define (problem p) (:domain d) (:objects a - item) \
             (:init (ready a) (= (fuel) 1)) (:goal (done a)))",
        )
        .unwrap();
        let features = content_features(&domain, &problem);
        assert!(features.contains(&PddlFeature::NegativePreconditions));
        assert!(features.contains(&PddlFeature::UniversalPreconditions));
        assert!(features.contains(&PddlFeature::NumericFluents));
        assert!(features.contains(&PddlFeature::ConditionalEffects));
    }

    #[test]
    fn derives_undeclared_equality_disjunction_and_existential_features() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :typing) (:types item) \
             (:predicates (ready ?x - item) (done)) \
             (:action finish :parameters (?x - item ?y - item) \
               :precondition (or (not (= ?x ?y)) (exists (?z - item) (ready ?z))) \
               :effect (done)))",
        )
        .unwrap();
        let problem = problem31_from_pddl(
            "(define (problem p) (:domain d) (:objects a b - item) \
             (:init (ready a)) (:goal (done)))",
        )
        .unwrap();
        let features = content_features(&domain, &problem);
        assert!(features.contains(&PddlFeature::Equality));
        assert!(features.contains(&PddlFeature::Disjunction));
        assert!(features.contains(&PddlFeature::ExistentialPreconditions));
    }
}
