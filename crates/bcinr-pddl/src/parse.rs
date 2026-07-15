//! PDDL 3.1 text → wasm4pm_compat::pddl canonical types.
#![allow(unused_imports)]

use crate::error::Pddl8Error;
use pddl::{
    parsers::Span, AssignOp, BinaryOp, ConditionalEffect, ConstraintGoalDefinition,
    ConstraintGoalDefinitionInner, DurationConstraint as PddlDurationConstraint, DurationOperator,
    DurationValue, DurativeActionEffect, DurativeActionGoalDefinition, FluentExpression,
    FunctionHead, GoalDefinition, InitElement, MetricFluentExpression, MultiOp, Optimization,
    Parser, PreconditionGoalDefinition, PredicateAtomicFormula, PreferenceGoalDefinition,
    PreferenceTimedGoalDefinition, PrimitiveEffect, SimpleDurationConstraint, StructureDef,
    TimedEffect, TimedGoalDefinition,
};
use wasm4pm_compat::pddl::{
    CompareOp, DerivedPredicate, DurationConstraint, DurativeAction, Metric, MetricDir, MetricExpr,
    NumericEffect, NumericExpr, NumericOp, Pddl31Action, Pddl31Domain, Pddl31Problem,
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem, PddlCondition, PddlConstraint,
    PddlEffect, PddlEvent, PddlFunction, PddlPreference, PddlProcess, PddlType, TimeSpecifier,
    TimedLiteral, TrajectoryConstraint, PDDL8_MAX_ARITY, PDDL8_MAX_CONJUNCTS, PDDL8_MAX_PARAMS,
};

// ─────────────────────────────────────────────────────────────────────────────
// Public API: existing functions (updated to populate PDDL 3.1 extended fields)
// ─────────────────────────────────────────────────────────────────────────────

