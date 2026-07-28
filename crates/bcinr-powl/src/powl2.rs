//! Recursive POWL 2.0 model and compiler.
//!
//! POWL 2.0 extends the original activity/partial-order/do-redo constructors
//! with choice graphs. A choice graph contains child POWL models as vertices,
//! designated start/end vertices, and directed edges; every vertex must lie on
//! at least one start-to-end path. Compilation selects one admitted path under
//! an explicit policy and lowers the resulting recursive model to the same
//! executable `tape::v2::PowlTape` consumed by `scheduler_v2`.

use std::collections::{BTreeSet, VecDeque};

use crate::tape::v2::{OpKind, Powl64Op, PowlTape};

/// Hard bound shared with the 64-slot POWL v2 tape.
pub const POWL2_MAX_NODES: usize = 64;
/// Hard bound on choice-graph path traversal, preventing an unbounded cycle.
pub const POWL2_MAX_CHOICE_STEPS: usize = 64;

/// One recursive POWL 2.0 model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Powl2Model {
    Activity(String),
    Silent,
    Sequence(Vec<Powl2Model>),
    PartialOrder {
        children: Vec<Powl2Model>,
        /// Strict precedence edges over child indices.
        edges: Vec<(usize, usize)>,
    },
    ChoiceGraph {
        children: Vec<Powl2Model>,
        edges: Vec<(usize, usize)>,
        start: usize,
        end: usize,
    },
    DoRedo {
        body: Box<Powl2Model>,
        redo: Box<Powl2Model>,
        /// Maximum admitted redo count. Zero means no redo, not unbounded.
        max_redos: u8,
    },
}

/// Typed model/compile refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Powl2Error {
    EmptySequence,
    EmptyPartialOrder,
    EmptyChoiceGraph,
    /// Fewer children than the definition admits. Def 3.7 states `n >= 2` for
    /// both the partial order and the choice graph; a one-child model is not
    /// something the paper defines.
    ArityBelowMinimum {
        constructor: &'static str,
        found: usize,
        minimum: usize,
    },
    InvalidEdge {
        from: usize,
        to: usize,
        len: usize,
    },
    InvalidChoiceEndpoint {
        endpoint: usize,
        len: usize,
    },
    /// Def 3.6 requires exactly one node with no incoming edge (`start`) and
    /// exactly one with no outgoing edge (`finish`).
    ChoiceEndpointNotUnique {
        node: usize,
        role: &'static str,
    },
    PartialOrderCycle,
    ChoiceNodeOffStartEndPath {
        node: usize,
    },
    ChoicePathBoundExceeded {
        limit: usize,
    },
    ChoicePolicyReturnedInvalidSuccessor {
        from: usize,
        selected: usize,
    },
    TapeFull,
    LabelSlabFull,
}

impl std::fmt::Display for Powl2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySequence => write!(f, "POWL 2.0 sequence has no children"),
            Self::EmptyPartialOrder => write!(f, "POWL 2.0 partial order has no children"),
            Self::EmptyChoiceGraph => write!(f, "POWL 2.0 choice graph has no vertices"),
            Self::ArityBelowMinimum {
                constructor,
                found,
                minimum,
            } => write!(
                f,
                "POWL 2.0 {constructor} has {found} children; Def 3.7 requires at least {minimum}"
            ),
            Self::InvalidEdge { from, to, len } => {
                write!(f, "POWL 2.0 edge {from}->{to} is outside 0..{len}")
            }
            Self::InvalidChoiceEndpoint { endpoint, len } => write!(
                f,
                "POWL 2.0 choice endpoint {endpoint} is not the Def 3.6 sentinel \
                 encoding for {len} children (expected start={len}, finish={})",
                len + 1
            ),
            Self::ChoiceEndpointNotUnique { node, role } => write!(
                f,
                "POWL 2.0 choice graph has a second {role} at node {node}; Def 3.6 \
                 requires exactly one"
            ),
            Self::PartialOrderCycle => write!(f, "POWL 2.0 partial order contains a cycle"),
            Self::ChoiceNodeOffStartEndPath { node } => write!(
                f,
                "POWL 2.0 choice vertex {node} is not on a start-to-end path"
            ),
            Self::ChoicePathBoundExceeded { limit } => {
                write!(f, "POWL 2.0 choice traversal exceeded {limit} steps")
            }
            Self::ChoicePolicyReturnedInvalidSuccessor { from, selected } => write!(
                f,
                "choice policy selected non-successor {selected} from vertex {from}"
            ),
            Self::TapeFull => write!(f, "POWL 2.0 compilation exceeded 64 tape slots"),
            Self::LabelSlabFull => write!(f, "POWL 2.0 activity labels exceeded slab capacity"),
        }
    }
}

