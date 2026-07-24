//! Safe, deterministic rewrite operations over the recursive POWL process IR.
//!
//! Rewrites produce before/after identities and never select choices, execute
//! activities, or claim semantic equivalence beyond the declared operation.

use std::collections::VecDeque;
use std::fmt;

use bcinr_mfw_ir::Digest;

use crate::powl2::{validate_powl2, Powl2Model};
use crate::process_toolkit::{
    diff_processes, process_digest, process_node, process_nodes, transitive_reduction, ProcessDiff,
    ProcessNodeRef, ProcessToolkitError,
};

/// Evidence for one exact structural rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRewriteWitness {
    pub target: ProcessNodeRef,
    pub before: Digest,
    pub target_before: Digest,
    pub after: Digest,
}

/// Exact activity-selection evidence for a process slice.
///
/// Node references identify activities in the input model. The nested rewrite
/// witness binds that selection to the exact before/after process identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSliceWitness {
    pub rewrite: ProcessRewriteWitness,
    pub retained_activities: Vec<ProcessNodeRef>,
    pub removed_activities: Vec<ProcessNodeRef>,
}

/// Optimistic process patch. The expected target root prevents stale replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessPatch {
    pub target: ProcessNodeRef,
    pub expected_target: Digest,
    pub replacement: Powl2Model,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessRewriteError {
    Toolkit(ProcessToolkitError),
    StaleTarget { expected: Digest, actual: Digest },
}

impl fmt::Display for ProcessRewriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POWL process rewrite refused: {self:?}")
    }
}

impl std::error::Error for ProcessRewriteError {}

impl From<ProcessToolkitError> for ProcessRewriteError {
    fn from(value: ProcessToolkitError) -> Self {
        Self::Toolkit(value)
    }
}

/// Find every activity with an exact label and return stable structural references.
pub fn find_activities(
    model: &Powl2Model,
    label: &str,
) -> Result<Vec<ProcessNodeRef>, ProcessToolkitError> {
    Ok(process_nodes(model)?
        .into_iter()
        .filter_map(|(node, value)| match value {
            Powl2Model::Activity(candidate) if candidate == label => Some(node),
            _ => None,
        })
        .collect())
}

/// Manufacture an optimistic patch against the current target identity.
pub fn prepare_process_patch(
    model: &Powl2Model,
    target: ProcessNodeRef,
    replacement: Powl2Model,
) -> Result<ProcessPatch, ProcessRewriteError> {
    validate_powl2(model).map_err(ProcessToolkitError::InvalidModel)?;
    validate_powl2(&replacement).map_err(ProcessToolkitError::InvalidModel)?;
    let expected_target = process_digest(process_node(model, &target)?);
    Ok(ProcessPatch {
        target,
        expected_target,
        replacement,
    })
}

/// Apply a patch only when the selected node still has the expected identity.
pub fn apply_process_patch(
    model: &Powl2Model,
    patch: &ProcessPatch,
) -> Result<(Powl2Model, ProcessRewriteWitness), ProcessRewriteError> {
    validate_powl2(model).map_err(ProcessToolkitError::InvalidModel)?;
    let actual = process_digest(process_node(model, &patch.target)?);
    if actual != patch.expected_target {
        return Err(ProcessRewriteError::StaleTarget {
            expected: patch.expected_target,
            actual,
        });
    }
    replace_process_node(model, &patch.target, patch.replacement.clone())
}

/// Replace one exact node and return roots suitable for a pass receipt.
pub fn replace_process_node(
    model: &Powl2Model,
    target: &ProcessNodeRef,
    replacement: Powl2Model,
) -> Result<(Powl2Model, ProcessRewriteWitness), ProcessRewriteError> {
    validate_powl2(model).map_err(ProcessToolkitError::InvalidModel)?;
    validate_powl2(&replacement).map_err(ProcessToolkitError::InvalidModel)?;
    let target_before = process_digest(process_node(model, target)?);
    let before = process_digest(model);
    let rewritten = replace_at(model, target.path(), target, &replacement)?;
    validate_powl2(&rewritten).map_err(ProcessToolkitError::InvalidModel)?;
    let after = process_digest(&rewritten);
    Ok((
        rewritten,
        ProcessRewriteWitness {
            target: target.clone(),
            before,
            target_before,
            after,
        },
    ))
}