pub fn domain_from_pddl(text: &str) -> Result<Pddl8Domain, Pddl8Error> {
    let (_, dom) = pddl::Domain::parse(Span::new(text))
        .map_err(|e| Pddl8Error::ParseError(format!("{e:?}")))?;

    let name = dom.name().to_string();

    let predicates = dom
        .predicates()
        .values()
        .iter()
        .map(|p| {
            let arity = p.variables().value().len();
            if arity > PDDL8_MAX_ARITY {
                Err(Pddl8Error::BoundExceeded {
                    what: "predicate arity",
                    limit: PDDL8_MAX_ARITY as u8,
                    got: arity,
                })
            } else {
                Ok((p.name().to_string(), arity as u8))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let actions = dom
        .structure()
        .values()
        .iter()
        .filter_map(|sd| {
            if let StructureDef::Action(a) = sd {
                Some(a)
            } else {
                None
            }
        })
        .map(lower_action)
        .collect::<Result<Vec<_>, _>>()?;

    // PDDL 3.1 extended fields
    let types = lower_types(dom.types());
    let functions = lower_functions(dom.functions());
    let durative_actions = dom
        .structure()
        .values()
        .iter()
        .filter_map(|sd| {
            if let StructureDef::DurativeAction(da) = sd {
                Some(da.as_ref())
            } else {
                None
            }
        })
        .map(lower_durative_action)
        .collect();
    let derived = dom
        .structure()
        .values()
        .iter()
        .filter_map(|sd| {
            if let StructureDef::Derived(dp) = sd {
                Some(dp)
            } else {
                None
            }
        })
        .map(lower_derived_predicate)
        .collect();

    Ok(Pddl8Domain {
        name,
        predicates,
        actions,
        types,
        functions,
        durative_actions,
        derived,
        constraints: vec![],
        processes: vec![],
        events: vec![],
    })
}

pub fn problem_from_pddl(text: &str) -> Result<Pddl8Problem, Pddl8Error> {
    let (_, prob) = pddl::Problem::parse(Span::new(text))
        .map_err(|e| Pddl8Error::ParseError(format!("{e:?}")))?;

    let objects: Vec<String> = prob
        .objects()
        .values()
        .value()
        .iter()
        .map(|t| t.value().to_string())
        .collect();
    let object_types: Vec<(String, String)> = prob
        .objects()
        .values()
        .value()
        .iter()
        .map(|t| (t.value().to_string(), type_to_string(t.type_())))
        .collect();

    let (init, timed_inits, fn_values) = lower_init_full(prob.init())?;
    let goal = lower_precond_defs(prob.goals())?;

    let metric = prob.metric_spec().as_ref().map(lower_metric);

    Ok(Pddl8Problem {
        name: prob.name().to_string(),
        domain: prob.domain().to_string(),
        objects,
        object_types,
        init,
        goal,
        timed_inits,
        fn_values,
        preferences: vec![],
        metric,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API: new PDDL 3.1 full-fidelity functions
// ─────────────────────────────────────────────────────────────────────────────

pub fn domain31_from_pddl(text: &str) -> Result<Pddl31Domain, Pddl8Error> {
    let (_, dom) = pddl::Domain::parse(Span::new(text))
        .map_err(|e| Pddl8Error::ParseError(format!("{e:?}")))?;

    let name = dom.name().to_string();

    let requirements = dom
        .requirements()
        .iter()
        .map(|r| format!("{r:?}"))
        .collect();

    let types = lower_types(dom.types());
    let functions = lower_functions(dom.functions());

    let predicates = dom
        .predicates()
        .values()
        .iter()
        .map(|p| {
            let typed_params = lower_typed_variables(p.variables());
            (p.name().to_string(), typed_params)
        })
        .collect();

    let mut actions = Vec::new();
    let mut durative_actions = Vec::new();
    let mut derived = Vec::new();

    for sd in dom.structure().values().iter() {
        match sd {
            StructureDef::Action(a) => {
                actions.push(lower_action31(a));
            }
            StructureDef::DurativeAction(da) => {
                durative_actions.push(lower_durative_action(da.as_ref()));
            }
            StructureDef::Derived(dp) => {
                derived.push(lower_derived_predicate(dp));
            }
        }
    }

    let constraints = lower_constraint_gd(dom.constraints());

    Ok(Pddl31Domain {
        name,
        requirements,
        types,
        predicates,
        functions,
        actions,
        durative_actions,
        derived,
        constraints,
        processes: vec![],
        events: vec![],
    })
}

pub fn problem31_from_pddl(text: &str) -> Result<Pddl31Problem, Pddl8Error> {
    let (_, prob) = pddl::Problem::parse(Span::new(text))
        .map_err(|e| Pddl8Error::ParseError(format!("{e:?}")))?;

    let objects = prob
        .objects()
        .values()
        .value()
        .iter()
        .map(|t| {
            let type_name = type_to_string(t.type_());
            (t.value().to_string(), type_name)
        })
        .collect();

    let mut init_atoms = Vec::new();
    let mut init_fn_values = Vec::new();
    let mut timed_inits = Vec::new();

    for el in prob.init().iter() {
        match el {
            InitElement::Literal(lit) => {
                if let pddl::Literal::AtomicFormula(pddl::AtomicFormula::Predicate(p)) = lit {
                    init_atoms.push(Pddl8Atom {
                        pred: p.predicate().to_string(),
                        args: p.values().iter().map(|n| n.to_string()).collect(),
                    });
                }
            }
            InitElement::At(time, lit) => {
                let t = num_f64(*time);
                match lit {
                    pddl::Literal::AtomicFormula(af) => {
                        if let pddl::AtomicFormula::Predicate(p) = af {
                            timed_inits.push(TimedLiteral {
                                time: t,
                                atom: Pddl8Atom {
                                    pred: p.predicate().to_string(),
                                    args: p.values().iter().map(|n| n.to_string()).collect(),
                                },
                                negated: false,
                            });
                        }
                    }
                    pddl::Literal::NotAtomicFormula(af) => {
                        if let pddl::AtomicFormula::Predicate(p) = af {
                            timed_inits.push(TimedLiteral {
                                time: t,
                                atom: Pddl8Atom {
                                    pred: p.predicate().to_string(),
                                    args: p.values().iter().map(|n| n.to_string()).collect(),
                                },
                                negated: true,
                            });
                        }
                    }
                }
            }
            InitElement::IsValue(bft, num) => {
                let func = PddlFunction {
                    name: bft.symbol().to_string(),
                    params: bft.names().iter().map(|n| n.to_string()).collect(),
                };
                init_fn_values.push((func, num_f64(*num)));
            }
            InitElement::IsObject(_, _) => {
                // Object fluent initial assignment — TODO: extend if needed
            }
        }
    }

    let goal = lower_goal_full(prob.goals());
    let metric = prob.metric_spec().as_ref().map(lower_metric);

    Ok(Pddl31Problem {
        name: prob.name().to_string(),
        domain: prob.domain().to_string(),
        objects,
        init_atoms,
        init_fn_values,
        timed_inits,
        goal,
        preferences: vec![],
        metric,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Numeric conversion helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a `pddl::Number` (backed by `f32`) to `f64`.
#[inline]
fn num_f64(n: pddl::Number) -> f64 {
    // pddl::Number derefs to f32 via Deref<Target=f32>.
    let v: f32 = *n;
    f64::from(v)
}

// ─────────────────────────────────────────────────────────────────────────────
// Existing internal lowering helpers (updated)
// ─────────────────────────────────────────────────────────────────────────────

fn lower_action(a: &pddl::ActionDefinition) -> Result<Pddl8ActionSchema, Pddl8Error> {
    let params: Vec<String> = a
        .parameters()
        .value()
        .iter()
        .map(|t| format!("?{}", t.value()))
        .collect();
    if params.len() > PDDL8_MAX_PARAMS {
        return Err(Pddl8Error::BoundExceeded {
            what: "action parameters",
            limit: PDDL8_MAX_PARAMS as u8,
            got: params.len(),
        });
    }

    let preconditions = lower_precond_defs(a.precondition())?;
    if preconditions.len() > PDDL8_MAX_CONJUNCTS {
        return Err(Pddl8Error::BoundExceeded {
            what: "precondition atoms",
            limit: PDDL8_MAX_CONJUNCTS as u8,
            got: preconditions.len(),
        });
    }

    let (add_effects, del_effects) = lower_effects(a.effect())?;

    // Extended fields
    let typed_params = lower_typed_variables(a.parameters());
    let condition = {
        let c = lower_precond_defs_full(a.precondition());
        // Only store if non-trivial
        match &c {
            PddlCondition::And(v) if v.is_empty() => None,
            _ => Some(c),
        }
    };
    let effects = lower_effect_list(a.effect());

    Ok(Pddl8ActionSchema {
        name: a.symbol().to_string(),
        params,
        preconditions,
        add_effects,
        del_effects,
        typed_params,
        condition,
        effects,
        numeric_effects: vec![],
    })
}

fn lower_precond_defs(
    defs: &pddl::PreconditionGoalDefinitions,
) -> Result<Vec<Pddl8Atom>, Pddl8Error> {
    let mut out = Vec::new();
    for def in defs.iter() {
        collect_precond_def(def, &mut out);
    }
    Ok(out)
}

fn collect_precond_def(def: &PreconditionGoalDefinition, out: &mut Vec<Pddl8Atom>) {
    match def {
        PreconditionGoalDefinition::Preference(pref) => collect_pref_gd(pref, out),
        PreconditionGoalDefinition::Forall(_, inner) => {
            for d in inner.iter() {
                collect_precond_def(d, out);
            }
        }
    }
}

fn collect_pref_gd(pref: &PreferenceGoalDefinition, out: &mut Vec<Pddl8Atom>) {
    match pref {
        PreferenceGoalDefinition::Goal(gd) => collect_gd(gd, out),
        PreferenceGoalDefinition::Preference(_) => {}
    }
}

fn collect_gd(gd: &GoalDefinition, out: &mut Vec<Pddl8Atom>) {
    match gd {
        GoalDefinition::AtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) {
                out.push(atom);
            }
        }
        GoalDefinition::And(cs) => {
            for c in cs {
                collect_gd(c, out);
            }
        }
        GoalDefinition::Literal(lit) => {
            use pddl::Literal;
            if let Literal::AtomicFormula(af) = lit {
                if let Some(atom) = lower_af_term(af) {
                    out.push(atom);
                }
            }
        }
        GoalDefinition::Not(inner) => {
            // STRIPS8: negations can't be positive atoms; skip for STRIPS8 compatibility.
            // Full condition is available via Pddl8ActionSchema::condition.
            let _ = inner;
        }
        GoalDefinition::Or(cs) => {
            // STRIPS8: flatten OR into atoms (lossy). Full condition in extended field.
            for c in cs {
                collect_gd(c, out);
            }
        }
        GoalDefinition::Imply(a, b) => {
            collect_gd(a, out);
            collect_gd(b, out);
        }
        GoalDefinition::Exists(_, body) => {
            collect_gd(body, out);
        }
        GoalDefinition::ForAll(_, body) => {
            collect_gd(body, out);
        }
        GoalDefinition::FluentComparison(_) => {
            // Numeric comparison — not representable as a STRIPS8 atom.
            // Full fidelity is available via the `PddlCondition` algebra
            // (see `lower_condition`), used by durative-action grounding.
        }
    }
}

fn lower_effects(
    eff: &Option<pddl::Effects>,
) -> Result<(Vec<Pddl8Atom>, Vec<Pddl8Atom>), Pddl8Error> {
    let Some(effects) = eff else {
        return Ok((vec![], vec![]));
    };
    let mut adds = Vec::new();
    let mut dels = Vec::new();
    for ce in effects.iter() {
        collect_conditional_effect(ce, &mut adds, &mut dels);
    }
    Ok((adds, dels))
}

fn collect_conditional_effect(
    ce: &ConditionalEffect,
    adds: &mut Vec<Pddl8Atom>,
    dels: &mut Vec<Pddl8Atom>,
) {
    match ce {
        ConditionalEffect::Effect(pe) => collect_primitive_effect(pe, adds, dels),
        ConditionalEffect::Forall(f) => {
            for inner in f.effects.iter() {
                collect_conditional_effect(inner, adds, dels);
            }
        }
        ConditionalEffect::When(w) => {
            for pe in w.effect.clone() {
                collect_primitive_effect(&pe, adds, dels);
            }
        }
    }
}

fn collect_primitive_effect(
    pe: &PrimitiveEffect,
    adds: &mut Vec<Pddl8Atom>,
    dels: &mut Vec<Pddl8Atom>,
) {
    match pe {
        PrimitiveEffect::AtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) {
                adds.push(atom);
            }
        }
        PrimitiveEffect::NotAtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) {
                dels.push(atom);
            }
        }
        PrimitiveEffect::AssignNumericFluent(_, _, _) => {
            // Numeric effect — available via PddlEffect algebra in extended fields.
        }
        PrimitiveEffect::AssignObjectFluent(_, _) => {
            // Object fluent effect — not in STRIPS8.
        }
    }
}

fn lower_af_term(af: &pddl::AtomicFormula<pddl::Term>) -> Option<Pddl8Atom> {
    if let pddl::AtomicFormula::Predicate(p) = af {
        Some(lower_pred_af(p))
    } else {
        None
    }
}

fn lower_pred_af(p: &PredicateAtomicFormula<pddl::Term>) -> Pddl8Atom {
    Pddl8Atom {
        pred: p.predicate().to_string(),
        args: p
            .values()
            .iter()
            .map(|t| match t {
                pddl::Term::Name(n) => n.to_string(),
                pddl::Term::Variable(v) => format!("?{v}"),
                pddl::Term::Function(_) => "_".to_string(),
            })
            .collect(),
    }
}

/// `(init_atoms, timed_inits, fn_values)` — [`lower_init_full`]'s return
/// shape, factored into a named alias per clippy's `type_complexity` lint.
type LoweredInit = (Vec<Pddl8Atom>, Vec<TimedLiteral>, Vec<(PddlFunction, f64)>);

/// Returns (init_atoms, timed_inits, fn_values) from an InitElements.
fn lower_init_full(init: &pddl::InitElements) -> Result<LoweredInit, Pddl8Error> {
    let mut atoms = Vec::new();
    let mut timed = Vec::new();
    let mut fn_vals = Vec::new();

    for el in init.iter() {
        match el {
            InitElement::Literal(lit) => {
                if let pddl::Literal::AtomicFormula(pddl::AtomicFormula::Predicate(p)) = lit {
                    atoms.push(Pddl8Atom {
                        pred: p.predicate().to_string(),
                        args: p.values().iter().map(|n| n.to_string()).collect(),
                    });
                }
            }
            InitElement::At(time, lit) => {
                let t = num_f64(*time);
                match lit {
                    pddl::Literal::AtomicFormula(af) => {
                        if let pddl::AtomicFormula::Predicate(p) = af {
                            timed.push(TimedLiteral {
                                time: t,
                                atom: Pddl8Atom {
                                    pred: p.predicate().to_string(),
                                    args: p.values().iter().map(|n| n.to_string()).collect(),
                                },
                                negated: false,
                            });
                        }
                    }
                    pddl::Literal::NotAtomicFormula(af) => {
                        if let pddl::AtomicFormula::Predicate(p) = af {
                            timed.push(TimedLiteral {
                                time: t,
                                atom: Pddl8Atom {
                                    pred: p.predicate().to_string(),
                                    args: p.values().iter().map(|n| n.to_string()).collect(),
                                },
                                negated: true,
                            });
                        }
                    }
                }
            }
            InitElement::IsValue(bft, num) => {
                let func = PddlFunction {
                    name: bft.symbol().to_string(),
                    params: bft.names().iter().map(|n| n.to_string()).collect(),
                };
                fn_vals.push((func, num_f64(*num)));
            }
            InitElement::IsObject(_, _) => {}
        }
    }
    Ok((atoms, timed, fn_vals))
}

// ─────────────────────────────────────────────────────────────────────────────
// PDDL 3.1 lowering helpers
// ─────────────────────────────────────────────────────────────────────────────

fn lower_types(types: &pddl::Types) -> Vec<PddlType> {
    types
        .values()
        .value()
        .iter()
        .map(|t| {
            let parent = match t.type_() {
                pddl::Type::Exactly(pt) => {
                    let s = pt.to_string();
                    if s == "object" {
                        None
                    } else {
                        Some(s)
                    }
                }
                pddl::Type::EitherOf(pts) => pts.first().map(|pt| pt.to_string()),
            };
            PddlType {
                name: t.value().to_string(),
                parent,
            }
        })
        .collect()
}

fn lower_functions(functions: &pddl::Functions) -> Vec<PddlFunction> {
    functions
        .values()
        .values()
        .iter()
        .map(|ft| {
            let skel = ft.value_ref();
            PddlFunction {
                name: skel.symbol().to_string(),
                params: skel
                    .variables()
                    .value()
                    .iter()
                    .map(|v| format!("?{}", v.value()))
                    .collect(),
            }
        })
        .collect()
}

fn lower_typed_variables(params: &pddl::TypedVariables) -> Vec<(String, String)> {
    params
        .value()
        .iter()
        .map(|t| {
            let var = format!("?{}", t.value());
            let ty = type_to_string(t.type_());
            (var, ty)
        })
        .collect()
}

fn type_to_string(t: &pddl::Type) -> String {
    match t {
        pddl::Type::Exactly(pt) => pt.to_string(),
        pddl::Type::EitherOf(pts) => pts
            .first()
            .map(|pt| pt.to_string())
            .unwrap_or_else(|| "object".to_string()),
    }
}

fn lower_action31(a: &pddl::ActionDefinition) -> Pddl31Action {
    let params = lower_typed_variables(a.parameters());
    let precondition = lower_precond_defs_full(a.precondition());
    let effect = lower_effect_list(a.effect());
    Pddl31Action {
        name: a.symbol().to_string(),
        params,
        precondition,
        effect,
    }
}

fn lower_precond_defs_full(defs: &pddl::PreconditionGoalDefinitions) -> PddlCondition {
    let conjuncts: Vec<PddlCondition> = defs.iter().map(lower_precond_def_full).collect();
    match conjuncts.len() {
        0 => PddlCondition::And(vec![]),
        1 => conjuncts.into_iter().next().unwrap(),
        _ => PddlCondition::And(conjuncts),
    }
}

fn lower_precond_def_full(def: &PreconditionGoalDefinition) -> PddlCondition {
    match def {
        PreconditionGoalDefinition::Preference(pref) => lower_pref_gd_full(pref),
        PreconditionGoalDefinition::Forall(vars, inner) => {
            let typed_vars = lower_typed_variables(vars);
            let body_parts: Vec<_> = inner.iter().map(lower_precond_def_full).collect();
            let body_cond = match body_parts.len() {
                0 => PddlCondition::And(vec![]),
                1 => body_parts.into_iter().next().unwrap(),
                _ => PddlCondition::And(body_parts),
            };
            PddlCondition::Forall {
                vars: typed_vars,
                body: Box::new(body_cond),
            }
        }
    }
}

fn lower_pref_gd_full(pref: &PreferenceGoalDefinition) -> PddlCondition {
    match pref {
        PreferenceGoalDefinition::Goal(gd) => lower_condition(gd),
        PreferenceGoalDefinition::Preference(_) => {
            // TODO: lower named preferences into PddlCondition
            PddlCondition::And(vec![])
        }
    }
}

/// Lower a GoalDefinition into the full PddlCondition algebra.
fn lower_condition(gd: &GoalDefinition) -> PddlCondition {
    match gd {
        GoalDefinition::AtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) {
                PddlCondition::Atom(atom)
            } else {
                PddlCondition::And(vec![])
            }
        }
        GoalDefinition::Literal(lit) => match lit {
            pddl::Literal::AtomicFormula(af) => {
                if let Some(atom) = lower_af_term(af) {
                    PddlCondition::Atom(atom)
                } else {
                    PddlCondition::And(vec![])
                }
            }
            pddl::Literal::NotAtomicFormula(af) => {
                if let Some(atom) = lower_af_term(af) {
                    PddlCondition::Not(Box::new(PddlCondition::Atom(atom)))
                } else {
                    PddlCondition::And(vec![])
                }
            }
        },
        GoalDefinition::And(cs) => PddlCondition::And(cs.iter().map(lower_condition).collect()),
        GoalDefinition::Or(cs) => PddlCondition::Or(cs.iter().map(lower_condition).collect()),
        GoalDefinition::Not(inner) => PddlCondition::Not(Box::new(lower_condition(inner))),
        GoalDefinition::Imply(a, b) => {
            PddlCondition::Imply(Box::new(lower_condition(a)), Box::new(lower_condition(b)))
        }
        GoalDefinition::Exists(vars, body) => PddlCondition::Exists {
            vars: lower_typed_variables(vars),
            body: Box::new(lower_condition(body)),
        },
        GoalDefinition::ForAll(vars, body) => PddlCondition::Forall {
            vars: lower_typed_variables(vars),
            body: Box::new(lower_condition(body)),
        },
        GoalDefinition::FluentComparison(fc) => {
            use pddl::BinaryComparison;
            let op = match fc.comparison() {
                BinaryComparison::GreaterOrEqual => CompareOp::Ge,
                BinaryComparison::LessThanOrEqual => CompareOp::Le,
                BinaryComparison::GreaterThan => CompareOp::Gt,
                BinaryComparison::LessThan => CompareOp::Lt,
                BinaryComparison::Equal => CompareOp::Eq,
            };
            PddlCondition::Compare(
                lower_fluent_expression(fc.first()),
                op,
                lower_fluent_expression(fc.second()),
            )
        }
    }
}

fn lower_goal_full(defs: &pddl::PreconditionGoalDefinitions) -> PddlCondition {
    lower_precond_defs_full(defs)
}

fn lower_effect_list(eff: &Option<pddl::Effects>) -> Vec<PddlEffect> {
    let Some(effects) = eff else {
        return vec![];
    };
    effects.iter().map(lower_conditional_effect_full).collect()
}

fn lower_conditional_effect_full(ce: &ConditionalEffect) -> PddlEffect {
    match ce {
        ConditionalEffect::Effect(pe) => lower_primitive_effect_full(pe),
        ConditionalEffect::Forall(f) => {
            let vars = lower_typed_variables(&f.variables);
            let effects = f
                .effects
                .iter()
                .map(lower_conditional_effect_full)
                .collect();
            PddlEffect::Forall { vars, effects }
        }
        ConditionalEffect::When(w) => {
            let condition = lower_condition(&w.condition);
            let effects = w
                .effect
                .clone()
                .into_iter()
                .map(|pe| lower_primitive_effect_full(&pe))
                .collect();
            PddlEffect::When { condition, effects }
        }
    }
}

fn lower_primitive_effect_full(pe: &PrimitiveEffect) -> PddlEffect {
    match pe {
        PrimitiveEffect::AtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) {
                PddlEffect::Add(atom)
            } else {
                PddlEffect::Add(Pddl8Atom {
                    pred: "_".to_string(),
                    args: vec![],
                })
            }
        }
        PrimitiveEffect::NotAtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) {
                PddlEffect::Del(atom)
            } else {
                PddlEffect::Del(Pddl8Atom {
                    pred: "_".to_string(),
                    args: vec![],
                })
            }
        }
        PrimitiveEffect::AssignNumericFluent(op, head, exp) => {
            let func = lower_function_head(head);
            let expr = lower_fluent_expression(exp);
            PddlEffect::Numeric(lower_assign_op_numeric(op, func, expr))
        }
        PrimitiveEffect::AssignObjectFluent(_, _) => {
            // TODO: Object fluent effects
            PddlEffect::Add(Pddl8Atom {
                pred: "_object_fluent".to_string(),
                args: vec![],
            })
        }
    }
}

