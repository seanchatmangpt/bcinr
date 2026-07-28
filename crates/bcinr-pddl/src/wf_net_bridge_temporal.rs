//! Temporal-rail analog of [`crate::wf_net_bridge`]: materialize a
//! `wasm4pm_compat::pddl::TemporalPlan` (found by
//! [`crate::ground::GroundTemporalProblem::find_temporal_plan`]) into a
//! `bcinr_powl::wf_net::WfNet`, then run it through
//! `bcinr_powl::wf_to_powl::convert` (Algorithm 3) to recover hierarchical
//! POWL 2.0 structure -- exactly like the classical rail's bridge, but for
//! durative actions with `[start, start+duration)` intervals instead of a
//! flat STRIPS occurrence sequence.
//!
//! # Signature deviation from the classical bridge, and why
//!
//! `wf_net_bridge::causal_plan_to_partial_order` takes two arguments
//! (`epoch: &GroundedPlanningEpoch`, `causal_plan: &CausalPlan`) because the
//! occurrence sequence and the action bodies it needs for labels live in
//! separate types. The same split exists here, for a stronger reason:
//! `TemporalPlanStep` (see `wasm4pm_compat::pddl`) carries only
//! `start_time`, `duration`, `action_name` (the *schema* name, not the
//! grounded label), and `args` -- it has **no condition/effect fields at
//! all**. A genuine footprint-disjointness check is therefore structurally
//! impossible from `&TemporalPlan` alone; the conditions/effects only exist
//! on `GroundDurativeAction` (`crate::ground::GroundTemporalProblem::
//! durative_actions`). Every function below that needs a footprint or a
//! real label therefore takes `problem: &GroundTemporalProblem` alongside
//! `plan: &TemporalPlan`, mirroring the classical bridge's two-argument
//! shape rather than force-fitting a single-argument signature that could
//! only ever default every pair to `Dependent` (safe, but making the
//! footprint check dead code) or fabricate a footprint that isn't there
//! (unsound). Steps are matched back to their `GroundDurativeAction` by
//! `(schema_name, args)` equality -- the same identity `TemporalPlanStep` is
//! built from in `GroundTemporalProblem::find_temporal_plan_with_fn_overrides`.
//!
//! # Independence criterion
//!
//! Two steps are the temporal analog of independent -- safe to leave
//! unordered in the partial order, and so eligible for `recompose` to
//! expose as concurrent branches -- only when **both** hold:
//!
//! 1. **Genuine interval overlap**: `[a.start, a.start+a.duration)` and
//!    `[b.start, b.start+b.duration)` actually intersect. Non-overlapping
//!    steps are left ordered (dependent) even if their footprints are
//!    disjoint -- there is no evidence in the plan that running them
//!    concurrently is meaningful, only that it might be safe, and this
//!    module follows the same "never invent Independent without a passing
//!    check" discipline as `crate::causal::PddlCausalAnalyzer`.
//! 2. **Disjoint footprints**: the two `GroundDurativeAction`s' combined
//!    at-start/over-all/at-end conditions and effects touch no atom or
//!    numeric fluent in common under a write/write, write/read, or
//!    read/write pairing (read/read overlap is fine -- reading the same
//!    fact from two branches commutes). Footprint extraction is a
//!    conservative static walk of `PddlCondition`/`PddlEffect`; a
//!    `Forall`/`Exists` anywhere in either action makes that action's
//!    footprint `unbounded` (its true ground footprint ranges over the
//!    object universe, which this module has no access to -- `ground::
//!    QuantifierDomain` is private to the `ground` module) and such an
//!    action is never found independent of anything. `PddlEffect::When`'s
//!    condition is folded into the reads set and its (conditionally-fired)
//!    effects are folded into the writes set unconditionally -- an
//!    over-approximation of writes that can only turn a would-be
//!    independent pair into a dependent one, never the reverse.
//!
//! Either sub-check failing, or either step's `GroundDurativeAction` not
//! being resolvable in `problem.durative_actions` at all, defaults the pair
//! to dependent.

use std::collections::BTreeSet;

use bcinr_powl::powl2::Powl2Model;
use bcinr_powl::wf_net::{NetError, WfNet};
use bcinr_powl::wf_to_powl::{convert_and_verify, Refusal, RefusalReason, DEFAULT_DEPTH_BUDGET};
use wasm4pm_compat::pddl::{
    NumericEffect, NumericExpr, Pddl8Atom, Pddl8GroundAtom, PddlCondition, PddlEffect,
    TemporalPlan, TemporalPlanStep,
};

use crate::ground::{GroundDurativeAction, GroundTemporalProblem};

