//! PDDL8 grounding and forward-search plan finding.

use crate::error::Pddl8Error;
use std::collections::{BTreeSet, HashMap};
use wasm4pm_compat::pddl::{
    CompareOp, DurationConstraint, DurativeAction, NumericExpr, Pddl8ActionSchema, Pddl8Atom,
    Pddl8Domain, Pddl8GroundAction, Pddl8GroundAtom, Pddl8Problem, Pddl8Tape, PddlCondition,
    PddlEffect, PddlFunction, TemporalPlan, TemporalPlanStep, TimedLiteral, PDDL8_MAX_GROUND,
    PDDL8_MAX_PLAN_DEPTH,
};

pub struct GroundProblem {
    pub initial_state: BTreeSet<Pddl8GroundAtom>,
    pub goal: Vec<Pddl8GroundAtom>,
    pub actions: Vec<Pddl8GroundAction>,
    /// precondition atom -> indices of actions that require it. Lets
    /// `find_plan`'s BFS only consider actions that could possibly apply at
    /// a given state instead of linearly scanning every ground action.
    action_index: HashMap<Pddl8GroundAtom, Vec<usize>>,
    /// Indices of actions with no preconditions — always candidates.
    always_applicable: Vec<usize>,
}

/// Object/type lookup used to restrict grounding to type-compatible bindings.
///
/// Built once per `GroundProblem`/`GroundTemporalProblem`. A parameter with no
/// entry in `typed_params`, or a required type of `"object"` (PDDL's
/// universal type), matches every object — this is what keeps untyped/legacy
/// domains behaving exactly as before.
struct TypeIndex {
    /// object name -> declared type (objects absent from `object_types` are
    /// treated as type `"object"`, matching untyped-domain semantics).
    object_type: HashMap<String, String>,
    /// type name -> parent type name, for `(:types child - parent)` subtyping.
    parent: HashMap<String, String>,
}

impl TypeIndex {
    fn build(domain: &Pddl8Domain, problem: &Pddl8Problem) -> Self {
        let object_type = problem.object_types.iter().cloned().collect();
        let parent = domain
            .types
            .iter()
            .filter_map(|t| t.parent.clone().map(|p| (t.name.clone(), p)))
            .collect();
        Self { object_type, parent }
    }

    /// Does `obj`'s actual (or inherited) type satisfy `required`?
    fn satisfies(&self, obj: &str, required: &str) -> bool {
        if required == "object" {
            return true;
        }
        let mut cur: &str = self.object_type.get(obj).map(String::as_str).unwrap_or("object");
        loop {
            if cur == required {
                return true;
            }
            match self.parent.get(cur) {
                Some(p) => cur = p.as_str(),
                None => return false,
            }
        }
    }
}

