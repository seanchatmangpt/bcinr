#!/usr/bin/env python3
"""Apply the exact `at` predicate / temporal-wrapper parser correction.

This utility is temporary release tooling. It mutates only the checked-out
`parse.rs`; the proof workflow formats and tests that source, then stores it as
an unreferenced Git blob. It never moves a branch or tag reference.
"""

from __future__ import annotations

from pathlib import Path

PARSE_PATH = Path("crates/bcinr-pddl/src/parse.rs")


def find_line(lines: list[str], predicate, start: int = 0, end: int | None = None) -> int:
    stop = len(lines) if end is None else end
    for index in range(start, stop):
        if predicate(lines[index]):
            return index
    raise RuntimeError(f"source anchor not found between lines {start} and {stop}")


def function_bounds(lines: list[str], signature: str, next_signature: str) -> tuple[int, int]:
    start = find_line(lines, lambda line: line.startswith(signature))
    end = find_line(lines, lambda line: line.startswith(next_signature), start + 1)
    return start, end


def replace_line(lines: list[str], index: int, replacement: list[str]) -> None:
    lines[index : index + 1] = replacement


def remove_arm_range(
    lines: list[str],
    start: int,
    end: int,
    arm_start: str,
    next_arm: str,
) -> None:
    left = find_line(lines, lambda line: line.strip() == arm_start, start, end)
    right = find_line(lines, lambda line: line.strip().startswith(next_arm), left + 1, end)
    del lines[left:right]


def main() -> None:
    lines = PARSE_PATH.read_text().splitlines()

    condition_start, condition_end = function_bounds(
        lines,
        "fn parse_condition(expr: &SExpr)",
        "fn parse_effects(expr: &SExpr)",
    )
    condition_head = find_line(
        lines,
        lambda line: line.strip() == "let head = list[0].atom()?;",
        condition_start,
        condition_end,
    )
    replace_line(
        lines,
        condition_head,
        [
            "    let head = list[0].atom()?;",
            "    if list.len() == 3 && matches!(&list[2], SExpr::List(_)) {",
            "        let time = match (head, list[1].atom()?) {",
            '            ("at", "start") => Some(TimeSpecifier::AtStart),',
            '            ("at", "end") => Some(TimeSpecifier::AtEnd),',
            '            ("over", "all") => Some(TimeSpecifier::OverAll),',
            "            _ => None,",
            "        };",
            "        if let Some(time) = time {",
            "            return Ok(PddlCondition::Timed(",
            "                time,",
            "                Box::new(parse_condition(&list[2])?),",
            "            ));",
            "        }",
            "    }",
        ],
    )
    condition_end += 14
    remove_arm_range(
        lines,
        condition_start,
        condition_end,
        '"at" => {',
        '"increase" | "decrease"',
    )

    effect_start, effect_end = function_bounds(
        lines,
        "fn parse_effect(expr: &SExpr)",
        "fn parse_init(",
    )
    effect_head = find_line(
        lines,
        lambda line: line.strip() == ".atom()?;",
        effect_start,
        effect_end,
    )
    replace_line(
        lines,
        effect_head,
        [
            "        .atom()?;",
            "    if list.len() == 3 && matches!(&list[2], SExpr::List(_)) {",
            "        let time = match (head, list[1].atom()?) {",
            '            ("at", "start") => Some(TimeSpecifier::AtStart),',
            '            ("at", "end") => Some(TimeSpecifier::AtEnd),',
            "            _ => None,",
            "        };",
            "        if let Some(time) = time {",
            "            let nested = parse_effects(&list[2])?;",
            "            if nested.len() != 1 {",
            "                return Err(Pddl8Error::ParseError(",
            '                    "timed effect must wrap exactly one effect; put (and ...) outside each wrapper"',
            "                        .into(),",
            "                ));",
            "            }",
            "            return Ok(PddlEffect::Timed(time, Box::new(nested[0].clone())));",
            "        }",
            "    }",
        ],
    )
    effect_end += 17
    remove_arm_range(
        lines,
        effect_start,
        effect_end,
        '"at" => {',
        '"increase" | "decrease"',
    )

    init_start, init_end = function_bounds(lines, "fn parse_init(", "fn parse_function(expr: &SExpr)")
    init_match = find_line(
        lines,
        lambda line: line.strip() == "match list[0].atom()? {",
        init_start,
        init_end,
    )
    replace_line(
        lines,
        init_match,
        [
            "        let head = list[0].atom()?;",
            '        if head == "at" && list.len() == 3 && matches!(&list[2], SExpr::List(_)) {',
            "            if let Ok(time) = list[1].atom()?.parse::<f64>() {",
            "                let literal = list[2].list()?;",
            "                let (negated, atom_expr) =",
            '                    if literal.first().and_then(|value| value.atom().ok()) == Some("not") {',
            '                        (true, single_arg(literal, "not")?)',
            "                    } else {",
            "                        (false, &list[2])",
            "                    };",
            "                problem.timed_inits.push(TimedLiteral {",
            "                    time,",
            "                    atom: parse_atom(atom_expr)?,",
            "                    negated,",
            "                });",
            "                continue;",
            "            }",
            "        }",
            "        match head {",
        ],
    )
    init_end += 18
    remove_arm_range(lines, init_start, init_end, '"at" => {', '"not" => {')

    test_anchor = find_line(
        lines,
        lambda line: line.strip() == "fn preserves_boolean_quantified_and_conditional_trees() {",
    )
    regression = [
        "    #[test]",
        "    fn distinguishes_at_predicates_from_temporal_wrappers() {",
        "        let domain = domain31_from_pddl(",
        '            r#"(define (domain temporal-at)',
        "                (:requirements :strips :durative-actions)",
        "                (:predicates (at ?x ?l) (ready ?x))",
        "                (:durative-action hold",
        "                    :parameters (?x ?l)",
        "                    :duration (= ?duration 1)",
        "                    :condition (and (at start (at ?x ?l)) (over all (ready ?x)))",
        '                    :effect (at end (ready ?x))))"#,',
        "        )",
        "        .unwrap();",
        "        assert_eq!(domain.durative_actions.len(), 1);",
        "        assert!(matches!(",
        "            domain.durative_actions[0].conditions[0],",
        "            PddlCondition::Timed(TimeSpecifier::AtStart, _)",
        "        ));",
        "",
        "        let problem = problem31_from_pddl(",
        '            r#"(define (problem temporal-at-p)',
        "                (:domain temporal-at)",
        "                (:objects thing place)",
        "                (:init (at thing place) (at 2 (ready thing)))",
        '                (:goal (at thing place)))"#,',
        "        )",
        "        .unwrap();",
        "        assert_eq!(problem.init_atoms.len(), 1);",
        '        assert_eq!(problem.init_atoms[0].pred, "at");',
        "        assert_eq!(problem.timed_inits.len(), 1);",
        "    }",
        "",
        "    #[test]",
    ]
    lines[test_anchor - 1 : test_anchor] = regression

    PARSE_PATH.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
