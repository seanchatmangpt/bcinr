//! Dependency-free PDDL 3.1 parser and lowering boundary.
//!
//! The previous implementation delegated syntax and AST ownership to the
//! EUPL-licensed `pddl` crate and then flattened several constructs while
//! lowering. This parser admits PDDL directly into the canonical
//! `wasm4pm_compat::pddl` types. Unsupported constructs are refused with a
//! typed parse error instead of being silently dropped.

use crate::error::Pddl8Error;
use crate::sexpr::{parse_one, SExpr};
use wasm4pm_compat::pddl::{
    CompareOp, DerivedPredicate, DurationConstraint, DurativeAction, Metric, MetricDir, MetricExpr,
    NumericEffect, NumericExpr, NumericOp, Pddl31Action, Pddl31Domain, Pddl31Problem,
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem, PddlCondition, PddlConstraint,
    PddlEffect, PddlEvent, PddlFunction, PddlPreference, PddlProcess, PddlType, TimeSpecifier,
    TimedLiteral, TrajectoryConstraint, PDDL8_MAX_ARITY, PDDL8_MAX_CONJUNCTS, PDDL8_MAX_PARAMS,
};

/// Detectable refusal fingerprint for PDDL+ continuous effects. The canonical
/// external effect enum has no continuous-rate variant, so the parser preserves
/// evidence of use and the admission gate refuses the domain.
pub const CONTINUOUS_EFFECT_SENTINEL_PRED: &str = "__bcinr_unsupported_continuous_effect";

/// Detectable refusal fingerprint for object-valued fluent assignments. The
/// canonical external effect enum models numeric fluents only.
pub const OBJECT_FLUENT_SENTINEL_PRED: &str = "__bcinr_unsupported_object_fluent";

pub fn domain_from_pddl(text: &str) -> Result<Pddl8Domain, Pddl8Error> {
    let full = domain31_from_pddl(text)?;
    let actions = full
        .actions
        .iter()
        .map(action31_to_pddl8)
        .collect::<Result<Vec<_>, _>>()?;
    let predicates = full
        .predicates
        .iter()
        .map(|(name, params)| {
            if params.len() > PDDL8_MAX_ARITY {
                Err(Pddl8Error::BoundExceeded {
                    what: "predicate arity",
                    limit: PDDL8_MAX_ARITY,
                    got: params.len(),
                })
            } else {
                Ok((name.clone(), params.len() as u8))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Pddl8Domain {
        name: full.name,
        predicates,
        actions,
        types: full.types,
        functions: full.functions,
        durative_actions: full.durative_actions,
        derived: full.derived,
        constraints: full.constraints,
        processes: full.processes,
        events: full.events,
    })
}

pub fn problem_from_pddl(text: &str) -> Result<Pddl8Problem, Pddl8Error> {
    let full = problem31_from_pddl(text)?;
    let goal = legacy_positive_atoms(&full.goal);
    Ok(Pddl8Problem {
        name: full.name,
        domain: full.domain,
        objects: full.objects.iter().map(|(name, _)| name.clone()).collect(),
        object_types: full.objects,
        init: full.init_atoms,
        goal,
        fn_values: full.init_fn_values,
        timed_inits: full.timed_inits,
        preferences: full.preferences,
        metric: full.metric,
    })
}

pub fn domain31_from_pddl(text: &str) -> Result<Pddl31Domain, Pddl8Error> {
    let root = parse_one(text)?;
    let items = define_items(&root, "domain")?;
    let name = declaration_name(items, "domain")?;

    let mut domain = Pddl31Domain {
        name,
        ..Pddl31Domain::default()
    };

    for section in items.iter().skip(2) {
        let Some(head) = section.head() else {
            continue;
        };
        match head {
            ":requirements" => {
                domain.requirements = section
                    .list()?
                    .iter()
                    .skip(1)
                    .map(|item| requirement_name(item.atom()?))
                    .collect::<Result<Vec<_>, Pddl8Error>>()?;
            }
            ":types" => domain.types = parse_types(&section.list()?[1..])?,
            ":predicates" => {
                domain.predicates = parse_predicates(&section.list()?[1..])?;
            }
            ":functions" => {
                domain.functions = parse_functions(&section.list()?[1..])?;
            }
            ":action" => domain.actions.push(parse_action(section)?),
            ":durative-action" => domain
                .durative_actions
                .push(parse_durative_action(section)?),
            ":derived" => domain.derived.push(parse_derived(section)?),
            ":constraints" => {
                domain
                    .constraints
                    .extend(parse_domain_constraints(section)?);
            }
            ":process" => domain.processes.push(parse_process(section)?),
            ":event" => domain.events.push(parse_event(section)?),
            ":constants" => {
                return Err(Pddl8Error::ParseError(
                    "domain constants are not representable; move them to problem :objects".into(),
                ));
            }
            other => {
                return Err(Pddl8Error::ParseError(format!(
                    "unsupported domain section {other:?}"
                )));
            }
        }
    }
    Ok(domain)
}

pub fn problem31_from_pddl(text: &str) -> Result<Pddl31Problem, Pddl8Error> {
    let root = parse_one(text)?;
    let items = define_items(&root, "problem")?;
    let name = declaration_name(items, "problem")?;
    let mut problem = Pddl31Problem {
        name,
        ..Pddl31Problem::default()
    };

    for section in items.iter().skip(2) {
        let Some(head) = section.head() else {
            continue;
        };
        match head {
            ":domain" => {
                problem.domain = section
                    .list()?
                    .get(1)
                    .ok_or_else(|| Pddl8Error::ParseError("missing problem :domain".into()))?
                    .atom()?
                    .to_string();
            }
            ":objects" => problem.objects = parse_typed_names(&section.list()?[1..])?,
            ":init" => parse_init(section, &mut problem)?,
            ":goal" => {
                let goal = section
                    .list()?
                    .get(1)
                    .ok_or_else(|| Pddl8Error::ParseError("missing :goal expression".into()))?;
                if contains_head(goal, "preference") {
                    return Err(Pddl8Error::ParseError(
                        "preferences nested inside :goal are not representable; use :constraints"
                            .into(),
                    ));
                }
                problem.goal = parse_condition(goal)?;
            }
            ":constraints" => {
                let expr = section.list()?.get(1).ok_or_else(|| {
                    Pddl8Error::ParseError("missing :constraints expression".into())
                })?;
                problem.preferences.extend(parse_problem_constraints(expr)?);
            }
            ":metric" => problem.metric = Some(parse_metric(section)?),
            other => {
                return Err(Pddl8Error::ParseError(format!(
                    "unsupported problem section {other:?}"
                )));
            }
        }
    }

    if problem.domain.is_empty() {
        return Err(Pddl8Error::ParseError("problem is missing :domain".into()));
    }
    Ok(problem)
}

fn define_items<'a>(root: &'a SExpr, kind: &str) -> Result<&'a [SExpr], Pddl8Error> {
    let items = root.list()?;
    if items.first().and_then(|item| item.atom().ok()) != Some("define") {
        return Err(Pddl8Error::ParseError(
            "document must start with (define ...)".into(),
        ));
    }
    let declaration = items
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError("missing define declaration".into()))?;
    if declaration.head() != Some(kind) {
        return Err(Pddl8Error::ParseError(format!(
            "expected ({kind} <name>) declaration"
        )));
    }
    Ok(items)
}