impl GroundProblem {
    pub fn build(
        domain: &Pddl8Domain,
        problem: &Pddl8Problem,
        max_ground: Option<usize>,
    ) -> Result<Self, Pddl8Error> {
        let limit = max_ground.unwrap_or(PDDL8_MAX_GROUND);

        let initial_state: BTreeSet<Pddl8GroundAtom> = problem
            .init
            .iter()
            .map(|a| Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
            .collect();

        let goal: Vec<Pddl8GroundAtom> = problem
            .goal
            .iter()
            .map(|a| Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
            .collect();

        let objects = &problem.objects;
        let type_index = TypeIndex::build(domain, problem);
        let mut actions = Vec::new();

        for schema in &domain.actions {
            ground_schema(schema, objects, &type_index, &mut actions)?;
            if actions.len() > limit {
                return Err(Pddl8Error::BoundExceeded {
                    what: "ground actions",
                    limit: limit as u8,
                    got: actions.len(),
                });
            }
        }

        if actions.is_empty() {
            return Err(Pddl8Error::EmptyGrounding);
        }

        let mut action_index: HashMap<Pddl8GroundAtom, Vec<usize>> = HashMap::new();
        let mut always_applicable: Vec<usize> = Vec::new();
        for (i, action) in actions.iter().enumerate() {
            if action.preconditions.is_empty() {
                always_applicable.push(i);
            }
            for p in &action.preconditions {
                action_index.entry(p.clone()).or_default().push(i);
            }
        }

        Ok(Self { initial_state, goal, actions, action_index, always_applicable })
    }

    /// BFS forward search — returns a `Pddl8Tape` ready for execution.
    pub fn find_plan(&self) -> Result<Pddl8Tape, Pddl8Error> {
        use std::collections::VecDeque;

        let goal_set: BTreeSet<Pddl8GroundAtom> = self.goal.iter().cloned().collect();
        let mut queue: VecDeque<(BTreeSet<Pddl8GroundAtom>, Vec<usize>)> = VecDeque::new();
        let mut visited: std::collections::HashSet<Vec<Pddl8GroundAtom>> = Default::default();

        let init_sorted: Vec<Pddl8GroundAtom> = self.initial_state.iter().cloned().collect();
        visited.insert(init_sorted);
        queue.push_back((self.initial_state.clone(), vec![]));

        while let Some((state, path)) = queue.pop_front() {
            if path.len() > PDDL8_MAX_PLAN_DEPTH {
                continue;
            }
            if goal_set.iter().all(|g| state.contains(g)) {
                let plan: Vec<Pddl8GroundAction> =
                    path.into_iter().map(|i| self.actions[i].clone()).collect();
                return Ok(Pddl8Tape::from_plan(plan));
            }
            // Only consider actions that could possibly apply: always-applicable
            // (no preconditions) plus those keyed by an atom currently true.
            // Full precondition check below still runs per candidate — this just
            // avoids scanning the whole action list at every BFS node.
            let mut candidates: BTreeSet<usize> = self.always_applicable.iter().copied().collect();
            for atom in state.iter() {
                if let Some(idxs) = self.action_index.get(atom) {
                    candidates.extend(idxs.iter().copied());
                }
            }
            for i in candidates {
                let action = &self.actions[i];
                if action.preconditions.iter().all(|p| state.contains(p)) {
                    let mut next = state.clone();
                    for d in &action.del_effects { next.remove(d); }
                    for a in &action.add_effects { next.insert(a.clone()); }
                    let sorted: Vec<Pddl8GroundAtom> = next.iter().cloned().collect();
                    if !visited.contains(&sorted) {
                        visited.insert(sorted);
                        let mut p2 = path.clone();
                        p2.push(i);
                        queue.push_back((next, p2));
                    }
                }
            }
        }

        Err(Pddl8Error::NoAdmittedPlan)
    }
}

// ---------------------------------------------------------------------------
// PDDL 3.1 temporal grounding
// ---------------------------------------------------------------------------

/// A grounded durative action — duration bounds resolved, conditions/effects kept.
#[derive(Clone)]
pub struct GroundDurativeAction {
    pub schema_name: String,
    pub label: String,
    /// Bound object names, in schema parameter order (empty for zero-param schemas).
    pub args: Vec<String>,
    pub duration_min: f64,
    pub duration_max: f64,
    pub conditions: Vec<PddlCondition>,
    pub effects: Vec<PddlEffect>,
}

/// Grounded temporal problem: classical + durative actions, numeric fluents, timed inits.
#[derive(Clone)]
pub struct GroundTemporalProblem {
    pub initial_atoms: BTreeSet<Pddl8GroundAtom>,
    pub initial_fn_values: HashMap<String, f64>,
    pub timed_inits: Vec<TimedLiteral>,
    pub goal: PddlCondition,
    pub actions: Vec<Pddl8GroundAction>,
    pub durative_actions: Vec<GroundDurativeAction>,
}

impl GroundTemporalProblem {
    /// Build a `GroundTemporalProblem` from domain + problem.
    pub fn build(domain: &Pddl8Domain, problem: &Pddl8Problem) -> Result<Self, Pddl8Error> {
        let initial_atoms: BTreeSet<Pddl8GroundAtom> = problem
            .init
            .iter()
            .map(|a| Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
            .collect();

        let initial_fn_values: HashMap<String, f64> = problem
            .fn_values
            .iter()
            .map(|(f, v)| (fn_key(f), *v))
            .collect();

        let timed_inits = problem.timed_inits.clone();

        let goal = PddlCondition::And(
            problem.goal.iter().map(|a| PddlCondition::Atom(a.clone())).collect(),
        );

        // Ground classical actions
        let type_index = TypeIndex::build(domain, problem);
        let mut actions = Vec::new();
        for schema in &domain.actions {
            ground_schema(schema, &problem.objects, &type_index, &mut actions)?;
            if actions.len() > PDDL8_MAX_GROUND {
                return Err(Pddl8Error::BoundExceeded {
                    what: "ground actions",
                    limit: PDDL8_MAX_GROUND as u8,
                    got: actions.len(),
                });
            }
        }

        // Ground durative actions over objects, mirroring `ground_schema` for
        // classical actions: enumerate type-compatible bindings for each
        // schema's params and substitute the bound object names into the
        // schema's conditions/effects.
        let mut durative_actions = Vec::new();
        for da in &domain.durative_actions {
            ground_durative_schema(da, &problem.objects, &type_index, &mut durative_actions)?;
            if durative_actions.len() > PDDL8_MAX_GROUND {
                return Err(Pddl8Error::BoundExceeded {
                    what: "ground durative actions",
                    limit: PDDL8_MAX_GROUND as u8,
                    got: durative_actions.len(),
                });
            }
        }

        Ok(Self { initial_atoms, initial_fn_values, timed_inits, goal, actions, durative_actions })
    }

    /// Forward-chaining temporal planner using a priority queue ordered by time.
    ///
    /// - Starts with initial_atoms + initial_fn_values at t=0
    /// - Applies at-start effects immediately when an action is selected
    /// - Advances time to the next action completion
    /// - Applies at-end effects on completion
    /// - Checks goal (PddlCondition) after each completion
    /// - Limits to PDDL8_MAX_PLAN_DEPTH iterations
    pub fn find_temporal_plan(&self) -> Result<TemporalPlan, Pddl8Error> {
        self.find_temporal_plan_with_fn_overrides(&HashMap::new())
    }

    /// Same as `find_temporal_plan`, but with `overrides` merged into a
    /// cloned copy of `initial_fn_values` before planning starts — lets
    /// callers probe a perturbed numeric fluent (e.g. capacity sensitivity
    /// in `schedule_analysis::replan_with_perturbed_capacity`) without
    /// cloning the whole `GroundTemporalProblem` (grounded actions,
    /// conditions, atoms), just the small fn_values map.
    pub fn find_temporal_plan_with_fn_overrides(
        &self,
        overrides: &HashMap<String, f64>,
    ) -> Result<TemporalPlan, Pddl8Error> {
        let mut state = self.initial_atoms.clone();
        let mut fn_vals = self.initial_fn_values.clone();
        for (k, v) in overrides {
            fn_vals.insert(k.clone(), *v);
        }
        let mut steps: Vec<TemporalPlanStep> = Vec::new();
        let mut current_time = 0.0_f64;

        // Apply t=0 timed initial literals
        for til in &self.timed_inits {
            if til.time == 0.0 {
                let ga = Pddl8GroundAtom { pred: til.atom.pred.clone(), args: til.atom.args.clone() };
                if til.negated { state.remove(&ga); } else { state.insert(ga); }
            }
        }

        // Pending completions: (end_time, action_idx)
        let mut pending: Vec<(f64, usize)> = Vec::new();

        for _iteration in 0..PDDL8_MAX_PLAN_DEPTH {
            // Apply timed inits that have triggered
            for til in &self.timed_inits {
                if til.time > 0.0 && til.time <= current_time {
                    let ga = Pddl8GroundAtom { pred: til.atom.pred.clone(), args: til.atom.args.clone() };
                    if til.negated { state.remove(&ga); } else { state.insert(ga); }
                }
            }

            // Check goal
            if eval_condition(&self.goal, &state, &fn_vals) {
                let makespan = steps
                    .iter()
                    .map(|s| s.start_time + s.duration)
                    .fold(0.0_f64, f64::max);
                return Ok(TemporalPlan { steps, makespan, metric_value: None });
            }

            // Try to schedule every applicable durative action at this tick.
            // Re-scan after each start (bounded by durative_actions.len() passes)
            // so at-start effects (e.g. numeric capacity decrements) from one
            // start are visible when checking the next candidate's
            // preconditions in the same tick — this is what lets concurrent
            // starts correctly gate on shared resource fluents.
            let mut scheduled = false;
            let mut started_this_tick: BTreeSet<usize> = BTreeSet::new();
            for _pass in 0..self.durative_actions.len().max(1) {
                let mut started_this_pass = false;
                for (i, da) in self.durative_actions.iter().enumerate() {
                    if started_this_tick.contains(&i) { continue; }
                    // An action already in flight (started but not yet
                    // completed) must not be started again against itself —
                    // its own "already running" state isn't otherwise
                    // tracked for actions with no exclusive lock predicate
                    // (e.g. one that only consumes/releases a shared
                    // numeric fluent): only its *finished* effect blocks a
                    // restart, so without this guard the same grounded
                    // instance can be scheduled concurrently with itself.
                    if pending.iter().any(|(_, idx)| *idx == i) { continue; }
                    let applicable = da.conditions.iter().all(|c| {
                        eval_condition(c, &state, &fn_vals)
                    });
                    if !applicable { continue; }

                    let dur = da.duration_min;
                    let end = current_time + dur;

                    // Apply at-start effects
                    for eff in &da.effects {
                        apply_effect_at_start(eff, &mut state, &mut fn_vals);
                    }

                    steps.push(TemporalPlanStep {
                        start_time: current_time,
                        duration: dur,
                        action_name: da.schema_name.clone(),
                        args: da.args.clone(),
                    });
                    pending.push((end, i));
                    scheduled = true;
                    started_this_pass = true;
                    started_this_tick.insert(i);
                }
                if !started_this_pass { break; }
            }

            // Advance to the next pending completion
            if let Some(min_pos) = pending
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(p, _)| p)
            {
                let (end, idx) = pending.remove(min_pos);
                current_time = end;
                let da = &self.durative_actions[idx];
                for eff in &da.effects {
                    apply_effect_at_end(eff, &mut state, &mut fn_vals);
                }
            } else if !scheduled {
                break;
            }
        }

        // Final goal check after last completion
        if eval_condition(&self.goal, &state, &fn_vals) {
            let makespan = steps
                .iter()
                .map(|s| s.start_time + s.duration)
                .fold(0.0_f64, f64::max);
            return Ok(TemporalPlan { steps, makespan, metric_value: None });
        }

        Err(Pddl8Error::NoAdmittedPlan)
    }
}

/// Evaluate a `PddlCondition` against a ground state.
pub fn eval_condition(
    cond: &PddlCondition,
    state: &BTreeSet<Pddl8GroundAtom>,
    fn_vals: &HashMap<String, f64>,
) -> bool {
    match cond {
        PddlCondition::Atom(a) => {
            state.contains(&Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() })
        }
        PddlCondition::Not(inner) => !eval_condition(inner, state, fn_vals),
        PddlCondition::And(subs) => subs.iter().all(|s| eval_condition(s, state, fn_vals)),
        PddlCondition::Or(subs) => subs.iter().any(|s| eval_condition(s, state, fn_vals)),
        PddlCondition::Imply(lhs, rhs) => {
            !eval_condition(lhs, state, fn_vals) || eval_condition(rhs, state, fn_vals)
        }
        PddlCondition::Timed(_, inner) => eval_condition(inner, state, fn_vals),
        PddlCondition::Forall { .. } => true,
        PddlCondition::Exists { .. } => false,
        PddlCondition::Compare(lhs, op, rhs) => {
            let l = eval_numeric(lhs, fn_vals);
            let r = eval_numeric(rhs, fn_vals);
            match op {
                CompareOp::Ge => l >= r,
                CompareOp::Le => l <= r,
                CompareOp::Gt => l > r,
                CompareOp::Lt => l < r,
                CompareOp::Eq => (l - r).abs() < 1e-9,
            }
        }
    }
}

fn apply_effect_at_start(
    eff: &PddlEffect,
    state: &mut BTreeSet<Pddl8GroundAtom>,
    fn_vals: &mut HashMap<String, f64>,
) {
    use wasm4pm_compat::pddl::TimeSpecifier;
    match eff {
        PddlEffect::Timed(TimeSpecifier::AtStart, inner) => apply_effect_ground(inner, state, fn_vals),
        PddlEffect::Timed(_, _) => {}
        other => apply_effect_ground(other, state, fn_vals),
    }
}

fn apply_effect_at_end(
    eff: &PddlEffect,
    state: &mut BTreeSet<Pddl8GroundAtom>,
    fn_vals: &mut HashMap<String, f64>,
) {
    use wasm4pm_compat::pddl::TimeSpecifier;
    if let PddlEffect::Timed(TimeSpecifier::AtEnd, inner) = eff {
        apply_effect_ground(inner, state, fn_vals);
    }
}

fn apply_effect_ground(
    eff: &PddlEffect,
    state: &mut BTreeSet<Pddl8GroundAtom>,
    fn_vals: &mut HashMap<String, f64>,
) {
    match eff {
        PddlEffect::Add(a) => {
            state.insert(Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() });
        }
        PddlEffect::Del(a) => {
            state.remove(&Pddl8GroundAtom { pred: a.pred.clone(), args: a.args.clone() });
        }
        PddlEffect::Numeric(ne) => apply_numeric_effect(ne, fn_vals),
        PddlEffect::When { effects, .. } => {
            for e in effects { apply_effect_ground(e, state, fn_vals); }
        }
        PddlEffect::Forall { effects, .. } => {
            for e in effects { apply_effect_ground(e, state, fn_vals); }
        }
        PddlEffect::Timed(_, inner) => apply_effect_ground(inner, state, fn_vals),
    }
}