impl std::error::Error for Powl2Error {}

/// Explicit policy for generalized choices and do-redo repetition.
pub trait Powl2ChoicePolicy {
    /// Select one outgoing successor from `successors`.
    fn select_successor(
        &mut self,
        graph_depth: usize,
        current: usize,
        successors: &[usize],
    ) -> usize;

    /// Return true to execute another `redo -> body` cycle.
    fn select_redo(&mut self, loop_depth: usize, completed_redos: u8, max_redos: u8) -> bool;
}

/// Deterministic policy: lowest-index successor and no loop redo.
#[derive(Debug, Default)]
pub struct LowestIndexPolicy;

impl Powl2ChoicePolicy for LowestIndexPolicy {
    fn select_successor(
        &mut self,
        _graph_depth: usize,
        _current: usize,
        successors: &[usize],
    ) -> usize {
        successors.iter().copied().min().unwrap_or(usize::MAX)
    }

    fn select_redo(&mut self, _loop_depth: usize, _completed_redos: u8, _max_redos: u8) -> bool {
        false
    }
}

/// Choice evidence emitted during compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceSelection {
    pub graph_depth: usize,
    pub from: usize,
    pub to: usize,
}

/// Executable compiled POWL 2.0 model plus selected-choice evidence.
#[derive(Debug)]
pub struct CompiledPowl2 {
    pub tape: PowlTape,
    pub selected_choices: Vec<ChoiceSelection>,
    pub activity_slots: Vec<(u8, u16)>,
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    entries: u64,
    exits: u64,
}

struct Compiler<'a, P> {
    tape: PowlTape,
    policy: &'a mut P,
    selected_choices: Vec<ChoiceSelection>,
    activity_slots: Vec<(u8, u16)>,
}

/// Validate and compile a recursive POWL 2.0 model.
pub fn compile_powl2<P: Powl2ChoicePolicy>(
    model: &Powl2Model,
    policy: &mut P,
) -> Result<CompiledPowl2, Powl2Error> {
    validate_powl2(model)?;
    let mut compiler = Compiler {
        tape: PowlTape::new(),
        policy,
        selected_choices: Vec::new(),
        activity_slots: Vec::new(),
    };
    let root = compiler.compile(model, 0)?;
    let entry = compiler.push_op(OpKind::Silent, None)?;
    wire(&mut compiler.tape, 1u64 << entry, root.entries);
    let exit = compiler.push_op(OpKind::Silent, None)?;
    wire(&mut compiler.tape, root.exits, 1u64 << exit);
    compiler.tape.entry_op = entry;
    compiler.tape.exit_op = exit;
    recompute_fan_out(&mut compiler.tape);
    Ok(CompiledPowl2 {
        tape: compiler.tape,
        selected_choices: compiler.selected_choices,
        activity_slots: compiler.activity_slots,
    })
}

