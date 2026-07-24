#!/usr/bin/env python3
"""Apply the `at` predicate / temporal-wrapper parser correction in place."""

from pathlib import Path

PARSE_PATH = Path("crates/bcinr-pddl/src/parse.rs")


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one source anchor, found {count}")
    return source.replace(old, new, 1)


def replace_in_function(
    source: str,
    function_start: str,
    next_function: str,
    old: str,
    new: str,
    label: str,
) -> str:
    start = source.index(function_start)
    end = source.index(next_function, start)
    section = source[start:end]
    section = replace_once(section, old, new, label)
    return source[:start] + section + source[end:]


def main() -> None:
    source = PARSE_PATH.read_text()
    regression_name = "fn distinguishes_at_predicates_from_temporal_wrappers()"
    if regression_name in source:
        return

    helper_anchor = "fn parse_condition(expr: &SExpr) -> Result<PddlCondition, Pddl8Error> {"
    helpers = '''fn is_temporal_condition(list: &[SExpr]) -> bool {
    if list.len() != 3 || !matches!(&list[2], SExpr::List(_)) {
        return false;
    }
    matches!(
        (list[0].atom().ok(), list[1].atom().ok()),
        (Some("at"), Some("start" | "end")) | (Some("over"), Some("all"))
    )
}

fn is_temporal_effect(list: &[SExpr]) -> bool {
    list.len() == 3
        && matches!(&list[2], SExpr::List(_))
        && matches!(
            (list[0].atom().ok(), list[1].atom().ok()),
            (Some("at"), Some("start" | "end"))
        )
}

fn is_timed_initial_literal(list: &[SExpr]) -> bool {
    list.len() == 3
        && matches!(&list[2], SExpr::List(_))
        && list[1]
            .atom()
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .is_some()
}

'''
    source = replace_once(source, helper_anchor, helpers + helper_anchor, "helper insertion")

    source = replace_in_function(
        source,
        helper_anchor,
        "fn parse_numeric_comparison(",
        '        "at" => {',
        '        "at" if is_temporal_condition(list) => {',
        "condition at guard",
    )
    source = replace_in_function(
        source,
        helper_anchor,
        "fn parse_numeric_comparison(",
        '        "over" => {',
        '        "over" if is_temporal_condition(list) => {',
        "condition over guard",
    )
    source = replace_in_function(
        source,
        "fn parse_effect(expr: &SExpr)",
        "fn parse_function_term(",
        '        "at" => {',
        '        "at" if is_temporal_effect(list) => {',
        "effect at guard",
    )
    source = replace_in_function(
        source,
        "fn parse_init(",
        "fn parse_metric(",
        '            "at" => {',
        '            "at" if is_timed_initial_literal(list) => {',
        "initial at guard",
    )

    test_anchor = '''    #[test]
    fn rejects_unrepresentable_domain_constants() {'''
    regression = '''    #[test]
    fn distinguishes_at_predicates_from_temporal_wrappers() {
        let domain = domain31_from_pddl(
            r#"(define (domain temporal-at)
                (:requirements :strips :durative-actions)
                (:predicates (at ?x ?l) (ready ?x))
                (:durative-action hold
                    :parameters (?x ?l)
                    :duration (= ?duration 1)
                    :condition (and (at start (at ?x ?l)) (over all (ready ?x)))
                    :effect (at end (ready ?x))))"#,
        )
        .unwrap();
        assert_eq!(domain.durative_actions.len(), 1);
        assert!(matches!(
            domain.durative_actions[0].conditions[0],
            PddlCondition::Timed(TimeSpecifier::AtStart, _)
        ));

        let problem = problem31_from_pddl(
            r#"(define (problem temporal-at-p)
                (:domain temporal-at)
                (:objects thing place)
                (:init (at thing place) (at 2 (ready thing)))
                (:goal (at thing place)))"#,
        )
        .unwrap();
        assert_eq!(problem.init_atoms.len(), 1);
        assert_eq!(problem.init_atoms[0].pred, "at");
        assert_eq!(problem.timed_inits.len(), 1);
        assert!(matches!(problem.goal, PddlCondition::Atom(ref atom) if atom.pred == "at"));
    }

    #[test]
    fn rejects_unrepresentable_domain_constants() {'''
    source = replace_once(source, test_anchor, regression, "regression insertion")
    PARSE_PATH.write_text(source)


if __name__ == "__main__":
    main()