fn lower_assign_op_numeric(op: &AssignOp, func: PddlFunction, expr: NumericExpr) -> NumericEffect {
    match op {
        AssignOp::Assign => NumericEffect::Assign(func, expr),
        AssignOp::ScaleUp => NumericEffect::ScaleUp(func, expr),
        AssignOp::ScaleDown => NumericEffect::ScaleDown(func, expr),
        AssignOp::Increase => NumericEffect::Increase(func, expr),
        AssignOp::Decrease => NumericEffect::Decrease(func, expr),
    }
}

fn lower_function_head(head: &FunctionHead) -> PddlFunction {
    match head {
        FunctionHead::Simple(sym) => PddlFunction {
            name: sym.to_string(),
            params: vec![],
        },
        FunctionHead::WithTerms(sym, terms) => PddlFunction {
            name: sym.to_string(),
            params: terms
                .iter()
                .map(|t| match t {
                    pddl::Term::Name(n) => n.to_string(),
                    pddl::Term::Variable(v) => format!("?{v}"),
                    pddl::Term::Function(_) => "_".to_string(),
                })
                .collect(),
        },
    }
}

fn lower_fluent_expression(fe: &FluentExpression) -> NumericExpr {
    match fe {
        FluentExpression::Number(n) => NumericExpr::Number(num_f64(*n)),
        FluentExpression::Function(head) => {
            let func = lower_function_head(head);
            NumericExpr::FunctionTerm(func.name, func.params)
        }
        FluentExpression::Negative(inner) => {
            NumericExpr::Neg(Box::new(lower_fluent_expression(inner)))
        }
        FluentExpression::BinaryOp(op, lhs, rhs) => NumericExpr::BinOp {
            op: lower_binary_op(op),
            lhs: Box::new(lower_fluent_expression(lhs)),
            rhs: Box::new(lower_fluent_expression(rhs)),
        },
        FluentExpression::MultiOp(op, lhs, rhs) => {
            let nop = lower_multi_op(op);
            rhs.iter()
                .fold(lower_fluent_expression(lhs), |acc, r| NumericExpr::BinOp {
                    op: nop,
                    lhs: Box::new(acc),
                    rhs: Box::new(lower_fluent_expression(r)),
                })
        }
    }
}

