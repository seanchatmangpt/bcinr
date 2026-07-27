//! Real, non-vacuous numeric-effect independence check for
//! [`ExactGroundAction`] pairs.
//!
//! `PddlCausalAnalyzer` (see `crate::causal`) sets `numeric_flow` vacuously
//! `true` for every independent pair because it operates on the legacy
//! `wasm4pm_compat::pddl::Pddl8GroundAction`, which has no numeric-fluent
//! slot at all. `ExactGroundAction` (`crate::ground_v2`) is the ADL-capable
//! ground-action type whose `effects: Vec<PddlEffect>` carries
//! `PddlEffect::Numeric(NumericEffect)` entries, so it has everything needed
//! to check whether two actions' numeric effects genuinely commute.
//!
//! This module is standalone: it does not call any `crate::ground_v2`
//! function, only reads its public types, and is not wired into
//! `PddlCausalAnalyzer::analyze` (that would require threading
//! `ExactGroundAction` data through `GroundedPlanningEpoch`, which only
//! carries `Pddl8GroundAction` — a larger refactor out of scope here).
//!
//! Per this crate's stated safety direction (see `crate::causal`'s module
//! doc comment): under-approximating independence is safe, over-approximating
//! is not. Every decision below defaults to "does not commute" whenever the
//! two actions' numeric effects are not provably order-independent.

use std::collections::{BTreeMap, BTreeSet};

use bcinr_mfw_ir::{Digest, FluentId, NumericFlowWitness};
use wasm4pm_compat::pddl::{NumericEffect, NumericExpr, PddlEffect};

use crate::ground_v2::ExactGroundAction;

/// Which of the five numeric-effect operators a [`NumericEffect`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericOpKind {
    Assign,
    Increase,
    Decrease,
    ScaleUp,
    ScaleDown,
}

fn numeric_op_kind(effect: &NumericEffect) -> NumericOpKind {
    match effect {
        NumericEffect::Assign(_, _) => NumericOpKind::Assign,
        NumericEffect::Increase(_, _) => NumericOpKind::Increase,
        NumericEffect::Decrease(_, _) => NumericOpKind::Decrease,
        NumericEffect::ScaleUp(_, _) => NumericOpKind::ScaleUp,
        NumericEffect::ScaleDown(_, _) => NumericOpKind::ScaleDown,
    }
}

/// The bare function symbol a numeric effect targets (params dropped
/// deliberately — see [`touched_functions`]'s doc comment for why).
fn numeric_function_name(effect: &NumericEffect) -> &str {
    match effect {
        NumericEffect::Assign(function, _)
        | NumericEffect::Increase(function, _)
        | NumericEffect::Decrease(function, _)
        | NumericEffect::ScaleUp(function, _)
        | NumericEffect::ScaleDown(function, _) => function.name.as_str(),
    }
}

fn numeric_expr(effect: &NumericEffect) -> &NumericExpr {
    match effect {
        NumericEffect::Assign(_, expr)
        | NumericEffect::Increase(_, expr)
        | NumericEffect::Decrease(_, expr)
        | NumericEffect::ScaleUp(_, expr)
        | NumericEffect::ScaleDown(_, expr) => expr,
    }
}

/// Own independent walk of the `PddlEffect` tree (mirrors the shape
/// `ground_v2::validate_effect` and `ground_v2::collect_effect` recurse
/// over -- `When`/`Forall`/`Timed` wrappers pass through, `Numeric` is the
/// leaf of interest -- but calls neither of those functions, or any other
/// `ground_v2::` function: this module shares no code with the solver's
/// effect-application logic).
fn walk_numeric_effects<'a>(
    effect: &'a PddlEffect,
    out: &mut BTreeMap<String, Vec<&'a NumericEffect>>,
) {
    match effect {
        PddlEffect::Numeric(numeric) => {
            out.entry(numeric_function_name(numeric).to_string())
                .or_default()
                .push(numeric);
        }
        PddlEffect::When { effects, .. } => {
            for nested in effects {
                walk_numeric_effects(nested, out);
            }
        }
        PddlEffect::Forall { effects, .. } => {
            for nested in effects {
                walk_numeric_effects(nested, out);
            }
        }
        PddlEffect::Timed(_, inner) => walk_numeric_effects(inner, out),
        PddlEffect::Add(_) | PddlEffect::Del(_) => {}
    }
}

fn numeric_effects_by_function(action: &ExactGroundAction) -> BTreeMap<String, Vec<&NumericEffect>> {
    let mut out = BTreeMap::new();
    for effect in &action.effects {
        walk_numeric_effects(effect, &mut out);
    }
    out
}

/// Every function name touched by a `PddlEffect::Numeric` entry anywhere in
/// `action.effects`, recursing into `When`/`Forall`/`Timed` wrappers.
///
/// Deliberately keyed by bare function *name* (`PddlFunction::name`), not
/// the full grounded `name(arg,arg)` key: collapsing distinct argument
/// bindings of the same function symbol into one key is the conservative
/// direction (it can only make two actions look like they share a touched
/// fluent when they do not, never the reverse), matching this module's
/// stated under-approximate-independence discipline.
pub fn touched_functions(action: &ExactGroundAction) -> BTreeSet<String> {
    numeric_effects_by_function(action).into_keys().collect()
}