fn apply_numeric_effect(
    ne: &wasm4pm_compat::pddl::NumericEffect,
    fn_vals: &mut HashMap<String, f64>,
) {
    use wasm4pm_compat::pddl::NumericEffect;
    match ne {
        NumericEffect::Assign(f, expr) => {
            let v = eval_numeric(expr, fn_vals);
            fn_vals.insert(fn_key(f), v);
        }
        NumericEffect::Increase(f, expr) => {
            let v = eval_numeric(expr, fn_vals);
            *fn_vals.entry(fn_key(f)).or_insert(0.0) += v;
        }
        NumericEffect::Decrease(f, expr) => {
            let v = eval_numeric(expr, fn_vals);
            *fn_vals.entry(fn_key(f)).or_insert(0.0) -= v;
        }
        NumericEffect::ScaleUp(f, expr) => {
            let v = eval_numeric(expr, fn_vals);
            *fn_vals.entry(fn_key(f)).or_insert(1.0) *= v;
        }
        NumericEffect::ScaleDown(f, expr) => {
            let v = eval_numeric(expr, fn_vals);
            let entry = fn_vals.entry(fn_key(f)).or_insert(1.0);
            if v != 0.0 { *entry /= v; }
        }
    }
}

fn eval_numeric(expr: &NumericExpr, fn_vals: &HashMap<String, f64>) -> f64 {
    use wasm4pm_compat::pddl::{NumericExpr, NumericOp};
    match expr {
        NumericExpr::Number(n) => *n,
        NumericExpr::FunctionTerm(name, args) => {
            let key = if args.is_empty() {
                name.clone()
            } else {
                format!("{}({})", name, args.join(","))
            };
            *fn_vals.get(&key).unwrap_or(&0.0)
        }
        NumericExpr::BinOp { op, lhs, rhs } => {
            let l = eval_numeric(lhs, fn_vals);
            let r = eval_numeric(rhs, fn_vals);
            match op {
                NumericOp::Add => l + r,
                NumericOp::Sub => l - r,
                NumericOp::Mul => l * r,
                NumericOp::Div => if r != 0.0 { l / r } else { 0.0 },
            }
        }
        NumericExpr::Neg(inner) => -eval_numeric(inner, fn_vals),
    }
}

