//! Arbitrary-arity, arbitrary-depth multifractal consequence cascade.
//!
//! # Why this exists alongside [`crate::allocator`]
//!
//! [`crate::allocator::allocate`] is the *certified* cascade: branchless
//! ($CC=1$), allocation-free, side-channel resilient -- and, as the price of
//! those properties, hard-bounded to `N = 8` nodes and depth `<= 8`
//! (`parent[x & 7]`, eight literal `flow_step` calls) with a single scalar `q`
//! applied identically at every level. Those bounds are not a defect of that
//! function; they are what make it constant-time. Widening it would destroy
//! the property that justifies it.
//!
//! This module is the *analysis* cascade: it allocates, it branches, and in
//! exchange it takes a tree of any shape and a **lens per level**. It is meant
//! for structural allocation over a real hierarchy (e.g. a POWL 2.0 process
//! tree, whose depth and arity are set by the process, not by a register
//! width), not for a hot control loop. Both compute the same measure; pick by
//! whether you need the constant-time guarantee or the unbounded shape.
//!
//! # The measure
//!
//! At each parent `p` with children `c_1..c_k`, the local *escort*
//! distribution at lens `q` is
//!
//! ```text
//!   L_q(c_i | p) = m(c_i)^q / SUM_j m(c_j)^q
//! ```
//!
//! and a node's consequence mass is the product down its root path:
//!
//! ```text
//!   pi(v) = PRODUCT over edges (p -> c) on root..v of L_q(c | p)
//! ```
//!
//! This is a multiplicative cascade of escort measures -- the standard
//! multifractal construction. What makes it genuinely *multi*fractal rather
//! than one global exponent is that `q` is read **per depth**: `lenses[d]`
//! applies to nodes at depth `d`. A single-element `lenses` slice reproduces
//! the monofractal, one-exponent behaviour of the certified path.
//!
//! # Determinism
//!
//! Lenses are integers, and `m^q` is computed by repeated `saturating_mul`
//! (and one `saturating_div` for negative `q`) over Q16.16
//! [`NonNegativeFixed`] -- no `powf`, no libm, no floating point anywhere. The
//! result is bit-identical on every platform, exactly as the certified path
//! is. Integer lenses are not a restriction in practice: the lens vocabulary
//! this crate is built around (exploitation `2`, proportional `1`, coverage
//! `0`, rare `-1`) is already integral.

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::fixed::NonNegativeFixed;

/// Largest `|q|` a lens may take. Bounds the repeated-multiplication loop in
/// [`escort_weight`]; `2` (exploitation) through `-1` (rare) sit far inside it.
pub const MAX_LENS_MAGNITUDE: u32 = 16;

/// A tree of masses to cascade over. `parent[i]` is `None` for a root.
///
/// Multiple roots are allowed: they are treated as one sibling group and
/// normalized against each other, matching `allocator`'s own `root_w_sum`
/// handling.
///
/// `rho[v]` splits an internal node's flow between the two paths of the
/// measure (see [`consequence_mass`]): `ONE` is pure descendant (flow reaches
/// leaves only by passing through every intervening level), `ZERO` is pure
/// flat (flow lands directly on the subtree's leaves, skipping the levels
/// between). [`CascadeTree::new`] pins `rho = ONE` for every node;
/// [`CascadeTree::with_rho`] takes it per node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadeTree {
    parent: Vec<Option<usize>>,
    mass: Vec<NonNegativeFixed>,
    rho: Vec<NonNegativeFixed>,
}

