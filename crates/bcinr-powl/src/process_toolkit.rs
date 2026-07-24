//! Deterministic manipulation and analysis utilities for the recursive POWL 2.0 IR.
//!
//! The functions in this module never schedule or actuate application work. They
//! manufacture new process values, structural witnesses, metrics, and projections
//! that callers can validate and receipt before execution.

use std::collections::{BTreeSet, VecDeque};
use std::fmt;

use bcinr_mfw_ir::Digest;

use crate::powl2::{validate_powl2, Powl2Error, Powl2Model};

/// Stable structural path to a node in a recursive POWL model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessNodeRef(Vec<u16>);

impl ProcessNodeRef {
    pub const fn root() -> Self {
        Self(Vec::new())
    }

    pub fn child(&self, index: usize) -> Result<Self, ProcessToolkitError> {
        let index =
            u16::try_from(index).map_err(|_| ProcessToolkitError::NodeIndexOverflow(index))?;
        let mut path = self.0.clone();
        path.push(index);
        Ok(Self(path))
    }

    pub fn path(&self) -> &[u16] {
        &self.0
    }

    pub fn stable_id(&self) -> String {
        if self.0.is_empty() {
            "root".to_string()
        } else {
            let suffix = self
                .0
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join("_");
            format!("n_{suffix}")
        }
    }
}

/// Process-toolkit refusal with precise structural context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessToolkitError {
    InvalidModel(Powl2Error),
    InvalidNode(ProcessNodeRef),
    InvalidEdge { from: usize, to: usize, len: usize },
    Cycle,
    NodeIndexOverflow(usize),
}

impl fmt::Display for ProcessToolkitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POWL process toolkit refused: {self:?}")
    }
}

impl std::error::Error for ProcessToolkitError {}

impl From<Powl2Error> for ProcessToolkitError {
    fn from(value: Powl2Error) -> Self {
        Self::InvalidModel(value)
    }
}

/// Structural metrics over one recursive POWL model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcessMetrics {
    pub nodes: usize,
    pub activities: usize,
    pub silent: usize,
    pub sequences: usize,
    pub partial_orders: usize,
    pub choice_graphs: usize,
    pub do_redo: usize,
    pub max_depth: usize,
    pub max_parallel_width: usize,
}

/// Exact structural difference between two POWL models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessDiff {
    pub before: Digest,
    pub after: Digest,
    pub added_activities: Vec<String>,
    pub removed_activities: Vec<String>,
    pub structure_changed: bool,
}

/// Manufacture an activity node.
pub fn activity(label: impl Into<String>) -> Powl2Model {
    Powl2Model::Activity(label.into())
}

/// Manufacture a silent node.
pub const fn silent() -> Powl2Model {
    Powl2Model::Silent
}

/// Manufacture a sequence and validate it.
pub fn sequence(children: Vec<Powl2Model>) -> Result<Powl2Model, ProcessToolkitError> {
    let model = Powl2Model::Sequence(children);
    validate_powl2(&model)?;
    Ok(model)
}

/// Manufacture a partial order and validate all edges and acyclicity.
pub fn partial_order(
    children: Vec<Powl2Model>,
    edges: Vec<(usize, usize)>,
) -> Result<Powl2Model, ProcessToolkitError> {
    let model = Powl2Model::PartialOrder { children, edges };
    validate_powl2(&model)?;
    Ok(model)
}

/// Manufacture a generalized choice graph and validate start/end coverage.
pub fn choice_graph(
    children: Vec<Powl2Model>,
    edges: Vec<(usize, usize)>,
    start: usize,
    end: usize,
) -> Result<Powl2Model, ProcessToolkitError> {
    let model = Powl2Model::ChoiceGraph {
        children,
        edges,
        start,
        end,
    };
    validate_powl2(&model)?;
    Ok(model)
}

/// Manufacture a bounded do-redo process and validate its children.
pub fn do_redo(
    body: Powl2Model,
    redo: Powl2Model,
    max_redos: u8,
) -> Result<Powl2Model, ProcessToolkitError> {
    let model = Powl2Model::DoRedo {
        body: Box::new(body),
        redo: Box::new(redo),
        max_redos,
    };
    validate_powl2(&model)?;
    Ok(model)
}

/// Content identity for the exact recursive process structure.
pub fn process_digest(model: &Powl2Model) -> Digest {
    let mut bytes = b"bcinr:powl2:process-structure:v1\0".to_vec();
    encode_model(model, &mut bytes);
    Digest::hash(&bytes)
}