/// Validate every recursive constructor without selecting a path.
pub fn validate_powl2(model: &Powl2Model) -> Result<(), Powl2Error> {
    match model {
        Powl2Model::Activity(_) | Powl2Model::Silent => Ok(()),
        Powl2Model::Sequence(children) => {
            if children.is_empty() {
                return Err(Powl2Error::EmptySequence);
            }
            children.iter().try_for_each(validate_powl2)
        }
        Powl2Model::PartialOrder { children, edges } => {
            if children.is_empty() {
                return Err(Powl2Error::EmptyPartialOrder);
            }
            if children.len() < crate::generated_arity::PARTIAL_ORDER_MIN_CHILDREN {
                return Err(Powl2Error::ArityBelowMinimum {
                    constructor: "partial order",
                    found: children.len(),
                    minimum: crate::generated_arity::PARTIAL_ORDER_MIN_CHILDREN,
                });
            }
            validate_edges(children.len(), edges)?;
            validate_acyclic(children.len(), edges)?;
            children.iter().try_for_each(validate_powl2)
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => {
            if children.is_empty() {
                return Err(Powl2Error::EmptyChoiceGraph);
            }
            if children.len() < crate::generated_arity::CHOICE_GRAPH_MIN_CHILDREN {
                return Err(Powl2Error::ArityBelowMinimum {
                    constructor: "choice graph",
                    found: children.len(),
                    minimum: crate::generated_arity::CHOICE_GRAPH_MIN_CHILDREN,
                });
            }
            // Def 3.6: `N = X union {start, finish}` with `start, finish NOT
            // IN X`, and the sentinels are "not included in the execution
            // sequence". They are therefore NOT child indices -- the encoding
            // is `start = n`, `finish = n + 1` over `n` children, which is
            // what `wf_to_powl::execution_flow` emits and what
            // `language.rs`'s `node < n_children` guard implements.
            //
            // This previously required `start < children.len()`, i.e. it
            // rejected the paper-conformant encoding, and so rejected every
            // choice graph the SM branch of Algorithm 3 can produce. No test
            // routed converter output through here, which is why it stood.
            let n = children.len();
            if *start != n || *end != n + 1 {
                return Err(Powl2Error::InvalidChoiceEndpoint {
                    endpoint: if *start != n { *start } else { *end },
                    len: n,
                });
            }
            // Edges may name the two sentinels, so the bound is `n + 2` here.
            // `PartialOrder` keeps the tighter `n` bound -- it has no
            // sentinels.
            validate_edges(n + 2, edges)?;
            validate_choice_unique_endpoints(n, edges, *start, *end)?;
            validate_choice_coverage(n, edges, *start, *end)?;
            children.iter().try_for_each(validate_powl2)
        }
        Powl2Model::DoRedo { body, redo, .. } => {
            validate_powl2(body)?;
            validate_powl2(redo)
        }
    }
}

impl<P: Powl2ChoicePolicy> Compiler<'_, P> {
    fn compile(&mut self, model: &Powl2Model, depth: usize) -> Result<Segment, Powl2Error> {
        match model {
            Powl2Model::Activity(label) => {
                let slot = self.push_op(OpKind::Activity, Some(label))?;
                Ok(singleton(slot))
            }
            Powl2Model::Silent => {
                let slot = self.push_op(OpKind::Silent, None)?;
                Ok(singleton(slot))
            }
            Powl2Model::Sequence(children) => self.compile_sequence(children, depth + 1),
            Powl2Model::PartialOrder { children, edges } => {
                self.compile_partial_order(children, edges, depth + 1)
            }
            Powl2Model::ChoiceGraph {
                children,
                edges,
                start,
                end,
            } => self.compile_choice_graph(children, edges, *start, *end, depth + 1),
            Powl2Model::DoRedo {
                body,
                redo,
                max_redos,
            } => {
                let mut segment = self.compile(body, depth + 1)?;
                let mut completed_redos = 0u8;
                while completed_redos < *max_redos
                    && self.policy.select_redo(depth, completed_redos, *max_redos)
                {
                    let redo_segment = self.compile(redo, depth + 1)?;
                    wire(&mut self.tape, segment.exits, redo_segment.entries);
                    let next_body = self.compile(body, depth + 1)?;
                    wire(&mut self.tape, redo_segment.exits, next_body.entries);
                    segment.exits = next_body.exits;
                    completed_redos = completed_redos.saturating_add(1);
                }
                Ok(segment)
            }
        }
    }

    fn compile_sequence(
        &mut self,
        children: &[Powl2Model],
        depth: usize,
    ) -> Result<Segment, Powl2Error> {
        let mut segment = self.compile(&children[0], depth)?;
        for child in &children[1..] {
            let next = self.compile(child, depth)?;
            wire(&mut self.tape, segment.exits, next.entries);
            segment.exits = next.exits;
        }
        Ok(segment)
    }

    fn compile_partial_order(
        &mut self,
        children: &[Powl2Model],
        edges: &[(usize, usize)],
        depth: usize,
    ) -> Result<Segment, Powl2Error> {
        let segments = children
            .iter()
            .map(|child| self.compile(child, depth))
            .collect::<Result<Vec<_>, _>>()?;
        let mut incoming = vec![false; children.len()];
        let mut outgoing = vec![false; children.len()];
        for &(from, to) in edges {
            wire(&mut self.tape, segments[from].exits, segments[to].entries);
            outgoing[from] = true;
            incoming[to] = true;
        }
        let entries = segments
            .iter()
            .enumerate()
            .filter(|(index, _)| !incoming[*index])
            .fold(0u64, |mask, (_, segment)| mask | segment.entries);
        let exits = segments
            .iter()
            .enumerate()
            .filter(|(index, _)| !outgoing[*index])
            .fold(0u64, |mask, (_, segment)| mask | segment.exits);
        Ok(Segment { entries, exits })
    }

    fn compile_choice_graph(
        &mut self,
        children: &[Powl2Model],
        edges: &[(usize, usize)],
        start: usize,
        end: usize,
        depth: usize,
    ) -> Result<Segment, Powl2Error> {
        let mut current = start;
        let mut path = vec![start];
        for _ in 0..POWL2_MAX_CHOICE_STEPS {
            if current == end {
                // Def 3.6: the sentinels "are not included in the execution
                // sequence", so drop them before compiling the traversed
                // path. Same rule `language.rs`'s `node < n_children` guard
                // applies on the denotational side.
                let executed: Vec<usize> = path
                    .iter()
                    .copied()
                    .filter(|&n| n < children.len())
                    .collect();
                return self.compile_sequence_indices(children, &executed, depth);
            }
            let mut successors = edges
                .iter()
                .filter_map(|&(from, to)| (from == current).then_some(to))
                .collect::<Vec<_>>();
            successors.sort_unstable();
            successors.dedup();
            let selected = self.policy.select_successor(depth, current, &successors);
            if !successors.contains(&selected) {
                return Err(Powl2Error::ChoicePolicyReturnedInvalidSuccessor {
                    from: current,
                    selected,
                });
            }
            self.selected_choices.push(ChoiceSelection {
                graph_depth: depth,
                from: current,
                to: selected,
            });
            current = selected;
            path.push(current);
        }
        Err(Powl2Error::ChoicePathBoundExceeded {
            limit: POWL2_MAX_CHOICE_STEPS,
        })
    }

    fn compile_sequence_indices(
        &mut self,
        children: &[Powl2Model],
        path: &[usize],
        depth: usize,
    ) -> Result<Segment, Powl2Error> {
        // A start->finish path touching no real child contributes nothing to
        // the execution sequence; compile it as a silent step rather than
        // indexing an empty slice.
        let Some(&first) = path.first() else {
            return self.compile(&Powl2Model::Silent, depth);
        };
        let mut segment = self.compile(&children[first], depth)?;
        for &index in &path[1..] {
            let next = self.compile(&children[index], depth)?;
            wire(&mut self.tape, segment.exits, next.entries);
            segment.exits = next.exits;
        }
        Ok(segment)
    }

    fn push_op(&mut self, kind: OpKind, label: Option<&str>) -> Result<u8, Powl2Error> {
        let mut op = Powl64Op::silent();
        op.op_kind = kind;
        let slot = self.tape.push(op).ok_or(Powl2Error::TapeFull)?;
        if let Some(label) = label {
            let offset = self.tape.label_slab.intern(label);
            if offset == u16::MAX {
                return Err(Powl2Error::LabelSlabFull);
            }
            self.activity_slots.push((slot, offset));
        }
        Ok(slot)
    }
}