/// Typed refusal from the cascade. Every variant is a real condition under
/// which no correct answer exists -- none is a silent degradation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadeRefusal {
    /// `parent` and `mass` describe different numbers of nodes.
    LengthMismatch { parents: usize, masses: usize },
    /// `parent[node]` names a node index that does not exist.
    ParentOutOfRange {
        node: usize,
        parent: usize,
        len: usize,
    },
    /// Walking `node` toward a root exceeded the node count, so the `parent`
    /// relation contains a cycle. The `N`-generic counterpart of
    /// [`crate::allocator::check_hierarchy_acyclic`]'s bounded ancestor
    /// doubling.
    Cyclic { node: usize },
    /// A non-empty tree in which every node has a parent -- no root to start
    /// the cascade from. Implied by `Cyclic` in most cases but reported
    /// distinctly when the tree is small enough to have no cycle witness.
    NoRoot,
    /// A lens exceeded [`MAX_LENS_MAGNITUDE`].
    ExponentOutOfRange { lens: i32, max_magnitude: u32 },
    /// A sibling group's escort weights all came out zero, so the local
    /// distribution `w_i / SUM w_j` has no denominator. Refused rather than
    /// guarded to `ONE`, which is exactly the silent-degradation failure the
    /// cyclic-parent bug in `allocator` used to have.
    DegenerateSiblingSet { parent: Option<usize> },
    /// The escort weights over a node's *subtree leaves* all came out zero
    /// while that node has a non-zero flat part (`rho < ONE`), so the flat
    /// path has no denominator. Only reachable when the flat path actually
    /// carries mass -- a degenerate leaf set under `rho == ONE` is harmless
    /// and is not refused.
    DegenerateSubtreeLeaves { node: usize },
    /// A zero-mass node under a negative lens. `0^q` for `q < 0` is
    /// unbounded, so its escort weight is undefined -- there is no correct
    /// finite answer, and saturating to `MAX` would silently make a
    /// zero-mass node dominate every sibling.
    ZeroMassUnderNegativeLens { node: usize, lens: i32 },
}

impl CascadeTree {
    /// Build a tree with `rho = ONE` everywhere -- the pure descendant
    /// cascade, in which flow reaches a leaf only by passing through every
    /// intervening level. Validates index ranges and length agreement;
    /// acyclicity is checked later, by [`consequence_mass`], where the depth
    /// walk that would detect it happens anyway.
    pub fn new(
        parent: Vec<Option<usize>>,
        mass: Vec<NonNegativeFixed>,
    ) -> Result<Self, CascadeRefusal> {
        let rho = vec![NonNegativeFixed::ONE; parent.len()];
        Self::with_rho(parent, mass, rho)
    }

    /// Build a tree with an explicit per-node `rho`, the general two-path
    /// measure. See [`consequence_mass`] for what the two paths are.
    pub fn with_rho(
        parent: Vec<Option<usize>>,
        mass: Vec<NonNegativeFixed>,
        rho: Vec<NonNegativeFixed>,
    ) -> Result<Self, CascadeRefusal> {
        if parent.len() != mass.len() {
            return Err(CascadeRefusal::LengthMismatch {
                parents: parent.len(),
                masses: mass.len(),
            });
        }
        if parent.len() != rho.len() {
            return Err(CascadeRefusal::LengthMismatch {
                parents: parent.len(),
                masses: rho.len(),
            });
        }
        let len = parent.len();
        for (node, slot) in parent.iter().enumerate() {
            if let Some(p) = slot {
                if *p >= len {
                    return Err(CascadeRefusal::ParentOutOfRange {
                        node,
                        parent: *p,
                        len,
                    });
                }
            }
        }
        Ok(Self { parent, mass, rho })
    }