/// Return every node with its stable recursive path in preorder.
pub fn process_nodes(
    model: &Powl2Model,
) -> Result<Vec<(ProcessNodeRef, &Powl2Model)>, ProcessToolkitError> {
    validate_powl2(model)?;
    let mut nodes = Vec::new();
    walk_nodes(model, &ProcessNodeRef::root(), &mut nodes)?;
    Ok(nodes)
}

/// Resolve a stable recursive node reference.
pub fn process_node<'a>(
    model: &'a Powl2Model,
    node: &ProcessNodeRef,
) -> Result<&'a Powl2Model, ProcessToolkitError> {
    let mut current = model;
    for &index in node.path() {
        let child_models = children(current);
        current = child_models
            .get(index as usize)
            .copied()
            .ok_or_else(|| ProcessToolkitError::InvalidNode(node.clone()))?;
    }
    Ok(current)
}

/// Normalize representation-only structure while preserving declared process meaning.
///
/// This pass flattens nested sequences, removes sequence-local silent nodes,
/// deduplicates edges, and transitively reduces partial-order edges. It does not
/// select choices, unroll loops, or reorder activities.
pub fn normalize_process(model: &Powl2Model) -> Result<Powl2Model, ProcessToolkitError> {
    validate_powl2(model)?;
    let normalized = normalize_inner(model)?;
    validate_powl2(&normalized)?;
    Ok(normalized)
}

/// Compute deterministic process metrics.
pub fn process_metrics(model: &Powl2Model) -> Result<ProcessMetrics, ProcessToolkitError> {
    validate_powl2(model)?;
    let mut metrics = ProcessMetrics::default();
    observe_metrics(model, 0, &mut metrics)?;
    Ok(metrics)
}

/// Exact transitive reduction for a bounded directed acyclic graph.
pub fn transitive_reduction(
    node_count: usize,
    edges: &[(usize, usize)],
) -> Result<Vec<(usize, usize)>, ProcessToolkitError> {
    let edges = canonical_edges(node_count, edges)?;
    topological_layers(node_count, &edges)?;
    let mut reduced = Vec::new();
    for &(from, to) in &edges {
        if !reachable_without_edge(node_count, &edges, from, to, (from, to)) {
            reduced.push((from, to));
        }
    }
    Ok(reduced)
}

/// Deterministic topological layers. Each layer is the complete ready set under
/// the supplied precedence relation after prior layers are removed.
pub fn topological_layers(
    node_count: usize,
    edges: &[(usize, usize)],
) -> Result<Vec<Vec<usize>>, ProcessToolkitError> {
    let edges = canonical_edges(node_count, edges)?;
    let mut indegree = vec![0usize; node_count];
    let mut successors = vec![Vec::<usize>::new(); node_count];
    for &(from, to) in &edges {
        indegree[to] += 1;
        successors[from].push(to);
    }
    let mut ready = (0..node_count)
        .filter(|&node| indegree[node] == 0)
        .collect::<BTreeSet<_>>();
    let mut layers = Vec::new();
    let mut visited = 0usize;
    while !ready.is_empty() {
        let layer = ready.iter().copied().collect::<Vec<_>>();
        ready.clear();
        visited += layer.len();
        for &node in &layer {
            for &successor in &successors[node] {
                indegree[successor] -= 1;
                if indegree[successor] == 0 {
                    ready.insert(successor);
                }
            }
        }
        layers.push(layer);
    }
    if visited != node_count {
        return Err(ProcessToolkitError::Cycle);
    }
    Ok(layers)
}

/// Exact maximum antichain width for a bounded DAG, computed through Dilworth's
/// theorem as `node_count - maximum_bipartite_matching(transitive_closure)`.
pub fn antichain_width(
    node_count: usize,
    edges: &[(usize, usize)],
) -> Result<usize, ProcessToolkitError> {
    let edges = canonical_edges(node_count, edges)?;
    topological_layers(node_count, &edges)?;
    let mut reachability = vec![vec![false; node_count]; node_count];
    for &(from, to) in &edges {
        reachability[from][to] = true;
    }
    for pivot in 0..node_count {
        for from in 0..node_count {
            if !reachability[from][pivot] {
                continue;
            }
            for to in 0..node_count {
                reachability[from][to] |= reachability[pivot][to];
            }
        }
    }

    fn augment(
        left: usize,
        reachability: &[Vec<bool>],
        seen: &mut [bool],
        matched_left: &mut [Option<usize>],
    ) -> bool {
        for right in 0..reachability.len() {
            if !reachability[left][right] || seen[right] {
                continue;
            }
            seen[right] = true;
            if matched_left[right]
                .is_none_or(|previous| augment(previous, reachability, seen, matched_left))
            {
                matched_left[right] = Some(left);
                return true;
            }
        }
        false
    }

    let mut matched_left = vec![None; node_count];
    let mut matching = 0usize;
    for left in 0..node_count {
        let mut seen = vec![false; node_count];
        if augment(left, &reachability, &mut seen, &mut matched_left) {
            matching += 1;
        }
    }
    Ok(node_count.saturating_sub(matching))
}