fn lower_da_fluent_expression(fe: &pddl::DurativeActionFluentExpression) -> NumericExpr {
    use pddl::DurativeActionFluentExpression;
    match fe {
        DurativeActionFluentExpression::Assign(op, head, inner) => {
            // A nested assignment expression inside a duration-relative fluent
            // expression; lower it as the inner expression's value (the
            // assignment's effect is applied by the outer TimedEffect, this
            // path only matters for the value it evaluates to).
            let _ = (op, head);
            lower_da_fluent_expression(inner)
        }
        DurativeActionFluentExpression::BinaryOp(op, lhs, rhs) => NumericExpr::BinOp {
            op: lower_binary_op(op),
            lhs: Box::new(lower_da_fluent_expression(lhs)),
            rhs: Box::new(lower_da_fluent_expression(rhs)),
        },
        DurativeActionFluentExpression::MultiOp(op, lhs, rhs) => {
            let nop = lower_multi_op(op);
            rhs.iter().fold(lower_da_fluent_expression(lhs), |acc, r| {
                NumericExpr::BinOp {
                    op: nop,
                    lhs: Box::new(acc),
                    rhs: Box::new(lower_da_fluent_expression(r)),
                }
            })
        }
        DurativeActionFluentExpression::Negative(inner) => {
            NumericExpr::Neg(Box::new(lower_da_fluent_expression(inner)))
        }
        DurativeActionFluentExpression::Duration => {
            // `?duration` reference — not tracked as a numeric fluent key;
            // callers needing the actual duration value read it from the
            // GroundDurativeAction's resolved duration_min/duration_max instead.
            NumericExpr::Number(0.0)
        }
        DurativeActionFluentExpression::FluentExpression(inner) => lower_fluent_expression(inner),
    }
}