fn singleton(slot: u8) -> Segment {
    let bit = 1u64 << slot;
    Segment {
        entries: bit,
        exits: bit,
    }
}

fn wire(tape: &mut PowlTape, from_mask: u64, to_mask: u64) {
    let mut from = from_mask;
    while from != 0 {
        let index = from.trailing_zeros() as usize;
        from &= from - 1;
        tape.ops[index].succ_mask |= to_mask;
    }
    let mut to = to_mask;
    while to != 0 {
        let index = to.trailing_zeros() as usize;
        to &= to - 1;
        tape.ops[index].pred_mask |= from_mask;
    }
}

fn recompute_fan_out(tape: &mut PowlTape) {
    for index in 0..tape.len as usize {
        tape.ops[index].fan_out = tape.ops[index].succ_mask.count_ones() as u8;
    }
}

fn validate_edges(len: usize, edges: &[(usize, usize)]) -> Result<(), Powl2Error> {
    for &(from, to) in edges {
        if from >= len || to >= len {
            return Err(Powl2Error::InvalidEdge { from, to, len });
        }
    }
    Ok(())
}

fn validate_acyclic(len: usize, edges: &[(usize, usize)]) -> Result<(), Powl2Error> {
    let mut indegree = vec![0usize; len];
    let mut outgoing = vec![Vec::new(); len];
    for &(from, to) in edges {
        indegree[to] += 1;
        outgoing[from].push(to);
    }
    let mut queue = VecDeque::new();
    for (index, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            queue.push_back(index);
        }
    }
    let mut visited = 0usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        for &next in &outgoing[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                queue.push_back(next);
            }
        }
    }
    if visited == len {
        Ok(())
    } else {
        Err(Powl2Error::PartialOrderCycle)
    }
}

