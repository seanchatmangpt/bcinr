//! From-scratch, structurally independent PDDL plan validator.
//!
//! Deliberately shares no code with `ground_v2.rs`'s `eval_condition` /
//! `apply_action` / `TypeIndex` -- a validator that calls into the solver it
//! is checking proves nothing about that solver's own bugs. Everything here
//! (type satisfaction, condition evaluation, quantifier enumeration, effect
//! application) is re-derived from `Pddl31Domain`/`Pddl31Problem`'s public
//! fields and the ground actions already embedded in a `Pddl8Tape`.
//!
//! Scope: boolean fluents only. `Pddl8GroundAction`'s precondition/effect
//! lists are atom-only by the time they reach a tape op, so a validator
//! built from tape replay alone cannot see numeric state regardless of
//! effort spent here. A goal-level `Compare` is reported as
//! `NumericConditionUnchecked`, not silently assumed true.

use std::collections::{BTreeMap, BTreeSet};

use wasm4pm_compat::pddl::{Pddl31Domain, Pddl31Problem, Pddl8GroundAtom, Pddl8Tape, PddlCondition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanViolation {
    PreconditionUnsatisfied { step: usize, action: String, atom: String },
    GoalNotReached,
    ActionRepeated { label: String, count: usize },
    NumericConditionUnchecked,
    TemporalConditionUnchecked,
}

impl std::fmt::Display for PlanViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreconditionUnsatisfied { step, action, atom } => {
                write!(f, "step {step} ({action}): precondition {atom} not satisfied")
            }
            Self::GoalNotReached => write!(f, "goal not reached at end of plan"),
            Self::ActionRepeated { label, count } => {
                write!(f, "action {label} fired {count} times: no ground action may repeat")
            }
            Self::NumericConditionUnchecked => {
                write!(f, "goal contains a numeric comparison this validator cannot verify")
            }
            Self::TemporalConditionUnchecked => {
                write!(f, "goal contains a timed condition this validator cannot verify")
            }
        }
    }
}

impl std::error::Error for PlanViolation {}

/// Type-satisfaction index, built fresh from `Pddl31Domain`/`Pddl31Problem`'s
/// public fields -- deliberately not `ground_v2::TypeIndex`.
struct TypeCheck {
    object_type: BTreeMap<String, String>,
    parent: BTreeMap<String, String>,
    all_objects: Vec<String>,
}

impl TypeCheck {
    fn build(domain: &Pddl31Domain, problem: &Pddl31Problem) -> Self {
        Self {
            object_type: problem.objects.iter().cloned().collect(),
            parent: domain
                .types
                .iter()
                .filter_map(|t| t.parent.clone().map(|p| (t.name.clone(), p)))
                .collect(),
            all_objects: problem.objects.iter().map(|(name, _)| name.clone()).collect(),
        }
    }

    fn satisfies(&self, object: &str, required: &str) -> bool {
        if required == "object" {
            return true;
        }
        let mut current = self.object_type.get(object).map(String::as_str).unwrap_or("object");
        loop {
            if current == required {
                return true;
            }
            match self.parent.get(current) {
                Some(p) => current = p,
                None => return false,
            }
        }
    }

    fn candidates(&self, required: &str) -> Vec<String> {
        self.all_objects.iter().filter(|o| self.satisfies(o, required)).cloned().collect()
    }
}