/// Longest precedence-chain length in nodes for a bounded DAG.
pub fn critical_path_length(
    node_count: usize,
    edges: &[(usize, usize)],
) -> Result<usize, ProcessToolkitError> {
    let edges = canonical_edges(node_count, edges)?;
    let layers = topological_layers(node_count, &edges)?;
    if node_count == 0 {
        return Ok(0);
    }
    let mut predecessors = vec![Vec::<usize>::new(); node_count];
    for (from, to) in edges {
        predecessors[to].push(from);
    }
    let mut distance = vec![1usize; node_count];
    for layer in layers {
        for node in layer {
            distance[node] = predecessors[node]
                .iter()
                .map(|&predecessor| distance[predecessor].saturating_add(1))
                .max()
                .unwrap_or(1);
        }
    }
    Ok(distance.into_iter().max().unwrap_or(0))
}

/// Compare exact process identity and activity vocabulary.
pub fn diff_processes(before: &Powl2Model, after: &Powl2Model) -> ProcessDiff {
    let before_labels = activity_labels(before);
    let after_labels = activity_labels(after);
    let before_root = process_digest(before);
    let after_root = process_digest(after);
    ProcessDiff {
        before: before_root,
        after: after_root,
        added_activities: after_labels.difference(&before_labels).cloned().collect(),
        removed_activities: before_labels.difference(&after_labels).cloned().collect(),
        structure_changed: before_root != after_root,
    }
}

/// Deterministic Mermaid projection of recursive containment and process edges.
pub fn process_to_mermaid(model: &Powl2Model) -> Result<String, ProcessToolkitError> {
    validate_powl2(model)?;
    let mut lines = vec!["flowchart TD".to_string()];
    render_mermaid(model, &ProcessNodeRef::root(), &mut lines)?;
    Ok(lines.join("\n"))
}

/// Stable human-readable explanation for logs, tests, and approval packets.
pub fn explain_process(model: &Powl2Model) -> Result<String, ProcessToolkitError> {
    let metrics = process_metrics(model)?;
    Ok(format!(
        "POWL process {}: nodes={}, activities={}, silent={}, sequences={}, partial_orders={}, choice_graphs={}, do_redo={}, max_depth={}, max_parallel_width={}",
        process_digest(model),
        metrics.nodes,
        metrics.activities,
        metrics.silent,
        metrics.sequences,
        metrics.partial_orders,
        metrics.choice_graphs,
        metrics.do_redo,
        metrics.max_depth,
        metrics.max_parallel_width,
    ))
}