/// Resolve a `DurationConstraint` to (min, max) f64 bounds.
fn resolve_duration(dc: &DurationConstraint) -> (f64, f64) {
    match dc {
        DurationConstraint::Eq(expr) => {
            let v = eval_numeric(expr, &HashMap::new());
            (v, v)
        }
        DurationConstraint::Gte(expr) => {
            let v = eval_numeric(expr, &HashMap::new());
            (v, f64::INFINITY)
        }
        DurationConstraint::Lte(expr) => {
            let v = eval_numeric(expr, &HashMap::new());
            (0.0, v)
        }
        DurationConstraint::And(parts) => {
            let mut lo = 0.0_f64;
            let mut hi = f64::INFINITY;
            for p in parts {
                let (a, b) = resolve_duration(p);
                lo = lo.max(a);
                hi = hi.min(b);
            }
            (lo, hi)
        }
    }
}

/// Stable string key for a `PddlFunction`.
fn fn_key(f: &PddlFunction) -> String {
    if f.params.is_empty() {
        f.name.clone()
    } else {
        format!("{}({})", f.name, f.params.join(","))
    }
}

fn ground_schema(
    schema: &Pddl8ActionSchema,
    objects: &[String],
    type_index: &TypeIndex,
    out: &mut Vec<Pddl8GroundAction>,
) -> Result<(), Pddl8Error> {
    let n = schema.params.len();
    if n == 0 {
        if let Some(ga) = instantiate(schema, &HashMap::new()) {
            out.push(ga);
        }
        return Ok(());
    }

    // Per-parameter candidate lists, restricted to type-compatible objects
    // when the schema declares a type for that parameter — this is what
    // shrinks grounding from |objects|^n to ∏ᵢ |objects_of_type(paramᵢ)|.
    // A parameter absent from `typed_params` falls back to the full object
    // list, preserving exact behavior for untyped/legacy domains.
    let typed: HashMap<&str, &str> = schema
        .typed_params
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let candidates: Vec<Vec<&String>> = schema
        .params
        .iter()
        .map(|p| match typed.get(p.as_str()) {
            Some(required) => objects.iter().filter(|o| type_index.satisfies(o, required)).collect(),
            None => objects.iter().collect(),
        })
        .collect();
    if candidates.iter().any(|c| c.is_empty()) {
        return Ok(());
    }

    let mut indices = vec![0usize; n];
    loop {
        let binding: HashMap<String, String> = schema
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), candidates[i][indices[i]].clone()))
            .collect();
        if let Some(ga) = instantiate(schema, &binding) {
            out.push(ga);
        }
        // odometer increment, now bounded per-slot by candidates[i].len()
        let mut pos = n;
        loop {
            if pos == 0 { return Ok(()); }
            pos -= 1;
            indices[pos] += 1;
            if indices[pos] < candidates[pos].len() { break; }
            indices[pos] = 0;
        }
    }
}

