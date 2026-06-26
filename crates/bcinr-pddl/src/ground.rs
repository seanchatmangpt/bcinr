//! PDDL8 grounding and forward-search plan finding.

use crate::error::Pddl8Error;
use std::collections::{BTreeSet, HashMap};
use wasm4pm_compat::pddl::{
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8GroundAction, Pddl8GroundAtom, Pddl8Problem,
    Pddl8Tape, PDDL8_MAX_GROUND, PDDL8_MAX_PLAN_DEPTH,
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