fn normalize_inner(model: &Powl2Model) -> Result<Powl2Model, ProcessToolkitError> {
    match model {
        Powl2Model::Activity(label) => Ok(Powl2Model::Activity(label.clone())),
        Powl2Model::Silent => Ok(Powl2Model::Silent),
        Powl2Model::Sequence(children) => {
            let mut flattened = Vec::new();
            for child in children {
                match normalize_inner(child)? {
                    Powl2Model::Silent => {}
                    Powl2Model::Sequence(nested) => flattened.extend(nested),
                    normalized => flattened.push(normalized),
                }
            }
            Ok(match flattened.len() {
                0 => Powl2Model::Silent,
                1 => flattened.pop().expect("length checked"),
                _ => Powl2Model::Sequence(flattened),
            })
        }
        Powl2Model::PartialOrder { children, edges } => {
            let children = children
                .iter()
                .map(normalize_inner)
                .collect::<Result<Vec<_>, _>>()?;
            let edges = transitive_reduction(children.len(), edges)?;
            Ok(Powl2Model::PartialOrder { children, edges })
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => {
            let children = children
                .iter()
                .map(normalize_inner)
                .collect::<Result<Vec<_>, _>>()?;
            let edges = canonical_edges(children.len(), edges)?;
            Ok(Powl2Model::ChoiceGraph {
                children,
                edges,
                start: *start,
                end: *end,
            })
        }
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => Ok(Powl2Model::DoRedo {
            body: Box::new(normalize_inner(body)?),
            redo: Box::new(normalize_inner(redo)?),
            max_redos: *max_redos,
        }),
    }
}

fn observe_metrics(
    model: &Powl2Model,
    depth: usize,
    metrics: &mut ProcessMetrics,
) -> Result<(), ProcessToolkitError> {
    metrics.nodes += 1;
    metrics.max_depth = metrics.max_depth.max(depth);
    match model {
        Powl2Model::Activity(_) => {
            metrics.activities += 1;
            metrics.max_parallel_width = metrics.max_parallel_width.max(1);
        }
        Powl2Model::Silent => metrics.silent += 1,
        Powl2Model::Sequence(children) => {
            metrics.sequences += 1;
            for child in children {
                observe_metrics(child, depth + 1, metrics)?;
            }
        }
        Powl2Model::PartialOrder { children, edges } => {
            metrics.partial_orders += 1;
            let width = antichain_width(children.len(), edges)?;
            metrics.max_parallel_width = metrics.max_parallel_width.max(width);
            for child in children {
                observe_metrics(child, depth + 1, metrics)?;
            }
        }
        Powl2Model::ChoiceGraph { children, .. } => {
            metrics.choice_graphs += 1;
            for child in children {
                observe_metrics(child, depth + 1, metrics)?;
            }
        }
        Powl2Model::DoRedo { body, redo, .. } => {
            metrics.do_redo += 1;
            observe_metrics(body, depth + 1, metrics)?;
            observe_metrics(redo, depth + 1, metrics)?;
        }
    }
    Ok(())
}

fn activity_labels(model: &Powl2Model) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    collect_activity_labels(model, &mut labels);
    labels
}

fn collect_activity_labels(model: &Powl2Model, labels: &mut BTreeSet<String>) {
    match model {
        Powl2Model::Activity(label) => {
            labels.insert(label.clone());
        }
        Powl2Model::Silent => {}
        Powl2Model::Sequence(children)
        | Powl2Model::PartialOrder { children, .. }
        | Powl2Model::ChoiceGraph { children, .. } => {
            for child in children {
                collect_activity_labels(child, labels);
            }
        }
        Powl2Model::DoRedo { body, redo, .. } => {
            collect_activity_labels(body, labels);
            collect_activity_labels(redo, labels);
        }
    }
}

fn walk_nodes<'a>(
    model: &'a Powl2Model,
    node: &ProcessNodeRef,
    output: &mut Vec<(ProcessNodeRef, &'a Powl2Model)>,
) -> Result<(), ProcessToolkitError> {
    output.push((node.clone(), model));
    for (index, child) in children(model).into_iter().enumerate() {
        walk_nodes(child, &node.child(index)?, output)?;
    }
    Ok(())
}

fn children(model: &Powl2Model) -> Vec<&Powl2Model> {
    match model {
        Powl2Model::Activity(_) | Powl2Model::Silent => Vec::new(),
        Powl2Model::Sequence(children)
        | Powl2Model::PartialOrder { children, .. }
        | Powl2Model::ChoiceGraph { children, .. } => children.iter().collect(),
        Powl2Model::DoRedo { body, redo, .. } => vec![body, redo],
    }
}

fn canonical_edges(
    node_count: usize,
    edges: &[(usize, usize)],
) -> Result<Vec<(usize, usize)>, ProcessToolkitError> {
    let mut canonical = BTreeSet::new();
    for &(from, to) in edges {
        if from >= node_count || to >= node_count {
            return Err(ProcessToolkitError::InvalidEdge {
                from,
                to,
                len: node_count,
            });
        }
        canonical.insert((from, to));
    }
    Ok(canonical.into_iter().collect())
}

fn reachable_without_edge(
    node_count: usize,
    edges: &[(usize, usize)],
    start: usize,
    target: usize,
    skipped: (usize, usize),
) -> bool {
    let mut queue = VecDeque::from([start]);
    let mut visited = vec![false; node_count];
    visited[start] = true;
    while let Some(node) = queue.pop_front() {
        for &(from, to) in edges {
            if (from, to) == skipped || from != node || visited[to] {
                continue;
            }
            if to == target {
                return true;
            }
            visited[to] = true;
            queue.push_back(to);
        }
    }
    false
}