/// Ground a `DurativeAction` schema over `objects`, mirroring `ground_schema`
/// for classical actions: enumerate type-compatible bindings for `da.params`
/// and substitute the bound object names into the schema's conditions and
/// effects. A zero-param schema collapses to exactly one ground instance.
fn ground_durative_schema(
    da: &DurativeAction,
    objects: &[String],
    type_index: &TypeIndex,
    out: &mut Vec<GroundDurativeAction>,
) -> Result<(), Pddl8Error> {
    let n = da.params.len();
    let (dur_min, dur_max) = resolve_duration(&da.duration);

    if n == 0 {
        out.push(GroundDurativeAction {
            schema_name: da.name.clone(),
            label: da.name.clone(),
            args: vec![],
            duration_min: dur_min,
            duration_max: dur_max,
            conditions: da.conditions.clone(),
            effects: da.effects.clone(),
        });
        return Ok(());
    }

    // Per-parameter candidate lists, restricted to type-compatible objects —
    // same scheme as `ground_schema`'s `typed_params` lookup.
    let candidates: Vec<Vec<&String>> = da
        .params
        .iter()
        .map(|(_, required_type)| {
            objects.iter().filter(|o| type_index.satisfies(o, required_type)).collect()
        })
        .collect();
    if candidates.iter().any(|c| c.is_empty()) {
        return Ok(());
    }

    let mut indices = vec![0usize; n];
    loop {
        let binding: HashMap<String, String> = da
            .params
            .iter()
            .enumerate()
            .map(|(i, (p, _))| (p.clone(), candidates[i][indices[i]].clone()))
            .collect();

        let args: Vec<String> =
            da.params.iter().filter_map(|(p, _)| binding.get(p)).cloned().collect();
        let label = format!("{}({})", da.name, args.join(","));

        out.push(GroundDurativeAction {
            schema_name: da.name.clone(),
            label,
            args,
            duration_min: dur_min,
            duration_max: dur_max,
            conditions: da.conditions.iter().map(|c| subst_condition(c, &binding)).collect(),
            effects: da.effects.iter().map(|e| subst_effect(e, &binding)).collect(),
        });

        // odometer increment, bounded per-slot by candidates[i].len()
        let mut pos = n;
        loop {
            if pos == 0 { return Ok(()); }
            pos -= 1;
            indices[pos] += 1;
            if indices[pos] < candidates[pos].len() { break; }
            indices[pos] = 0;
        }
    }
}