/// Def 3.6 bullets 3 and 4: `start` is the *unique* node with no incoming
/// edge, `finish` the unique node with no outgoing edge.
///
/// Nothing checked these, so a graph with a second source or sink passed
/// validation and was not a choice graph. `wf_net.rs:100-119` already applies
/// exactly this test to WF-nets; this is the same shape over `0..n` plus the
/// two sentinels.
fn validate_choice_unique_endpoints(
    n: usize,
    edges: &[(usize, usize)],
    start: usize,
    end: usize,
) -> Result<(), Powl2Error> {
    for node in 0..n {
        if !edges.iter().any(|&(_, to)| to == node) {
            return Err(Powl2Error::ChoiceEndpointNotUnique {
                node,
                role: "source",
            });
        }
        if !edges.iter().any(|&(from, _)| from == node) {
            return Err(Powl2Error::ChoiceEndpointNotUnique { node, role: "sink" });
        }
    }
    if edges.iter().any(|&(_, to)| to == start) {
        return Err(Powl2Error::ChoiceEndpointNotUnique {
            node: start,
            role: "source",
        });
    }
    if edges.iter().any(|&(from, _)| from == end) {
        return Err(Powl2Error::ChoiceEndpointNotUnique {
            node: end,
            role: "sink",
        });
    }
    Ok(())
}

fn validate_choice_coverage(
    len: usize,
    edges: &[(usize, usize)],
    start: usize,
    end: usize,
) -> Result<(), Powl2Error> {
    // Traversal arrays must span the sentinels; the coverage loop below still
    // only requires the `n` real children to lie on a start->finish path.
    let forward = reachable(len + 2, edges, start, false);
    let backward = reachable(len + 2, edges, end, true);
    for node in 0..len {
        if !forward.contains(&node) || !backward.contains(&node) {
            return Err(Powl2Error::ChoiceNodeOffStartEndPath { node });
        }
    }
    Ok(())
}