fn declaration_name(items: &[SExpr], kind: &str) -> Result<String, Pddl8Error> {
    items[1]
        .list()?
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError(format!("missing {kind} name")))?
        .atom()
        .map(str::to_string)
}

fn requirement_name(surface: &str) -> Result<String, Pddl8Error> {
    let name = match surface.trim_start_matches(':') {
        "strips" => "Strips",
        "typing" => "Typing",
        "negative-preconditions" => "NegativePreconditions",
        "disjunctive-preconditions" => "DisjunctivePreconditions",
        "equality" => "Equality",
        "existential-preconditions" => "ExistentialPreconditions",
        "universal-preconditions" => "UniversalPreconditions",
        "quantified-preconditions" => "QuantifiedPreconditions",
        "conditional-effects" => "ConditionalEffects",
        "fluents" => "Fluents",
        "numeric-fluents" => "NumericFluents",
        "object-fluents" => "ObjectFluents",
        "adl" => "Adl",
        "durative-actions" => "DurativeActions",
        "duration-inequalities" => "DurationInequalities",
        "continuous-effects" => "ContinuousEffects",
        "derived-predicates" => "DerivedPredicates",
        "timed-initial-literals" => "TimedInitialLiterals",
        "preferences" => "Preferences",
        "constraints" => "Constraints",
        "action-costs" => "ActionCosts",
        "time" => "Time",
        other => {
            return Err(Pddl8Error::ParseError(format!(
                "unknown PDDL requirement :{other}"
            )));
        }
    };
    Ok(name.to_string())
}

fn parse_types(items: &[SExpr]) -> Result<Vec<PddlType>, Pddl8Error> {
    Ok(parse_typed_names(items)?
        .into_iter()
        .map(|(name, parent)| PddlType {
            name,
            parent: (parent != "object").then_some(parent),
        })
        .collect())
}

fn parse_predicates(items: &[SExpr]) -> Result<Vec<(String, Vec<(String, String)>)>, Pddl8Error> {
    items
        .iter()
        .map(|expr| {
            let list = expr.list()?;
            let name = list
                .first()
                .ok_or_else(|| Pddl8Error::ParseError("empty predicate declaration".into()))?
                .atom()?
                .to_string();
            let params = parse_typed_names(&list[1..])?;
            if params.len() > PDDL8_MAX_ARITY {
                return Err(Pddl8Error::BoundExceeded {
                    what: "predicate arity",
                    limit: PDDL8_MAX_ARITY,
                    got: params.len(),
                });
            }
            Ok((name, params))
        })
        .collect()
}