    #[must_use]
    pub fn rho(&self) -> &[NonNegativeFixed] {
        &self.rho
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    #[must_use]
    pub fn parent(&self) -> &[Option<usize>] {
        &self.parent
    }

    #[must_use]
    pub fn mass(&self) -> &[NonNegativeFixed] {
        &self.mass
    }
}

/// `m^q` in Q16.16, by repeated multiplication -- no `powf`, no libm.
///
/// `q == 0` is `ONE` for any mass (including zero: the coverage lens weights
/// every sibling equally by construction).
pub fn escort_weight(
    mass: NonNegativeFixed,
    lens: i32,
    node: usize,
) -> Result<NonNegativeFixed, CascadeRefusal> {
    let magnitude = lens.unsigned_abs();
    if magnitude > MAX_LENS_MAGNITUDE {
        return Err(CascadeRefusal::ExponentOutOfRange {
            lens,
            max_magnitude: MAX_LENS_MAGNITUDE,
        });
    }
    if lens == 0 {
        return Ok(NonNegativeFixed::ONE);
    }
    let mut accumulated = NonNegativeFixed::ONE;
    for _ in 0..magnitude {
        accumulated = accumulated.saturating_mul(mass);
    }
    if lens > 0 {
        return Ok(accumulated);
    }
    if accumulated.to_bits() == 0 {
        return Err(CascadeRefusal::ZeroMassUnderNegativeLens { node, lens });
    }
    Ok(NonNegativeFixed::ONE.saturating_div(accumulated))
}

/// Depth of every node (roots at depth 0), refusing on a cyclic `parent`.
///
/// Walks each node toward a root, capping the walk at the node count: a
/// well-formed forest reaches a root within `len` hops from any node, so
/// exceeding that is exactly a cycle witness.
fn depths(tree: &CascadeTree) -> Result<Vec<usize>, CascadeRefusal> {
    let len = tree.len();
    let mut depth = vec![0usize; len];
    for node in 0..len {
        let mut steps = 0usize;
        let mut cursor = node;
        while let Some(p) = tree.parent[cursor] {
            steps += 1;
            if steps > len {
                return Err(CascadeRefusal::Cyclic { node });
            }
            cursor = p;
        }
        depth[node] = steps;
    }
    Ok(depth)
}

/// Consequence mass for every node, under a lens per level and the two-path
/// flow split.
///
/// # The two paths
///
/// At an internal node `v` carrying flow `f`, the measure splits:
///
/// ```text
///   descendant part = rho[v] * f
///       -> immediate children, weighted by an escort over CHILDREN
///   flat part       = (ONE - rho[v]) * f
///       -> every leaf of v's subtree, weighted by an escort over those
///          LEAVES, skipping every intervening level
/// ```
///
/// This mirrors `allocator::flow_step`'s `desc_part`/`flat_part` split
/// exactly, so this function is the `N`-generic, per-level-lens
/// generalization of the certified measure rather than a different one that
/// happens to share a name. `rho == ONE` everywhere reduces it to the pure
/// multiplicative escort cascade `pi(v) = PRODUCT of L_q(child | parent)`,
/// bit-for-bit.
///
/// # Lenses
///
/// `lenses[d]` is the exponent applied to nodes at depth `d`; depths past the
/// end of the slice reuse its last entry (so a one-element slice is the
/// single-exponent, monofractal case, and an empty slice is the coverage lens
/// `q = 0` throughout). Escort weights for the flat path use the *leaf's* own
/// depth, which is what makes the flat path per-level too.
///
/// # What the returned values mean
///
/// One value per node, in the tree's own index order. For a leaf it is the
/// total mass that arrived, by either path
/// (`flow + flat`, matching `allocator`'s `res[x] = flat_alloc[x] +
/// alloc_flow[x]`). For an internal node it is the flow that *passed through*
/// it before splitting -- which is what makes "how much does this subtree
/// matter" answerable at every depth, where `allocator`'s leaf-only output
/// cannot answer it. Leaves sum to `ONE` (up to Q16.16 flooring).
pub fn consequence_mass(
    tree: &CascadeTree,
    lenses: &[i32],
) -> Result<Vec<NonNegativeFixed>, CascadeRefusal> {
    let len = tree.len();
    if len == 0 {
        return Ok(Vec::new());
    }
    let depth = depths(tree)?;
    if !tree.parent.iter().any(Option::is_none) {
        return Err(CascadeRefusal::NoRoot);
    }

    let lens_at = |d: usize| -> i32 {
        if lenses.is_empty() {
            0
        } else {
            lenses[d.min(lenses.len() - 1)]
        }
    };

    // One escort weight per node, reused by both the child escort and the
    // subtree-leaf escort -- `allocator` likewise derives `child_w` and
    // `leaf_w` from the same per-node masses.
    let mut weight = Vec::with_capacity(len);
    for node in 0..len {
        weight.push(escort_weight(tree.mass[node], lens_at(depth[node]), node)?);
    }

    let is_leaf: Vec<bool> = (0..len)
        .map(|node| !tree.parent.iter().any(|p| *p == Some(node)))
        .collect();
    let children: Vec<Vec<usize>> = (0..len)
        .map(|v| (0..len).filter(|c| tree.parent[*c] == Some(v)).collect())
        .collect();

    // Subtree leaves per node, built by walking each leaf up to its root. The
    // depth walk already proved acyclicity, so this terminates.
    let mut subtree_leaves: Vec<Vec<usize>> = vec![Vec::new(); len];
    for leaf in (0..len).filter(|node| is_leaf[*node]) {
        let mut cursor = tree.parent[leaf];
        while let Some(ancestor) = cursor {
            subtree_leaves[ancestor].push(leaf);
            cursor = tree.parent[ancestor];
        }
    }

    // `flow` is what arrives via the descendant path; `flat` is what lands
    // directly on a leaf from some ancestor's flat part.
    let mut flow = vec![NonNegativeFixed::ZERO; len];
    let mut flat = vec![NonNegativeFixed::ZERO; len];

    let roots: Vec<usize> = (0..len)
        .filter(|node| tree.parent[*node].is_none())
        .collect();
    let mut root_total = NonNegativeFixed::ZERO;
    for &root in &roots {
        root_total = root_total.saturating_add(weight[root]);
    }
    if root_total.to_bits() == 0 {
        return Err(CascadeRefusal::DegenerateSiblingSet { parent: None });
    }
    for &root in &roots {
        flow[root] = weight[root].saturating_div(root_total);
    }

    // Depth order guarantees a node's inbound flow is final before it splits.
    let mut order: Vec<usize> = (0..len).collect();
    order.sort_by_key(|node| depth[*node]);

    for v in order {
        if is_leaf[v] {
            continue;
        }
        let descendant_part = tree.rho[v].saturating_mul(flow[v]);
        let flat_part = NonNegativeFixed::ONE
            .saturating_sub(tree.rho[v])
            .saturating_mul(flow[v]);

        let mut child_total = NonNegativeFixed::ZERO;
        for &c in &children[v] {
            child_total = child_total.saturating_add(weight[c]);
        }
        if child_total.to_bits() == 0 {
            return Err(CascadeRefusal::DegenerateSiblingSet { parent: Some(v) });
        }
        for &c in &children[v] {
            let share = weight[c].saturating_div(child_total);
            flow[c] = flow[c].saturating_add(descendant_part.saturating_mul(share));
        }

        // Only walk the flat path when it actually carries mass: under
        // `rho == ONE` a degenerate subtree-leaf set is harmless, and
        // refusing on it would break the pure-cascade case for no reason.
        if flat_part.to_bits() == 0 {
            continue;
        }
        let mut leaf_total = NonNegativeFixed::ZERO;
        for &x in &subtree_leaves[v] {
            leaf_total = leaf_total.saturating_add(weight[x]);
        }
        if leaf_total.to_bits() == 0 {
            return Err(CascadeRefusal::DegenerateSubtreeLeaves { node: v });
        }
        for &x in &subtree_leaves[v] {
            let share = weight[x].saturating_div(leaf_total);
            flat[x] = flat[x].saturating_add(flat_part.saturating_mul(share));
        }
    }

    Ok((0..len)
        .map(|node| flow[node].saturating_add(flat[node]))
        .collect())
}