/// Does `expr` read `function` anywhere in its tree? Used to detect
/// self-referencing increase/decrease amounts (e.g. `(increase (f) (f))`),
/// which are not order-independent.
fn expr_references_function(expr: &NumericExpr, function: &str) -> bool {
    match expr {
        NumericExpr::Number(_) => false,
        NumericExpr::FunctionTerm(name, _) => name == function,
        NumericExpr::BinOp { lhs, rhs, .. } => {
            expr_references_function(lhs, function) || expr_references_function(rhs, function)
        }
        NumericExpr::Neg(inner) => expr_references_function(inner, function),
    }
}

/// Decide, for a single shared function, whether one numeric effect from
/// each action commutes. See this module's doc comment: default is `false`.
fn pairwise_numeric_commute(function: &str, a: &NumericEffect, b: &NumericEffect) -> bool {
    let op_a = numeric_op_kind(a);
    let op_b = numeric_op_kind(b);

    // Rule: an assign effect touching a function any other effect also
    // touches does NOT commute (assign overwrites; the last write wins, so
    // order matters -- including assign paired with another assign).
    if op_a == NumericOpKind::Assign || op_b == NumericOpKind::Assign {
        return false;
    }

    let is_scale = |op: NumericOpKind| matches!(op, NumericOpKind::ScaleUp | NumericOpKind::ScaleDown);
    let is_incdec = |op: NumericOpKind| matches!(op, NumericOpKind::Increase | NumericOpKind::Decrease);

    // Rule: a scale-up/scale-down paired with an increase/decrease on the
    // same function does NOT commute (multiplication and addition do not
    // commute with each other in general).
    if (is_scale(op_a) && is_incdec(op_b)) || (is_incdec(op_a) && is_scale(op_b)) {
        return false;
    }

    // Rule: two increase (or two decrease) effects on the same function
    // commute if both expressions are order-independent -- i.e. neither
    // reads the very function both are writing, so the combined update is
    // old (+/-) expr_a (+/-) expr_b regardless of which fires first.
    if op_a == op_b && is_incdec(op_a) {
        return !expr_references_function(numeric_expr(a), function)
            && !expr_references_function(numeric_expr(b), function);
    }

    // Everything else (increase/decrease mixed pairs, two scale-ups, two
    // scale-downs, a scale-up paired with a scale-down) is not covered by
    // an explicit commute rule above -- conservative default: false.
    false
}

/// Do `a` and `b`'s numeric effects genuinely commute?
///
/// Trivially `true` when [`touched_functions`] are disjoint. Otherwise every
/// shared function must independently pass [`pairwise_numeric_commute`]; a
/// single non-commuting shared function makes the whole pair non-commuting.
///
/// If either action fires more than one numeric effect on the same shared
/// function (e.g. from separate `when` branches, which may or may not both
/// trigger at run time), this is already an ambiguous case this checker does
/// not attempt to disambiguate -- conservatively `false`.
pub fn numeric_effects_commute(a: &ExactGroundAction, b: &ExactGroundAction) -> bool {
    let by_function_a = numeric_effects_by_function(a);
    let by_function_b = numeric_effects_by_function(b);

    for (function, effects_a) in &by_function_a {
        let Some(effects_b) = by_function_b.get(function) else {
            continue;
        };
        if effects_a.len() != 1 || effects_b.len() != 1 {
            return false;
        }
        if !pairwise_numeric_commute(function, effects_a[0], effects_b[0]) {
            return false;
        }
    }
    true
}

/// Deterministic (not guaranteed collision-free) 32-bit id for a function
/// name -- same digest/id pattern as `crate::causal::atom_id`, adapted for
/// function names instead of ground atoms.
fn fluent_id(function_name: &str) -> FluentId {
    let digest = Digest::hash(function_name.as_bytes());
    let b = digest.as_bytes();
    FluentId(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Same style as `crate::causal::effects_digest`, adapted to hash the set of
/// touched function names instead of add/del atom labels.
fn flow_digest(touched: &BTreeSet<String>) -> Digest {
    let mut buf = Vec::new();
    for name in touched {
        buf.extend_from_slice(name.as_bytes());
        buf.push(0);
    }
    Digest::hash(&buf)
}

/// Build a real (non-vacuous) [`NumericFlowWitness`] for the pair `(a, b)`:
/// `commute` from [`numeric_effects_commute`], `touched_fluents` from the
/// real union of [`touched_functions`], and `flow_digest` hashed over that
/// same touched-name set.
pub fn compute_real_numeric_flow_witness(
    a: &ExactGroundAction,
    b: &ExactGroundAction,
) -> NumericFlowWitness {
    let touched: BTreeSet<String> = touched_functions(a)
        .union(&touched_functions(b))
        .cloned()
        .collect();
    let commute = numeric_effects_commute(a, b);
    let touched_fluents: BTreeSet<FluentId> = touched.iter().map(|name| fluent_id(name)).collect();

    NumericFlowWitness {
        commute,
        flow_digest: flow_digest(&touched),
        touched_fluents,
    }
}
