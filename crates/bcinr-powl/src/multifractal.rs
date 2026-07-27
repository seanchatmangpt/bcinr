//! Multifractal consequence allocation over a POWL 2.0 process hierarchy.
//!
//! This is the join between the two halves of the system that previously had
//! no edge between them: [`crate::powl2::Powl2Model`] supplies a hierarchy
//! whose depth and arity are set by the process, and
//! [`bcinr_cmca::cascade`] supplies a multiplicative escort cascade that can
//! run over a tree of any shape with a **lens per level**.
//!
//! The result answers "how much does this subtree matter, under lens `q`, at
//! this depth" for every node of a process -- the structural allocation
//! question, asked at every scale of a workflow rather than at one flat level.
//!
//! # Which cascade this uses, and why
//!
//! Not [`bcinr_cmca::allocator::allocate`]: that one is the certified
//! branchless path, and it is hard-bounded to `N = 8` nodes at depth `<= 8`
//! with a single scalar `q`. A POWL model recomposed from a real plan exceeds
//! `N = 8` at a single level routinely (each `recompose`d child carries
//! several tau gates), and the whole point of a *multi*fractal reading is a
//! different exponent at different depths. So this uses
//! [`bcinr_cmca::cascade::consequence_mass`], which trades constant time for
//! unbounded shape. Both compute the same measure where their domains overlap.
//!
//! # Masses are the caller's to supply
//!
//! [`uniform_mass`] weights every node equally, which is the honest default
//! when nothing better is known -- it makes the cascade report pure structural
//! share. Real allocation wants real masses (cost, duration, business value,
//! recomputation cost), which only the caller has. Supply them via
//! [`consequence_mass`]'s `mass_of` argument.

use bcinr_cmca::cascade::{consequence_mass as cascade_mass, CascadeRefusal, CascadeTree};
use bcinr_cmca::fixed::NonNegativeFixed;
use std::collections::BTreeMap;

use crate::powl2::Powl2Model;
use crate::process_toolkit::{process_nodes, ProcessNodeRef, ProcessToolkitError};

/// Refusal from multifractal allocation over a process model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultifractalError {
    /// The model itself is structurally invalid.
    Process(ProcessToolkitError),
    /// The cascade refused -- see [`CascadeRefusal`] for the precise reason
    /// (cyclic parent, degenerate sibling set, zero mass under a negative
    /// lens, ...).
    Cascade(CascadeRefusal),
}

impl std::fmt::Display for MultifractalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Process(error) => write!(f, "invalid process model: {error}"),
            Self::Cascade(refusal) => write!(f, "cascade refused: {refusal:?}"),
        }
    }
}

impl std::error::Error for MultifractalError {}

impl From<ProcessToolkitError> for MultifractalError {
    fn from(value: ProcessToolkitError) -> Self {
        Self::Process(value)
    }
}

impl From<CascadeRefusal> for MultifractalError {
    fn from(value: CascadeRefusal) -> Self {
        Self::Cascade(value)
    }
}

/// Weight every node equally. Safe under every lens (including negative ones,
/// which are undefined on zero mass), and reports pure structural share.
#[must_use]
pub fn uniform_mass(_node: &Powl2Model) -> NonNegativeFixed {
    NonNegativeFixed::ONE
}

/// Flatten a process model into the parent/mass arrays the cascade consumes.
///
/// Node order is [`process_nodes`]' preorder, so the returned indices line up
/// with the [`ProcessNodeRef`]s [`consequence_mass`] pairs them back with. The
/// root (empty path) is the tree's single root; every other node's parent is
/// the node whose path is its own path minus the last step.
pub fn cascade_tree(
    model: &Powl2Model,
    mass_of: impl Fn(&Powl2Model) -> NonNegativeFixed,
) -> Result<(CascadeTree, Vec<ProcessNodeRef>), MultifractalError> {
    let nodes = process_nodes(model)?;
    let index_of: BTreeMap<&[u16], usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, (reference, _))| (reference.path(), index))
        .collect();

    let mut parent = Vec::with_capacity(nodes.len());
    let mut mass = Vec::with_capacity(nodes.len());
    let mut refs = Vec::with_capacity(nodes.len());
    for (reference, node) in &nodes {
        let path = reference.path();
        parent.push(match path.len() {
            0 => None,
            depth => index_of.get(&path[..depth - 1]).copied(),
        });
        mass.push(mass_of(node));
        refs.push(reference.clone());
    }
    Ok((CascadeTree::new(parent, mass)?, refs))
}

/// Consequence mass for every node of a process, under a lens per level.
///
/// `lenses[d]` applies at depth `d` (root is depth 0); depths past the slice
/// reuse its last entry, so a one-element slice reproduces single-exponent
/// behaviour and an empty slice is the coverage lens throughout. The returned
/// pairs are in [`process_nodes`] preorder, and the root's mass is `ONE`
/// (it is the only member of its sibling group).
///
/// # Examples
///
/// ```
/// use bcinr_powl::multifractal::{consequence_mass, uniform_mass};
/// use bcinr_powl::process_toolkit::{activity, partial_order};
///
/// let model = partial_order(vec![activity("a"), activity("b")], vec![]).unwrap();
/// // Proportional at the root, exploitation one level down.
/// let allocated = consequence_mass(&model, &[1, 2], uniform_mass).unwrap();
/// assert_eq!(allocated.len(), 3); // the partial order plus its two children
/// ```
pub fn consequence_mass(
    model: &Powl2Model,
    lenses: &[i32],
    mass_of: impl Fn(&Powl2Model) -> NonNegativeFixed,
) -> Result<Vec<(ProcessNodeRef, NonNegativeFixed)>, MultifractalError> {
    let (tree, refs) = cascade_tree(model, mass_of)?;
    let allocated = cascade_mass(&tree, lenses)?;
    Ok(refs.into_iter().zip(allocated).collect())
}