fn reachable(
    len: usize,
    edges: &[(usize, usize)],
    origin: usize,
    reverse: bool,
) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([origin]);
    while let Some(node) = queue.pop_front() {
        if !seen.insert(node) {
            continue;
        }
        for &(from, to) in edges {
            let (source, target) = if reverse { (to, from) } else { (from, to) };
            if source == node && target < len && !seen.contains(&target) {
                queue.push_back(target);
            }
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use crate::scheduler::StableMaximalSelector;
    use crate::scheduler_v2::{execute_v2, PowlV2TickOutcome};
    use crate::tape::v2::ConcurrencyGuardTable;

    use super::*;

    fn labels(compiled: &CompiledPowl2) -> Vec<&str> {
        compiled
            .activity_slots
            .iter()
            .map(|(_, offset)| compiled.tape.label_slab.get(*offset))
            .collect()
    }

    #[test]
    fn original_powl_partial_order_runs_independent_children_together() {
        let model = Powl2Model::Sequence(vec![
            Powl2Model::PartialOrder {
                children: vec![
                    Powl2Model::Activity("a".into()),
                    Powl2Model::Activity("b".into()),
                ],
                edges: vec![],
            },
            Powl2Model::Activity("c".into()),
        ]);
        let compiled = compile_powl2(&model, &mut LowestIndexPolicy).unwrap();
        let mut selector = StableMaximalSelector;
        let (state, outcome) = execute_v2(
            &compiled.tape,
            &mut selector,
            &ConcurrencyGuardTable::empty(),
            8,
        );
        assert_eq!(outcome, PowlV2TickOutcome::Complete);
        assert_eq!(state.tick, 4); // entry, {a,b}, c, exit
        assert_eq!(labels(&compiled), vec!["a", "b", "c"]);
    }

    #[test]
    fn powl2_choice_graph_selects_one_valid_start_end_path() {
        let model = Powl2Model::ChoiceGraph {
            children: vec![
                Powl2Model::Activity("start".into()),
                Powl2Model::Activity("left".into()),
                Powl2Model::Activity("right".into()),
                Powl2Model::Activity("end".into()),
            ],
            // Def 3.6 encoding: sentinels are node 4 (start) and 5 (finish),
            // outside the 4-element child set. Previously this used child
            // indices 0 and 3, which `validate_powl2` accepted and the paper
            // does not define.
            edges: vec![(4, 0), (0, 1), (0, 2), (1, 3), (2, 3), (3, 5)],
            start: 4,
            end: 5,
        };
        let compiled = compile_powl2(&model, &mut LowestIndexPolicy).unwrap();
        assert_eq!(labels(&compiled), vec!["start", "left", "end"]);
        // The traversal now enters from and exits to the sentinels, so those
        // hops appear in the record. They are part of the path even though
        // Def 3.6 excludes the sentinels from the execution *sequence* --
        // which is why `labels` above is unchanged.
        assert_eq!(
            compiled.selected_choices,
            vec![
                ChoiceSelection {
                    graph_depth: 1,
                    from: 4,
                    to: 0,
                },
                ChoiceSelection {
                    graph_depth: 1,
                    from: 0,
                    to: 1,
                },
                ChoiceSelection {
                    graph_depth: 1,
                    from: 1,
                    to: 3,
                },
                ChoiceSelection {
                    graph_depth: 1,
                    from: 3,
                    to: 5,
                },
            ]
        );
    }

    #[test]
    fn choice_graph_refuses_a_vertex_off_every_start_end_path() {
        let model = Powl2Model::ChoiceGraph {
            children: vec![
                Powl2Model::Activity("start".into()),
                Powl2Model::Activity("end".into()),
                Powl2Model::Activity("orphan".into()),
            ],
            // Sentinels 3/4 over 3 real children. Child 2 is reachable from
            // nothing and reaches nothing, so it lies off every
            // start->finish path.
            edges: vec![(3, 0), (0, 1), (1, 4), (2, 2)],
            start: 3,
            end: 4,
        };
        assert_eq!(
            validate_powl2(&model),
            Err(Powl2Error::ChoiceNodeOffStartEndPath { node: 2 })
        );
    }

    #[test]
    fn partial_order_refuses_cycles() {
        let model = Powl2Model::PartialOrder {
            children: vec![Powl2Model::Silent, Powl2Model::Silent],
            edges: vec![(0, 1), (1, 0)],
        };
        assert_eq!(validate_powl2(&model), Err(Powl2Error::PartialOrderCycle));
    }

    struct OneRedo;

    impl Powl2ChoicePolicy for OneRedo {
        fn select_successor(
            &mut self,
            _graph_depth: usize,
            _current: usize,
            successors: &[usize],
        ) -> usize {
            successors[0]
        }

        fn select_redo(&mut self, _loop_depth: usize, completed_redos: u8, _max_redos: u8) -> bool {
            completed_redos == 0
        }
    }

    #[test]
    fn do_redo_is_bounded_and_unrolled_into_executable_tape() {
        let model = Powl2Model::DoRedo {
            body: Box::new(Powl2Model::Activity("body".into())),
            redo: Box::new(Powl2Model::Activity("redo".into())),
            max_redos: 3,
        };
        let compiled = compile_powl2(&model, &mut OneRedo).unwrap();
        assert_eq!(labels(&compiled), vec!["body", "redo", "body"]);
    }
}