/// Rename activity vocabulary without changing recursive process geometry.
pub fn map_activity_labels(
    model: &Powl2Model,
    mut map: impl FnMut(&str) -> String,
) -> Result<(Powl2Model, ProcessRewriteWitness), ProcessRewriteError> {
    validate_powl2(model).map_err(ProcessToolkitError::InvalidModel)?;
    let before = process_digest(model);
    let rewritten = map_labels(model, &mut map);
    validate_powl2(&rewritten).map_err(ProcessToolkitError::InvalidModel)?;
    let after = process_digest(&rewritten);
    Ok((
        rewritten,
        ProcessRewriteWitness {
            target: ProcessNodeRef::root(),
            before,
            target_before: before,
            after,
        },
    ))
}

/// Recursively remove only transitively implied partial-order edges.
///
/// Choice-graph edges are intentionally untouched: removing a direct choice edge
/// can change the paths available to a selection policy even when reachability is
/// unchanged. The returned witness binds the exact input and output identities.
pub fn reduce_transitive_process_edges(
    model: &Powl2Model,
) -> Result<(Powl2Model, ProcessRewriteWitness), ProcessRewriteError> {
    witnessed_root_rewrite(model, reduce_transitive_edges_inner)
}

/// Eliminate redundant silent nodes while preserving explicit choice and loop boundaries.
///
/// Sequence-local silent nodes are removed. Direct silent children of a partial
/// order are projected away while preserving reachability among retained children.
/// Direct silent vertices in a choice graph and silent `DoRedo` branches remain
/// explicit because removing them can change selection or repetition semantics.
pub fn eliminate_redundant_silent_nodes(
    model: &Powl2Model,
) -> Result<(Powl2Model, ProcessRewriteWitness), ProcessRewriteError> {
    witnessed_root_rewrite(model, eliminate_silent_inner)
}

/// Project a process onto selected activities and return exact selection evidence.
///
/// Activities rejected by `retain` become silent before conservative silent-node
/// elimination. This is an explicit semantic slice, not an equivalence claim and
/// not execution authority. Choice and loop boundaries remain explicit.
pub fn slice_process_activities(
    model: &Powl2Model,
    mut retain: impl FnMut(&str) -> bool,
) -> Result<(Powl2Model, ProcessSliceWitness), ProcessRewriteError> {
    validate_powl2(model).map_err(ProcessToolkitError::InvalidModel)?;
    let before = process_digest(model);
    let mut retained_activities = Vec::new();
    let mut removed_activities = Vec::new();
    let projected = project_activity_slice(
        model,
        &ProcessNodeRef::root(),
        &mut retain,
        &mut retained_activities,
        &mut removed_activities,
    )?;
    let rewritten = eliminate_silent_inner(&projected)?;
    validate_powl2(&rewritten).map_err(ProcessToolkitError::InvalidModel)?;
    let after = process_digest(&rewritten);
    Ok((
        rewritten,
        ProcessSliceWitness {
            rewrite: ProcessRewriteWitness {
                target: ProcessNodeRef::root(),
                before,
                target_before: before,
                after,
            },
            retained_activities,
            removed_activities,
        },
    ))
}

/// Diff two validated process graphs without manufacturing new standing.
///
/// `ProcessDiff` already carries the exact before/after digests. Validation here
/// prevents malformed process values from entering a comparison receipt.
pub fn diff_validated_processes(
    before: &Powl2Model,
    after: &Powl2Model,
) -> Result<ProcessDiff, ProcessRewriteError> {
    validate_powl2(before).map_err(ProcessToolkitError::InvalidModel)?;
    validate_powl2(after).map_err(ProcessToolkitError::InvalidModel)?;
    Ok(diff_processes(before, after))
}

