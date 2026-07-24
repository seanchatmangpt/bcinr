//! Exact bounded classical PDDL grounder and search rail.
//!
//! This module consumes `Pddl31Domain`/`Pddl31Problem` directly, preserving
//! Boolean conditions, equality, quantifiers, numeric fluents, and
//! conditional/quantified effects. It is separate from the legacy PDDL8
//! compatibility grounder so richer semantics cannot be silently flattened.
//!
//! Admitted scope is explicit: classical actions only. Durative actions,
//! timed literals, PDDL+ processes/events, derived predicates, trajectory
//! constraints, preferences, and plan metrics return typed refusals.

use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

use wasm4pm_compat::pddl::{
    CompareOp, NumericEffect, NumericExpr, NumericOp, Pddl31Action, Pddl31Domain, Pddl31Problem,
    Pddl8Atom, Pddl8GroundAction, Pddl8GroundAtom, Pddl8Tape, Pddl8TapeOp, PddlCondition,
    PddlEffect, PddlFunction, PddlType,
};

use crate::capability::{CapabilityProfile, PddlFeature, SemanticSupport};

/// Default structural bounds for exact classical search.
pub const EXACT_MAX_GROUND_ACTIONS: usize = 65_536;
pub const EXACT_MAX_PLAN_DEPTH: usize = 64;
pub const EXACT_MAX_SEARCH_STATES: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExactClassicalError {
    DurativeActionsUnsupported,
    TimedInitialLiteralsUnsupported,
    ProcessesUnsupported,
    EventsUnsupported,
    DerivedPredicatesUnsupported,
    TrajectoryConstraintsUnsupported,
    PreferencesUnsupported,
    MetricsUnsupported,
    TimedConditionUnsupported,
    TimedEffectUnsupported,
    ContinuousEffectUnsupported,
    ObjectFluentUnsupported,
    GroundActionBoundExceeded { limit: usize, observed: usize },
    PlanDepthBoundExceeded { limit: usize },
    SearchStateBoundExceeded { limit: usize },
    NoPlan,
    ConflictingNumericEffects { function: String },
    DivisionByZero { function: String },
    TapeFull,
}

impl std::fmt::Display for ExactClassicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DurativeActionsUnsupported => write!(f, "durative actions require the temporal rail"),
            Self::TimedInitialLiteralsUnsupported => {
                write!(f, "timed initial literals require the temporal rail")
            }
            Self::ProcessesUnsupported => write!(f, "PDDL+ processes require the hybrid rail"),
            Self::EventsUnsupported => write!(f, "PDDL+ events require the hybrid rail"),
            Self::DerivedPredicatesUnsupported => {
                write!(f, "derived predicates are not admitted by the exact classical rail")
            }
            Self::TrajectoryConstraintsUnsupported => {
                write!(f, "trajectory constraints require a trace-monitoring rail")
            }
            Self::PreferencesUnsupported => {
                write!(f, "preferences require a soft-constraint optimization rail")
            }
            Self::MetricsUnsupported => write!(f, "metrics require an optimizing search rail"),
            Self::TimedConditionUnsupported => write!(f, "timed condition in classical action"),
            Self::TimedEffectUnsupported => write!(f, "timed effect in classical action"),
            Self::ContinuousEffectUnsupported => write!(f, "continuous effect is unsupported"),
            Self::ObjectFluentUnsupported => write!(f, "object fluent assignment is unsupported"),
            Self::GroundActionBoundExceeded { limit, observed } => write!(
                f,
                "ground-action bound exceeded: observed {observed}, limit {limit}"
            ),
            Self::PlanDepthBoundExceeded { limit } => {
                write!(f, "plan depth bound exceeded: {limit}")
            }
            Self::SearchStateBoundExceeded { limit } => {
                write!(f, "search state bound exceeded: {limit}")
            }
            Self::NoPlan => write!(f, "reachable state space exhausted without a plan"),
            Self::ConflictingNumericEffects { function } => {
                write!(f, "multiple simultaneous numeric effects target {function}")
            }
            Self::DivisionByZero { function } => {
                write!(f, "numeric scale-down by zero for {function}")
            }
            Self::TapeFull => write!(f, "plan exceeds the 64-slot PDDL tape bound"),
        }
    }
}