fn parse_functions(items: &[SExpr]) -> Result<Vec<PddlFunction>, Pddl8Error> {
    let mut functions = Vec::new();
    let mut index = 0usize;
    while index < items.len() {
        match &items[index] {
            SExpr::List(list) => {
                let name = list
                    .first()
                    .ok_or_else(|| Pddl8Error::ParseError("empty function declaration".into()))?
                    .atom()?
                    .to_string();
                let params = parse_typed_names(&list[1..])?
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect();
                functions.push(PddlFunction { name, params });
                index += 1;
            }
            SExpr::Atom(atom) if atom == "-" => {
                index += 2;
            }
            SExpr::Atom(atom) => {
                return Err(Pddl8Error::ParseError(format!(
                    "unexpected token {atom:?} in :functions"
                )));
            }
        }
    }
    Ok(functions)
}

fn parse_typed_names(items: &[SExpr]) -> Result<Vec<(String, String)>, Pddl8Error> {
    let mut out = Vec::new();
    let mut pending = Vec::<String>::new();
    let mut index = 0usize;
    while index < items.len() {
        let token = items[index].atom()?;
        if token == "-" {
            let typ = items
                .get(index + 1)
                .ok_or_else(|| Pddl8Error::ParseError("typed list ends after '-'".into()))?
                .atom()?
                .to_string();
            if pending.is_empty() {
                return Err(Pddl8Error::ParseError(
                    "typed list '-' has no preceding names".into(),
                ));
            }
            out.extend(pending.drain(..).map(|name| (name, typ.clone())));
            index += 2;
        } else {
            pending.push(token.to_string());
            index += 1;
        }
    }
    out.extend(pending.into_iter().map(|name| (name, "object".into())));
    Ok(out)
}

fn parse_action(expr: &SExpr) -> Result<Pddl31Action, Pddl8Error> {
    let list = expr.list()?;
    let name = list
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError(":action missing name".into()))?
        .atom()?
        .to_string();
    let params = parse_typed_names(field_list(list, ":parameters")?)?;
    if params.len() > PDDL8_MAX_PARAMS {
        return Err(Pddl8Error::BoundExceeded {
            what: "action parameters",
            limit: PDDL8_MAX_PARAMS,
            got: params.len(),
        });
    }
    let precondition = field_expr(list, ":precondition")
        .map(parse_condition)
        .transpose()?
        .unwrap_or_else(|| PddlCondition::And(vec![]));
    let effect = field_expr(list, ":effect")
        .map(parse_effects)
        .transpose()?
        .unwrap_or_default();
    Ok(Pddl31Action {
        name,
        params,
        precondition,
        effect,
    })
}

fn action31_to_pddl8(action: &Pddl31Action) -> Result<Pddl8ActionSchema, Pddl8Error> {
    let params: Vec<String> = action.params.iter().map(|(name, _)| name.clone()).collect();
    let preconditions = legacy_positive_atoms(&action.precondition);
    if preconditions.len() > PDDL8_MAX_CONJUNCTS {
        return Err(Pddl8Error::BoundExceeded {
            what: "precondition atoms",
            limit: PDDL8_MAX_CONJUNCTS,
            got: preconditions.len(),
        });
    }
    let mut add_effects = Vec::new();
    let mut del_effects = Vec::new();
    let mut numeric_effects = Vec::new();
    collect_legacy_effects(
        &action.effect,
        &mut add_effects,
        &mut del_effects,
        &mut numeric_effects,
    );
    if add_effects.len() + del_effects.len() > PDDL8_MAX_CONJUNCTS {
        return Err(Pddl8Error::BoundExceeded {
            what: "effect atoms",
            limit: PDDL8_MAX_CONJUNCTS,
            got: add_effects.len() + del_effects.len(),
        });
    }
    Ok(Pddl8ActionSchema {
        name: action.name.clone(),
        params,
        preconditions,
        add_effects,
        del_effects,
        typed_params: action.params.clone(),
        condition: Some(action.precondition.clone()),
        effects: action.effect.clone(),
        numeric_effects,
    })
}

fn collect_legacy_effects(
    effects: &[PddlEffect],
    adds: &mut Vec<Pddl8Atom>,
    dels: &mut Vec<Pddl8Atom>,
    numeric: &mut Vec<NumericEffect>,
) {
    for effect in effects {
        match effect {
            PddlEffect::Add(atom) => adds.push(atom.clone()),
            PddlEffect::Del(atom) => dels.push(atom.clone()),
            PddlEffect::Numeric(effect) => numeric.push(effect.clone()),
            PddlEffect::Timed(_, _) | PddlEffect::Forall { .. } | PddlEffect::When { .. } => {}
        }
    }
}

fn parse_durative_action(expr: &SExpr) -> Result<DurativeAction, Pddl8Error> {
    let list = expr.list()?;
    let name = list
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError(":durative-action missing name".into()))?
        .atom()?
        .to_string();
    let params = parse_typed_names(field_list(list, ":parameters")?)?;
    if params.len() > PDDL8_MAX_PARAMS {
        return Err(Pddl8Error::BoundExceeded {
            what: "durative action parameters",
            limit: PDDL8_MAX_PARAMS,
            got: params.len(),
        });
    }
    let duration = parse_duration(
        field_expr(list, ":duration")
            .ok_or_else(|| Pddl8Error::ParseError("durative action missing :duration".into()))?,
    )?;
    let condition = field_expr(list, ":condition")
        .map(parse_condition)
        .transpose()?
        .unwrap_or_else(|| PddlCondition::And(vec![]));
    let conditions = match condition {
        PddlCondition::And(parts) => parts,
        other => vec![other],
    };
    let effects = field_expr(list, ":effect")
        .map(parse_effects)
        .transpose()?
        .unwrap_or_default();
    Ok(DurativeAction {
        name,
        params,
        duration,
        conditions,
        effects,
    })
}

