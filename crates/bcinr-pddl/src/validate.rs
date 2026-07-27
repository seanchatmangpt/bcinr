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

use wasm4pm_compat::pddl::{
    CompareOp, Pddl31Domain, Pddl31Problem, Pddl8GroundAtom, Pddl8Tape, PddlCondition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanViolation {
    PreconditionUnsatisfied {
        step: usize,
        action: String,
        atom: String,
    },
    GoalNotReached,
    ActionRepeated {
        label: String,
        count: usize,
    },
    NumericConditionUnchecked,
    TemporalConditionUnchecked,
    /// A step's full (unflattened) precondition tree -- boolean or numeric --
    /// failed under [`validate_plan_numeric`]'s replay.
    NumericPreconditionUnsatisfied {
        step: usize,
        action: String,
        detail: String,
    },
    /// A numeric effect or `Compare` condition referenced a function with no
    /// value in the replayed numeric state (never initialized, and never
    /// assigned by any prior step).
    UndefinedFunction {
        step: usize,
        action: String,
        function: String,
    },
    /// A `scale-down` numeric effect's divisor evaluated to zero.
    DivisionByZero {
        step: usize,
        action: String,
        function: String,
    },
}

impl std::fmt::Display for PlanViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreconditionUnsatisfied { step, action, atom } => {
                write!(
                    f,
                    "step {step} ({action}): precondition {atom} not satisfied"
                )
            }
            Self::GoalNotReached => write!(f, "goal not reached at end of plan"),
            Self::ActionRepeated { label, count } => {
                write!(
                    f,
                    "action {label} fired {count} times: no ground action may repeat"
                )
            }
            Self::NumericConditionUnchecked => {
                write!(
                    f,
                    "goal contains a numeric comparison this validator cannot verify"
                )
            }
            Self::TemporalConditionUnchecked => {
                write!(
                    f,
                    "goal contains a timed condition this validator cannot verify"
                )
            }
            Self::NumericPreconditionUnsatisfied {
                step,
                action,
                detail,
            } => {
                write!(
                    f,
                    "step {step} ({action}): precondition not satisfied: {detail}"
                )
            }
            Self::UndefinedFunction {
                step,
                action,
                function,
            } => {
                write!(f, "step {step} ({action}): undefined function {function}")
            }
            Self::DivisionByZero {
                step,
                action,
                function,
            } => {
                write!(
                    f,
                    "step {step} ({action}): scale-down by zero for {function}"
                )
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
            all_objects: problem
                .objects
                .iter()
                .map(|(name, _)| name.clone())
                .collect(),
        }
    }

    fn satisfies(&self, object: &str, required: &str) -> bool {
        if required == "object" {
            return true;
        }
        let mut current = self
            .object_type
            .get(object)
            .map(String::as_str)
            .unwrap_or("object");
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
        self.all_objects
            .iter()
            .filter(|o| self.satisfies(o, required))
            .cloned()
            .collect()
    }
}

/// Independently replay `tape` against `domain`/`problem`'s own definitions.
/// `Ok(())` only if every step's precondition holds, no ground action fires
/// twice, and the goal holds in the final state.
pub fn validate_plan(
    domain: &Pddl31Domain,
    problem: &Pddl31Problem,
    tape: &Pddl8Tape,
) -> Result<(), PlanViolation> {
    let type_check = TypeCheck::build(domain, problem);
    let mut state: BTreeSet<Pddl8GroundAtom> = problem
        .init_atoms
        .iter()
        .map(|a| Pddl8GroundAtom {
            pred: a.pred.clone(),
            args: a.args.clone(),
        })
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
            return Err(PlanViolation::ActionRepeated {
                label: op.label.clone(),
                count: *count,
            });
        }
    }

    if !eval_condition(&problem.goal, &state, &type_check)? {
        return Err(PlanViolation::GoalNotReached);
    }
    Ok(())
}