fn lower_binary_op(op: &BinaryOp) -> NumericOp {
    match op {
        BinaryOp::Addition => NumericOp::Add,
        BinaryOp::Subtraction => NumericOp::Sub,
        BinaryOp::Multiplication => NumericOp::Mul,
        BinaryOp::Division => NumericOp::Div,
    }
}

fn lower_multi_op(op: &MultiOp) -> NumericOp {
    match op {
        MultiOp::Addition => NumericOp::Add,
        MultiOp::Multiplication => NumericOp::Mul,
    }
}

fn lower_durative_action(da: &pddl::DurativeActionDefinition) -> DurativeAction {
    let name = da.symbol().to_string();
    let params = lower_typed_variables(da.parameters());

    let duration = da
        .duration()
        .as_ref()
        .map(lower_duration_constraint)
        .unwrap_or(DurationConstraint::Eq(NumericExpr::Number(0.0)));

    let conditions = da.condition().as_ref().map(lower_da_gd).unwrap_or_default();

    let effects = da
        .effect()
        .as_ref()
        .map(lower_da_effect)
        .unwrap_or_default();

    DurativeAction {
        name,
        params,
        duration,
        conditions,
        effects,
    }
}

fn lower_duration_constraint(dc: &PddlDurationConstraint) -> DurationConstraint {
    match dc {
        PddlDurationConstraint::Single(sdc) => lower_simple_duration_constraint(sdc),
        PddlDurationConstraint::All(sdcs) => {
            DurationConstraint::And(sdcs.iter().map(lower_simple_duration_constraint).collect())
        }
    }
}