fn parse_derived(expr: &SExpr) -> Result<DerivedPredicate, Pddl8Error> {
    let list = expr.list()?;
    if list.len() != 3 {
        return Err(Pddl8Error::ParseError(
            ":derived requires a head and body".into(),
        ));
    }
    Ok(DerivedPredicate {
        head: parse_atom(&list[1])?,
        body: parse_condition(&list[2])?,
    })
}

fn parse_process(expr: &SExpr) -> Result<PddlProcess, Pddl8Error> {
    let list = expr.list()?;
    let name = list
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError(":process missing name".into()))?
        .atom()?
        .to_string();
    let params = parse_typed_names(field_list(list, ":parameters")?)?;
    let precondition = field_expr(list, ":precondition")
        .map(parse_condition)
        .transpose()?
        .unwrap_or_else(|| PddlCondition::And(vec![]));
    let effects = field_expr(list, ":effect")
        .map(parse_effects)
        .transpose()?
        .unwrap_or_default();
    let mut numeric = Vec::new();
    for effect in effects {
        match effect {
            PddlEffect::Numeric(value) => numeric.push(value),
            other => {
                return Err(Pddl8Error::ParseError(format!(
                    "PDDL+ process {name:?} contains non-numeric effect {other:?}"
                )));
            }
        }
    }
    Ok(PddlProcess {
        name,
        params,
        precondition,
        effects: numeric,
    })
}

fn parse_event(expr: &SExpr) -> Result<PddlEvent, Pddl8Error> {
    let list = expr.list()?;
    let name = list
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError(":event missing name".into()))?
        .atom()?
        .to_string();
    Ok(PddlEvent {
        name,
        params: parse_typed_names(field_list(list, ":parameters")?)?,
        precondition: field_expr(list, ":precondition")
            .map(parse_condition)
            .transpose()?
            .unwrap_or_else(|| PddlCondition::And(vec![])),
        effects: field_expr(list, ":effect")
            .map(parse_effects)
            .transpose()?
            .unwrap_or_default(),
    })
}

fn field_expr<'a>(items: &'a [SExpr], key: &str) -> Option<&'a SExpr> {
    items
        .iter()
        .position(|item| matches!(item, SExpr::Atom(atom) if atom == key))
        .and_then(|index| items.get(index + 1))
}

fn field_list<'a>(items: &'a [SExpr], key: &str) -> Result<&'a [SExpr], Pddl8Error> {
    match field_expr(items, key) {
        Some(expr) => expr.list(),
        None => Ok(&[]),
    }
}

fn parse_atom(expr: &SExpr) -> Result<Pddl8Atom, Pddl8Error> {
    let list = expr.list()?;
    let pred = list
        .first()
        .ok_or_else(|| Pddl8Error::ParseError("empty atom".into()))?
        .atom()?
        .to_string();
    let args = list[1..]
        .iter()
        .map(|item| item.atom().map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;
    if args.len() > PDDL8_MAX_ARITY {
        return Err(Pddl8Error::BoundExceeded {
            what: "atom arity",
            limit: PDDL8_MAX_ARITY,
            got: args.len(),
        });
    }
    Ok(Pddl8Atom { pred, args })
}

fn parse_condition(expr: &SExpr) -> Result<PddlCondition, Pddl8Error> {
    let list = expr.list()?;
    if list.is_empty() {
        return Ok(PddlCondition::And(vec![]));
    }
    let head = list[0].atom()?;
    match head {
        "and" => Ok(PddlCondition::And(
            list[1..]
                .iter()
                .map(parse_condition)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "or" => Ok(PddlCondition::Or(
            list[1..]
                .iter()
                .map(parse_condition)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "not" => Ok(PddlCondition::Not(Box::new(parse_condition(single_arg(
            list, "not",
        )?)?))),
        "imply" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError("imply requires two operands".into()));
            }
            Ok(PddlCondition::Imply(
                Box::new(parse_condition(&list[1])?),
                Box::new(parse_condition(&list[2])?),
            ))
        }
        "forall" | "exists" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError(format!(
                    "{head} requires variables and a body"
                )));
            }
            let vars = parse_typed_names(list[1].list()?)?;
            let body = Box::new(parse_condition(&list[2])?);
            if head == "forall" {
                Ok(PddlCondition::Forall { vars, body })
            } else {
                Ok(PddlCondition::Exists { vars, body })
            }
        }
        "at" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError(
                    "timed condition requires (at start|end <condition>)".into(),
                ));
            }
            let time = match list[1].atom()? {
                "start" => TimeSpecifier::AtStart,
                "end" => TimeSpecifier::AtEnd,
                other => {
                    return Err(Pddl8Error::ParseError(format!(
                        "invalid timed condition qualifier {other:?}"
                    )));
                }
            };
            Ok(PddlCondition::Timed(
                time,
                Box::new(parse_condition(&list[2])?),
            ))
        }
        "over" => {
            if list.len() != 3 || list[1].atom()? != "all" {
                return Err(Pddl8Error::ParseError(
                    "timed invariant must be (over all <condition>)".into(),
                ));
            }
            Ok(PddlCondition::Timed(
                TimeSpecifier::OverAll,
                Box::new(parse_condition(&list[2])?),
            ))
        }
        ">=" | "<=" | ">" | "<" => parse_numeric_comparison(head, &list[1..]),
        "=" if list.len() == 3 && is_numeric_operand(&list[1]) => {
            parse_numeric_comparison(head, &list[1..])
        }
        "=" => Ok(PddlCondition::Atom(Pddl8Atom {
            pred: "=".into(),
            args: list[1..]
                .iter()
                .map(|item| item.atom().map(str::to_string))
                .collect::<Result<Vec<_>, _>>()?,
        })),
        _ => Ok(PddlCondition::Atom(parse_atom(expr)?)),
    }
}