/// Independently replay `tape` against `domain`/`problem`'s own definitions.
/// `Ok(())` only if every step's precondition holds, no ground action fires
/// twice, and the goal holds in the final state.
pub fn validate_plan(domain: &Pddl31Domain, problem: &Pddl31Problem, tape: &Pddl8Tape) -> Result<(), PlanViolation> {
    let type_check = TypeCheck::build(domain, problem);
    let mut state: BTreeSet<Pddl8GroundAtom> = problem
        .init_atoms
        .iter()
        .map(|a| Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
        .collect();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();

    for (step, op) in tape.ops.iter().enumerate() {
        let action = &op.action;
        for pre in &action.preconditions {
            if !state.contains(pre) {
                return Err(PlanViolation::PreconditionUnsatisfied {
                    step,
                    action: op.label.clone(),
                    atom: pre.label(),
                });
            }
        }
        for d in &action.del_effects {
            state.remove(d);
        }
        for a in &action.add_effects {
            state.insert(a.clone());
        }

        let count = seen.entry(op.label.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            return Err(PlanViolation::ActionRepeated { label: op.label.clone(), count: *count });
        }
    }

    if !eval_condition(&problem.goal, &state, &type_check)? {
        return Err(PlanViolation::GoalNotReached);
    }
    Ok(())
}

fn eval_condition(cond: &PddlCondition, state: &BTreeSet<Pddl8GroundAtom>, tc: &TypeCheck) -> Result<bool, PlanViolation> {
    match cond {
        PddlCondition::Atom(a) => {
            Ok(state.contains(&Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() }))
        }
        PddlCondition::Not(inner) => Ok(!eval_condition(inner, state, tc)?),
        PddlCondition::And(parts) => {
            for p in parts {
                if !eval_condition(p, state, tc)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        PddlCondition::Or(parts) => {
            for p in parts {
                if eval_condition(p, state, tc)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        PddlCondition::Imply(left, right) => {
            Ok(!eval_condition(left, state, tc)? || eval_condition(right, state, tc)?)
        }
        PddlCondition::Forall { vars, body } => {
            let mut all_true = true;
            for_each_binding(vars, tc, &mut BTreeMap::new(), &mut |binding| {
                if all_true {
                    let substituted = substitute(body, binding);
                    if !matches!(eval_condition(&substituted, state, tc), Ok(true)) {
                        all_true = false;
                    }
                }
            });
            Ok(all_true)
        }
        PddlCondition::Exists { vars, body } => {
            let mut any_true = false;
            for_each_binding(vars, tc, &mut BTreeMap::new(), &mut |binding| {
                if !any_true {
                    let substituted = substitute(body, binding);
                    if matches!(eval_condition(&substituted, state, tc), Ok(true)) {
                        any_true = true;
                    }
                }
            });
            Ok(any_true)
        }
        PddlCondition::Compare(_, _, _) => Err(PlanViolation::NumericConditionUnchecked),
        PddlCondition::Timed(_, _) => Err(PlanViolation::TemporalConditionUnchecked),
    }
}

fn for_each_binding(
    vars: &[(String, String)],
    tc: &TypeCheck,
    binding: &mut BTreeMap<String, String>,
    f: &mut impl FnMut(&BTreeMap<String, String>),
) {
    for_each_binding_rec(vars, 0, tc, binding, f);
}

fn for_each_binding_rec(
    vars: &[(String, String)],
    index: usize,
    tc: &TypeCheck,
    binding: &mut BTreeMap<String, String>,
    f: &mut impl FnMut(&BTreeMap<String, String>),
) {
    if index == vars.len() {
        f(binding);
        return;
    }
    let (name, required_type) = &vars[index];
    for obj in tc.candidates(required_type) {
        binding.insert(name.clone(), obj);
        for_each_binding_rec(vars, index + 1, tc, binding, f);
        binding.remove(name);
    }
}

/// Substitute `?var` tokens in `cond` with the bound object names in
/// `binding`, leaving unbound variables (from an outer quantifier) as-is.
fn substitute(cond: &PddlCondition, binding: &BTreeMap<String, String>) -> PddlCondition {
    match cond {
        PddlCondition::Atom(a) => PddlCondition::Atom(wasm4pm_compat::pddl::Pddl8Atom {
            pred: a.pred.clone(),
            args: a.args.iter().map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone())).collect(),
        }),
        PddlCondition::Not(inner) => PddlCondition::Not(Box::new(substitute(inner, binding))),
        PddlCondition::And(parts) => PddlCondition::And(parts.iter().map(|p| substitute(p, binding)).collect()),
        PddlCondition::Or(parts) => PddlCondition::Or(parts.iter().map(|p| substitute(p, binding)).collect()),
        PddlCondition::Imply(l, r) => {
            PddlCondition::Imply(Box::new(substitute(l, binding)), Box::new(substitute(r, binding)))
        }
        PddlCondition::Forall { vars, body } => {
            PddlCondition::Forall { vars: vars.clone(), body: Box::new(substitute(body, binding)) }
        }
        PddlCondition::Exists { vars, body } => {
            PddlCondition::Exists { vars: vars.clone(), body: Box::new(substitute(body, binding)) }
        }
        PddlCondition::Compare(op, l, r) => PddlCondition::Compare(op.clone(), l.clone(), r.clone()),
        PddlCondition::Timed(spec, inner) => PddlCondition::Timed(spec.clone(), Box::new(substitute(inner, binding))),
    }
}