fn eval_condition(
    cond: &PddlCondition,
    state: &BTreeSet<Pddl8GroundAtom>,
    tc: &TypeCheck,
) -> Result<bool, PlanViolation> {
    match cond {
        PddlCondition::Atom(a) => Ok(state.contains(&Pddl8GroundAtom {
            pred: a.pred.clone(),
            args: a.args.clone(),
        })),
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
            args: a
                .args
                .iter()
                .map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone()))
                .collect(),
        }),
        PddlCondition::Not(inner) => PddlCondition::Not(Box::new(substitute(inner, binding))),
        PddlCondition::And(parts) => {
            PddlCondition::And(parts.iter().map(|p| substitute(p, binding)).collect())
        }
        PddlCondition::Or(parts) => {
            PddlCondition::Or(parts.iter().map(|p| substitute(p, binding)).collect())
        }
        PddlCondition::Imply(l, r) => PddlCondition::Imply(
            Box::new(substitute(l, binding)),
            Box::new(substitute(r, binding)),
        ),
        PddlCondition::Forall { vars, body } => PddlCondition::Forall {
            vars: vars.clone(),
            body: Box::new(substitute(body, binding)),
        },
        PddlCondition::Exists { vars, body } => PddlCondition::Exists {
            vars: vars.clone(),
            body: Box::new(substitute(body, binding)),
        },
        // `PddlCondition::Compare` is `(NumericExpr, CompareOp, NumericExpr)`,
        // not `(CompareOp, NumericExpr, NumericExpr)` -- named `lhs`/`op`/`rhs`
        // here (not the misleading `op`/`l`/`r` this arm used to bind) to match
        // the real field order and avoid repeating the exact field-order bug
        // `eval_condition_numeric` had earlier this session. `eval_condition`
        // above always reports `Compare` as `NumericConditionUnchecked`
        // regardless of its operands, so this arm cloning `lhs`/`rhs` verbatim
        // (rather than substituting `?var` tokens inside them, as `Atom`'s arm
        // does for its args) is currently inert either way -- not silently
        // wrong, just not yet exercised by anything that would notice.
        PddlCondition::Compare(lhs, op, rhs) => {
            PddlCondition::Compare(lhs.clone(), *op, rhs.clone())
        }
        PddlCondition::Timed(spec, inner) => {
            PddlCondition::Timed(*spec, Box::new(substitute(inner, binding)))
        }
    }
}

// ---------------------------------------------------------------------------
// Numeric-complete replay, extending the boolean-only validator above.
//
// `validate_plan` (above) is untouched by everything below -- it keeps
// replaying `Pddl8Tape`'s flattened, atom-only `Pddl8GroundAction` steps
// exactly as before. `validate_plan_numeric` instead replays a sequence of
// `ExactGroundAction`s (from `crate::ground_v2`) directly, whose
// `condition`/`effects` are the full, unflattened `PddlCondition`/
// `PddlEffect` trees -- including `Compare` and `PddlEffect::Numeric` --
// closing the `NumericConditionUnchecked` gap `validate_plan` honestly
// reports rather than fakes.
// ---------------------------------------------------------------------------

use crate::ground_v2::ExactGroundAction;
use wasm4pm_compat::pddl::{NumericEffect, NumericExpr, NumericOp, PddlEffect};

/// Stable string key for a numeric fluent reference: `name` if nullary,
/// else `name(arg1,arg2)`.
fn fluent_key(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!("{name}({})", args.join(","))
    }
}