/// Read/write footprint of a single grounded durative action, folded across
/// all of its at-start/over-all/at-end conditions and effects. See the
/// module doc comment for exactly what `unbounded` means and why.
#[derive(Default)]
struct Footprint {
    reads: BTreeSet<Pddl8GroundAtom>,
    writes: BTreeSet<Pddl8GroundAtom>,
    read_fluents: BTreeSet<String>,
    write_fluents: BTreeSet<String>,
    unbounded: bool,
}

fn to_ground(a: &Pddl8Atom) -> Pddl8GroundAtom {
    Pddl8GroundAtom {
        pred: a.pred.clone(),
        args: a.args.clone(),
    }
}

/// Stable string key for a numeric fluent reference, matching
/// `ground::fn_key`'s / `ground::eval_numeric`'s `{name}({args})` format --
/// that private helper isn't reachable from here, so this is a deliberate,
/// format-identical re-derivation, not a divergent scheme.
fn fluent_key(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!("{name}({})", args.join(","))
    }
}

fn walk_numeric_reads(expr: &NumericExpr, fp: &mut Footprint) {
    match expr {
        NumericExpr::Number(_) => {}
        NumericExpr::FunctionTerm(name, args) => {
            fp.read_fluents.insert(fluent_key(name, args));
        }
        NumericExpr::BinOp { lhs, rhs, .. } => {
            walk_numeric_reads(lhs, fp);
            walk_numeric_reads(rhs, fp);
        }
        NumericExpr::Neg(inner) => walk_numeric_reads(inner, fp),
    }
}

fn walk_condition(cond: &PddlCondition, fp: &mut Footprint) {
    match cond {
        PddlCondition::Atom(a) => {
            fp.reads.insert(to_ground(a));
        }
        PddlCondition::Not(inner) | PddlCondition::Timed(_, inner) => walk_condition(inner, fp),
        PddlCondition::And(subs) | PddlCondition::Or(subs) => {
            for s in subs {
                walk_condition(s, fp);
            }
        }
        PddlCondition::Imply(lhs, rhs) => {
            walk_condition(lhs, fp);
            walk_condition(rhs, fp);
        }
        // Quantified range is not resolvable here -- see module doc comment.
        PddlCondition::Forall { .. } | PddlCondition::Exists { .. } => {
            fp.unbounded = true;
        }
        PddlCondition::Compare(lhs, _op, rhs) => {
            walk_numeric_reads(lhs, fp);
            walk_numeric_reads(rhs, fp);
        }
    }
}

fn walk_effect(eff: &PddlEffect, fp: &mut Footprint) {
    match eff {
        PddlEffect::Add(a) | PddlEffect::Del(a) => {
            fp.writes.insert(to_ground(a));
        }
        PddlEffect::Numeric(ne) => {
            let (f, expr) = match ne {
                NumericEffect::Assign(f, e)
                | NumericEffect::Increase(f, e)
                | NumericEffect::Decrease(f, e)
                | NumericEffect::ScaleUp(f, e)
                | NumericEffect::ScaleDown(f, e) => (f, e),
            };
            fp.write_fluents.insert(fluent_key(&f.name, &f.params));
            walk_numeric_reads(expr, fp);
        }
        PddlEffect::Timed(_, inner) => walk_effect(inner, fp),
        // Quantified range is not resolvable here -- see module doc comment.
        PddlEffect::Forall { .. } => {
            fp.unbounded = true;
        }
        PddlEffect::When { condition, effects } => {
            walk_condition(condition, fp);
            for e in effects {
                walk_effect(e, fp);
            }
        }
    }
}

fn compute_footprint(da: &GroundDurativeAction) -> Footprint {
    let mut fp = Footprint::default();
    for c in &da.conditions {
        walk_condition(c, &mut fp);
    }
    for e in &da.effects {
        walk_effect(e, &mut fp);
    }
    fp
}

/// `true` only if neither footprint is `unbounded` and no write/write,
/// write/read, or read/write pair shares an atom or fluent. Read/read
/// sharing is fine -- see the module doc comment.
fn footprints_disjoint(a: &Footprint, b: &Footprint) -> bool {
    if a.unbounded || b.unbounded {
        return false;
    }
    a.writes.is_disjoint(&b.writes)
        && a.writes.is_disjoint(&b.reads)
        && b.writes.is_disjoint(&a.reads)
        && a.write_fluents.is_disjoint(&b.write_fluents)
        && a.write_fluents.is_disjoint(&b.read_fluents)
        && b.write_fluents.is_disjoint(&a.read_fluents)
}

/// Genuine half-open interval overlap: `[a.start, a.start+a.duration)` and
/// `[b.start, b.start+b.duration)` share a nonempty sub-interval.
fn intervals_overlap(a: &TemporalPlanStep, b: &TemporalPlanStep) -> bool {
    let a_end = a.start_time + a.duration;
    let b_end = b.start_time + b.duration;
    a.start_time < b_end && b.start_time < a_end
}