fn witnessed_root_rewrite(
    model: &Powl2Model,
    rewrite: impl FnOnce(&Powl2Model) -> Result<Powl2Model, ProcessRewriteError>,
) -> Result<(Powl2Model, ProcessRewriteWitness), ProcessRewriteError> {
    validate_powl2(model).map_err(ProcessToolkitError::InvalidModel)?;
    let before = process_digest(model);
    let rewritten = rewrite(model)?;
    validate_powl2(&rewritten).map_err(ProcessToolkitError::InvalidModel)?;
    let after = process_digest(&rewritten);
    Ok((
        rewritten,
        ProcessRewriteWitness {
            target: ProcessNodeRef::root(),
            before,
            target_before: before,
            after,
        },
    ))
}

fn reduce_transitive_edges_inner(model: &Powl2Model) -> Result<Powl2Model, ProcessRewriteError> {
    match model {
        Powl2Model::Activity(label) => Ok(Powl2Model::Activity(label.clone())),
        Powl2Model::Silent => Ok(Powl2Model::Silent),
        Powl2Model::Sequence(children) => Ok(Powl2Model::Sequence(
            children
                .iter()
                .map(reduce_transitive_edges_inner)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Powl2Model::PartialOrder { children, edges } => {
            let children = children
                .iter()
                .map(reduce_transitive_edges_inner)
                .collect::<Result<Vec<_>, _>>()?;
            let edges = transitive_reduction(children.len(), edges)?;
            Ok(Powl2Model::PartialOrder { children, edges })
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => Ok(Powl2Model::ChoiceGraph {
            children: children
                .iter()
                .map(reduce_transitive_edges_inner)
                .collect::<Result<Vec<_>, _>>()?,
            edges: edges.clone(),
            start: *start,
            end: *end,
        }),
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => Ok(Powl2Model::DoRedo {
            body: Box::new(reduce_transitive_edges_inner(body)?),
            redo: Box::new(reduce_transitive_edges_inner(redo)?),
            max_redos: *max_redos,
        }),
    }
}

fn eliminate_silent_inner(model: &Powl2Model) -> Result<Powl2Model, ProcessRewriteError> {
    match model {
        Powl2Model::Activity(label) => Ok(Powl2Model::Activity(label.clone())),
        Powl2Model::Silent => Ok(Powl2Model::Silent),
        Powl2Model::Sequence(children) => {
            let mut retained = Vec::new();
            for child in children {
                let rewritten = eliminate_silent_inner(child)?;
                if !matches!(rewritten, Powl2Model::Silent) {
                    retained.push(rewritten);
                }
            }
            Ok(collapse_sequence(retained))
        }
        Powl2Model::PartialOrder { children, edges } => {
            let children = children
                .iter()
                .map(eliminate_silent_inner)
                .collect::<Result<Vec<_>, _>>()?;
            project_partial_order_without_silent(children, edges)
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => Ok(Powl2Model::ChoiceGraph {
            children: children
                .iter()
                .map(eliminate_silent_inner)
                .collect::<Result<Vec<_>, _>>()?,
            edges: edges.clone(),
            start: *start,
            end: *end,
        }),
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => Ok(Powl2Model::DoRedo {
            body: Box::new(eliminate_silent_inner(body)?),
            redo: Box::new(eliminate_silent_inner(redo)?),
            max_redos: *max_redos,
        }),
    }
}

fn collapse_sequence(mut children: Vec<Powl2Model>) -> Powl2Model {
    match children.len() {
        0 => Powl2Model::Silent,
        1 => children.pop().expect("length checked"),
        _ => Powl2Model::Sequence(children),
    }
}

fn project_partial_order_without_silent(
    children: Vec<Powl2Model>,
    edges: &[(usize, usize)],
) -> Result<Powl2Model, ProcessRewriteError> {
    let node_count = children.len();
    let mut remap = vec![None; node_count];
    let mut retained = Vec::with_capacity(node_count);
    for (old_index, child) in children.into_iter().enumerate() {
        if matches!(child, Powl2Model::Silent) {
            continue;
        }
        remap[old_index] = Some(retained.len());
        retained.push(child);
    }
    match retained.len() {
        0 => return Ok(Powl2Model::Silent),
        1 => return Ok(retained.pop().expect("length checked")),
        _ => {}
    }

    let mut projected_edges = Vec::new();
    for (old_from, new_from) in remap.iter().copied().enumerate() {
        let Some(new_from) = new_from else {
            continue;
        };
        for (old_to, new_to) in remap.iter().copied().enumerate() {
            let Some(new_to) = new_to else {
                continue;
            };
            if old_from != old_to && graph_reachable(node_count, edges, old_from, old_to) {
                projected_edges.push((new_from, new_to));
            }
        }
    }
    let edges = transitive_reduction(retained.len(), &projected_edges)?;
    Ok(Powl2Model::PartialOrder {
        children: retained,
        edges,
    })
}

fn graph_reachable(
    node_count: usize,
    edges: &[(usize, usize)],
    start: usize,
    target: usize,
) -> bool {
    let mut queue = VecDeque::from([start]);
    let mut visited = vec![false; node_count];
    visited[start] = true;
    while let Some(node) = queue.pop_front() {
        for &(from, to) in edges {
            if from != node || visited[to] {
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

fn project_activity_slice(
    model: &Powl2Model,
    node: &ProcessNodeRef,
    retain: &mut impl FnMut(&str) -> bool,
    retained_activities: &mut Vec<ProcessNodeRef>,
    removed_activities: &mut Vec<ProcessNodeRef>,
) -> Result<Powl2Model, ProcessRewriteError> {
    match model {
        Powl2Model::Activity(label) => {
            if retain(label) {
                retained_activities.push(node.clone());
                Ok(Powl2Model::Activity(label.clone()))
            } else {
                removed_activities.push(node.clone());
                Ok(Powl2Model::Silent)
            }
        }
        Powl2Model::Silent => Ok(Powl2Model::Silent),
        Powl2Model::Sequence(children) => {
            let mut projected = Vec::with_capacity(children.len());
            for (index, child) in children.iter().enumerate() {
                projected.push(project_activity_slice(
                    child,
                    &node.child(index)?,
                    retain,
                    retained_activities,
                    removed_activities,
                )?);
            }
            Ok(Powl2Model::Sequence(projected))
        }
        Powl2Model::PartialOrder { children, edges } => {
            let mut projected = Vec::with_capacity(children.len());
            for (index, child) in children.iter().enumerate() {
                projected.push(project_activity_slice(
                    child,
                    &node.child(index)?,
                    retain,
                    retained_activities,
                    removed_activities,
                )?);
            }
            Ok(Powl2Model::PartialOrder {
                children: projected,
                edges: edges.clone(),
            })
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => {
            let mut projected = Vec::with_capacity(children.len());
            for (index, child) in children.iter().enumerate() {
                projected.push(project_activity_slice(
                    child,
                    &node.child(index)?,
                    retain,
                    retained_activities,
                    removed_activities,
                )?);
            }
            Ok(Powl2Model::ChoiceGraph {
                children: projected,
                edges: edges.clone(),
                start: *start,
                end: *end,
            })
        }
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => Ok(Powl2Model::DoRedo {
            body: Box::new(project_activity_slice(
                body,
                &node.child(0)?,
                retain,
                retained_activities,
                removed_activities,
            )?),
            redo: Box::new(project_activity_slice(
                redo,
                &node.child(1)?,
                retain,
                retained_activities,
                removed_activities,
            )?),
            max_redos: *max_redos,
        }),
    }
}

fn replace_at(
    model: &Powl2Model,
    path: &[u16],
    target: &ProcessNodeRef,
    replacement: &Powl2Model,
) -> Result<Powl2Model, ProcessRewriteError> {
    let Some((&head, tail)) = path.split_first() else {
        return Ok(replacement.clone());
    };
    let index = head as usize;
    match model {
        Powl2Model::Sequence(children) => {
            let mut children = children.clone();
            let child = children
                .get(index)
                .cloned()
                .ok_or_else(|| ProcessToolkitError::InvalidNode(target.clone()))?;
            let rewritten_child = replace_at(&child, tail, target, replacement)?;
            children[index] = rewritten_child;
            Ok(Powl2Model::Sequence(children))
        }
        Powl2Model::PartialOrder { children, edges } => {
            let mut children = children.clone();
            let child = children
                .get(index)
                .cloned()
                .ok_or_else(|| ProcessToolkitError::InvalidNode(target.clone()))?;
            let rewritten_child = replace_at(&child, tail, target, replacement)?;
            children[index] = rewritten_child;
            Ok(Powl2Model::PartialOrder {
                children,
                edges: edges.clone(),
            })
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => {
            let mut children = children.clone();
            let child = children
                .get(index)
                .cloned()
                .ok_or_else(|| ProcessToolkitError::InvalidNode(target.clone()))?;
            let rewritten_child = replace_at(&child, tail, target, replacement)?;
            children[index] = rewritten_child;
            Ok(Powl2Model::ChoiceGraph {
                children,
                edges: edges.clone(),
                start: *start,
                end: *end,
            })
        }
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => match index {
            0 => Ok(Powl2Model::DoRedo {
                body: Box::new(replace_at(body, tail, target, replacement)?),
                redo: redo.clone(),
                max_redos: *max_redos,
            }),
            1 => Ok(Powl2Model::DoRedo {
                body: body.clone(),
                redo: Box::new(replace_at(redo, tail, target, replacement)?),
                max_redos: *max_redos,
            }),
            _ => Err(ProcessToolkitError::InvalidNode(target.clone()).into()),
        },
        Powl2Model::Activity(_) | Powl2Model::Silent => {
            Err(ProcessToolkitError::InvalidNode(target.clone()).into())
        }
    }
}

fn map_labels(model: &Powl2Model, map: &mut impl FnMut(&str) -> String) -> Powl2Model {
    match model {
        Powl2Model::Activity(label) => Powl2Model::Activity(map(label)),
        Powl2Model::Silent => Powl2Model::Silent,
        Powl2Model::Sequence(children) => Powl2Model::Sequence(
            children
                .iter()
                .map(|child| map_labels(child, map))
                .collect(),
        ),
        Powl2Model::PartialOrder { children, edges } => Powl2Model::PartialOrder {
            children: children
                .iter()
                .map(|child| map_labels(child, map))
                .collect(),
            edges: edges.clone(),
        },
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => Powl2Model::ChoiceGraph {
            children: children
                .iter()
                .map(|child| map_labels(child, map))
                .collect(),
            edges: edges.clone(),
            start: *start,
            end: *end,
        },
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => Powl2Model::DoRedo {
            body: Box::new(map_labels(body, map)),
            redo: Box::new(map_labels(redo, map)),
            max_redos: *max_redos,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process_toolkit::{activity, partial_order, sequence, silent};

    #[test]
    fn patch_refuses_stale_target() {
        let original = sequence(vec![activity("a"), activity("b")]).unwrap();
        let target = ProcessNodeRef::root().child(1).unwrap();
        let patch = prepare_process_patch(&original, target, activity("c")).unwrap();
        let changed = sequence(vec![activity("a"), activity("other")]).unwrap();
        assert!(matches!(
            apply_process_patch(&changed, &patch),
            Err(ProcessRewriteError::StaleTarget { .. })
        ));
    }

    #[test]
    fn activity_mapping_preserves_geometry() {
        let original = sequence(vec![activity("a"), activity("b")]).unwrap();
        let (mapped, witness) =
            map_activity_labels(&original, |label| format!("cmd::{label}")).unwrap();
        assert_eq!(find_activities(&mapped, "cmd::a").unwrap().len(), 1);
        assert_ne!(witness.before, witness.after);
    }

    #[test]
    fn transitive_reduction_is_recursive_and_witnessed() {
        let nested = partial_order(
            vec![activity("a"), activity("b"), activity("c")],
            vec![(0, 1), (1, 2), (0, 2)],
        )
        .unwrap();
        let original = sequence(vec![nested, activity("done")]).unwrap();
        let (rewritten, witness) = reduce_transitive_process_edges(&original).unwrap();
        let Powl2Model::Sequence(children) = &rewritten else {
            panic!("sequence geometry should remain explicit");
        };
        let Powl2Model::PartialOrder { edges, .. } = &children[0] else {
            panic!("nested partial order should remain explicit");
        };
        assert_eq!(edges, &vec![(0, 1), (1, 2)]);
        assert_eq!(witness.target, ProcessNodeRef::root());
        assert_ne!(witness.before, witness.after);
        validate_powl2(&rewritten).unwrap();
    }

    #[test]
    fn silent_partial_order_nodes_are_bypassed_without_losing_precedence() {
        let original = partial_order(
            vec![activity("a"), silent(), activity("b")],
            vec![(0, 1), (1, 2)],
        )
        .unwrap();
        let (rewritten, witness) = eliminate_redundant_silent_nodes(&original).unwrap();
        assert_eq!(
            rewritten,
            Powl2Model::PartialOrder {
                children: vec![activity("a"), activity("b")],
                edges: vec![(0, 1)],
            }
        );
        assert_ne!(witness.before, witness.after);
        validate_powl2(&rewritten).unwrap();
    }

    #[test]
    fn activity_slice_records_exact_input_nodes_and_validated_diff() {
        let original = sequence(vec![activity("a"), activity("drop"), activity("c")]).unwrap();
        let (sliced, witness) =
            slice_process_activities(&original, |label| label != "drop").unwrap();
        assert_eq!(sliced, sequence(vec![activity("a"), activity("c")]).unwrap());
        assert_eq!(witness.retained_activities.len(), 2);
        assert_eq!(witness.removed_activities.len(), 1);
        assert_eq!(witness.removed_activities[0].stable_id(), "n_1");
        let diff = diff_validated_processes(&original, &sliced).unwrap();
        assert_eq!(diff.before, witness.rewrite.before);
        assert_eq!(diff.after, witness.rewrite.after);
        assert_eq!(diff.removed_activities, vec!["drop".to_string()]);
    }

    #[test]
    fn activity_slice_preserves_choice_vertices_as_explicit_silent_nodes() {
        let original = Powl2Model::ChoiceGraph {
            children: vec![activity("start"), activity("drop"), activity("end")],
            edges: vec![(0, 1), (1, 2)],
            start: 0,
            end: 2,
        };
        let (sliced, _) =
            slice_process_activities(&original, |label| label != "drop").unwrap();
        assert_eq!(
            sliced,
            Powl2Model::ChoiceGraph {
                children: vec![activity("start"), silent(), activity("end")],
                edges: vec![(0, 1), (1, 2)],
                start: 0,
                end: 2,
            }
        );
        validate_powl2(&sliced).unwrap();
    }

    #[test]
    fn validated_diff_refuses_a_malformed_process() {
        let malformed = Powl2Model::Sequence(Vec::new());
        assert!(diff_validated_processes(&malformed, &activity("ok")).is_err());
    }
}