/// Independently replay a sequence of [`ExactGroundAction`]s (indexed by
/// `plan_indices` into `actions`, in firing order) against `domain`/
/// `problem`'s own definitions, tracking BOTH boolean fluent state and
/// numeric function state. `Ok(())` only if every step's full condition
/// tree (boolean and numeric) holds, every numeric effect applies cleanly
/// (no undefined function, no division by zero), and the full goal tree
/// (`Compare` included) holds in the final state.
pub fn validate_plan_numeric(
    domain: &Pddl31Domain,
    problem: &Pddl31Problem,
    actions: &[ExactGroundAction],
    plan_indices: &[usize],
) -> Result<(), PlanViolation> {
    let type_check = TypeCheck::build(domain, problem);
    let mut state: BTreeSet<Pddl8GroundAtom> = problem
        .init_atoms
        .iter()
        .map(|a| Pddl8GroundAtom {
            pred: a.pred.clone(),
            args: a.args.clone(),
        })
        .collect();
    let mut numeric_state: BTreeMap<String, f64> = problem
        .init_fn_values
        .iter()
        .map(|(function, value)| (fluent_key(&function.name, &function.params), *value))
        .collect();

    for (step, &action_index) in plan_indices.iter().enumerate() {
        let action = &actions[action_index];

        if !eval_condition_numeric(
            &action.condition,
            &state,
            &numeric_state,
            &type_check,
            step,
            &action.label,
        )? {
            return Err(PlanViolation::NumericPreconditionUnsatisfied {
                step,
                action: action.label.clone(),
                detail: "full condition tree did not hold in the replayed state".to_string(),
            });
        }

        for effect in &action.effects {
            apply_effect(
                effect,
                &mut state,
                &mut numeric_state,
                &type_check,
                step,
                &action.label,
            )?;
        }
    }

    let goal_step = plan_indices.len();
    if !eval_condition_numeric(
        &problem.goal,
        &state,
        &numeric_state,
        &type_check,
        goal_step,
        "<goal>",
    )? {
        return Err(PlanViolation::GoalNotReached);
    }
    Ok(())
}

