//! Safe, deterministic rewrite operations over the recursive POWL process IR.
//!
//! Rewrites produce before/after identities and never select choices, execute
//! activities, or claim semantic equivalence beyond the declared operation.

use std::fmt;

use bcinr_mfw_ir::Digest;

use crate::powl2::{validate_powl2, Powl2Model};
use crate::process_toolkit::{
    process_digest, process_node, process_nodes, ProcessNodeRef, ProcessToolkitError,
};

/// Evidence for one exact structural rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRewriteWitness {
    pub target: ProcessNodeRef,
    pub before: Digest,
    pub target_before: Digest,
    pub after: Digest,
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
        Powl2Model::Sequence(children) => {
            Powl2Model::Sequence(children.iter().map(|child| map_labels(child, map)).collect())
        }
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
    use crate::process_toolkit::{activity, sequence};

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
}
