//! PDDL 3.1 text → wasm4pm_compat::pddl canonical types.

use crate::error::Pddl8Error;
use pddl::{ConditionalEffect, GoalDefinition, InitElement, Parser, PreferenceGoalDefinition,
    PredicateAtomicFormula, PreconditionGoalDefinition, PrimitiveEffect, StructureDef,
    parsers::Span};
use wasm4pm_compat::pddl::{
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem,
    PDDL8_MAX_ARITY, PDDL8_MAX_CONJUNCTS, PDDL8_MAX_PARAMS,
};

pub fn domain_from_pddl(text: &str) -> Result<Pddl8Domain, Pddl8Error> {
    let (_, dom) = pddl::Domain::parse(Span::new(text))
        .map_err(|e| Pddl8Error::ParseError(format!("{e:?}")))?;

    let name = dom.name().to_string();

    let predicates = dom.predicates().values().iter().map(|p| {
        let arity = p.variables().value().len();
        if arity > PDDL8_MAX_ARITY {
            Err(Pddl8Error::BoundExceeded { what: "predicate arity", limit: PDDL8_MAX_ARITY as u8, got: arity })
        } else {
            Ok((p.name().to_string(), arity as u8))
        }
    }).collect::<Result<Vec<_>, _>>()?;

    let actions = dom.structure().values().iter().filter_map(|sd| {
        if let StructureDef::Action(a) = sd { Some(a) } else { None }
    }).map(lower_action).collect::<Result<Vec<_>, _>>()?;

    Ok(Pddl8Domain { name, predicates, actions })
}

pub fn problem_from_pddl(text: &str) -> Result<Pddl8Problem, Pddl8Error> {
    let (_, prob) = pddl::Problem::parse(Span::new(text))
        .map_err(|e| Pddl8Error::ParseError(format!("{e:?}")))?;

    let objects: Vec<String> = prob.objects().values().value().iter()
        .map(|t| t.value().to_string())
        .collect();

    let init = lower_init(prob.init())?;
    let goal = lower_precond_defs(prob.goals())?;

    Ok(Pddl8Problem {
        name: prob.name().to_string(),
        domain: prob.domain().to_string(),
        objects,
        init,
        goal,
    })
}

fn lower_action(a: &pddl::ActionDefinition) -> Result<Pddl8ActionSchema, Pddl8Error> {
    let params: Vec<String> = a.parameters().value().iter()
        .map(|t| format!("?{}", t.value()))
        .collect();
    if params.len() > PDDL8_MAX_PARAMS {
        return Err(Pddl8Error::BoundExceeded { what: "action parameters", limit: PDDL8_MAX_PARAMS as u8, got: params.len() });
    }

    let preconditions = lower_precond_defs(a.precondition())?;
    if preconditions.len() > PDDL8_MAX_CONJUNCTS {
        return Err(Pddl8Error::BoundExceeded { what: "precondition atoms", limit: PDDL8_MAX_CONJUNCTS as u8, got: preconditions.len() });
    }

    let (add_effects, del_effects) = lower_effects(a.effect())?;

    Ok(Pddl8ActionSchema {
        name: a.symbol().to_string(),
        params,
        preconditions,
        add_effects,
        del_effects,
    })
}

fn lower_precond_defs(defs: &pddl::PreconditionGoalDefinitions) -> Result<Vec<Pddl8Atom>, Pddl8Error> {
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
            for d in inner.iter() { collect_precond_def(d, out); }
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
            if let Some(atom) = lower_af_term(af) { out.push(atom); }
        }
        GoalDefinition::And(cs) => {
            for c in cs { collect_gd(c, out); }
        }
        GoalDefinition::Literal(lit) => {
            use pddl::Literal;
            if let Literal::AtomicFormula(af) = lit {
                if let Some(atom) = lower_af_term(af) { out.push(atom); }
            }
        }
        _ => {}
    }
}

fn lower_effects(eff: &Option<pddl::Effects>) -> Result<(Vec<Pddl8Atom>, Vec<Pddl8Atom>), Pddl8Error> {
    let Some(effects) = eff else { return Ok((vec![], vec![])); };
    let mut adds = Vec::new();
    let mut dels = Vec::new();
    for ce in effects.iter() {
        collect_conditional_effect(ce, &mut adds, &mut dels);
    }
    Ok((adds, dels))
}

fn collect_conditional_effect(ce: &ConditionalEffect, adds: &mut Vec<Pddl8Atom>, dels: &mut Vec<Pddl8Atom>) {
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

fn collect_primitive_effect(pe: &PrimitiveEffect, adds: &mut Vec<Pddl8Atom>, dels: &mut Vec<Pddl8Atom>) {
    match pe {
        PrimitiveEffect::AtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) { adds.push(atom); }
        }
        PrimitiveEffect::NotAtomicFormula(af) => {
            if let Some(atom) = lower_af_term(af) { dels.push(atom); }
        }
        _ => {}
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
        args: p.values().iter().map(|t| match t {
            pddl::Term::Name(n) => n.to_string(),
            pddl::Term::Variable(v) => format!("?{v}"),
            pddl::Term::Function(_) => "_".to_string(),
        }).collect(),
    }
}

fn lower_init(init: &pddl::InitElements) -> Result<Vec<Pddl8Atom>, Pddl8Error> {
    let mut out = Vec::new();
    for el in init.iter() {
        if let InitElement::Literal(lit) = el {
            use pddl::Literal;
            if let Literal::AtomicFormula(af) = lit {
                if let pddl::AtomicFormula::Predicate(p) = af {
                    out.push(Pddl8Atom {
                        pred: p.predicate().to_string(),
                        args: p.values().iter().map(|n| n.to_string()).collect(),
                    });
                }
            }
        }
    }
    Ok(out)
}