fn parse_numeric_comparison(head: &str, operands: &[SExpr]) -> Result<PddlCondition, Pddl8Error> {
    if operands.len() != 2 {
        return Err(Pddl8Error::ParseError(format!(
            "numeric comparison {head} requires two operands"
        )));
    }
    let op = match head {
        ">=" => CompareOp::Ge,
        "<=" => CompareOp::Le,
        ">" => CompareOp::Gt,
        "<" => CompareOp::Lt,
        "=" => CompareOp::Eq,
        _ => unreachable!(),
    };
    Ok(PddlCondition::Compare(
        parse_numeric(&operands[0])?,
        op,
        parse_numeric(&operands[1])?,
    ))
}

fn parse_numeric(expr: &SExpr) -> Result<NumericExpr, Pddl8Error> {
    match expr {
        SExpr::Atom(atom) => atom
            .parse::<f64>()
            .map(NumericExpr::Number)
            .map_err(|_| Pddl8Error::ParseError(format!("invalid numeric atom {atom:?}"))),
        SExpr::List(list) => {
            if list.is_empty() {
                return Err(Pddl8Error::ParseError("empty numeric expression".into()));
            }
            let head = list[0].atom()?;
            match head {
                "+" | "-" | "*" | "/" if list.len() == 3 => {
                    let op = match head {
                        "+" => NumericOp::Add,
                        "-" => NumericOp::Sub,
                        "*" => NumericOp::Mul,
                        "/" => NumericOp::Div,
                        _ => unreachable!(),
                    };
                    Ok(NumericExpr::BinOp {
                        op,
                        lhs: Box::new(parse_numeric(&list[1])?),
                        rhs: Box::new(parse_numeric(&list[2])?),
                    })
                }
                "-" if list.len() == 2 => Ok(NumericExpr::Neg(Box::new(parse_numeric(&list[1])?))),
                _ => Ok(NumericExpr::FunctionTerm(
                    head.to_string(),
                    list[1..]
                        .iter()
                        .map(|item| item.atom().map(str::to_string))
                        .collect::<Result<Vec<_>, _>>()?,
                )),
            }
        }
    }
}

fn is_numeric_operand(expr: &SExpr) -> bool {
    match expr {
        SExpr::List(_) => true,
        SExpr::Atom(atom) => atom.parse::<f64>().is_ok(),
    }
}

fn parse_effects(expr: &SExpr) -> Result<Vec<PddlEffect>, Pddl8Error> {
    let list = expr.list()?;
    if list.is_empty() {
        return Ok(vec![]);
    }
    if list[0].atom()? == "and" {
        let mut out = Vec::new();
        for child in &list[1..] {
            out.extend(parse_effects(child)?);
        }
        return Ok(out);
    }
    Ok(vec![parse_effect(expr)?])
}