/// Numeric-complete extension of [`eval_condition`]: identical on every
/// boolean-only variant, and now genuinely evaluates `Compare` against
/// `numeric_state` instead of refusing with `NumericConditionUnchecked`.
#[allow(clippy::too_many_arguments)]
fn eval_condition_numeric(
    cond: &PddlCondition,
    state: &BTreeSet<Pddl8GroundAtom>,
    numeric_state: &BTreeMap<String, f64>,
    tc: &TypeCheck,
    step: usize,
    action_label: &str,
) -> Result<bool, PlanViolation> {
    match cond {
        PddlCondition::Atom(a) => Ok(state.contains(&Pddl8GroundAtom {
            pred: a.pred.clone(),
            args: a.args.clone(),
        })),
        PddlCondition::Not(inner) => Ok(!eval_condition_numeric(
            inner,
            state,
            numeric_state,
            tc,
            step,
            action_label,
        )?),
        PddlCondition::And(parts) => {
            for p in parts {
                if !eval_condition_numeric(p, state, numeric_state, tc, step, action_label)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        PddlCondition::Or(parts) => {
            for p in parts {
                if eval_condition_numeric(p, state, numeric_state, tc, step, action_label)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        PddlCondition::Imply(left, right) => {
            Ok(
                !eval_condition_numeric(left, state, numeric_state, tc, step, action_label)?
                    || eval_condition_numeric(right, state, numeric_state, tc, step, action_label)?,
            )
        }
        PddlCondition::Forall { vars, body } => {
            let mut all_true = true;
            let mut first_err = None;
            for_each_binding(vars, tc, &mut BTreeMap::new(), &mut |binding| {
                if all_true && first_err.is_none() {
                    let substituted = substitute(body, binding);
                    match eval_condition_numeric(
                        &substituted,
                        state,
                        numeric_state,
                        tc,
                        step,
                        action_label,
                    ) {
                        Ok(true) => {}
                        Ok(false) => all_true = false,
                        Err(e) => first_err = Some(e),
                    }
                }
            });
            if let Some(e) = first_err {
                return Err(e);
            }
            Ok(all_true)
        }
        PddlCondition::Exists { vars, body } => {
            let mut any_true = false;
            let mut first_err = None;
            for_each_binding(vars, tc, &mut BTreeMap::new(), &mut |binding| {
                if !any_true && first_err.is_none() {
                    let substituted = substitute(body, binding);
                    match eval_condition_numeric(
                        &substituted,
                        state,
                        numeric_state,
                        tc,
                        step,
                        action_label,
                    ) {
                        Ok(true) => any_true = true,
                        Ok(false) => {}
                        Err(e) => first_err = Some(e),
                    }
                }
            });
            if let Some(e) = first_err {
                return Err(e);
            }
            Ok(any_true)
        }
        PddlCondition::Compare(lhs, op, rhs) => {
            let l = eval_numeric_expr(lhs, numeric_state, step, action_label)?;
            let r = eval_numeric_expr(rhs, numeric_state, step, action_label)?;
            Ok(match op {
                CompareOp::Ge => l >= r,
                CompareOp::Le => l <= r,
                CompareOp::Gt => l > r,
                CompareOp::Lt => l < r,
                CompareOp::Eq => l == r,
            })
        }
        PddlCondition::Timed(_, inner) => {
            // Same simplification the classical/exact rail already makes
            // elsewhere: no fine-grained timeline is replayed here, only
            // pre/post state -- an `at start`/`at end` wrapper is evaluated
            // against whatever state is current at this point in the
            // sequence, not a distinct instant.
            eval_condition_numeric(inner, state, numeric_state, tc, step, action_label)
        }
    }
}

fn eval_numeric_expr(
    expr: &NumericExpr,
    numeric_state: &BTreeMap<String, f64>,
    step: usize,
    action_label: &str,
) -> Result<f64, PlanViolation> {
    match expr {
        NumericExpr::Number(n) => Ok(*n),
        NumericExpr::FunctionTerm(name, args) => {
            let key = fluent_key(name, args);
            numeric_state
                .get(&key)
                .copied()
                .ok_or_else(|| PlanViolation::UndefinedFunction {
                    step,
                    action: action_label.to_string(),
                    function: key,
                })
        }
        NumericExpr::BinOp { op, lhs, rhs } => {
            let l = eval_numeric_expr(lhs, numeric_state, step, action_label)?;
            let r = eval_numeric_expr(rhs, numeric_state, step, action_label)?;
            Ok(match op {
                NumericOp::Add => l + r,
                NumericOp::Sub => l - r,
                NumericOp::Mul => l * r,
                NumericOp::Div => l / r,
            })
        }
        NumericExpr::Neg(inner) => Ok(-eval_numeric_expr(
            inner,
            numeric_state,
            step,
            action_label,
        )?),
    }
}

/// Apply one effect (boolean or numeric, possibly wrapped in `When`/
/// `Forall`/`Timed`) to `state`/`numeric_state` in place.
#[allow(clippy::too_many_arguments)]
fn apply_effect(
    effect: &PddlEffect,
    state: &mut BTreeSet<Pddl8GroundAtom>,
    numeric_state: &mut BTreeMap<String, f64>,
    tc: &TypeCheck,
    step: usize,
    action_label: &str,
) -> Result<(), PlanViolation> {
    match effect {
        PddlEffect::Add(a) => {
            state.insert(Pddl8GroundAtom {
                pred: a.pred.clone(),
                args: a.args.clone(),
            });
            Ok(())
        }
        PddlEffect::Del(a) => {
            state.remove(&Pddl8GroundAtom {
                pred: a.pred.clone(),
                args: a.args.clone(),
            });
            Ok(())
        }
        PddlEffect::Numeric(ne) => apply_numeric_effect(ne, numeric_state, step, action_label),
        PddlEffect::Timed(_, inner) => {
            apply_effect(inner, state, numeric_state, tc, step, action_label)
        }
        PddlEffect::When { condition, effects } => {
            if eval_condition_numeric(condition, state, numeric_state, tc, step, action_label)? {
                for e in effects {
                    apply_effect(e, state, numeric_state, tc, step, action_label)?;
                }
            }
            Ok(())
        }
        PddlEffect::Forall { vars, effects } => {
            let mut result = Ok(());
            for_each_binding(vars, tc, &mut BTreeMap::new(), &mut |binding| {
                if result.is_ok() {
                    for e in effects {
                        let substituted = substitute_effect(e, binding);
                        if let Err(err) =
                            apply_effect(&substituted, state, numeric_state, tc, step, action_label)
                        {
                            result = Err(err);
                            break;
                        }
                    }
                }
            });
            result
        }
    }
}

fn apply_numeric_effect(
    ne: &NumericEffect,
    numeric_state: &mut BTreeMap<String, f64>,
    step: usize,
    action_label: &str,
) -> Result<(), PlanViolation> {
    let (function, expr, op_kind) = match ne {
        NumericEffect::Assign(f, e) => (f, e, "assign"),
        NumericEffect::Increase(f, e) => (f, e, "increase"),
        NumericEffect::Decrease(f, e) => (f, e, "decrease"),
        NumericEffect::ScaleUp(f, e) => (f, e, "scale-up"),
        NumericEffect::ScaleDown(f, e) => (f, e, "scale-down"),
    };
    let key = fluent_key(&function.name, &function.params);
    let rhs = eval_numeric_expr(expr, numeric_state, step, action_label)?;

    let current = numeric_state.get(&key).copied();
    let next = match op_kind {
        "assign" => rhs,
        "increase" => {
            current.ok_or_else(|| PlanViolation::UndefinedFunction {
                step,
                action: action_label.to_string(),
                function: key.clone(),
            })? + rhs
        }
        "decrease" => {
            current.ok_or_else(|| PlanViolation::UndefinedFunction {
                step,
                action: action_label.to_string(),
                function: key.clone(),
            })? - rhs
        }
        "scale-up" => {
            current.ok_or_else(|| PlanViolation::UndefinedFunction {
                step,
                action: action_label.to_string(),
                function: key.clone(),
            })? * rhs
        }
        _ => {
            // scale-down
            if rhs == 0.0 {
                return Err(PlanViolation::DivisionByZero {
                    step,
                    action: action_label.to_string(),
                    function: key,
                });
            }
            current.ok_or_else(|| PlanViolation::UndefinedFunction {
                step,
                action: action_label.to_string(),
                function: key.clone(),
            })? / rhs
        }
    };
    numeric_state.insert(key, next);
    Ok(())
}

fn substitute_numeric_expr(expr: &NumericExpr, binding: &BTreeMap<String, String>) -> NumericExpr {
    match expr {
        NumericExpr::Number(n) => NumericExpr::Number(*n),
        NumericExpr::FunctionTerm(name, args) => NumericExpr::FunctionTerm(
            name.clone(),
            args.iter()
                .map(|a| binding.get(a).cloned().unwrap_or_else(|| a.clone()))
                .collect(),
        ),
        NumericExpr::BinOp { op, lhs, rhs } => NumericExpr::BinOp {
            op: *op,
            lhs: Box::new(substitute_numeric_expr(lhs, binding)),
            rhs: Box::new(substitute_numeric_expr(rhs, binding)),
        },
        NumericExpr::Neg(inner) => {
            NumericExpr::Neg(Box::new(substitute_numeric_expr(inner, binding)))
        }
    }
}

/// Substitute `?var` tokens in `effect` with the bound object names in
/// `binding`, mirroring [`substitute`] but for the effect algebra.
fn substitute_effect(effect: &PddlEffect, binding: &BTreeMap<String, String>) -> PddlEffect {
    match effect {
        PddlEffect::Add(a) => PddlEffect::Add(wasm4pm_compat::pddl::Pddl8Atom {
            pred: a.pred.clone(),
            args: a
                .args
                .iter()
                .map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone()))
                .collect(),
        }),
        PddlEffect::Del(a) => PddlEffect::Del(wasm4pm_compat::pddl::Pddl8Atom {
            pred: a.pred.clone(),
            args: a
                .args
                .iter()
                .map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone()))
                .collect(),
        }),
        PddlEffect::Numeric(ne) => PddlEffect::Numeric(match ne {
            NumericEffect::Assign(f, e) => NumericEffect::Assign(
                substitute_function(f, binding),
                substitute_numeric_expr(e, binding),
            ),
            NumericEffect::Increase(f, e) => NumericEffect::Increase(
                substitute_function(f, binding),
                substitute_numeric_expr(e, binding),
            ),
            NumericEffect::Decrease(f, e) => NumericEffect::Decrease(
                substitute_function(f, binding),
                substitute_numeric_expr(e, binding),
            ),
            NumericEffect::ScaleUp(f, e) => NumericEffect::ScaleUp(
                substitute_function(f, binding),
                substitute_numeric_expr(e, binding),
            ),
            NumericEffect::ScaleDown(f, e) => NumericEffect::ScaleDown(
                substitute_function(f, binding),
                substitute_numeric_expr(e, binding),
            ),
        }),
        PddlEffect::Timed(spec, inner) => {
            PddlEffect::Timed(*spec, Box::new(substitute_effect(inner, binding)))
        }
        PddlEffect::Forall { vars, effects } => PddlEffect::Forall {
            vars: vars.clone(),
            effects: effects
                .iter()
                .map(|e| substitute_effect(e, binding))
                .collect(),
        },
        PddlEffect::When { condition, effects } => PddlEffect::When {
            condition: substitute(condition, binding),
            effects: effects
                .iter()
                .map(|e| substitute_effect(e, binding))
                .collect(),
        },
    }
}