fn lower_simple_duration_constraint(sdc: &SimpleDurationConstraint) -> DurationConstraint {
    match sdc {
        SimpleDurationConstraint::Op(op, val) => {
            let expr = lower_duration_value(val);
            match op {
                DurationOperator::Equal => DurationConstraint::Eq(expr),
                DurationOperator::GreaterOrEqual => DurationConstraint::Gte(expr),
                DurationOperator::LessThanOrEqual => DurationConstraint::Lte(expr),
            }
        }
        SimpleDurationConstraint::At(_, inner) => {
            // Timed constraint — lower the inner, ignoring the time tag
            lower_simple_duration_constraint(inner)
        }
    }
}

fn lower_duration_value(val: &DurationValue) -> NumericExpr {
    match val {
        DurationValue::Number(n) => NumericExpr::Number(num_f64(*n)),
        DurationValue::FluentExpression(fe) => lower_fluent_expression(fe),
    }
}

fn lower_da_gd(gd: &DurativeActionGoalDefinition) -> Vec<PddlCondition> {
    match gd {
        DurativeActionGoalDefinition::Timed(pref_timed) => {
            vec![lower_pref_timed_gd(pref_timed)]
        }
        DurativeActionGoalDefinition::And(cs) => cs.iter().flat_map(lower_da_gd).collect(),
        DurativeActionGoalDefinition::Forall(vars, inner) => {
            let typed_vars = lower_typed_variables(vars);
            let body_parts = lower_da_gd(inner);
            let body = match body_parts.len() {
                0 => PddlCondition::And(vec![]),
                1 => body_parts.into_iter().next().unwrap(),
                _ => PddlCondition::And(body_parts),
            };
            vec![PddlCondition::Forall {
                vars: typed_vars,
                body: Box::new(body),
            }]
        }
    }
}

fn lower_pref_timed_gd(ptgd: &PreferenceTimedGoalDefinition) -> PddlCondition {
    match ptgd {
        PreferenceTimedGoalDefinition::Required(tgd) => lower_timed_gd(tgd),
        PreferenceTimedGoalDefinition::Preference(_, tgd) => lower_timed_gd(tgd),
    }
}