fn parse_effect(expr: &SExpr) -> Result<PddlEffect, Pddl8Error> {
    let list = expr.list()?;
    let head = list
        .first()
        .ok_or_else(|| Pddl8Error::ParseError("empty effect".into()))?
        .atom()?;
    match head {
        "not" => Ok(PddlEffect::Del(parse_atom(single_arg(list, "not")?)?)),
        "assign" | "increase" | "decrease" | "scale-up" | "scale-down" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError(format!(
                    "{head} effect requires a fluent and expression"
                )));
            }
            if contains_atom(&list[2], "#t") {
                return Ok(PddlEffect::Add(Pddl8Atom {
                    pred: CONTINUOUS_EFFECT_SENTINEL_PRED.into(),
                    args: vec![head.into()],
                }));
            }
            let function = parse_function_term(&list[1])?;
            let value = match parse_numeric(&list[2]) {
                Ok(value) => value,
                Err(_) if head == "assign" => {
                    return Ok(PddlEffect::Add(Pddl8Atom {
                        pred: OBJECT_FLUENT_SENTINEL_PRED.into(),
                        args: vec![function.name],
                    }));
                }
                Err(error) => return Err(error),
            };
            let numeric = match head {
                "assign" => NumericEffect::Assign(function, value),
                "increase" => NumericEffect::Increase(function, value),
                "decrease" => NumericEffect::Decrease(function, value),
                "scale-up" => NumericEffect::ScaleUp(function, value),
                "scale-down" => NumericEffect::ScaleDown(function, value),
                _ => unreachable!(),
            };
            Ok(PddlEffect::Numeric(numeric))
        }
        "when" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError(
                    "when effect requires condition and effects".into(),
                ));
            }
            Ok(PddlEffect::When {
                condition: parse_condition(&list[1])?,
                effects: parse_effects(&list[2])?,
            })
        }
        "forall" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError(
                    "forall effect requires variables and effects".into(),
                ));
            }
            Ok(PddlEffect::Forall {
                vars: parse_typed_names(list[1].list()?)?,
                effects: parse_effects(&list[2])?,
            })
        }
        "at" => {
            if list.len() != 3 {
                return Err(Pddl8Error::ParseError(
                    "timed effect requires (at start|end <effect>)".into(),
                ));
            }
            let time = match list[1].atom()? {
                "start" => TimeSpecifier::AtStart,
                "end" => TimeSpecifier::AtEnd,
                other => {
                    return Err(Pddl8Error::ParseError(format!(
                        "invalid timed effect qualifier {other:?}"
                    )));
                }
            };
            let nested = parse_effects(&list[2])?;
            if nested.len() != 1 {
                return Err(Pddl8Error::ParseError(
                    "timed effect must wrap exactly one effect; put (and ...) outside each wrapper"
                        .into(),
                ));
            }
            Ok(PddlEffect::Timed(time, Box::new(nested[0].clone())))
        }
        _ => Ok(PddlEffect::Add(parse_atom(expr)?)),
    }
}

