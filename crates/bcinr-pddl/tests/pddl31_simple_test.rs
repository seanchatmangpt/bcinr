// Simple test to verify basic PDDL 3.1 features

use bcinr_pddl::{domain_from_pddl, problem_from_pddl};

#[test]
fn test_equality_parses() {
    let domain = domain_from_pddl(
        r#"(define (domain eq-test)
             (:requirements :strips)
             (:predicates (p))
             (:action a
               :parameters ()
               :precondition ()
               :effect (p)))"#,
    )
    .unwrap();
    assert_eq!(domain.actions.len(), 1);
}

#[test]
fn test_conditional_effects_parse() {
    let domain = domain_from_pddl(
        r#"(define (domain cond-test)
             (:requirements :strips :conditional-effects)
             (:predicates (p) (q))
             (:action a
               :parameters ()
               :precondition ()
               :effect (when (p) (q))))"#,
    )
    .unwrap();
    assert_eq!(domain.actions.len(), 1);
}

#[test]
fn test_derived_predicates_parse() {
    let domain = domain_from_pddl(
        r#"(define (domain derived-test)
             (:requirements :strips :derived-predicates)
             (:predicates (p) (q))
             (:derived (q) (p))
             (:action a
               :parameters ()
               :precondition ()
               :effect (p)))"#,
    )
    .unwrap();
    assert_eq!(domain.derived.len(), 1);
}
