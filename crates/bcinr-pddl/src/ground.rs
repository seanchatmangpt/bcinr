//! PDDL8 grounding and forward-search plan finding.

use crate::error::Pddl8Error;
use std::collections::{BTreeSet, HashMap};
use wasm4pm_compat::pddl::{
    DurationConstraint, NumericExpr, Pddl8ActionSchema, Pddl8Atom, Pddl8Domain,
    Pddl8GroundAction, Pddl8GroundAtom, Pddl8Problem, Pddl8Tape, PddlCondition, PddlEffect,
    PddlFunction, TemporalPlan, TemporalPlanStep, TimedLiteral, PDDL8_MAX_GROUND,
    PDDL8_MAX_PLAN_DEPTH,
};

pub struct GroundProblem {
    pub initial_state: BTreeSet<Pddl8GroundAtom>,
    pub goal: Vec<Pddl8GroundAtom>,
    pub actions: Vec<Pddl8GroundAction>,
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
        let mut actions = Vec::new();

        for schema in &domain.actions {
            ground_schema(schema, objects, &mut actions)?;
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

        Ok(Self { initial_state, goal, actions })
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
            for (i, action) in self.actions.iter().enumerate() {
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
pub struct GroundDurativeAction {
    pub schema_name: String,
    pub label: String,
    pub duration_min: f64,
    pub duration_max: f64,
    pub conditions: Vec<PddlCondition>,
    pub effects: Vec<PddlEffect>,
}

/// Grounded temporal problem: classical + durative actions, numeric fluents, timed inits.
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
        let mut actions = Vec::new();
        for schema in &domain.actions {
            ground_schema(schema, &problem.objects, &mut actions)?;
            if actions.len() > PDDL8_MAX_GROUND {
                return Err(Pddl8Error::BoundExceeded {
                    what: "ground actions",
                    limit: PDDL8_MAX_GROUND as u8,
                    got: actions.len(),
                });
            }
        }

        // Ground durative actions
        let mut durative_actions = Vec::new();
        for da in &domain.durative_actions {
            let (dur_min, dur_max) = resolve_duration(&da.duration);
            durative_actions.push(GroundDurativeAction {
                schema_name: da.name.clone(),
                label: da.name.clone(),
                duration_min: dur_min,
                duration_max: dur_max,
                conditions: da.conditions.clone(),
                effects: da.effects.clone(),
            });
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
        let mut state = self.initial_atoms.clone();
        let mut fn_vals = self.initial_fn_values.clone();
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

            // Try to schedule an applicable durative action
            let mut scheduled = false;
            for (i, da) in self.durative_actions.iter().enumerate() {
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
                    args: vec![],
                });
                pending.push((end, i));
                scheduled = true;
                break;
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
    out: &mut Vec<Pddl8GroundAction>,
) -> Result<(), Pddl8Error> {
    let n = schema.params.len();
    if n == 0 {
        if let Some(ga) = instantiate(schema, &HashMap::new()) {
            out.push(ga);
        }
        return Ok(());
    }
    let mut indices = vec![0usize; n];
    loop {
        let binding: HashMap<String, String> = schema
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), objects[indices[i]].clone()))
            .collect();
        if let Some(ga) = instantiate(schema, &binding) {
            out.push(ga);
        }
        // odometer increment
        let mut pos = n;
        loop {
            if pos == 0 { return Ok(()); }
            pos -= 1;
            indices[pos] += 1;
            if indices[pos] < objects.len() { break; }
            indices[pos] = 0;
        }
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