/// Substitute schema parameter variables (e.g. `?v`) with bound object names
/// throughout a `Pddl8Atom`'s args. Non-variable args (and variables absent
/// from `binding`, e.g. universally-quantified ones) pass through unchanged.
fn subst_atom(a: &Pddl8Atom, binding: &HashMap<String, String>) -> Pddl8Atom {
    Pddl8Atom {
        pred: a.pred.clone(),
        args: a
            .args
            .iter()
            .map(|arg| {
                if Pddl8Atom::is_variable(arg) {
                    binding.get(arg).cloned().unwrap_or_else(|| arg.clone())
                } else {
                    arg.clone()
                }
            })
            .collect(),
    }
}

fn subst_numeric(expr: &NumericExpr, binding: &HashMap<String, String>) -> NumericExpr {
    match expr {
        NumericExpr::Number(n) => NumericExpr::Number(*n),
        NumericExpr::FunctionTerm(name, args) => NumericExpr::FunctionTerm(
            name.clone(),
            args.iter()
                .map(|arg| {
                    if Pddl8Atom::is_variable(arg) {
                        binding.get(arg).cloned().unwrap_or_else(|| arg.clone())
                    } else {
                        arg.clone()
                    }
                })
                .collect(),
        ),
        NumericExpr::BinOp { op, lhs, rhs } => NumericExpr::BinOp {
            op: *op,
            lhs: Box::new(subst_numeric(lhs, binding)),
            rhs: Box::new(subst_numeric(rhs, binding)),
        },
        NumericExpr::Neg(inner) => NumericExpr::Neg(Box::new(subst_numeric(inner, binding))),
    }
}