impl std::error::Error for ExactClassicalError {}

/// Capability profile implemented exactly by this module.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExactClassicalCapabilityProfile;

impl CapabilityProfile for ExactClassicalCapabilityProfile {
    fn support(&self, feature: PddlFeature) -> SemanticSupport {
        match feature {
            PddlFeature::Strips
            | PddlFeature::Typing
            | PddlFeature::NegativePreconditions
            | PddlFeature::Disjunction
            | PddlFeature::Equality
            | PddlFeature::ExistentialPreconditions
            | PddlFeature::UniversalPreconditions
            | PddlFeature::ConditionalEffects
            | PddlFeature::NumericFluents
            | PddlFeature::NumericEffects => SemanticSupport::Exact,
            PddlFeature::DurativeActions
            | PddlFeature::TimedInitialLiterals
            | PddlFeature::DerivedPredicates
            | PddlFeature::TrajectoryConstraints
            | PddlFeature::Preferences
            | PddlFeature::Metrics => SemanticSupport::Unsupported,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExactGroundAction {
    pub schema_name: String,
    pub label: String,
    pub args: Vec<String>,
    pub condition: PddlCondition,
    pub effects: Vec<PddlEffect>,
    pub legacy_action: Pddl8GroundAction,
}

#[derive(Debug, Clone)]
struct ExactState {
    facts: BTreeSet<Pddl8GroundAtom>,
    functions: BTreeMap<String, f64>,
}

/// Exact bounded classical planning problem.
#[derive(Debug, Clone)]
pub struct ExactClassicalProblem {
    pub initial_facts: BTreeSet<Pddl8GroundAtom>,
    pub initial_functions: BTreeMap<String, f64>,
    pub goal: PddlCondition,
    pub actions: Vec<ExactGroundAction>,
    objects: Vec<String>,
    type_index: TypeIndex,
}

#[derive(Debug, Clone)]
struct TypeIndex {
    object_type: BTreeMap<String, String>,
    parent: BTreeMap<String, String>,
}

impl TypeIndex {
    fn build(types: &[PddlType], objects: &[(String, String)]) -> Self {
        Self {
            object_type: objects.iter().cloned().collect(),
            parent: types
                .iter()
                .filter_map(|typ| typ.parent.clone().map(|parent| (typ.name.clone(), parent)))
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
                Some(parent) => current = parent,
                None => return false,
            }
        }
    }
}

impl ExactClassicalProblem {
    pub fn build(
        domain: &Pddl31Domain,
        problem: &Pddl31Problem,
        max_ground_actions: usize,
    ) -> Result<Self, ExactClassicalError> {
        validate_scope(domain, problem)?;
        let objects = problem
            .objects
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let type_index = TypeIndex::build(&domain.types, &problem.objects);
        let mut actions = Vec::new();
        for schema in &domain.actions {
            ground_action_schema(schema, &objects, &type_index, &mut actions)?;
            if actions.len() > max_ground_actions {
                return Err(ExactClassicalError::GroundActionBoundExceeded {
                    limit: max_ground_actions,
                    observed: actions.len(),
                });
            }
        }
        let initial_facts = problem
            .init_atoms
            .iter()
            .map(atom_to_ground)
            .collect::<BTreeSet<_>>();
        let initial_functions = problem
            .init_fn_values
            .iter()
            .map(|(function, value)| (function_key(function), *value))
            .collect::<BTreeMap<_, _>>();
        Ok(Self {
            initial_facts,
            initial_functions,
            goal: problem.goal.clone(),
            actions,
            objects,
            type_index,
        })
    }

    /// Exact breadth-first search over the admitted classical semantics.
    pub fn find_plan(
        &self,
        max_depth: usize,
        max_states: usize,
    ) -> Result<Pddl8Tape, ExactClassicalError> {
        let initial = ExactState {
            facts: self.initial_facts.clone(),
            functions: self.initial_functions.clone(),
        };
        if eval_condition(
            &self.goal,
            &initial,
            &self.objects,
            &self.type_index,
        ) {
            return Ok(Pddl8Tape { ops: Vec::new() });
        }

        let mut queue = VecDeque::from([(initial.clone(), Vec::<usize>::new())]);
        let mut visited = HashSet::from([state_key(&initial)]);
        let mut depth_cut = false;

        while let Some((state, path)) = queue.pop_front() {
            if visited.len() > max_states {
                return Err(ExactClassicalError::SearchStateBoundExceeded { limit: max_states });
            }
            if path.len() >= max_depth {
                depth_cut = true;
                continue;
            }
            for (action_index, action) in self.actions.iter().enumerate() {
                let Some(next) = apply_action(action, &state, &self.objects, &self.type_index)? else {
                    continue;
                };
                let key = state_key(&next);
                if !visited.insert(key) {
                    continue;
                }
                let mut next_path = path.clone();
                next_path.push(action_index);
                if eval_condition(&self.goal, &next, &self.objects, &self.type_index) {
                    return self.path_to_tape(&next_path);
                }
                queue.push_back((next, next_path));
            }
        }

        if depth_cut {
            Err(ExactClassicalError::PlanDepthBoundExceeded { limit: max_depth })
        } else {
            Err(ExactClassicalError::NoPlan)
        }
    }

    fn path_to_tape(&self, path: &[usize]) -> Result<Pddl8Tape, ExactClassicalError> {
        if path.len() > 64 {
            return Err(ExactClassicalError::TapeFull);
        }
        let ops = path
            .iter()
            .enumerate()
            .map(|(index, action_index)| Pddl8TapeOp {
                index: index as u8,
                label: self.actions[*action_index].label.clone(),
                pred_mask: if index == 0 { 0 } else { 1u64 << (index - 1) },
                action: self.actions[*action_index].legacy_action.clone(),
            })
            .collect();
        Ok(Pddl8Tape { ops })
    }
}

fn validate_scope(
    domain: &Pddl31Domain,
    problem: &Pddl31Problem,
) -> Result<(), ExactClassicalError> {
    if !domain.durative_actions.is_empty() {
        return Err(ExactClassicalError::DurativeActionsUnsupported);
    }
    if !problem.timed_inits.is_empty() {
        return Err(ExactClassicalError::TimedInitialLiteralsUnsupported);
    }
    if !domain.processes.is_empty() {
        return Err(ExactClassicalError::ProcessesUnsupported);
    }
    if !domain.events.is_empty() {
        return Err(ExactClassicalError::EventsUnsupported);
    }
    if !domain.derived.is_empty() {
        return Err(ExactClassicalError::DerivedPredicatesUnsupported);
    }
    if !domain.constraints.is_empty() {
        return Err(ExactClassicalError::TrajectoryConstraintsUnsupported);
    }
    if !problem.preferences.is_empty() {
        return Err(ExactClassicalError::PreferencesUnsupported);
    }
    if problem.metric.is_some() {
        return Err(ExactClassicalError::MetricsUnsupported);
    }
    for action in &domain.actions {
        validate_condition(&action.precondition)?;
        for effect in &action.effect {
            validate_effect(effect)?;
        }
    }
    validate_condition(&problem.goal)
}

fn validate_condition(condition: &PddlCondition) -> Result<(), ExactClassicalError> {
    match condition {
        PddlCondition::Timed(_, _) => Err(ExactClassicalError::TimedConditionUnsupported),
        PddlCondition::Not(inner) => validate_condition(inner),
        PddlCondition::And(parts) | PddlCondition::Or(parts) => {
            parts.iter().try_for_each(validate_condition)
        }
        PddlCondition::Forall { body, .. } | PddlCondition::Exists { body, .. } => {
            validate_condition(body)
        }
        PddlCondition::Imply(left, right) => {
            validate_condition(left)?;
            validate_condition(right)
        }
        PddlCondition::Atom(_) | PddlCondition::Compare(_, _, _) => Ok(()),
    }
}

fn validate_effect(effect: &PddlEffect) -> Result<(), ExactClassicalError> {
    match effect {
        PddlEffect::Timed(_, _) => Err(ExactClassicalError::TimedEffectUnsupported),
        PddlEffect::Add(atom)
            if atom.pred == crate::parse::CONTINUOUS_EFFECT_SENTINEL_PRED =>
        {
            Err(ExactClassicalError::ContinuousEffectUnsupported)
        }
        PddlEffect::Add(atom) if atom.pred == crate::parse::OBJECT_FLUENT_SENTINEL_PRED => {
            Err(ExactClassicalError::ObjectFluentUnsupported)
        }
        PddlEffect::When { condition, effects } => {
            validate_condition(condition)?;
            effects.iter().try_for_each(validate_effect)
        }
        PddlEffect::Forall { effects, .. } => effects.iter().try_for_each(validate_effect),
        PddlEffect::Add(_) | PddlEffect::Del(_) | PddlEffect::Numeric(_) => Ok(()),
    }
}

fn ground_action_schema(
    schema: &Pddl31Action,
    objects: &[String],
    type_index: &TypeIndex,
    out: &mut Vec<ExactGroundAction>,
) -> Result<(), ExactClassicalError> {
    let candidates = schema
        .params
        .iter()
        .map(|(_, required)| {
            objects
                .iter()
                .filter(|object| type_index.satisfies(object, required))
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if candidates.iter().any(Vec::is_empty) {
        return Ok(());
    }
    enumerate_bindings(&schema.params, &candidates, 0, &mut BTreeMap::new(), &mut |binding| {
        let args = schema
            .params
            .iter()
            .map(|(parameter, _)| binding[parameter].clone())
            .collect::<Vec<_>>();
        let label = if args.is_empty() {
            schema.name.clone()
        } else {
            format!("{}({})", schema.name, args.join(","))
        };
        let condition = subst_condition(&schema.precondition, binding);
        let effects = schema
            .effect
            .iter()
            .map(|effect| subst_effect(effect, binding))
            .collect::<Vec<_>>();
        out.push(ExactGroundAction {
            schema_name: schema.name.clone(),
            label: label.clone(),
            args,
            legacy_action: legacy_action(&schema.name, &label, &condition, &effects),
            condition,
            effects,
        });
    });
    Ok(())
}

fn enumerate_bindings<F>(
    variables: &[(String, String)],
    candidates: &[Vec<String>],
    index: usize,
    binding: &mut BTreeMap<String, String>,
    callback: &mut F,
) where
    F: FnMut(&BTreeMap<String, String>),
{
    if index == variables.len() {
        callback(binding);
        return;
    }
    let variable = &variables[index].0;
    for object in &candidates[index] {
        binding.insert(variable.clone(), object.clone());
        enumerate_bindings(variables, candidates, index + 1, binding, callback);
        binding.remove(variable);
    }
}

fn apply_action(
    action: &ExactGroundAction,
    state: &ExactState,
    objects: &[String],
    type_index: &TypeIndex,
) -> Result<Option<ExactState>, ExactClassicalError> {
    if !eval_condition(&action.condition, state, objects, type_index) {
        return Ok(None);
    }
    let mut delta = EffectDelta::default();
    for effect in &action.effects {
        collect_effect(effect, state, objects, type_index, &mut delta)?;
    }
    let mut next = state.clone();
    for atom in delta.del {
        next.facts.remove(&atom);
    }
    next.facts.extend(delta.add);
    for (key, effect) in delta.numeric {
        let old = state.functions.get(&key).copied().unwrap_or(0.0);
        let value = match effect {
            GroundNumericEffect::Assign(value) => value,
            GroundNumericEffect::Increase(value) => old + value,
            GroundNumericEffect::Decrease(value) => old - value,
            GroundNumericEffect::ScaleUp(value) => old * value,
            GroundNumericEffect::ScaleDown(value) => {
                if value == 0.0 {
                    return Err(ExactClassicalError::DivisionByZero { function: key });
                }
                old / value
            }
        };
        next.functions.insert(key, value);
    }
    Ok(Some(next))
}

#[derive(Default)]
struct EffectDelta {
    add: BTreeSet<Pddl8GroundAtom>,
    del: BTreeSet<Pddl8GroundAtom>,
    numeric: BTreeMap<String, GroundNumericEffect>,
}

enum GroundNumericEffect {
    Assign(f64),
    Increase(f64),
    Decrease(f64),
    ScaleUp(f64),
    ScaleDown(f64),
}

fn collect_effect(
    effect: &PddlEffect,
    state: &ExactState,
    objects: &[String],
    type_index: &TypeIndex,
    delta: &mut EffectDelta,
) -> Result<(), ExactClassicalError> {
    match effect {
        PddlEffect::Add(atom) => {
            delta.add.insert(atom_to_ground(atom));
        }
        PddlEffect::Del(atom) => {
            delta.del.insert(atom_to_ground(atom));
        }
        PddlEffect::Numeric(effect) => {
            let (function, grounded) = ground_numeric_effect(effect, state);
            if delta.numeric.insert(function.clone(), grounded).is_some() {
                return Err(ExactClassicalError::ConflictingNumericEffects { function });
            }
        }
        PddlEffect::When { condition, effects } => {
            if eval_condition(condition, state, objects, type_index) {
                for nested in effects {
                    collect_effect(nested, state, objects, type_index, delta)?;
                }
            }
        }
        PddlEffect::Forall { vars, effects } => {
            let candidates = vars
                .iter()
                .map(|(_, required)| {
                    objects
                        .iter()
                        .filter(|object| type_index.satisfies(object, required))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            enumerate_bindings(vars, &candidates, 0, &mut BTreeMap::new(), &mut |binding| {
                for nested in effects {
                    let grounded = subst_effect(nested, binding);
                    // Scope validation has already removed every error except
                    // numeric conflicts/division, which are handled after this
                    // enumeration. Collection errors are impossible here.
                    collect_effect(&grounded, state, objects, type_index, delta)
                        .expect("validated quantified effect");
                }
            });
        }
        PddlEffect::Timed(_, _) => return Err(ExactClassicalError::TimedEffectUnsupported),
    }
    Ok(())
}

fn ground_numeric_effect(
    effect: &NumericEffect,
    state: &ExactState,
) -> (String, GroundNumericEffect) {
    match effect {
        NumericEffect::Assign(function, expression) => (
            function_key(function),
            GroundNumericEffect::Assign(eval_numeric(expression, state)),
        ),
        NumericEffect::Increase(function, expression) => (
            function_key(function),
            GroundNumericEffect::Increase(eval_numeric(expression, state)),
        ),
        NumericEffect::Decrease(function, expression) => (
            function_key(function),
            GroundNumericEffect::Decrease(eval_numeric(expression, state)),
        ),
        NumericEffect::ScaleUp(function, expression) => (
            function_key(function),
            GroundNumericEffect::ScaleUp(eval_numeric(expression, state)),
        ),
        NumericEffect::ScaleDown(function, expression) => (
            function_key(function),
            GroundNumericEffect::ScaleDown(eval_numeric(expression, state)),
        ),
    }
}

fn eval_condition(
    condition: &PddlCondition,
    state: &ExactState,
    objects: &[String],
    type_index: &TypeIndex,
) -> bool {
    match condition {
        PddlCondition::Atom(atom) if atom.pred == "=" && atom.args.len() == 2 => {
            atom.args[0] == atom.args[1]
        }
        PddlCondition::Atom(atom) => state.facts.contains(&atom_to_ground(atom)),
        PddlCondition::Not(inner) => !eval_condition(inner, state, objects, type_index),
        PddlCondition::And(parts) => parts
            .iter()
            .all(|part| eval_condition(part, state, objects, type_index)),
        PddlCondition::Or(parts) => parts
            .iter()
            .any(|part| eval_condition(part, state, objects, type_index)),
        PddlCondition::Imply(left, right) => {
            !eval_condition(left, state, objects, type_index)
                || eval_condition(right, state, objects, type_index)
        }
        PddlCondition::Forall { vars, body } => eval_quantified(
            vars,
            body,
            state,
            objects,
            type_index,
            true,
        ),
        PddlCondition::Exists { vars, body } => eval_quantified(
            vars,
            body,
            state,
            objects,
            type_index,
            false,
        ),
        PddlCondition::Compare(left, operator, right) => {
            let left = eval_numeric(left, state);
            let right = eval_numeric(right, state);
            match operator {
                CompareOp::Ge => left >= right,
                CompareOp::Le => left <= right,
                CompareOp::Gt => left > right,
                CompareOp::Lt => left < right,
                CompareOp::Eq => (left - right).abs() < 1e-9,
            }
        }
        PddlCondition::Timed(_, inner) => eval_condition(inner, state, objects, type_index),
    }
}

fn eval_quantified(
    vars: &[(String, String)],
    body: &PddlCondition,
    state: &ExactState,
    objects: &[String],
    type_index: &TypeIndex,
    require_all: bool,
) -> bool {
    let candidates = vars
        .iter()
        .map(|(_, required)| {
            objects
                .iter()
                .filter(|object| type_index.satisfies(object, required))
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut results = Vec::new();
    enumerate_bindings(vars, &candidates, 0, &mut BTreeMap::new(), &mut |binding| {
        results.push(eval_condition(
            &subst_condition(body, binding),
            state,
            objects,
            type_index,
        ));
    });
    if require_all {
        results.into_iter().all(|result| result)
    } else {
        results.into_iter().any(|result| result)
    }
}

fn eval_numeric(expression: &NumericExpr, state: &ExactState) -> f64 {
    match expression {
        NumericExpr::Number(value) => *value,
        NumericExpr::FunctionTerm(name, args) => {
            let key = if args.is_empty() {
                name.clone()
            } else {
                format!("{}({})", name, args.join(","))
            };
            state.functions.get(&key).copied().unwrap_or(0.0)
        }
        NumericExpr::BinOp { op, lhs, rhs } => {
            let left = eval_numeric(lhs, state);
            let right = eval_numeric(rhs, state);
            match op {
                NumericOp::Add => left + right,
                NumericOp::Sub => left - right,
                NumericOp::Mul => left * right,
                NumericOp::Div => {
                    if right == 0.0 {
                        f64::NAN
                    } else {
                        left / right
                    }
                }
            }
        }
        NumericExpr::Neg(inner) => -eval_numeric(inner, state),
    }
}

fn legacy_action(
    schema_name: &str,
    label: &str,
    condition: &PddlCondition,
    effects: &[PddlEffect],
) -> Pddl8GroundAction {
    let mut preconditions = Vec::new();
    collect_positive_atoms(condition, &mut preconditions);
    let mut add_effects = Vec::new();
    let mut del_effects = Vec::new();
    for effect in effects {
        match effect {
            PddlEffect::Add(atom) => add_effects.push(atom_to_ground(atom)),
            PddlEffect::Del(atom) => del_effects.push(atom_to_ground(atom)),
            _ => {}
        }
    }
    Pddl8GroundAction {
        schema_name: schema_name.to_string(),
        label: label.to_string(),
        preconditions,
        add_effects,
        del_effects,
    }
}

fn collect_positive_atoms(condition: &PddlCondition, out: &mut Vec<Pddl8GroundAtom>) {
    match condition {
        PddlCondition::Atom(atom) if atom.pred != "=" => out.push(atom_to_ground(atom)),
        PddlCondition::And(parts) => {
            for part in parts {
                collect_positive_atoms(part, out);
            }
        }
        _ => {}
    }
}

fn atom_to_ground(atom: &Pddl8Atom) -> Pddl8GroundAtom {
    Pddl8GroundAtom {
        pred: atom.pred.clone(),
        args: atom.args.clone(),
    }
}

fn function_key(function: &PddlFunction) -> String {
    if function.params.is_empty() {
        function.name.clone()
    } else {
        format!("{}({})", function.name, function.params.join(","))
    }
}

fn state_key(state: &ExactState) -> Vec<u8> {
    let mut bytes = Vec::new();
    for atom in &state.facts {
        bytes.extend_from_slice(atom.pred.as_bytes());
        bytes.push(0);
        for arg in &atom.args {
            bytes.extend_from_slice(arg.as_bytes());
            bytes.push(0);
        }
        bytes.push(0xff);
    }
    bytes.push(0xfe);
    for (key, value) in &state.functions {
        bytes.extend_from_slice(key.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    bytes
}

fn subst_atom(atom: &Pddl8Atom, binding: &BTreeMap<String, String>) -> Pddl8Atom {
    Pddl8Atom {
        pred: atom.pred.clone(),
        args: atom
            .args
            .iter()
            .map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone()))
            .collect(),
    }
}

fn subst_function(
    function: &PddlFunction,
    binding: &BTreeMap<String, String>,
) -> PddlFunction {
    PddlFunction {
        name: function.name.clone(),
        params: function
            .params
            .iter()
            .map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone()))
            .collect(),
    }
}

fn subst_numeric(
    expression: &NumericExpr,
    binding: &BTreeMap<String, String>,
) -> NumericExpr {
    match expression {
        NumericExpr::Number(value) => NumericExpr::Number(*value),
        NumericExpr::FunctionTerm(name, args) => NumericExpr::FunctionTerm(
            name.clone(),
            args.iter()
                .map(|arg| binding.get(arg).cloned().unwrap_or_else(|| arg.clone()))
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

fn subst_condition(
    condition: &PddlCondition,
    binding: &BTreeMap<String, String>,
) -> PddlCondition {
    match condition {
        PddlCondition::Atom(atom) => PddlCondition::Atom(subst_atom(atom, binding)),
        PddlCondition::Not(inner) => {
            PddlCondition::Not(Box::new(subst_condition(inner, binding)))
        }
        PddlCondition::And(parts) => PddlCondition::And(
            parts
                .iter()
                .map(|part| subst_condition(part, binding))
                .collect(),
        ),
        PddlCondition::Or(parts) => PddlCondition::Or(
            parts
                .iter()
                .map(|part| subst_condition(part, binding))
                .collect(),
        ),
        PddlCondition::Forall { vars, body } => PddlCondition::Forall {
            vars: vars.clone(),
            body: Box::new(subst_condition(body, binding)),
        },
        PddlCondition::Exists { vars, body } => PddlCondition::Exists {
            vars: vars.clone(),
            body: Box::new(subst_condition(body, binding)),
        },
        PddlCondition::Imply(left, right) => PddlCondition::Imply(
            Box::new(subst_condition(left, binding)),
            Box::new(subst_condition(right, binding)),
        ),
        PddlCondition::Timed(specifier, inner) => PddlCondition::Timed(
            *specifier,
            Box::new(subst_condition(inner, binding)),
        ),
        PddlCondition::Compare(left, operator, right) => PddlCondition::Compare(
            subst_numeric(left, binding),
            *operator,
            subst_numeric(right, binding),
        ),
    }
}

fn subst_effect(effect: &PddlEffect, binding: &BTreeMap<String, String>) -> PddlEffect {
    match effect {
        PddlEffect::Add(atom) => PddlEffect::Add(subst_atom(atom, binding)),
        PddlEffect::Del(atom) => PddlEffect::Del(subst_atom(atom, binding)),
        PddlEffect::Numeric(effect) => PddlEffect::Numeric(match effect {
            NumericEffect::Assign(function, expression) => NumericEffect::Assign(
                subst_function(function, binding),
                subst_numeric(expression, binding),
            ),
            NumericEffect::Increase(function, expression) => NumericEffect::Increase(
                subst_function(function, binding),
                subst_numeric(expression, binding),
            ),
            NumericEffect::Decrease(function, expression) => NumericEffect::Decrease(
                subst_function(function, binding),
                subst_numeric(expression, binding),
            ),
            NumericEffect::ScaleUp(function, expression) => NumericEffect::ScaleUp(
                subst_function(function, binding),
                subst_numeric(expression, binding),
            ),
            NumericEffect::ScaleDown(function, expression) => NumericEffect::ScaleDown(
                subst_function(function, binding),
                subst_numeric(expression, binding),
            ),
        }),
        PddlEffect::Timed(specifier, inner) => {
            PddlEffect::Timed(*specifier, Box::new(subst_effect(inner, binding)))
        }
        PddlEffect::Forall { vars, effects } => PddlEffect::Forall {
            vars: vars.clone(),
            effects: effects
                .iter()
                .map(|effect| subst_effect(effect, binding))
                .collect(),
        },
        PddlEffect::When { condition, effects } => PddlEffect::When {
            condition: subst_condition(condition, binding),
            effects: effects
                .iter()
                .map(|effect| subst_effect(effect, binding))
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain31_from_pddl, problem31_from_pddl};

    use super::*;

    fn solve(domain: &str, problem: &str) -> Result<Pddl8Tape, ExactClassicalError> {
        let domain = domain31_from_pddl(domain).unwrap();
        let problem = problem31_from_pddl(problem).unwrap();
        ExactClassicalProblem::build(&domain, &problem, 1_000)?.find_plan(16, 10_000)
    }

    #[test]
    fn negative_preconditions_are_load_bearing() {
        let domain = "(define (domain d) (:requirements :strips :negative-preconditions) \
            (:predicates (locked) (done)) \
            (:action finish :parameters () :precondition (not (locked)) :effect (done)))";
        let open = "(define (problem p) (:domain d) (:init) (:goal (done)))";
        let locked = "(define (problem p) (:domain d) (:init (locked)) (:goal (done)))";
        assert_eq!(solve(domain, open).unwrap().ops.len(), 1);
        assert_eq!(solve(domain, locked), Err(ExactClassicalError::NoPlan));
    }

    #[test]
    fn disjunction_and_object_equality_are_exact() {
        let domain = "(define (domain d) (:requirements :adl :typing) (:types item) \
            (:predicates (ready ?x - item) (backup ?x - item) (done ?x - item)) \
            (:action finish :parameters (?x - item ?y - item) \
              :precondition (and (not (= ?x ?y)) (or (ready ?x) (backup ?x))) \
              :effect (done ?x)))";
        let problem = "(define (problem p) (:domain d) (:objects a b - item) \
            (:init (backup a)) (:goal (done a)))";
        let tape = solve(domain, problem).unwrap();
        assert_eq!(tape.ops.len(), 1);
        assert_eq!(tape.ops[0].label, "finish(a,b)");
    }

    #[test]
    fn quantified_precondition_and_effect_are_exact() {
        let domain = "(define (domain d) (:requirements :adl :typing) (:types item) \
            (:predicates (ready ?x - item) (done ?x - item)) \
            (:action finish-all :parameters () \
              :precondition (forall (?x - item) (ready ?x)) \
              :effect (forall (?x - item) (when (ready ?x) (done ?x)))))";
        let problem = "(define (problem p) (:domain d) (:objects a b - item) \
            (:init (ready a) (ready b)) \
            (:goal (and (done a) (done b))))";
        assert_eq!(solve(domain, problem).unwrap().ops.len(), 1);
    }

    #[test]
    fn conditional_effect_false_branch_does_not_fire() {
        let domain = "(define (domain d) (:requirements :conditional-effects) \
            (:predicates (enabled) (done) (acted)) \
            (:action act :parameters () :precondition () \
              :effect (and (acted) (when (enabled) (done)))))";
        let problem = "(define (problem p) (:domain d) (:init) (:goal (done)))";
        assert_eq!(solve(domain, problem), Err(ExactClassicalError::NoPlan));
    }

    #[test]
    fn numeric_precondition_and_effect_are_exact() {
        let domain = "(define (domain d) (:requirements :strips :numeric-fluents) \
            (:predicates (done)) (:functions (fuel)) \
            (:action consume :parameters () :precondition (>= (fuel) 2) \
              :effect (and (decrease (fuel) 2) (done))))";
        let enough = "(define (problem p) (:domain d) (:init (= (fuel) 2)) (:goal (done)))";
        let insufficient =
            "(define (problem p) (:domain d) (:init (= (fuel) 1)) (:goal (done)))";
        assert_eq!(solve(domain, enough).unwrap().ops.len(), 1);
        assert_eq!(solve(domain, insufficient), Err(ExactClassicalError::NoPlan));
    }

    #[test]
    fn temporal_surface_is_typed_refusal_not_flattened() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :durative-actions) (:predicates (done)) \
             (:durative-action a :parameters () :duration (= ?duration 1) \
               :condition () :effect (at end (done))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(
            "(define (problem p) (:domain d) (:init) (:goal (done)))",
        )
        .unwrap();
        assert_eq!(
            ExactClassicalProblem::build(&domain, &problem, 10).unwrap_err(),
            ExactClassicalError::DurativeActionsUnsupported
        );
    }
}