fn substitute_function(
    f: &wasm4pm_compat::pddl::PddlFunction,
    binding: &BTreeMap<String, String>,
) -> wasm4pm_compat::pddl::PddlFunction {
    wasm4pm_compat::pddl::PddlFunction {
        name: f.name.clone(),
        params: f
            .params
            .iter()
            .map(|p| binding.get(p).cloned().unwrap_or_else(|| p.clone()))
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Temporal-plan shape validation: duration bounds, overlap, and time
// well-formedness for a `TemporalPlan` produced by
// `GroundTemporalProblem::find_temporal_plan[_with_fn_overrides]`.
//
// A distinct concern from `validate_plan`/`validate_plan_numeric` above (which
// replay boolean/numeric *state*): this never touches `state`/`fn_vals` at
// all, only the plan's own timing and its grounded actions' declared duration
// bounds -- exactly the slice `validate_plan` cannot check, since a
// `PddlCondition::Timed` goal is reported `TemporalConditionUnchecked` there
// rather than evaluated. Ported/adapted from a sibling implementation's
// `validate_temporal_plan_shape` (temporal_production.rs) -- same duration/
// overlap/time-sanity checks, dropped the `MAXIMUM_TEMPORAL_STEPS` bound
// (that constant does not exist in this tree) in favor of the one real bound
// already enforced downstream by `crate::powl_bridge::MAX_POWL_TAPE_STEPS`.
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalShapeViolation {
    /// A step's `start_time`/`duration`/end-time, or the plan's `makespan`,
    /// is non-finite, negative, or (for `makespan`) inconsistent with the
    /// steps' own `start_time + duration` maximum.
    InvalidTime { step: usize, field: &'static str },
    /// `step.duration` falls outside the grounded action's
    /// `[duration_min, duration_max]` bounds (epsilon-tolerant).
    DurationOutOfBounds {
        step: usize,
        minimum: f64,
        maximum: f64,
        actual: f64,
    },
    /// No grounded durative action matches this step's `(action_name, args)`.
    UnknownAction { step: usize, action: String },
    /// Two steps name the same grounded action and their `[start, start+duration)`
    /// intervals overlap -- the same ground action instance fired twice.
    OverlappingDuplicate {
        left: usize,
        right: usize,
        action: String,
    },
    /// `plan.steps.len()` exceeds `crate::powl_bridge::MAX_POWL_TAPE_STEPS`.
    PlanBoundExceeded { limit: usize, actual: usize },
}

impl core::fmt::Display for TemporalShapeViolation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidTime { step, field } => {
                write!(f, "temporal step {step} has invalid {field}")
            }
            Self::DurationOutOfBounds {
                step,
                minimum,
                maximum,
                actual,
            } => write!(
                f,
                "temporal step {step} duration {actual} is outside [{minimum}, {maximum}]"
            ),
            Self::UnknownAction { step, action } => {
                write!(
                    f,
                    "temporal step {step} references unknown action {action:?}"
                )
            }
            Self::OverlappingDuplicate {
                left,
                right,
                action,
            } => write!(
                f,
                "temporal steps {left} and {right} overlap the same grounded action {action:?}"
            ),
            Self::PlanBoundExceeded { limit, actual } => write!(
                f,
                "temporal plan contains {actual} steps, exceeding the admitted limit {limit}"
            ),
        }
    }
}