fn subst_function(f: &PddlFunction, binding: &HashMap<String, String>) -> PddlFunction {
    PddlFunction {
        name: f.name.clone(),
        params: f
            .params
            .iter()
            .map(|arg| {
                if Pddl8Atom::is_variable(arg) {
                    binding.get(arg).cloned().unwrap_or_else(|| arg.clone())
                } else {
                    arg.clone()
                }
            })
            .collect(),
    }
}

fn subst_condition(cond: &PddlCondition, binding: &HashMap<String, String>) -> PddlCondition {
    match cond {
        PddlCondition::Atom(a) => PddlCondition::Atom(subst_atom(a, binding)),
        PddlCondition::Not(inner) => PddlCondition::Not(Box::new(subst_condition(inner, binding))),
        PddlCondition::And(subs) => {
            PddlCondition::And(subs.iter().map(|s| subst_condition(s, binding)).collect())
        }
        PddlCondition::Or(subs) => {
            PddlCondition::Or(subs.iter().map(|s| subst_condition(s, binding)).collect())
        }
        PddlCondition::Forall { vars, body } => PddlCondition::Forall {
            vars: vars.clone(),
            body: Box::new(subst_condition(body, binding)),
        },
        PddlCondition::Exists { vars, body } => PddlCondition::Exists {
            vars: vars.clone(),
            body: Box::new(subst_condition(body, binding)),
        },
        PddlCondition::Imply(lhs, rhs) => PddlCondition::Imply(
            Box::new(subst_condition(lhs, binding)),
            Box::new(subst_condition(rhs, binding)),
        ),
        PddlCondition::Timed(spec, inner) => {
            PddlCondition::Timed(*spec, Box::new(subst_condition(inner, binding)))
        }
        PddlCondition::Compare(lhs, op, rhs) => {
            PddlCondition::Compare(subst_numeric(lhs, binding), *op, subst_numeric(rhs, binding))
        }
    }
}