/// Resolve a `TemporalPlanStep` back to the `GroundDurativeAction` it fired,
/// by `(schema_name, args)` identity -- see the module doc comment for why
/// `TemporalPlanStep` alone can't carry this.
fn find_ground_durative_action<'a>(
    problem: &'a GroundTemporalProblem,
    step: &TemporalPlanStep,
) -> Option<&'a GroundDurativeAction> {
    problem
        .durative_actions
        .iter()
        .find(|da| da.schema_name == step.action_name && da.args == step.args)
}

/// The step's resolved `GroundDurativeAction::label`, or (only if
/// `problem` and `plan` are mismatched) a synthesized `name(args)` fallback.
fn step_label(problem: &GroundTemporalProblem, step: &TemporalPlanStep) -> String {
    match find_ground_durative_action(problem, step) {
        Some(da) => da.label.clone(),
        None if step.args.is_empty() => step.action_name.clone(),
        None => format!("{}({})", step.action_name, step.args.join(",")),
    }
}

/// The temporal analog of `crate::causal::PddlCausalAnalyzer`'s
/// independence check: `true` only when `a` and `b` genuinely overlap in
/// time **and** their resolved `GroundDurativeAction` footprints are
/// disjoint. Defaults to `false` (dependent) whenever either step's ground
/// action cannot be resolved against `problem.durative_actions` -- never
/// invent independence without both checks genuinely passing.
#[must_use]
pub fn is_temporally_independent(
    problem: &GroundTemporalProblem,
    a: &TemporalPlanStep,
    b: &TemporalPlanStep,
) -> bool {
    if !intervals_overlap(a, b) {
        return false;
    }
    let da_a = match find_ground_durative_action(problem, a) {
        Some(da) => da,
        None => return false,
    };
    let da_b = match find_ground_durative_action(problem, b) {
        Some(da) => da,
        None => return false,
    };
    footprints_disjoint(&compute_footprint(da_a), &compute_footprint(da_b))
}

/// Build a flat `Powl2Model::PartialOrder` over the temporal plan's steps --
/// the temporal-rail analog of `wf_net_bridge`'s (private)
/// `causal_plan_to_partial_order`. Every step becomes a
/// `Powl2Model::Activity`; edges are added for every pair
/// `is_temporally_independent` does not clear, so only genuinely
/// overlapping, footprint-disjoint pairs are left unordered for
/// `recompose` to expose as concurrent branches.
#[must_use]
pub fn temporal_plan_to_partial_order(
    problem: &GroundTemporalProblem,
    plan: &TemporalPlan,
) -> Powl2Model {
    let steps = &plan.steps;
    if steps.is_empty() {
        return Powl2Model::Silent;
    }

    let children: Vec<Powl2Model> = steps
        .iter()
        .map(|step| Powl2Model::Activity(step_label(problem, step)))
        .collect();

    if children.len() == 1 {
        return children.into_iter().next().expect("length checked");
    }

    let mut edges = Vec::new();
    for i in 0..steps.len() {
        for j in (i + 1)..steps.len() {
            if !is_temporally_independent(problem, &steps[i], &steps[j]) {
                edges.push((i, j));
            }
        }
    }

    Powl2Model::PartialOrder { children, edges }
}

/// Materialize a temporal plan as an actual `WfNet` via recomposition. Errs
/// (rather than panics) if recomposition's own algorithm-internal
/// `WfNet::new` check fails -- see `bcinr_powl::recompose::recompose`'s doc
/// comment.
pub fn temporal_plan_to_wf_net(
    problem: &GroundTemporalProblem,
    plan: &TemporalPlan,
) -> Result<WfNet, bcinr_powl::recompose::RecomposeError> {
    let model = temporal_plan_to_partial_order(problem, plan);
    bcinr_powl::recompose::recompose(&model)
}

/// The full temporal-rail bridge: overlap+footprint analysis -> `WfNet` ->
/// Algorithm 3 decomposition. Mirrors `wf_net_bridge::causal_plan_to_powl2`
/// exactly, modulo the two-argument signature explained in the module doc
/// comment -- including its Theorem-1 gate and its `max_len` choice (the
/// recomposed net's transition count, *not* the plan's step count; see that
/// function's doc comment for why sizing the bound to visible activities
/// alone starves the replay's firing budget and refuses good models).
pub fn temporal_plan_to_powl2(
    problem: &GroundTemporalProblem,
    plan: &TemporalPlan,
) -> Result<Powl2Model, Refusal> {
    let net = temporal_plan_to_wf_net(problem, plan).map_err(|err| Refusal {
        reason: RefusalReason::NotRecomposable(err),
        net_hash: "0".repeat(64),
    })?;
    let max_len = net.transitions().len();
    convert_and_verify(&net, DEFAULT_DEPTH_BUDGET, max_len)
}