fn lower_timed_gd(tgd: &TimedGoalDefinition) -> PddlCondition {
    match tgd {
        TimedGoalDefinition::At(ts, gd) => {
            let ts_out = lower_time_specifier(ts);
            PddlCondition::Timed(ts_out, Box::new(lower_condition(gd)))
        }
        TimedGoalDefinition::Over(_interval, gd) => {
            // `over all` → OverAll
            PddlCondition::Timed(TimeSpecifier::OverAll, Box::new(lower_condition(gd)))
        }
    }
}

fn lower_time_specifier(ts: &pddl::TimeSpecifier) -> TimeSpecifier {
    match ts {
        pddl::TimeSpecifier::Start => TimeSpecifier::AtStart,
        pddl::TimeSpecifier::End => TimeSpecifier::AtEnd,
    }
}

fn lower_da_effect(effect: &DurativeActionEffect) -> Vec<PddlEffect> {
    match effect {
        DurativeActionEffect::Timed(te) => vec![lower_timed_effect(te)],
        DurativeActionEffect::All(effects) => effects.iter().flat_map(lower_da_effect).collect(),
        DurativeActionEffect::Forall(vars, inner) => {
            let typed_vars = lower_typed_variables(vars);
            let inner_effects = lower_da_effect(inner);
            vec![PddlEffect::Forall {
                vars: typed_vars,
                effects: inner_effects,
            }]
        }
        DurativeActionEffect::When(gd, te) => {
            let condition = lower_da_gd_condition(gd);
            let effects = vec![lower_timed_effect(te)];
            vec![PddlEffect::When { condition, effects }]
        }
    }
}

fn lower_da_gd_condition(gd: &DurativeActionGoalDefinition) -> PddlCondition {
    let parts = lower_da_gd(gd);
    match parts.len() {
        0 => PddlCondition::And(vec![]),
        1 => parts.into_iter().next().unwrap(),
        _ => PddlCondition::And(parts),
    }
}

/// The fabricated predicate name [`lower_timed_effect`]'s
/// `TimedEffect::ContinuousEffect` arm substitutes for a real continuous
/// numeric effect (e.g. `(increase fuel #t)` inside a `:durative-action`'s
/// `:effect`) — this crate has no representation for a continuous
/// (rate-based, over-duration) numeric change at all. The name is `pub(crate)`
/// rather than a private literal so
/// `capability::admit_planning_task`'s content scan
/// (`effect_uses_continuous_effect_sentinel`) can detect real usage of this
/// unsupported construct without re-deriving or duplicating the string —
/// see that function's doc comment for why a fingerprint scan, not a
/// declared-`:continuous-effects`-requirement check, is the right admission
/// gate here.
pub(crate) const CONTINUOUS_EFFECT_SENTINEL_PRED: &str = "_continuous_effect";

fn lower_timed_effect(te: &TimedEffect) -> PddlEffect {
    match te {
        TimedEffect::Conditional(ts, ec) => {
            let ts_out = lower_time_specifier(ts);
            let inner = lower_effect_condition_ec(ec);
            PddlEffect::Timed(ts_out, Box::new(inner))
        }
        TimedEffect::NumericFluent(ts, fa) => {
            let ts_out = lower_time_specifier(ts);
            let func = lower_function_head(fa.function());
            let expr = lower_da_fluent_expression(fa.function_expr());
            let numeric = lower_assign_op_numeric(fa.operation(), func, expr);
            PddlEffect::Timed(ts_out, Box::new(PddlEffect::Numeric(numeric)))
        }
        TimedEffect::ContinuousEffect(_, _, _) => {
            // This crate does not implement continuous effects (see
            // capability.rs's module doc comment). The fabricated atom
            // below is a *detectable sentinel*, not a real fact — it exists
            // so `capability::admit_planning_task` can refuse a domain that
            // actually uses this construct (see
            // `CONTINUOUS_EFFECT_SENTINEL_PRED`'s doc comment), closing the
            // "parsed but silently wrong" gap a bare drop would leave open.
            // It still reaches `GroundTemporalProblem`'s ground state as a
            // meaningless fact if a caller bypasses admission entirely
            // (calls `GroundTemporalProblem::build` directly on a domain
            // that was never run through `admit_planning_task`) — that
            // residual risk is the admission gate's job to close, not this
            // lowering function's.
            PddlEffect::Add(Pddl8Atom {
                pred: CONTINUOUS_EFFECT_SENTINEL_PRED.to_string(),
                args: vec![],
            })
        }
    }
}

fn lower_effect_condition_ec(ec: &pddl::EffectCondition) -> PddlEffect {
    let mut adds = Vec::new();
    let mut dels = Vec::new();
    let pes: Vec<&pddl::PrimitiveEffect> = match ec {
        pddl::EffectCondition::Single(pe) => vec![pe],
        pddl::EffectCondition::All(pes) => pes.iter().collect(),
    };
    for pe in pes {
        collect_primitive_effect(pe, &mut adds, &mut dels);
    }
    let mut effects: Vec<PddlEffect> = adds.into_iter().map(PddlEffect::Add).collect();
    effects.extend(dels.into_iter().map(PddlEffect::Del));
    match effects.len() {
        0 => PddlEffect::Add(Pddl8Atom {
            pred: "_empty".to_string(),
            args: vec![],
        }),
        1 => effects.remove(0),
        _ => PddlEffect::When {
            condition: PddlCondition::And(vec![]),
            effects,
        },
    }
}