fn subst_numeric_effect(
    ne: &wasm4pm_compat::pddl::NumericEffect,
    binding: &HashMap<String, String>,
) -> wasm4pm_compat::pddl::NumericEffect {
    use wasm4pm_compat::pddl::NumericEffect;
    match ne {
        NumericEffect::Assign(f, expr) => {
            NumericEffect::Assign(subst_function(f, binding), subst_numeric(expr, binding))
        }
        NumericEffect::Increase(f, expr) => {
            NumericEffect::Increase(subst_function(f, binding), subst_numeric(expr, binding))
        }
        NumericEffect::Decrease(f, expr) => {
            NumericEffect::Decrease(subst_function(f, binding), subst_numeric(expr, binding))
        }
        NumericEffect::ScaleUp(f, expr) => {
            NumericEffect::ScaleUp(subst_function(f, binding), subst_numeric(expr, binding))
        }
        NumericEffect::ScaleDown(f, expr) => {
            NumericEffect::ScaleDown(subst_function(f, binding), subst_numeric(expr, binding))
        }
    }
}

fn subst_effect(eff: &PddlEffect, binding: &HashMap<String, String>) -> PddlEffect {
    match eff {
        PddlEffect::Add(a) => PddlEffect::Add(subst_atom(a, binding)),
        PddlEffect::Del(a) => PddlEffect::Del(subst_atom(a, binding)),
        PddlEffect::Numeric(ne) => PddlEffect::Numeric(subst_numeric_effect(ne, binding)),
        PddlEffect::Timed(spec, inner) => {
            PddlEffect::Timed(*spec, Box::new(subst_effect(inner, binding)))
        }
        PddlEffect::Forall { vars, effects } => PddlEffect::Forall {
            vars: vars.clone(),
            effects: effects.iter().map(|e| subst_effect(e, binding)).collect(),
        },
        PddlEffect::When { condition, effects } => PddlEffect::When {
            condition: subst_condition(condition, binding),
            effects: effects.iter().map(|e| subst_effect(e, binding)).collect(),
        },
    }
}

fn instantiate(schema: &Pddl8ActionSchema, binding: &HashMap<String, String>) -> Option<Pddl8GroundAction> {
    fn ground_atom(a: &Pddl8Atom, binding: &HashMap<String, String>) -> Option<Pddl8GroundAtom> {
        let args: Option<Vec<String>> = a.args.iter().map(|arg| {
            if Pddl8Atom::is_variable(arg) { binding.get(arg).cloned() }
            else { Some(arg.clone()) }
        }).collect();
        args.map(|args| Pddl8GroundAtom { pred: a.pred.clone(), args })
    }

    let preconditions: Option<Vec<_>> = schema.preconditions.iter().map(|a| ground_atom(a, binding)).collect();
    let add_effects: Option<Vec<_>> = schema.add_effects.iter().map(|a| ground_atom(a, binding)).collect();
    let del_effects: Option<Vec<_>> = schema.del_effects.iter().map(|a| ground_atom(a, binding)).collect();

    let bound_args: Vec<String> = schema.params.iter().filter_map(|p| binding.get(p)).cloned().collect();
    let label = if bound_args.is_empty() {
        schema.name.clone()
    } else {
        format!("{}({})", schema.name, bound_args.join(","))
    };

    Some(Pddl8GroundAction {
        schema_name: schema.name.clone(),
        label,
        preconditions: preconditions?,
        add_effects: add_effects?,
        del_effects: del_effects?,
    })
}