fn encode_model(model: &Powl2Model, output: &mut Vec<u8>) {
    match model {
        Powl2Model::Activity(label) => {
            output.push(0);
            encode_bytes(label.as_bytes(), output);
        }
        Powl2Model::Silent => output.push(1),
        Powl2Model::Sequence(children) => {
            output.push(2);
            encode_usize(children.len(), output);
            for child in children {
                encode_model(child, output);
            }
        }
        Powl2Model::PartialOrder { children, edges } => {
            output.push(3);
            encode_usize(children.len(), output);
            for child in children {
                encode_model(child, output);
            }
            encode_edges(edges, output);
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => {
            output.push(4);
            encode_usize(children.len(), output);
            for child in children {
                encode_model(child, output);
            }
            encode_edges(edges, output);
            encode_usize(*start, output);
            encode_usize(*end, output);
        }
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => {
            output.push(5);
            encode_model(body, output);
            encode_model(redo, output);
            output.push(*max_redos);
        }
    }
}

fn encode_edges(edges: &[(usize, usize)], output: &mut Vec<u8>) {
    let edges = edges.iter().copied().collect::<BTreeSet<_>>();
    encode_usize(edges.len(), output);
    for (from, to) in edges {
        encode_usize(from, output);
        encode_usize(to, output);
    }
}

fn encode_bytes(bytes: &[u8], output: &mut Vec<u8>) {
    encode_usize(bytes.len(), output);
    output.extend_from_slice(bytes);
}

fn encode_usize(value: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(value as u64).to_le_bytes());
}

fn render_mermaid(
    model: &Powl2Model,
    node: &ProcessNodeRef,
    lines: &mut Vec<String>,
) -> Result<(), ProcessToolkitError> {
    let id = node.stable_id();
    lines.push(format!("    {id}[\"{}\"]", mermaid_label(model)));
    let child_models = children(model);
    for (index, child) in child_models.iter().enumerate() {
        let child_ref = node.child(index)?;
        render_mermaid(child, &child_ref, lines)?;
        lines.push(format!(
            "    {id} -. contains .-> {}",
            child_ref.stable_id()
        ));
    }
    match model {
        Powl2Model::Sequence(children) => {
            for index in 0..children.len().saturating_sub(1) {
                lines.push(format!(
                    "    {} --> {}",
                    node.child(index)?.stable_id(),
                    node.child(index + 1)?.stable_id()
                ));
            }
        }
        Powl2Model::PartialOrder { edges, .. } => {
            for &(from, to) in edges {
                lines.push(format!(
                    "    {} --> {}",
                    node.child(from)?.stable_id(),
                    node.child(to)?.stable_id()
                ));
            }
        }
        Powl2Model::ChoiceGraph { edges, .. } => {
            for &(from, to) in edges {
                lines.push(format!(
                    "    {} -. choice .-> {}",
                    node.child(from)?.stable_id(),
                    node.child(to)?.stable_id()
                ));
            }
        }
        Powl2Model::DoRedo { .. } => {
            lines.push(format!(
                "    {} -. redo .-> {}",
                node.child(1)?.stable_id(),
                node.child(0)?.stable_id()
            ));
        }
        Powl2Model::Activity(_) | Powl2Model::Silent => {}
    }
    Ok(())
}

fn mermaid_label(model: &Powl2Model) -> String {
    match model {
        Powl2Model::Activity(label) => format!("activity: {}", label.replace('"', "'")),
        Powl2Model::Silent => "silent".to_string(),
        Powl2Model::Sequence(_) => "sequence".to_string(),
        Powl2Model::PartialOrder { .. } => "partial order".to_string(),
        Powl2Model::ChoiceGraph { start, end, .. } => format!("choice {start}->{end}"),
        Powl2Model::DoRedo { max_redos, .. } => format!("do-redo <= {max_redos}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduction_removes_only_transitive_edges() {
        assert_eq!(
            transitive_reduction(3, &[(0, 1), (1, 2), (0, 2)]).unwrap(),
            vec![(0, 1), (1, 2)]
        );
    }

    #[test]
    fn normalization_flattens_sequences_without_selecting_behavior() {
        let model = sequence(vec![
            activity("a"),
            sequence(vec![silent(), activity("b")]).unwrap(),
        ])
        .unwrap();
        assert_eq!(
            normalize_process(&model).unwrap(),
            Powl2Model::Sequence(vec![activity("a"), activity("b")])
        );
    }

    #[test]
    fn antichain_and_critical_path_are_exact_for_diamond() {
        let edges = [(0, 1), (0, 2), (1, 3), (2, 3)];
        assert_eq!(antichain_width(4, &edges).unwrap(), 2);
        assert_eq!(critical_path_length(4, &edges).unwrap(), 3);
    }
}