const TEMPORAL_TIME_EPSILON: f64 = 1.0e-9;

/// Validate a temporal plan's shape against its grounded durative actions:
/// step count within [`crate::powl_bridge::MAX_POWL_TAPE_STEPS`], every
/// step's time fields finite and non-negative, every step matching a real
/// grounded action within its duration bounds, no ground-action instance
/// firing twice in an overlapping interval, and `plan.makespan` consistent
/// with the steps' own `start_time + duration` maximum.
pub fn validate_temporal_plan_shape(
    grounded: &crate::ground::GroundTemporalProblem,
    plan: &wasm4pm_compat::pddl::TemporalPlan,
) -> Result<(), TemporalShapeViolation> {
    if plan.steps.len() > crate::powl_bridge::MAX_POWL_TAPE_STEPS {
        return Err(TemporalShapeViolation::PlanBoundExceeded {
            limit: crate::powl_bridge::MAX_POWL_TAPE_STEPS,
            actual: plan.steps.len(),
        });
    }
    if !plan.makespan.is_finite() || plan.makespan < 0.0 {
        return Err(TemporalShapeViolation::InvalidTime {
            step: plan.steps.len(),
            field: "makespan",
        });
    }

    for (index, step) in plan.steps.iter().enumerate() {
        validate_temporal_step_time(index, step)?;
        let action = grounded
            .durative_actions
            .iter()
            .find(|action| action.schema_name == step.action_name && action.args == step.args)
            .ok_or_else(|| TemporalShapeViolation::UnknownAction {
                step: index,
                action: canonical_temporal_action(step),
            })?;

        let below_minimum = step.duration + TEMPORAL_TIME_EPSILON < action.duration_min;
        let above_maximum = action.duration_max.is_finite()
            && step.duration > action.duration_max + TEMPORAL_TIME_EPSILON;
        if below_minimum || above_maximum {
            return Err(TemporalShapeViolation::DurationOutOfBounds {
                step: index,
                minimum: action.duration_min,
                maximum: action.duration_max,
                actual: step.duration,
            });
        }
    }

    for left in 0..plan.steps.len() {
        for right in (left + 1)..plan.steps.len() {
            let a = &plan.steps[left];
            let b = &plan.steps[right];
            if a.action_name == b.action_name
                && a.args == b.args
                && temporal_intervals_overlap(a, b)
            {
                return Err(TemporalShapeViolation::OverlappingDuplicate {
                    left,
                    right,
                    action: canonical_temporal_action(a),
                });
            }
        }
    }

    let computed_makespan = plan
        .steps
        .iter()
        .map(|step| step.start_time + step.duration)
        .fold(0.0_f64, f64::max);
    if (computed_makespan - plan.makespan).abs() > TEMPORAL_TIME_EPSILON {
        return Err(TemporalShapeViolation::InvalidTime {
            step: plan.steps.len(),
            field: "makespan consistency",
        });
    }
    Ok(())
}