fn parse_function_term(expr: &SExpr) -> Result<PddlFunction, Pddl8Error> {
    let list = expr.list()?;
    Ok(PddlFunction {
        name: list
            .first()
            .ok_or_else(|| Pddl8Error::ParseError("empty function term".into()))?
            .atom()?
            .to_string(),
        params: list[1..]
            .iter()
            .map(|item| item.atom().map(str::to_string))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_duration(expr: &SExpr) -> Result<DurationConstraint, Pddl8Error> {
    let list = expr.list()?;
    if list.is_empty() {
        return Err(Pddl8Error::ParseError("empty duration constraint".into()));
    }
    match list[0].atom()? {
        "and" => Ok(DurationConstraint::And(
            list[1..]
                .iter()
                .map(parse_duration)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "=" | "<=" | ">=" => {
            if list.len() != 3 || list[1].atom()? != "?duration" {
                return Err(Pddl8Error::ParseError(
                    "duration constraint must compare ?duration with an expression".into(),
                ));
            }
            let value = parse_numeric(&list[2])?;
            Ok(match list[0].atom()? {
                "=" => DurationConstraint::Eq(value),
                "<=" => DurationConstraint::Lte(value),
                ">=" => DurationConstraint::Gte(value),
                _ => unreachable!(),
            })
        }
        other => Err(Pddl8Error::ParseError(format!(
            "unsupported duration operator {other:?}"
        ))),
    }
}

fn parse_init(section: &SExpr, problem: &mut Pddl31Problem) -> Result<(), Pddl8Error> {
    for item in section.list()?.iter().skip(1) {
        let list = item.list()?;
        if list.is_empty() {
            continue;
        }
        match list[0].atom()? {
            "=" => {
                if list.len() != 3 {
                    return Err(Pddl8Error::ParseError(
                        "initial fluent assignment requires two operands".into(),
                    ));
                }
                let function = parse_function_term(&list[1])?;
                let value = list[2].atom()?.parse::<f64>().map_err(|_| {
                    Pddl8Error::ParseError(
                        "object-valued fluent initial assignments are unsupported".into(),
                    )
                })?;
                problem.init_fn_values.push((function, value));
            }
            "at" => {
                if list.len() != 3 {
                    return Err(Pddl8Error::ParseError(
                        "timed initial literal requires time and literal".into(),
                    ));
                }
                let time = list[1].atom()?.parse::<f64>().map_err(|_| {
                    Pddl8Error::ParseError("timed initial literal has invalid time".into())
                })?;
                let literal = list[2].list()?;
                let (negated, atom_expr) =
                    if literal.first().and_then(|v| v.atom().ok()) == Some("not") {
                        (true, single_arg(literal, "not")?)
                    } else {
                        (false, &list[2])
                    };
                problem.timed_inits.push(TimedLiteral {
                    time,
                    atom: parse_atom(atom_expr)?,
                    negated,
                });
            }
            "not" => {
                return Err(Pddl8Error::ParseError(
                    "negative initial literals are not representable under closed-world PDDL8"
                        .into(),
                ));
            }
            _ => problem.init_atoms.push(parse_atom(item)?),
        }
    }
    Ok(())
}

fn parse_metric(section: &SExpr) -> Result<Metric, Pddl8Error> {
    let list = section.list()?;
    if list.len() != 3 {
        return Err(Pddl8Error::ParseError(
            ":metric requires minimize|maximize and an expression".into(),
        ));
    }
    let dir = match list[1].atom()? {
        "minimize" => MetricDir::Minimize,
        "maximize" => MetricDir::Maximize,
        other => {
            return Err(Pddl8Error::ParseError(format!(
                "invalid metric direction {other:?}"
            )));
        }
    };
    Ok(Metric {
        dir,
        expr: parse_metric_expr(&list[2])?,
    })
}

fn parse_metric_expr(expr: &SExpr) -> Result<MetricExpr, Pddl8Error> {
    match expr {
        SExpr::Atom(atom) => {
            if atom == "total-time" {
                Ok(MetricExpr::TotalTime)
            } else {
                atom.parse::<f64>()
                    .map(MetricExpr::Number)
                    .map_err(|_| Pddl8Error::ParseError(format!("invalid metric atom {atom:?}")))
            }
        }
        SExpr::List(list) => {
            if list.is_empty() {
                return Err(Pddl8Error::ParseError("empty metric expression".into()));
            }
            let head = list[0].atom()?;
            match head {
                "is-violated" => {
                    if list.len() != 2 {
                        return Err(Pddl8Error::ParseError(
                            "is-violated requires one preference name".into(),
                        ));
                    }
                    Ok(MetricExpr::IsViolated(list[1].atom()?.to_string()))
                }
                "+" | "-" | "*" | "/" if list.len() == 3 => {
                    let op = match head {
                        "+" => NumericOp::Add,
                        "-" => NumericOp::Sub,
                        "*" => NumericOp::Mul,
                        "/" => NumericOp::Div,
                        _ => unreachable!(),
                    };
                    Ok(MetricExpr::BinOp {
                        op,
                        lhs: Box::new(parse_metric_expr(&list[1])?),
                        rhs: Box::new(parse_metric_expr(&list[2])?),
                    })
                }
                _ => Ok(MetricExpr::FunctionTerm(
                    head.to_string(),
                    list[1..]
                        .iter()
                        .map(|item| item.atom().map(str::to_string))
                        .collect::<Result<Vec<_>, _>>()?,
                )),
            }
        }
    }
}

fn parse_domain_constraints(section: &SExpr) -> Result<Vec<PddlConstraint>, Pddl8Error> {
    let expr = section
        .list()?
        .get(1)
        .ok_or_else(|| Pddl8Error::ParseError("missing domain :constraints expression".into()))?;
    let constraints = parse_trajectory_many(expr)?;
    Ok(constraints
        .into_iter()
        .map(|constraint| PddlConstraint {
            name: None,
            constraint,
        })
        .collect())
}

fn parse_problem_constraints(expr: &SExpr) -> Result<Vec<PddlPreference>, Pddl8Error> {
    let list = expr.list()?;
    if list.first().and_then(|item| item.atom().ok()) == Some("and") {
        let mut out = Vec::new();
        for child in &list[1..] {
            out.extend(parse_problem_constraints(child)?);
        }
        return Ok(out);
    }
    if list.first().and_then(|item| item.atom().ok()) == Some("preference") {
        if list.len() != 3 {
            return Err(Pddl8Error::ParseError(
                "preference requires a name and constraint".into(),
            ));
        }
        return Ok(vec![PddlPreference {
            name: Some(list[1].atom()?.to_string()),
            constraint: parse_trajectory(&list[2])?,
        }]);
    }
    Ok(vec![PddlPreference {
        name: None,
        constraint: parse_trajectory(expr)?,
    }])
}

fn parse_trajectory_many(expr: &SExpr) -> Result<Vec<TrajectoryConstraint>, Pddl8Error> {
    let parsed = parse_trajectory(expr)?;
    match parsed {
        TrajectoryConstraint::And(parts) => Ok(parts),
        other => Ok(vec![other]),
    }
}

fn parse_trajectory(expr: &SExpr) -> Result<TrajectoryConstraint, Pddl8Error> {
    let list = expr.list()?;
    let head = list
        .first()
        .ok_or_else(|| Pddl8Error::ParseError("empty trajectory constraint".into()))?
        .atom()?;
    match head {
        "and" => Ok(TrajectoryConstraint::And(
            list[1..]
                .iter()
                .map(parse_trajectory)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "always" => Ok(TrajectoryConstraint::Always(Box::new(parse_condition(
            single_arg(list, "always")?,
        )?))),
        "sometime" => Ok(TrajectoryConstraint::Sometime(Box::new(parse_condition(
            single_arg(list, "sometime")?,
        )?))),
        "within" => {
            require_len(list, 3, "within")?;
            Ok(TrajectoryConstraint::Within(
                parse_number_atom(&list[1])?,
                Box::new(parse_condition(&list[2])?),
            ))
        }
        "at-most-once" => Ok(TrajectoryConstraint::AtMostOnce(Box::new(parse_condition(
            single_arg(list, "at-most-once")?,
        )?))),
        "sometime-before" | "sometime-after" => {
            require_len(list, 3, head)?;
            let first = Box::new(parse_condition(&list[1])?);
            let second = Box::new(parse_condition(&list[2])?);
            Ok(if head == "sometime-before" {
                TrajectoryConstraint::SometimeBefore(first, second)
            } else {
                TrajectoryConstraint::SometimeAfter(first, second)
            })
        }
        "always-within" => {
            require_len(list, 4, head)?;
            Ok(TrajectoryConstraint::AlwaysWithin(
                parse_number_atom(&list[1])?,
                Box::new(parse_condition(&list[2])?),
                Box::new(parse_condition(&list[3])?),
            ))
        }
        "hold-during" => {
            require_len(list, 4, head)?;
            Ok(TrajectoryConstraint::HoldDuring(
                parse_number_atom(&list[1])?,
                parse_number_atom(&list[2])?,
                Box::new(parse_condition(&list[3])?),
            ))
        }
        "hold-after" => {
            require_len(list, 3, head)?;
            Ok(TrajectoryConstraint::HoldAfter(
                parse_number_atom(&list[1])?,
                Box::new(parse_condition(&list[2])?),
            ))
        }
        other => Err(Pddl8Error::ParseError(format!(
            "unsupported trajectory constraint {other:?}"
        ))),
    }
}

fn legacy_positive_atoms(condition: &PddlCondition) -> Vec<Pddl8Atom> {
    match condition {
        PddlCondition::Atom(atom) if atom.pred != "=" => vec![atom.clone()],
        PddlCondition::And(parts) => parts.iter().flat_map(legacy_positive_atoms).collect(),
        _ => vec![],
    }
}

fn single_arg<'a>(list: &'a [SExpr], name: &str) -> Result<&'a SExpr, Pddl8Error> {
    if list.len() != 2 {
        return Err(Pddl8Error::ParseError(format!(
            "{name} requires exactly one argument"
        )));
    }
    Ok(&list[1])
}

fn require_len(list: &[SExpr], expected: usize, name: &str) -> Result<(), Pddl8Error> {
    if list.len() != expected {
        Err(Pddl8Error::ParseError(format!(
            "{name} requires {} operands",
            expected - 1
        )))
    } else {
        Ok(())
    }
}

fn parse_number_atom(expr: &SExpr) -> Result<f64, Pddl8Error> {
    expr.atom()?.parse::<f64>().map_err(|_| {
        Pddl8Error::ParseError(format!("expected number, found {:?}", expr.atom().ok()))
    })
}

fn contains_atom(expr: &SExpr, needle: &str) -> bool {
    match expr {
        SExpr::Atom(atom) => atom == needle,
        SExpr::List(items) => items.iter().any(|item| contains_atom(item, needle)),
    }
}

fn contains_head(expr: &SExpr, needle: &str) -> bool {
    expr.head() == Some(needle)
        || matches!(expr, SExpr::List(items) if items.iter().any(|item| contains_head(item, needle)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typed_logistics_without_cross_type_grounding_surface() {
        let domain = domain31_from_pddl(
            r#"
            (define (domain logistics)
              (:requirements :strips :typing)
              (:types package truck location)
              (:predicates (at ?x - object ?l - location) (in ?p - package ?t - truck))
              (:action load
                :parameters (?p - package ?t - truck ?l - location)
                :precondition (and (at ?p ?l) (at ?t ?l))
                :effect (and (in ?p ?t) (not (at ?p ?l)))))
            "#,
        )
        .unwrap();
        assert_eq!(domain.requirements, vec!["Strips", "Typing"]);
        assert_eq!(domain.actions[0].params[0], ("?p".into(), "package".into()));
    }

    #[test]
    fn preserves_quantified_conditions_and_conditional_effects() {
        let domain = domain31_from_pddl(
            r#"
            (define (domain quantified)
              (:requirements :adl)
              (:types item)
              (:predicates (ready ?x - item) (done ?x - item))
              (:action finish
                :parameters ()
                :precondition (forall (?x - item) (ready ?x))
                :effect (forall (?x - item) (when (ready ?x) (done ?x)))))
            "#,
        )
        .unwrap();
        assert!(matches!(
            domain.actions[0].precondition,
            PddlCondition::Forall { .. }
        ));
        assert!(matches!(
            domain.actions[0].effect[0],
            PddlEffect::Forall { .. }
        ));
    }

    #[test]
    fn parses_preferences_and_metric_without_dropping_them() {
        let problem = problem31_from_pddl(
            r#"
            (define (problem p)
              (:domain d)
              (:objects a)
              (:init (ready a))
              (:goal (done a))
              (:constraints (and (preference keep-ready (always (ready a)))))
              (:metric minimize (+ 1 (is-violated keep-ready))))
            "#,
        )
        .unwrap();
        assert_eq!(problem.preferences.len(), 1);
        assert!(problem.metric.is_some());
    }

    #[test]
    fn comments_and_uppercase_are_accepted() {
        let domain = domain31_from_pddl(
            "(DEFINE (DOMAIN D) ; comment\n (:PREDICATES (P)) (:ACTION A :PARAMETERS () :PRECONDITION () :EFFECT (P)))",
        )
        .unwrap();
        assert_eq!(domain.name, "d");
        assert_eq!(domain.actions[0].name, "a");
    }

    #[test]
    fn rejects_unrepresentable_domain_constants() {
        let error = domain31_from_pddl(
            "(define (domain d) (:types thing) (:constants x - thing) (:predicates (p)))",
        )
        .unwrap_err();
        assert!(matches!(error, Pddl8Error::ParseError(_)));
    }
}