fn lower_derived_predicate(dp: &pddl::DerivedPredicate) -> DerivedPredicate {
    let skel = dp.predicate();
    let head = Pddl8Atom {
        pred: skel.name().to_string(),
        args: skel
            .variables()
            .value()
            .iter()
            .map(|v| format!("?{}", v.value()))
            .collect(),
    };
    let body = lower_condition(dp.expression());
    DerivedPredicate { head, body }
}

fn lower_constraint_gd(cgd: &ConstraintGoalDefinition) -> Vec<PddlConstraint> {
    match cgd {
        ConstraintGoalDefinition::And(cs) => cs.iter().flat_map(lower_constraint_gd).collect(),
        ConstraintGoalDefinition::Forall(_, inner) => {
            // TODO: quantified domain constraints — lower inner for now
            lower_constraint_gd(inner)
        }
        other => {
            let tc = lower_trajectory_constraint(other);
            vec![PddlConstraint {
                name: None,
                constraint: tc,
            }]
        }
    }
}

fn lower_trajectory_constraint(cgd: &ConstraintGoalDefinition) -> TrajectoryConstraint {
    match cgd {
        ConstraintGoalDefinition::And(cs) => {
            TrajectoryConstraint::And(cs.iter().map(lower_trajectory_constraint).collect())
        }
        ConstraintGoalDefinition::Forall(_, inner) => {
            // TODO: proper forall lowering for trajectory constraints
            lower_trajectory_constraint(inner)
        }
        ConstraintGoalDefinition::AtEnd(gd) => {
            // at-end goal as Always constraint
            TrajectoryConstraint::Always(Box::new(lower_condition(gd)))
        }
        ConstraintGoalDefinition::Always(inner) => {
            TrajectoryConstraint::Always(Box::new(lower_con2gd_condition(inner)))
        }
        ConstraintGoalDefinition::Sometime(inner) => {
            TrajectoryConstraint::Sometime(Box::new(lower_con2gd_condition(inner)))
        }
        ConstraintGoalDefinition::Within(n, inner) => {
            TrajectoryConstraint::Within(num_f64(*n), Box::new(lower_con2gd_condition(inner)))
        }
        ConstraintGoalDefinition::AtMostOnce(inner) => {
            TrajectoryConstraint::AtMostOnce(Box::new(lower_con2gd_condition(inner)))
        }
        ConstraintGoalDefinition::SometimeAfter(a, b) => TrajectoryConstraint::SometimeAfter(
            Box::new(lower_con2gd_condition(a)),
            Box::new(lower_con2gd_condition(b)),
        ),
        ConstraintGoalDefinition::SometimeBefore(a, b) => TrajectoryConstraint::SometimeBefore(
            Box::new(lower_con2gd_condition(a)),
            Box::new(lower_con2gd_condition(b)),
        ),
        ConstraintGoalDefinition::AlwaysWithin(n, a, b) => TrajectoryConstraint::AlwaysWithin(
            num_f64(*n),
            Box::new(lower_con2gd_condition(a)),
            Box::new(lower_con2gd_condition(b)),
        ),
        ConstraintGoalDefinition::HoldDuring(n1, n2, inner) => TrajectoryConstraint::HoldDuring(
            num_f64(*n1),
            num_f64(*n2),
            Box::new(lower_con2gd_condition(inner)),
        ),
        ConstraintGoalDefinition::HoldAfter(n, inner) => {
            TrajectoryConstraint::HoldAfter(num_f64(*n), Box::new(lower_con2gd_condition(inner)))
        }
    }
}

fn lower_con2gd_condition(inner: &ConstraintGoalDefinitionInner) -> PddlCondition {
    match inner {
        ConstraintGoalDefinitionInner::Goal(gd) => lower_condition(gd),
        ConstraintGoalDefinitionInner::Nested(cgd) => {
            // Nested trajectory constraint embedded as condition — use And([]) placeholder
            // TODO: represent nested trajectory constraints properly
            let _ = cgd;
            PddlCondition::And(vec![])
        }
    }
}

fn lower_metric(metric: &pddl::MetricSpec) -> Metric {
    let dir = match metric.optimization() {
        Optimization::Minimize => MetricDir::Minimize,
        Optimization::Maximize => MetricDir::Maximize,
    };
    let expr = lower_metric_expr(metric.expression());
    Metric { dir, expr }
}

fn lower_metric_expr(expr: &MetricFluentExpression) -> MetricExpr {
    match expr {
        MetricFluentExpression::Number(n) => MetricExpr::Number(num_f64(*n)),
        MetricFluentExpression::TotalTime => MetricExpr::TotalTime,
        MetricFluentExpression::IsViolated(pref) => MetricExpr::IsViolated(pref.to_string()),
        MetricFluentExpression::Function(sym, names) => MetricExpr::FunctionTerm(
            sym.to_string(),
            names.iter().map(|n| n.to_string()).collect(),
        ),
        MetricFluentExpression::Negative(inner) => MetricExpr::BinOp {
            op: NumericOp::Sub,
            lhs: Box::new(MetricExpr::Number(0.0)),
            rhs: Box::new(lower_metric_expr(inner)),
        },
        MetricFluentExpression::BinaryOp(op, lhs, rhs) => MetricExpr::BinOp {
            op: lower_binary_op(op),
            lhs: Box::new(lower_metric_expr(lhs)),
            rhs: Box::new(lower_metric_expr(rhs)),
        },
        MetricFluentExpression::MultiOp(op, lhs, rhs) => {
            let nop = lower_multi_op(op);
            rhs.iter()
                .fold(lower_metric_expr(lhs), |acc, r| MetricExpr::BinOp {
                    op: nop,
                    lhs: Box::new(acc),
                    rhs: Box::new(lower_metric_expr(r)),
                })
        }
    }
}