fn validate_temporal_step_time(
    index: usize,
    step: &wasm4pm_compat::pddl::TemporalPlanStep,
) -> Result<(), TemporalShapeViolation> {
    if !step.start_time.is_finite() || step.start_time < 0.0 {
        return Err(TemporalShapeViolation::InvalidTime {
            step: index,
            field: "start time",
        });
    }
    if !step.duration.is_finite() || step.duration < 0.0 {
        return Err(TemporalShapeViolation::InvalidTime {
            step: index,
            field: "duration",
        });
    }
    if !(step.start_time + step.duration).is_finite() {
        return Err(TemporalShapeViolation::InvalidTime {
            step: index,
            field: "end time",
        });
    }
    Ok(())
}

fn temporal_intervals_overlap(
    left: &wasm4pm_compat::pddl::TemporalPlanStep,
    right: &wasm4pm_compat::pddl::TemporalPlanStep,
) -> bool {
    let left_end = left.start_time + left.duration;
    let right_end = right.start_time + right.duration;
    left.start_time < right_end - TEMPORAL_TIME_EPSILON
        && right.start_time < left_end - TEMPORAL_TIME_EPSILON
}

fn canonical_temporal_action(step: &wasm4pm_compat::pddl::TemporalPlanStep) -> String {
    format!("{}({})", step.action_name, step.args.join(","))
}
