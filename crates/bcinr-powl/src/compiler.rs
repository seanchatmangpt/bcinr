//! # POWL Compiler & Graph Verification Engines
//!
//! This module compiles a Partially Ordered Workflow Language (POWL) Abstract Syntax Tree (AST)
//! into a flat execution tape (`PowlTape`). In addition, it provides robust static analysis gates
//! to ensure correctness and determinism of the compiled workflow graph.
//!
//! ## Compilation Strategy
//!
//! The compiler performs a single recursive descent over the AST, allocating slots on a flat tape
//! sequentially. Each compilation step returns an internal `Segment` describing:
//! - **Entry Mask**: A bitmask of slots representing the starting points of execution for the subtree.
//! - **Exit Mask**: A bitmask of slots representing the termination points of execution for the subtree.
//!
//! Node wiring (via predecessor and successor bitmasks) is handled by linking the exit mask of
//! previous subtrees to the entry mask of subsequent subtrees.
//!
//! ## Two-Phase Verification Protocol
//!
//! After compiling the AST into a flat tape, the graph must pass two validation passes to ensure
//! correctness.
//!
//! ### Phase 1: Cycle Detection (Kahn's Algorithm)
//!
//! To ensure safe execution, the graph must be free of non-loop cycles. This is verified using
//! Kahn's topological sort algorithm:
//! 1. Compute the in-degree of all non-`LoopRedo` nodes, ignoring incoming edges from `LoopRedo` nodes.
//! 2. Perform a BFS traversal starting from nodes with an in-degree of 0.
//! 3. If the traversal visits fewer non-`LoopRedo` nodes than are present in the tape, a cycle is
//!    detected, and compilation fails.
//!
//! **Complexity**: $O(V + E)$ where $V \le 64$ is the number of tape slots.
//!
//! ### Phase 2: Reachability Validation (BP-TCRV)
//!
//! The tape must contain no unreachable execution paths. The Bit-Parallel Transitive Closure Reachability
//! Validation (BP-TCRV) algorithm determines if all active non-`LoopRedo` nodes are reachable from the
//! `entry_mask` of the tape.
//!
//! **Algorithm Steps**:
//! 1. Initialize the reachability matrix $R_i^{(0)} = \text{succ\_mask}_i \cup \{i\}$.
//! 2. Compute the transitive closure via a bit-parallel Roy-Warshall algorithm. For each pivot $k \in [0, 63]$:
//!    $$R_i^{(k+1)} = R_i^{(k)} \cup (R_k^{(k)} \text{ if } k \in R_i^{(k)})$$
//!    This is executed branchlessly using full-width bitmasks:
//!
//! ```text
//! let can_reach_k = (r[i] >> k) & 1;
//! let mask = 0u64.wrapping_sub(can_reach_k);
//! r[i] |= r[k] & mask;
//! ```
//! 3. Accumulate nodes reachable from the entry mask:
//!    $$\text{reachable\_from\_entry} = \bigcup_{e \in \text{entry\_mask}} R_e$$
//! 4. Check for containment: all active non-`LoopRedo` nodes must be present in `reachable_from_entry`.
//!
//! **Complexity**: $O(V^3 / 64)$ steps where $V = 64$. By fixing the matrix size to $64 \times 64$, the
//! execution is fully deterministic, branchless ($CC=1$), and constant-time, preventing timing leaks.

use crate::tape::{OpKind, PowlTape};

// ---------------------------------------------------------------------------
// Public AST
// ---------------------------------------------------------------------------

/// A node in the Partially Ordered Workflow Language (POWL) Abstract Syntax Tree.
///
/// POWL is a process modeling language that combines block-structured constructs
/// (like sequence, exclusive choice, and loops) with partial orders (where activity
/// dependency edges are defined explicitly).
///
/// This enum represents a node in the AST, which is recursively compiled into
/// a flat execution tape (`PowlTape`) for bounded, branchless, allocation-free execution.
pub enum PowlAstNode<'a> {
    /// A named activity transition.
    ///
    /// The activity is identified by a label string slice.
    Atom(&'a str),
    /// A silent/tau transition.
    ///
    /// Used as a placeholder transition that executes without side effects or activity labels.
    Silent,
    /// Sequential composition of child nodes.
    ///
    /// The child nodes execute strictly from left to right.
    Sequence(Vec<PowlAstNode<'a>>),
    /// Partial-order composition.
    ///
    /// Children can execute concurrently or with explicit precedence constraints.
    /// Dependency edges are represented as `(from_child_idx, to_child_idx)` pairs,
    /// denoting that `children[from_child_idx]` must complete before `children[to_child_idx]`
    /// can execute.
    PartialOrder {
        /// The list of child nodes in the partial order.
        children: Vec<PowlAstNode<'a>>,
        /// Directed acyclic dependency edges between children.
        edges: Vec<(usize, usize)>,
    },
    /// Exclusive choice (XOR split-join).
    ///
    /// Exactly one of the child branches will be chosen and executed, based on scheduler logic.
    XorChoice(Vec<PowlAstNode<'a>>),
    /// Loop composition.
    ///
    /// Executes the `body` node first. Once the `body` exits, the loop either exits
    /// (terminating the loop) or executes the `redo` node and cycles back to the `body`.
    /// The `max_iters` parameter defines the maximum number of redo cycles (0 indicates unlimited).
    Loop {
        /// The body of the loop, executed first.
        body: Box<PowlAstNode<'a>>,
        /// The redo path, executed before looping back to the body.
        redo: Box<PowlAstNode<'a>>,
        /// Bounded iteration count.
        max_iters: u8,
    },
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during POWL AST compilation or validation.
#[derive(Debug, PartialEq, Eq)]
pub enum CompileError {
    /// The compiled graph has more than 64 nodes, exceeding the capacity of the `PowlTape`.
    TapeFull,
    /// A sequential composition has no child nodes.
    EmptySequence,
    /// An exclusive choice has no branches.
    EmptyChoice,
    /// A partial order has no child nodes.
    EmptyPartialOrder,
    /// A partial order dependency edge references a non-existent child index.
    InvalidEdge {
        /// The source child index of the edge.
        from: usize,
        /// The destination child index of the edge.
        to: usize,
        /// The total number of children in the partial order.
        len: usize,
    },
    /// The compiled graph contains a non-loop cycle (e.g. mutual dependency in a partial order).
    Cycle,
    /// A node is unreachable from the entry nodes of the tape (excluding LoopRedo nodes).
    Unreachable,
    /// An exclusive choice (`XorChoice`) is nested inside a loop body or redo path.
    ///
    /// This is forbidden because loop iterations could re-enable unchosen XOR branches,
    /// violating safe execution semantics.
    XorInsideLoop {
        /// The tape slot index of the XOR dispatch node.
        xor_slot: u8,
        /// The entry slot index of the loop body.
        loop_body_entry: u8,
    },
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// A compiled sub-tree is described by:
/// - `entries`: bitmask of tape slots that are the first to fire.
/// - `exits`:  bitmask of tape slots that are the last to fire.
struct Segment {
    /// Bitmask of entry slot indices.
    entries: u64,
    /// Bitmask of exit slot indices.
    exits: u64,
}

/// Wire: every exit in `from` sets `to` as successor; every entry in `to` adds `from` as predecessor.
fn wire(tape: &mut PowlTape, from_exits: u64, to_entries: u64) {
    let mut bits = from_exits;
    while bits != 0 {
        let i = bits.trailing_zeros() as usize;
        tape.ops[i].succ_mask |= to_entries;
        bits &= bits - 1;
    }
    let mut bits = to_entries;
    while bits != 0 {
        let j = bits.trailing_zeros() as usize;
        tape.ops[j].pred_mask |= from_exits;
        bits &= bits - 1;
    }
}

// ---------------------------------------------------------------------------
// Per-variant compile helpers
// ---------------------------------------------------------------------------

fn compile_atom(tape: &mut PowlTape) -> Result<Segment, CompileError> {
    let idx = tape.alloc(OpKind::Atom).ok_or(CompileError::TapeFull)?;
    let bit = 1u64 << idx;
    Ok(Segment {
        entries: bit,
        exits: bit,
    })
}

fn compile_silent(tape: &mut PowlTape) -> Result<Segment, CompileError> {
    let idx = tape.alloc(OpKind::Silent).ok_or(CompileError::TapeFull)?;
    let bit = 1u64 << idx;
    Ok(Segment {
        entries: bit,
        exits: bit,
    })
}

fn compile_sequence<'a>(
    children: &'a [PowlAstNode<'a>],
    tape: &mut PowlTape,
) -> Result<Segment, CompileError> {
    if children.is_empty() {
        return Err(CompileError::EmptySequence);
    }
    let mut seg = compile_node(&children[0], tape)?;
    for child in &children[1..] {
        let next = compile_node(child, tape)?;
        wire(tape, seg.exits, next.entries);
        seg = Segment {
            entries: seg.entries,
            exits: next.exits,
        };
    }
    Ok(seg)
}

fn compile_partial_order<'a>(
    children: &'a [PowlAstNode<'a>],
    edges: &[(usize, usize)],
    tape: &mut PowlTape,
) -> Result<Segment, CompileError> {
    if children.is_empty() {
        return Err(CompileError::EmptyPartialOrder);
    }
    // Compile each child independently.
    let mut child_segs: Vec<Segment> = Vec::with_capacity(children.len());
    for child in children {
        child_segs.push(compile_node(child, tape)?);
    }

    // Wire explicit dependency edges.
    for &(from, to) in edges {
        if from >= children.len() || to >= children.len() {
            return Err(CompileError::InvalidEdge {
                from,
                to,
                len: children.len(),
            });
        }
        wire(tape, child_segs[from].exits, child_segs[to].entries);
    }

    // Collect overall entries/exits (children with no incoming/outgoing edges).
    let mut has_incoming = vec![false; children.len()];
    let mut has_outgoing = vec![false; children.len()];
    for &(from, to) in edges {
        has_outgoing[from] = true;
        has_incoming[to] = true;
    }

    let mut entries = 0u64;
    let mut exits = 0u64;
    for (i, seg) in child_segs.iter().enumerate() {
        if !has_incoming[i] {
            entries |= seg.entries;
        }
        if !has_outgoing[i] {
            exits |= seg.exits;
        }
    }

    // If there are multiple exits, emit a synthetic silent join.
    let exits = if exits.count_ones() > 1 {
        let join_idx = tape.alloc(OpKind::Join).ok_or(CompileError::TapeFull)?;
        let join_bit = 1u64 << join_idx;
        wire(tape, exits, join_bit);
        join_bit
    } else {
        exits
    };

    Ok(Segment { entries, exits })
}

fn compile_xor_choice<'a>(
    branches: &'a [PowlAstNode<'a>],
    tape: &mut PowlTape,
) -> Result<Segment, CompileError> {
    if branches.is_empty() {
        return Err(CompileError::EmptyChoice);
    }

    let dispatch_idx = tape
        .alloc(OpKind::XorDispatch)
        .ok_or(CompileError::TapeFull)?;
    let dispatch_bit = 1u64 << dispatch_idx;
    let join_idx = tape.alloc(OpKind::Join).ok_or(CompileError::TapeFull)?;
    let join_bit = 1u64 << join_idx;

    let mut branch_entries = 0u64;
    for branch in branches {
        let seg = compile_node(branch, tape)?;
        wire(tape, dispatch_bit, seg.entries);
        wire(tape, seg.exits, join_bit);
        branch_entries |= seg.entries;
    }

    tape.ops[dispatch_idx as usize].branch_mask = branch_entries;
    tape.ops[dispatch_idx as usize].branch_count = branches.len() as u8;
    // join pred_mask = branch_entries (all); scheduler suppresses unchosen branches
    // via choice_taken. See scheduler.rs for the XOR suppression protocol.
    tape.ops[join_idx as usize].pred_mask = branch_entries;

    Ok(Segment {
        entries: dispatch_bit,
        exits: join_bit,
    })
}

fn compile_loop<'a>(
    body: &'a PowlAstNode<'a>,
    redo: &'a PowlAstNode<'a>,
    max_iters: u8,
    tape: &mut PowlTape,
) -> Result<Segment, CompileError> {
    let pre_len = tape.len;
    let body_seg = compile_node(body, tape)?;
    let redo_seg = compile_node(redo, tape)?;

    // Wire body exits → redo entries (the redo path).
    wire(tape, body_seg.exits, redo_seg.entries);

    // Scan newly allocated slots for XorDispatch (forbidden inside loop body/redo).
    for i in pre_len as usize..tape.len as usize {
        if tape.ops[i].kind == OpKind::XorDispatch {
            let loop_body_entry = body_seg.entries.trailing_zeros() as u8;
            return Err(CompileError::XorInsideLoop {
                xor_slot: i as u8,
                loop_body_entry,
            });
        }
    }

    // Back-edge: redo exits → body entries via a LoopRedo slot.
    let back_idx = tape.alloc(OpKind::LoopRedo).ok_or(CompileError::TapeFull)?;
    let back_bit = 1u64 << back_idx;
    wire(tape, redo_seg.exits, back_bit);

    // succ_mask points to body entries; pred_mask on body entries NOT set
    // for the back-edge (would create a cycle in Kahn's check).
    tape.ops[back_idx as usize].succ_mask = body_seg.entries;
    // Store max_iters in branch_count (0 = unlimited).
    tape.ops[back_idx as usize].branch_count = max_iters;

    Ok(Segment {
        entries: body_seg.entries,
        exits: body_seg.exits,
    })
}

// ---------------------------------------------------------------------------
// Recursive compiler — thin dispatch
// ---------------------------------------------------------------------------

fn compile_node<'a>(
    node: &'a PowlAstNode<'a>,
    tape: &mut PowlTape,
) -> Result<Segment, CompileError> {
    match node {
        PowlAstNode::Atom(_label) => compile_atom(tape),
        PowlAstNode::Silent => compile_silent(tape),
        PowlAstNode::Sequence(children) => compile_sequence(children, tape),
        PowlAstNode::PartialOrder { children, edges } => {
            compile_partial_order(children, edges, tape)
        }
        PowlAstNode::XorChoice(branches) => compile_xor_choice(branches, tape),
        PowlAstNode::Loop {
            body,
            redo,
            max_iters,
        } => compile_loop(body, redo, *max_iters, tape),
    }
}

// ---------------------------------------------------------------------------
// Kahn's cycle detection (ignoring LoopRedo back-edges)
// ---------------------------------------------------------------------------

/// Compute in-degrees for non-LoopRedo slots, ignoring LoopRedo predecessors.
// `i` indexes both `tape.ops` (read) and `in_deg` (write) in the same loop
// body, and the inner `while` loop compares a second, unrelated index `j`
// against `n` -- clippy's `enumerate()`-based rewrite for `needless_range_loop`
// only tracks one iterated collection and doesn't apply cleanly here.
#[allow(clippy::needless_range_loop)]
fn build_in_degrees(tape: &PowlTape, n: usize) -> [u32; 64] {
    let mut in_deg = [0u32; 64];
    for i in 0..n {
        if tape.ops[i].kind == OpKind::LoopRedo {
            continue;
        }
        let mut preds = tape.ops[i].pred_mask;
        while preds != 0 {
            let j = preds.trailing_zeros() as usize;
            if j < n && tape.ops[j].kind != OpKind::LoopRedo {
                in_deg[i] += 1;
            }
            preds &= preds - 1;
        }
    }
    in_deg
}

/// BFS walk over non-LoopRedo slots; returns `Err(Cycle)` if any slot unreachable.
fn run_kahn_walk(tape: &PowlTape, n: usize, mut in_deg: [u32; 64]) -> Result<(), CompileError> {
    let mut queue: Vec<usize> = (0..n)
        .filter(|&i| in_deg[i] == 0 && tape.ops[i].kind != OpKind::LoopRedo)
        .collect();
    let mut visited = 0usize;

    while let Some(u) = queue.pop() {
        visited += 1;
        let mut succs = tape.ops[u].succ_mask;
        while succs != 0 {
            let v = succs.trailing_zeros() as usize;
            if v < n && tape.ops[v].kind != OpKind::LoopRedo {
                in_deg[v] = in_deg[v].saturating_sub(1);
                if in_deg[v] == 0 {
                    queue.push(v);
                }
            }
            succs &= succs - 1;
        }
    }

    let non_redo_count = (0..n)
        .filter(|&i| tape.ops[i].kind != OpKind::LoopRedo)
        .count();
    if visited < non_redo_count {
        Err(CompileError::Cycle)
    } else {
        Ok(())
    }
}

/// Phase 1 of the two-phase Kahn validation process: detect non-loop cycles.
///
/// This function executes Kahn's topological sort algorithm on the tape, ignoring loop redo
/// back-edges, to guarantee that no non-loop cycles exist.
///
/// # Algorithm & Complexity
///
/// 1. Computes in-degrees for all non-[`OpKind::LoopRedo`] slots, ignoring any predecessors
///    originating from [`OpKind::LoopRedo`] nodes.
/// 2. Performs a BFS traversal starting from slots with an in-degree of 0.
/// 3. Returns `Ok(())` if the count of visited non-LoopRedo slots equals the total number
///    of non-LoopRedo slots, otherwise returns `Err(CompileError::Cycle)`.
///
/// Complexity: $O(V + E)$ where $V \le 64$ is the number of active slots on the tape.
///
/// # Errors
///
/// Returns `Err(CompileError::Cycle)` if a non-loop cycle is detected in the graph.
///
/// # Examples
///
/// ```
/// use bcinr_powl::compiler::{compile_powl, check_full_graph_acyclic, PowlAstNode, CompileError};
///
/// // Create a valid sequence
/// let ast = PowlAstNode::Sequence(vec![
///     PowlAstNode::Atom("A"),
///     PowlAstNode::Atom("B"),
/// ]);
/// let tape = compile_powl(&ast).unwrap();
/// assert!(check_full_graph_acyclic(&tape).is_ok());
/// ```
pub fn check_full_graph_acyclic(tape: &PowlTape) -> Result<(), CompileError> {
    let n = tape.len as usize;
    let in_deg = build_in_degrees(tape, n);
    run_kahn_walk(tape, n, in_deg)
}

/// Validate reachability of all active non-LoopRedo nodes from the entry mask.
///
/// This function employs a Bit-Parallel Roy-Warshall transitive closure algorithm to verify
/// branchlessly that every active non-LoopRedo slot on the tape is reachable from the tape's
/// entry mask.
///
/// # Mathematical Contract (Hoare Logic)
///
/// ```text
/// { P(tape) }
/// bp_tcrv_validate_reachability(tape)
/// { Q(tape, result) }
/// ```
///
/// - **Precondition $P$**: The tape length satisfies $L \le 64$.
/// - **Postcondition $Q$**: The output `result` is:
///   - `!0u64` (all bits set / `u64::MAX`) if for every slot $i < L$ where `kind != LoopRedo`,
///     there exists a path from some entry $e \in \text{entry\_mask}$ to $i$ using successor edges.
///   - `0u64` (all bits zero) otherwise.
///
/// # Algorithm & Complexity
///
/// The algorithm is implemented branchlessly to comply with timing-invariant constraints:
/// 1. **Initialization**: Set $R_i^{(0)} = \text{succ\_mask}_i \cup \{i\}$ for all $i < 64$ within bounds.
/// 2. **Transitive Closure Propagation**: For each pivot $k$ from 0 to 63:
///    $$R_i^{(k+1)} = R_i^{(k)} \cup (R_k^{(k)} \text{ if } k \in R_i^{(k)})$$
///    This is calculated via bitwise masks:
///
/// ```text
/// let can_reach_k = (r[i] >> k) & 1;
/// let mask = 0u64.wrapping_sub(can_reach_k); // !0u64 if reachable, 0u64 otherwise
/// r[i] |= r[k] & mask;
/// ```
/// 3. **Reachability Analysis**: Compute the union of reachable nodes from the entry mask:
///    $$\text{reachable\_from\_entry} = \bigcup_{e \in \text{entry\_mask}} R_e$$
/// 4. **Validation**: Compare against the mask of required nodes (`must_be_reachable`).
///    Returns `!0u64` if `must_be_reachable & !reachable_from_entry == 0`, and `0u64` otherwise.
///
/// Complexity: $O(V^3 / 64)$ steps where $V = 64$. With a fixed bound, execution takes a constant
/// number of clock cycles, guaranteeing timing immunity.
///
/// # Examples
///
/// ```
/// use bcinr_powl::compiler::{compile_powl, bp_tcrv_validate_reachability, PowlAstNode};
///
/// let ast = PowlAstNode::Sequence(vec![
///     PowlAstNode::Atom("A"),
///     PowlAstNode::Atom("B"),
/// ]);
/// let tape = compile_powl(&ast).unwrap();
///
/// // Since B is reachable from A (which is the entry), this returns !0u64.
/// assert_eq!(bp_tcrv_validate_reachability(&tape), !0u64);
/// ```
#[must_use]
#[inline(always)]
pub fn bp_tcrv_validate_reachability(tape: &PowlTape) -> u64 {
    let mut r = [0u64; 64];
    let tape_len = tape.len as usize;
    let entry_mask = tape.entry_mask;

    // Step 1: Initialize the reachability matrix branchlessly.
    // Fixed loop bound of 64 allows complete compiler unrolling.
    for i in 0..64 {
        let in_bounds = (i < tape_len) as u64;
        let bounds_mask = 0u64.wrapping_sub(in_bounds);

        let succs = tape.ops[i].succ_mask & bounds_mask;
        r[i] = succs | (1u64 << i);
    }

    // Step 2: Bit-Parallel Roy-Warshall transitive closure propagation.
    // 64 iterations, fully deterministic.
    for k in 0..64 {
        let r_k = r[k];
        for i in 0..64 {
            let can_reach_k = (r[i] >> k) & 1;
            let mask = 0u64.wrapping_sub(can_reach_k);
            r[i] |= r_k & mask;
        }
    }

    // Step 3: Accumulate reachable set from entry mask branchlessly.
    let mut reachable_from_entry = 0u64;
    for i in 0..64 {
        let is_entry = (entry_mask >> i) & 1;
        let mask = 0u64.wrapping_sub(is_entry);
        reachable_from_entry |= r[i] & mask;
    }

    // Step 4: Construct mask of nodes requiring reachability.
    let mut must_be_reachable = 0u64;
    for i in 0..64 {
        let in_bounds = (i < tape_len) as u64;
        let is_not_redo = (tape.ops[i].kind != OpKind::LoopRedo) as u64;
        let active = in_bounds & is_not_redo;
        let mask = 0u64.wrapping_sub(active);
        must_be_reachable |= (1u64 << i) & mask;
    }

    // Step 5: Check for containment violations.
    let violation = must_be_reachable & !reachable_from_entry;
    let is_valid = (violation == 0) as u64;

    0u64.wrapping_sub(is_valid)
}

/// Phase 2 of the two-phase Kahn validation process: verify reachability.
///
/// Ensures that every active non-[`OpKind::LoopRedo`] slot is reachable from at least
/// one entry node in the entry mask, either directly or transitively via successor edges.
///
/// # Errors
///
/// Returns `Err(CompileError::Unreachable)` if any non-LoopRedo node is unreachable.
///
/// # Examples
///
/// ```
/// use bcinr_powl::compiler::{compile_powl, check_all_ops_reachable, PowlAstNode};
///
/// let ast = PowlAstNode::Sequence(vec![
///     PowlAstNode::Atom("A"),
///     PowlAstNode::Atom("B"),
/// ]);
/// let tape = compile_powl(&ast).unwrap();
/// assert!(check_all_ops_reachable(&tape).is_ok());
/// ```
pub fn check_all_ops_reachable(tape: &PowlTape) -> Result<(), CompileError> {
    let status = bp_tcrv_validate_reachability(tape);
    if status == !0u64 {
        Ok(())
    } else {
        Err(CompileError::Unreachable)
    }
}

fn kahn_check(tape: &PowlTape) -> Result<(), CompileError> {
    check_full_graph_acyclic(tape)?;
    check_all_ops_reachable(tape)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Compiles a POWL AST into a flat [`PowlTape`].
///
/// Compilation performs a recursive descent traversal over the AST structure to allocate tape
/// slots and wire execution flow (represented by predecessor and successor bitmasks).
/// It then performs a two-phase validation:
/// 1. **Cycle Detection**: Kahn's algorithm verifies the absence of non-loop cycles.
/// 2. **Reachability Check**: The bit-parallel Roy-Warshall algorithm verifies that all
///    non-loop nodes are reachable from the entry mask.
///
/// # Errors
///
/// Returns a [`CompileError`] if:
/// - The compiled graph exceeds 64 nodes ([`CompileError::TapeFull`]).
/// - Structural constraints are violated (e.g., empty sequences, invalid edges, nested XORs).
/// - Non-loop cycles are detected ([`CompileError::Cycle`]).
/// - Any node is unreachable from the entry mask ([`CompileError::Unreachable`]).
///
/// # Examples
///
/// ```
/// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
///
/// // Create a sequence workflow: Atom("A") -> Atom("B")
/// let ast = PowlAstNode::Sequence(vec![
///     PowlAstNode::Atom("A"),
///     PowlAstNode::Atom("B"),
/// ]);
///
/// let tape = compile_powl(&ast).unwrap();
/// assert_eq!(tape.len, 2);
/// assert_eq!(tape.entry_mask, 0b01);
/// assert_eq!(tape.ops[0].succ_mask, 0b10);
/// assert_eq!(tape.ops[1].pred_mask, 0b01);
/// ```
pub fn compile_powl(root: &PowlAstNode<'_>) -> Result<PowlTape, CompileError> {
    let mut tape = PowlTape::new();
    let seg = compile_node(root, &mut tape)?;
    tape.entry_mask = seg.entries;
    kahn_check(&tape)?;
    Ok(tape)
}

// =============================================================================
// v2 compiler — PowlModel -> tape::v2::PowlTape + ConcurrencyGuardTable
// =============================================================================

/// Compiles [`crate::model::PowlModel`] (the new planner-output IR) into the
/// existing v2 tape representation, mirroring [`crate::tape::v2`]'s pattern
/// of living alongside the legacy AST compiler above rather than replacing
/// it.
///
/// `PowlModel`'s node graph is already flat (one node per `ActionOccurrence`
/// plus precedence edges, not a nested AST), so this module does not reuse
/// the recursive-descent machinery above (`compile_node`/`wire`/`Segment`)
/// — there is no tree to recurse over. It *does* reuse the v2 tape's own
/// primitives unchanged: [`crate::tape::v2::PowlTape::push`],
/// [`crate::tape::v2::LabelSlab::intern`], and the established `pred_mask`/
/// `succ_mask` bit convention — no new op/tape encoding is invented here.
pub mod v2 {
    use std::collections::BTreeMap;

    use bcinr_mfw_ir::{ActionOccurrenceId, EventSet, PowlNodeId};

    use crate::model::{PowlModel, PowlNode};
    use crate::tape::v2::{
        CompiledNonFace, ConcurrencyGuardTable, OpKind as V2OpKind, Powl64Op as V2Op,
        PowlTape as V2Tape,
    };

    /// Errors from [`compile_powl_v2`]. Every variant names the exact node
    /// or action responsible — never a bare "compile failed".
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum CompileErrorV2 {
        /// More than 64 nodes — the v2 `PowlTape` (as opposed to
        /// `PowlTapeLarge`) has a hard 64-slot cap.
        TapeFull,
        /// `model.nodes` was not in dense ascending-`PowlNodeId` order
        /// (`node.id().0 == index as u64` for every position) — required
        /// so tape-slot index and `PowlNodeId` numeric value coincide,
        /// which is what lets the guard-table re-keying below work without
        /// a second indirection table.
        NodeIdNotDense {
            /// The index position where the mismatch occurred.
            position: usize,
            /// The node ID found at that position.
            found: PowlNodeId,
        },
        /// A [`crate::model::ChildWorkflowNode`] or
        /// [`crate::model::ExternalCutNode`] was encountered. These are
        /// UNSUPPORTED stubs (see `crate::model`'s doc comments) — this
        /// compiler refuses them with a typed error rather than silently
        /// dropping or mis-compiling them.
        UnsupportedNodeKind {
            /// The ID of the unsupported node.
            id: PowlNodeId,
        },
        /// A precedence edge referenced an `ActionOccurrenceId` absent
        /// from `model.provenance` — the edge cannot be wired to a tape
        /// slot.
        UnmappedActionInOrder(ActionOccurrenceId),
        /// A concurrency nonface's `EventSet` referenced a member value
        /// (a *position* in the source causal plan's occurrence list —
        /// see [`compile_powl_v2`]'s doc comment on the re-keying
        /// assumption) that is not a valid tape-slot index for this model
        /// (`>= model.nodes.len()`) — the nonface cannot be re-keyed into
        /// tape-slot space.
        UnmappedConcurrencySlot(usize),
    }

    /// The result of compiling a `PowlModel`: the existing v2 `PowlTape`
    /// plus the new concurrency guard table, plus a side map from each
    /// activity node's `PowlNodeId` to its interned label's offset in
    /// `tape.label_slab` (kept separate from `Powl64Op` itself so the
    /// already-landed, size-asserted 64-byte `Powl64Op` layout is not
    /// touched by this phase).
    /// Compiled representation of a POWL v2 model, including the flat tape,
    /// concurrency guards, and label offset mappings.
    #[derive(Debug)]
    pub struct CompiledPowlV2 {
        /// The compiled flat execution tape.
        pub tape: V2Tape,
        /// Concurrency guards that dynamically restrict concurrent execution.
        pub guards: ConcurrencyGuardTable,
        /// Mapping from each node's ID to its interned label offset in the tape's label slab.
        pub node_labels: BTreeMap<PowlNodeId, u16>,
    }

    /// Compiles a [`PowlModel`] into a [`CompiledPowlV2`].
    ///
    /// This compiles a flat [`PowlModel`] (a plan projection) into the v2 tape format
    /// [`PowlTape`](crate::tape::v2::PowlTape) along with its associated [`ConcurrencyGuardTable`].
    ///
    /// # Re-keying concurrency into tape-slot space
    ///
    /// `model.concurrency`'s `EventSet` members are **positions in the
    /// source causal plan's occurrence list** — the same convention
    /// `PddlConcurrencyAnalyzer::analyze` documents and implements
    /// (`crates/bcinr-pddl/src/concurrency.rs`, its `slot_of` map), *not*
    /// raw `ActionOccurrenceId` values (see `crate::model`'s corrected
    /// module doc comment). `PowlProjector::project` builds each
    /// `PowlNodeId` from that exact same position (`PowlNodeId(i)` for
    /// `causal.occurrences[i]`), so once this function's own
    /// `NodeIdNotDense` check above has passed (every `model.nodes[i]` has
    /// `id() == PowlNodeId(i)`), an `EventSet` member value *is* directly
    /// a valid tape-slot index / `PowlNodeId` numeric value — no
    /// `ActionOccurrenceId` round-trip is needed or correct here. This
    /// function bounds-checks each member against `model.nodes.len()` and
    /// re-keys it directly into the returned `ConcurrencyGuardTable`.
    ///
    /// Earlier code paths in this function (order-edge wiring above) still
    /// go through `model.provenance`'s `ActionOccurrenceId -> PowlNodeId`
    /// inversion, because `model.order`'s `PrecedenceEdge`s genuinely stay
    /// `ActionOccurrenceId`-keyed — concurrency and order use different
    /// key spaces in `PowlModel`, and conflating them is exactly the bug
    /// this function used to have (see
    /// `crates/bcinr-pddl/tests/mfw_capacity2_fixture.rs`'s
    /// `link4_adversarial_confirmed_bug_...`-style regression coverage,
    /// mirrored by this crate's own `non_dense_node_ids_are_rejected` and
    /// concurrency-re-keying tests below).
    ///
    /// # Errors
    ///
    /// Returns a [`CompileErrorV2`] if:
    /// - The number of nodes exceeds 64 ([`CompileErrorV2::TapeFull`]).
    /// - Node IDs are not densely packed starting from zero ([`CompileErrorV2::NodeIdNotDense`]).
    /// - Unsupported node kinds (such as child workflows or external cuts) are present ([`CompileErrorV2::UnsupportedNodeKind`]).
    /// - Precedence edges reference actions missing from provenance mapping ([`CompileErrorV2::UnmappedActionInOrder`]).
    /// - Concurrency event sets reference slots that are out of bounds ([`CompileErrorV2::UnmappedConcurrencySlot`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::compiler::v2::{compile_powl_v2, CompileErrorV2};
    /// use bcinr_powl::model::{PowlModel, PowlNode, ActivityNode};
    /// use bcinr_mfw_ir::{
    ///     ActionOccurrenceId, PowlNodeId, StrictPartialOrder, ExecutableConcurrencyComplex, Digest
    /// };
    /// use std::collections::BTreeMap;
    ///
    /// // Define two activity nodes
    /// let nodes = vec![
    ///     PowlNode::Activity(ActivityNode {
    ///         id: PowlNodeId(0),
    ///         label: "A".to_string(),
    ///         source_action: ActionOccurrenceId(0),
    ///     }),
    ///     PowlNode::Activity(ActivityNode {
    ///         id: PowlNodeId(1),
    ///         label: "B".to_string(),
    ///         source_action: ActionOccurrenceId(1),
    ///     }),
    /// ];
    ///
    /// let mut provenance = BTreeMap::new();
    /// provenance.insert(PowlNodeId(0), ActionOccurrenceId(0));
    /// provenance.insert(PowlNodeId(1), ActionOccurrenceId(1));
    ///
    /// let model = PowlModel {
    ///     nodes,
    ///     order: StrictPartialOrder::default(),
    ///     concurrency: ExecutableConcurrencyComplex {
    ///         event_count: 2,
    ///         minimal_nonfaces: vec![],
    ///         conflict_witnesses: BTreeMap::new(),
    ///         digest: Digest::hash(b"empty"),
    ///     },
    ///     provenance,
    /// };
    ///
    /// let compiled = compile_powl_v2(&model).unwrap();
    /// assert_eq!(compiled.tape.len, 2);
    /// ```
    pub fn compile_powl_v2(model: &PowlModel) -> Result<CompiledPowlV2, CompileErrorV2> {
        if model.nodes.len() > 64 {
            return Err(CompileErrorV2::TapeFull);
        }

        // Tape-slot index and PowlNodeId numeric value must coincide
        // (dense, ascending, zero-based) -- verified up front rather than
        // silently assumed.
        for (position, node) in model.nodes.iter().enumerate() {
            let id = node.id();
            if id.0 != position as u64 {
                return Err(CompileErrorV2::NodeIdNotDense {
                    position,
                    found: id,
                });
            }
        }

        let mut tape = V2Tape::new();
        let mut node_labels = BTreeMap::new();

        for node in &model.nodes {
            let mut op = V2Op::silent();
            match node {
                PowlNode::Activity(a) => {
                    op.op_kind = V2OpKind::Activity;
                    let offset = tape.label_slab.intern(&a.label);
                    node_labels.insert(a.id, offset);
                }
                PowlNode::Silent(_) => {
                    // op_kind already OpKind::Silent via V2Op::silent().
                }
                PowlNode::ChildWorkflow(c) => {
                    return Err(CompileErrorV2::UnsupportedNodeKind { id: c.id });
                }
                PowlNode::ExternalCut(e) => {
                    return Err(CompileErrorV2::UnsupportedNodeKind { id: e.id });
                }
            }
            tape.push(op).ok_or(CompileErrorV2::TapeFull)?;
        }

        // Invert provenance: ActionOccurrenceId -> PowlNodeId, used both to
        // wire order edges and to re-key the concurrency complex below.
        let mut action_to_node: BTreeMap<ActionOccurrenceId, PowlNodeId> = BTreeMap::new();
        for (node_id, action_id) in &model.provenance {
            action_to_node.insert(*action_id, *node_id);
        }

        // Wire precedence edges into pred_mask/succ_mask using the tape's
        // own established bit convention (mirrors compiler.rs's `wire()`
        // for the legacy tape, specialised to a flat edge list rather than
        // a recursive AST).
        for edge in &model.order.edges {
            let before_node = *action_to_node
                .get(&edge.before)
                .ok_or(CompileErrorV2::UnmappedActionInOrder(edge.before))?;
            let after_node = *action_to_node
                .get(&edge.after)
                .ok_or(CompileErrorV2::UnmappedActionInOrder(edge.after))?;
            let before_bit = 1u64 << before_node.0;
            let after_bit = 1u64 << after_node.0;
            tape.ops[before_node.0 as usize].succ_mask |= after_bit;
            tape.ops[after_node.0 as usize].pred_mask |= before_bit;
        }

        let n = tape.len as usize;
        tape.entry_op = (0..n)
            .find(|&i| tape.ops[i].pred_mask == 0)
            .map(|i| i as u8)
            .unwrap_or(0);
        tape.exit_op = (0..n)
            .rev()
            .find(|&i| tape.ops[i].succ_mask == 0)
            .map(|i| i as u8)
            .unwrap_or(0);

        // Re-key the concurrency complex: EventSet members are positions
        // in the source occurrence list, which (per the density check
        // above) coincide exactly with tape-slot indices / PowlNodeId
        // values — see this function's doc comment for why no
        // ActionOccurrenceId lookup belongs here.
        let mut guards = ConcurrencyGuardTable::empty();
        for nf in &model.concurrency.minimal_nonfaces {
            let mut members = EventSet::empty();
            for event_id in nf.members.iter_stable() {
                if event_id >= model.nodes.len() {
                    return Err(CompileErrorV2::UnmappedConcurrencySlot(event_id));
                }
                members.insert(event_id);
            }
            guards.nonfaces.push(CompiledNonFace {
                members,
                witness_digest: nf.witness_digest,
            });
        }

        Ok(CompiledPowlV2 {
            tape,
            guards,
            node_labels,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::model::{ActivityNode, SilentNode};
        use bcinr_mfw_ir::{
            ConcurrencyConflictWitness, Digest, ExecutableConcurrencyComplex, FluentId,
            MinimalNonFace, PrecedenceEdge, ResourceConflictWitness, StrictPartialOrder,
        };
        use std::collections::BTreeSet;

        fn activity(id: u64, label: &str, action: u32) -> PowlNode {
            PowlNode::Activity(ActivityNode {
                id: PowlNodeId(id),
                label: label.to_string(),
                source_action: ActionOccurrenceId(action),
            })
        }

        #[test]
        fn two_node_sequence_wires_pred_succ_and_entry_exit() {
            let mut order_edges = BTreeSet::new();
            order_edges.insert(PrecedenceEdge {
                before: ActionOccurrenceId(0),
                after: ActionOccurrenceId(1),
            });
            let mut provenance = BTreeMap::new();
            provenance.insert(PowlNodeId(0), ActionOccurrenceId(0));
            provenance.insert(PowlNodeId(1), ActionOccurrenceId(1));

            let model = PowlModel {
                nodes: vec![activity(0, "a", 0), activity(1, "b", 1)],
                order: StrictPartialOrder { edges: order_edges },
                concurrency: ExecutableConcurrencyComplex {
                    event_count: 2,
                    minimal_nonfaces: vec![],
                    conflict_witnesses: BTreeMap::new(),
                    digest: Digest::hash(b"empty"),
                },
                provenance,
            };

            let compiled = compile_powl_v2(&model).unwrap();
            assert_eq!(compiled.tape.len, 2);
            assert_eq!(compiled.tape.ops[0].succ_mask, 0b10);
            assert_eq!(compiled.tape.ops[1].pred_mask, 0b01);
            assert_eq!(compiled.tape.entry_op, 0);
            assert_eq!(compiled.tape.exit_op, 1);
            assert_eq!(
                compiled
                    .tape
                    .label_slab
                    .get(compiled.node_labels[&PowlNodeId(0)]),
                "a"
            );
            assert_eq!(
                compiled
                    .tape
                    .label_slab
                    .get(compiled.node_labels[&PowlNodeId(1)]),
                "b"
            );
            assert!(compiled.guards.nonfaces.is_empty());
        }

        #[test]
        fn silent_node_compiles_with_no_label() {
            let model = PowlModel {
                nodes: vec![PowlNode::Silent(SilentNode { id: PowlNodeId(0) })],
                order: StrictPartialOrder::default(),
                concurrency: ExecutableConcurrencyComplex {
                    event_count: 0,
                    minimal_nonfaces: vec![],
                    conflict_witnesses: BTreeMap::new(),
                    digest: Digest::hash(b"empty"),
                },
                provenance: BTreeMap::new(),
            };
            let compiled = compile_powl_v2(&model).unwrap();
            assert_eq!(compiled.tape.ops[0].op_kind, V2OpKind::Silent);
            assert!(compiled.node_labels.is_empty());
        }

        /// The "A, B, C can't all fire together" fixture, re-keyed into
        /// tape-slot space and checked with the exact same admit/reject
        /// assertions as `bcinr_mfw_ir::concurrency`'s own worked-complex
        /// test — proving the re-keying preserves admission semantics.
        #[test]
        fn concurrency_guard_table_mirrors_source_admission_semantics() {
            let abc = EventSet::empty().with(0).with(1).with(2);
            let witness = ConcurrencyConflictWitness {
                causal: None,
                temporal: None,
                resource: Some(ResourceConflictWitness {
                    actions: abc,
                    resource: FluentId(0),
                    capacity_milli: 2_000,
                    demanded_milli: 3_000,
                }),
            };
            let witness_digest = Digest::hash(b"abc-resource-conflict");
            let mut conflict_witnesses = BTreeMap::new();
            conflict_witnesses.insert(witness_digest, witness);

            let mut provenance = BTreeMap::new();
            provenance.insert(PowlNodeId(0), ActionOccurrenceId(0));
            provenance.insert(PowlNodeId(1), ActionOccurrenceId(1));
            provenance.insert(PowlNodeId(2), ActionOccurrenceId(2));

            let model = PowlModel {
                nodes: vec![
                    activity(0, "a", 0),
                    activity(1, "b", 1),
                    activity(2, "c", 2),
                ],
                order: StrictPartialOrder::default(),
                concurrency: ExecutableConcurrencyComplex {
                    event_count: 3,
                    minimal_nonfaces: vec![MinimalNonFace {
                        members: abc,
                        witness_digest,
                    }],
                    conflict_witnesses,
                    digest: Digest::hash(b"complex"),
                },
                provenance,
            };

            let compiled = compile_powl_v2(&model).unwrap();
            assert_eq!(compiled.guards.nonfaces.len(), 1);

            let empty = EventSet::empty();
            let a = EventSet::empty().with(0);
            let ab = EventSet::empty().with(0).with(1);
            let bc = EventSet::empty().with(1).with(2);
            let ac = EventSet::empty().with(0).with(2);
            for candidate in [empty, a, ab, bc, ac] {
                assert!(compiled.guards.admits(&candidate));
            }
            assert!(!compiled.guards.admits(&abc));
            let abcd = abc.with(3);
            assert!(!compiled.guards.admits(&abcd));
        }

        #[test]
        fn child_workflow_node_is_refused_not_silently_dropped() {
            let model = PowlModel {
                nodes: vec![PowlNode::ChildWorkflow(crate::model::ChildWorkflowNode {
                    id: PowlNodeId(0),
                })],
                order: StrictPartialOrder::default(),
                concurrency: ExecutableConcurrencyComplex {
                    event_count: 0,
                    minimal_nonfaces: vec![],
                    conflict_witnesses: BTreeMap::new(),
                    digest: Digest::hash(b"empty"),
                },
                provenance: BTreeMap::new(),
            };
            assert_eq!(
                compile_powl_v2(&model).unwrap_err(),
                CompileErrorV2::UnsupportedNodeKind { id: PowlNodeId(0) }
            );
        }

        #[test]
        fn non_dense_node_ids_are_rejected() {
            let model = PowlModel {
                // Two nodes but the second claims PowlNodeId(5), not 1.
                nodes: vec![activity(0, "a", 0), activity(5, "b", 1)],
                order: StrictPartialOrder::default(),
                concurrency: ExecutableConcurrencyComplex {
                    event_count: 0,
                    minimal_nonfaces: vec![],
                    conflict_witnesses: BTreeMap::new(),
                    digest: Digest::hash(b"empty"),
                },
                provenance: BTreeMap::new(),
            };
            assert_eq!(
                compile_powl_v2(&model).unwrap_err(),
                CompileErrorV2::NodeIdNotDense {
                    position: 1,
                    found: PowlNodeId(5)
                }
            );
        }

        #[test]
        fn more_than_64_nodes_is_tape_full() {
            let nodes: Vec<PowlNode> = (0..65).map(|i| activity(i, "x", i as u32)).collect();
            let model = PowlModel {
                nodes,
                order: StrictPartialOrder::default(),
                concurrency: ExecutableConcurrencyComplex {
                    event_count: 0,
                    minimal_nonfaces: vec![],
                    conflict_witnesses: BTreeMap::new(),
                    digest: Digest::hash(b"empty"),
                },
                provenance: BTreeMap::new(),
            };
            assert_eq!(
                compile_powl_v2(&model).unwrap_err(),
                CompileErrorV2::TapeFull
            );
        }

        /// End-to-end: project a real `CausalPlan` +
        /// `ExecutableConcurrencyComplex` via `PowlProjector`, then compile
        /// the resulting `PowlModel` -- proving the two new layers compose.
        #[test]
        fn compiles_a_real_projector_output_end_to_end() {
            use bcinr_mfw_ir::{
                ActionOccurrence, IndependenceRelation, PlanningEpochId,
                PowlProjector as PowlProjectorTrait,
            };

            let occurrences = vec![
                ActionOccurrence {
                    id: ActionOccurrenceId(0),
                    action: 10,
                },
                ActionOccurrence {
                    id: ActionOccurrenceId(1),
                    action: 11,
                },
                ActionOccurrence {
                    id: ActionOccurrenceId(2),
                    action: 12,
                },
            ];
            let mut edges = BTreeSet::new();
            edges.insert(PrecedenceEdge {
                before: ActionOccurrenceId(0),
                after: ActionOccurrenceId(2),
            });
            let causal = bcinr_mfw_ir::CausalPlan {
                epoch: PlanningEpochId(1),
                occurrences,
                precedes: StrictPartialOrder { edges },
                independence: IndependenceRelation::default(),
                support_edges: BTreeSet::new(),
                digest: Digest::hash(b"e2e-causal"),
            };
            let abc = EventSet::empty().with(0).with(1).with(2);
            let witness = ConcurrencyConflictWitness {
                causal: None,
                temporal: None,
                resource: Some(ResourceConflictWitness {
                    actions: abc,
                    resource: FluentId(0),
                    capacity_milli: 2_000,
                    demanded_milli: 3_000,
                }),
            };
            let witness_digest = Digest::hash(b"e2e-conflict");
            let mut conflict_witnesses = BTreeMap::new();
            conflict_witnesses.insert(witness_digest, witness);
            let concurrency = ExecutableConcurrencyComplex {
                event_count: 3,
                minimal_nonfaces: vec![MinimalNonFace {
                    members: abc,
                    witness_digest,
                }],
                conflict_witnesses,
                digest: Digest::hash(b"e2e-complex"),
            };

            let projector = crate::projection::PowlProjector;
            let (model, _witness) = projector.project(&causal, &concurrency).unwrap();
            let compiled = compile_powl_v2(&model).unwrap();

            assert_eq!(compiled.tape.len, 3);
            assert_eq!(compiled.guards.nonfaces.len(), 1);
            assert!(!compiled.guards.admits(&abc));
            assert_eq!(compiled.tape.ops[0].succ_mask, 1 << 2);
            assert_eq!(compiled.tape.ops[2].pred_mask, 1);
        }

        /// Regression test for the position-vs-`ActionOccurrenceId`
        /// conflation bug: a `PowlModel` built from a `CausalPlan` with
        /// sparse `ActionOccurrenceId`s (100,101,102 at positions 0,1,2)
        /// must still compile and re-key its concurrency guard table
        /// correctly — previously `compile_powl_v2` reinterpreted the
        /// `EventSet` member value `event_id` as
        /// `ActionOccurrenceId(event_id as u32)` directly, which only
        /// coincidentally worked when ids equaled positions. Paired with
        /// `crate::projection::tests::project_succeeds_with_sparse_non_positional_action_occurrence_ids`.
        #[test]
        fn compiles_real_projector_output_with_sparse_action_occurrence_ids() {
            use bcinr_mfw_ir::{
                ActionOccurrence, IndependenceRelation, PlanningEpochId,
                PowlProjector as PowlProjectorTrait,
            };

            let occurrences = vec![
                ActionOccurrence {
                    id: ActionOccurrenceId(100),
                    action: 10,
                },
                ActionOccurrence {
                    id: ActionOccurrenceId(101),
                    action: 11,
                },
                ActionOccurrence {
                    id: ActionOccurrenceId(102),
                    action: 12,
                },
            ];
            let causal = bcinr_mfw_ir::CausalPlan {
                epoch: PlanningEpochId(1),
                occurrences,
                precedes: StrictPartialOrder::default(),
                independence: IndependenceRelation::default(),
                support_edges: BTreeSet::new(),
                digest: Digest::hash(b"sparse-e2e-causal"),
            };
            // Positions {0, 1} (PddlConcurrencyAnalyzer-shaped), not the
            // raw ActionOccurrenceIds 100/101.
            let witness_digest = Digest::hash(b"sparse-e2e-conflict");
            let mut conflict_witnesses = BTreeMap::new();
            conflict_witnesses.insert(
                witness_digest,
                ConcurrencyConflictWitness {
                    causal: None,
                    temporal: None,
                    resource: None,
                },
            );
            let concurrency = ExecutableConcurrencyComplex {
                event_count: 3,
                minimal_nonfaces: vec![MinimalNonFace {
                    members: EventSet::empty().with(0).with(1),
                    witness_digest,
                }],
                conflict_witnesses,
                digest: Digest::hash(b"sparse-e2e-complex"),
            };

            let projector = crate::projection::PowlProjector;
            let (model, _witness) = projector.project(&causal, &concurrency).unwrap();
            let compiled = compile_powl_v2(&model)
                .expect("sparse ActionOccurrenceIds must not break concurrency re-keying");

            assert_eq!(compiled.guards.nonfaces.len(), 1);
            let ab = EventSet::empty().with(0).with(1);
            let ac = EventSet::empty().with(0).with(2);
            assert!(
                !compiled.guards.admits(&ab),
                "positions {{0,1}} must stay forbidden"
            );
            assert!(
                compiled.guards.admits(&ac),
                "positions {{0,2}} were never forbidden"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_single_atom() {
        let ast = PowlAstNode::Atom("a");
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.len, 1);
        assert_eq!(tape.entry_mask, 0b1);
    }

    #[test]
    fn compile_sequence_two() {
        let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")]);
        let tape = compile_powl(&ast).unwrap();
        // slot 0 = a, slot 1 = b
        assert_eq!(tape.len, 2);
        assert_eq!(tape.ops[0].succ_mask, 0b10);
        assert_eq!(tape.ops[1].pred_mask, 0b01);
    }

    #[test]
    fn compile_xor_choice() {
        let ast =
            PowlAstNode::XorChoice(vec![PowlAstNode::Atom("left"), PowlAstNode::Atom("right")]);
        let tape = compile_powl(&ast).unwrap();
        // dispatch(0) + left(1) + right(2) + join(3) — but dispatch alloc'd first
        // Actually: dispatch=0, join=1, left=2, right=3 (join alloc after dispatch)
        // No — see code: dispatch first, join second, then branches.
        // dispatch=0, join=1, branch_left=2, branch_right=3
        assert_eq!(tape.len, 4);
        assert_eq!(tape.ops[0].kind, OpKind::XorDispatch);
    }

    #[test]
    fn compile_partial_order_two_parallel() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        };
        let tape = compile_powl(&ast).unwrap();
        // a=0, b=1, join=2
        assert_eq!(tape.len, 3);
        // Both a and b are entries
        assert_eq!(tape.entry_mask & 0b11, 0b11);
    }

    #[test]
    fn compile_empty_sequence_error() {
        let ast = PowlAstNode::Sequence(vec![]);
        assert_eq!(compile_powl(&ast), Err(CompileError::EmptySequence));
    }

    #[test]
    fn compile_invalid_edge_error() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a")],
            edges: vec![(0, 5)],
        };
        assert!(matches!(
            compile_powl(&ast),
            Err(CompileError::InvalidEdge { .. })
        ));
    }

    // -------------------------------------------------------------------------
    // CompileError exhaustiveness — every variant must be reachable
    // -------------------------------------------------------------------------

    #[test]
    fn compile_error_tape_full() {
        // Build a Sequence of 65 atoms — tape only has 64 slots.
        let atoms: Vec<PowlAstNode<'_>> = (0..65).map(|_| PowlAstNode::Atom("x")).collect();
        let ast = PowlAstNode::Sequence(atoms);
        assert_eq!(compile_powl(&ast), Err(CompileError::TapeFull));
    }

    #[test]
    fn compile_error_empty_choice() {
        let ast = PowlAstNode::XorChoice(vec![]);
        assert_eq!(compile_powl(&ast), Err(CompileError::EmptyChoice));
    }

    #[test]
    fn compile_error_empty_partial_order() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![],
            edges: vec![],
        };
        assert_eq!(compile_powl(&ast), Err(CompileError::EmptyPartialOrder));
    }

    #[test]
    fn compile_deeply_nested_sequence_no_stack_overflow() {
        // Build Sequence([Sequence([... Atom("x") ...])] with depth 60.
        fn nest(depth: usize) -> PowlAstNode<'static> {
            if depth == 0 {
                PowlAstNode::Atom("x")
            } else {
                PowlAstNode::Sequence(vec![nest(depth - 1)])
            }
        }
        let ast = nest(60);
        let tape = compile_powl(&ast).expect("depth-60 nesting must not overflow");
        assert_eq!(tape.len, 1); // one Atom compiled
    }

    #[test]
    fn single_atom_op0_is_reachable_as_entry() {
        let ast = PowlAstNode::Atom("solo");
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.len, 1);
        assert_eq!(tape.entry_mask & 1, 1, "op 0 must be in entry_mask");
    }

    // ---------------------------------------------------------------------------
    // Proptests
    // ---------------------------------------------------------------------------

    #[test]
    fn compile_loop_with_max_iters() {
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("a")),
            redo: Box::new(PowlAstNode::Atom("b")),
            max_iters: 5,
        };
        let tape = compile_powl(&ast).unwrap();
        // body=0, redo=1, LoopRedo=2
        assert_eq!(tape.len, 3);
        // max_iters stored in branch_count of LoopRedo slot
        assert_eq!(tape.ops[2].branch_count, 5);
    }

    #[test]
    fn compile_unreachable_error() {
        // Manually craft a tape where a non-LoopRedo slot has no path from entry.
        // Use PartialOrder with an edge that makes slot 1 an entry but slot 0 is
        // wired with no predecessors yet never added to entry_mask.
        // Actually the compiler always sets entry_mask correctly, so Unreachable
        // can't happen from normal compilation. Test via check_all_ops_reachable directly.
        use super::{check_all_ops_reachable, CompileError};
        use crate::tape::{OpKind, PowlTape};

        let mut tape = PowlTape::new();
        // Slot 0: entry
        tape.alloc(OpKind::Atom);
        // Slot 1: not reachable from entry_mask
        tape.alloc(OpKind::Atom);
        tape.entry_mask = 0b01; // only slot 0 is entry; slot 1 is unreachable

        let result = check_all_ops_reachable(&tape);
        assert_eq!(result, Err(CompileError::Unreachable));
    }

    #[test]
    fn check_all_ops_reachable_ok_for_connected_graph() {
        use super::check_all_ops_reachable;

        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        assert!(check_all_ops_reachable(&tape).is_ok());
    }

    struct SimpleRng(u64);
    impl SimpleRng {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn next_range(&mut self, min: usize, max: usize) -> usize {
            let diff = max - min + 1;
            min + (self.next_u64() as usize % diff)
        }
    }

    fn oracle_check_reachability(tape: &PowlTape) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        // Seed queue with entry nodes
        for i in 0..tape.len {
            let idx = i as usize;
            if (tape.entry_mask & (1 << idx)) != 0 {
                queue.push_back(idx);
                visited.insert(idx);
            }
        }
        
        // BFS traversal
        while let Some(u) = queue.pop_front() {
            let succs = tape.ops[u].succ_mask;
            for v in 0..tape.len {
                let v_idx = v as usize;
                if (succs & (1 << v_idx)) != 0 && !visited.contains(&v_idx) {
                    visited.insert(v_idx);
                    queue.push_back(v_idx);
                }
            }
        }
        
        // Verify all active non-LoopRedo nodes are visited
        for i in 0..tape.len {
            let idx = i as usize;
            if tape.ops[idx].kind != OpKind::LoopRedo && !visited.contains(&idx) {
                return false;
            }
        }
        true
    }

    fn mutant1_identity_reachability_omission(tape: &PowlTape) -> u64 {
        let mut r = [0u64; 64];
        let tape_len = tape.len as usize;
        let entry_mask = tape.entry_mask;

        // Mutant 1: r[i] = succs; (omits self-reachability | (1u64 << i))
        for i in 0..64 {
            let in_bounds = (i < tape_len) as u64;
            let bounds_mask = 0u64.wrapping_sub(in_bounds);
            let succs = tape.ops[i].succ_mask & bounds_mask;
            r[i] = succs; // Mutant 1 omission
        }

        for k in 0..64 {
            let r_k = r[k];
            for i in 0..64 {
                let can_reach_k = (r[i] >> k) & 1;
                let mask = 0u64.wrapping_sub(can_reach_k);
                r[i] |= r_k & mask;
            }
        }

        let mut reachable_from_entry = 0u64;
        for i in 0..64 {
            let is_entry = (entry_mask >> i) & 1;
            let mask = 0u64.wrapping_sub(is_entry);
            reachable_from_entry |= r[i] & mask;
        }

        let mut must_be_reachable = 0u64;
        for i in 0..64 {
            let in_bounds = (i < tape_len) as u64;
            let is_not_redo = (tape.ops[i].kind != OpKind::LoopRedo) as u64;
            let active = in_bounds & is_not_redo;
            let mask = 0u64.wrapping_sub(active);
            must_be_reachable |= (1u64 << i) & mask;
        }

        let violation = must_be_reachable & !reachable_from_entry;
        let is_valid = (violation == 0) as u64;
        0u64.wrapping_sub(is_valid)
    }

    fn mutant2_pivot_index_skew(tape: &PowlTape) -> u64 {
        let mut r = [0u64; 64];
        let tape_len = tape.len as usize;
        let entry_mask = tape.entry_mask;

        for i in 0..64 {
            let in_bounds = (i < tape_len) as u64;
            let bounds_mask = 0u64.wrapping_sub(in_bounds);
            let succs = tape.ops[i].succ_mask & bounds_mask;
            r[i] = succs | (1u64 << i);
        }

        for k in 0..64 {
            // Mutant 2: let r_k = r[(k + 1) & 63];
            let r_k = r[(k + 1) & 63];
            for i in 0..64 {
                let can_reach_k = (r[i] >> k) & 1;
                let mask = 0u64.wrapping_sub(can_reach_k);
                r[i] |= r_k & mask;
            }
        }

        let mut reachable_from_entry = 0u64;
        for i in 0..64 {
            let is_entry = (entry_mask >> i) & 1;
            let mask = 0u64.wrapping_sub(is_entry);
            reachable_from_entry |= r[i] & mask;
        }

        let mut must_be_reachable = 0u64;
        for i in 0..64 {
            let in_bounds = (i < tape_len) as u64;
            let is_not_redo = (tape.ops[i].kind != OpKind::LoopRedo) as u64;
            let active = in_bounds & is_not_redo;
            let mask = 0u64.wrapping_sub(active);
            must_be_reachable |= (1u64 << i) & mask;
        }

        let violation = must_be_reachable & !reachable_from_entry;
        let is_valid = (violation == 0) as u64;
        0u64.wrapping_sub(is_valid)
    }

    fn mutant3_loop_redo_admittance_corruption(tape: &PowlTape) -> u64 {
        let mut r = [0u64; 64];
        let tape_len = tape.len as usize;
        let entry_mask = tape.entry_mask;

        for i in 0..64 {
            let in_bounds = (i < tape_len) as u64;
            let bounds_mask = 0u64.wrapping_sub(in_bounds);
            let succs = tape.ops[i].succ_mask & bounds_mask;
            r[i] = succs | (1u64 << i);
        }

        for k in 0..64 {
            let r_k = r[k];
            for i in 0..64 {
                let can_reach_k = (r[i] >> k) & 1;
                let mask = 0u64.wrapping_sub(can_reach_k);
                r[i] |= r_k & mask;
            }
        }

        let mut reachable_from_entry = 0u64;
        for i in 0..64 {
            let is_entry = (entry_mask >> i) & 1;
            let mask = 0u64.wrapping_sub(is_entry);
            reachable_from_entry |= r[i] & mask;
        }

        // Mutant 3: let active = in_bounds; (omits & is_not_redo check)
        let mut must_be_reachable = 0u64;
        for i in 0..64 {
            let in_bounds = (i < tape_len) as u64;
            let active = in_bounds; // Mutant 3 omission
            let mask = 0u64.wrapping_sub(active);
            must_be_reachable |= (1u64 << i) & mask;
        }

        let violation = must_be_reachable & !reachable_from_entry;
        let is_valid = (violation == 0) as u64;
        0u64.wrapping_sub(is_valid)
    }

    #[test]
    fn test_bp_tcrv_differential_and_mutants() {
        use super::bp_tcrv_validate_reachability;
        use crate::tape::OpKind;
        let mut rng = SimpleRng(0xACE1);
        let mut killed_mutant_1 = false;
        let mut killed_mutant_2 = false;
        let mut killed_mutant_3 = false;

        for _ in 0..10000 {
            let mut tape = PowlTape::new();
            let len = rng.next_range(1, 64);
            tape.len = len as u8;
            
            for i in 0..len {
                let kind_val = rng.next_range(0, 4);
                tape.ops[i].kind = match kind_val {
                    0 => OpKind::Atom,
                    1 => OpKind::Silent,
                    2 => OpKind::XorDispatch,
                    3 => OpKind::Join,
                    _ => OpKind::LoopRedo,
                };
                if rng.next_range(0, 10) == 0 {
                    tape.ops[i].kind = OpKind::LoopRedo;
                }
            }
            
            for i in 0..len {
                let mut succs = 0u64;
                for j in (i + 1)..len {
                    if rng.next_range(0, 3) == 0 {
                        succs |= 1u64 << j;
                    }
                }
                tape.ops[i].succ_mask = succs;
            }
            
            for i in 0..len {
                if tape.ops[i].kind == OpKind::LoopRedo {
                    let mut succs = 0u64;
                    for j in 0..i {
                        if rng.next_range(0, 3) == 0 {
                            succs |= 1u64 << j;
                        }
                    }
                    tape.ops[i].succ_mask = succs;
                }
            }

            for i in 0..len {
                let mut preds = 0u64;
                for j in 0..len {
                    if (tape.ops[j].succ_mask & (1u64 << i)) != 0 {
                        preds |= 1u64 << j;
                    }
                }
                tape.ops[i].pred_mask = preds;
            }

            let mut entry_mask = 0u64;
            for i in 0..len {
                if rng.next_range(0, 4) == 0 {
                    entry_mask |= 1u64 << i;
                }
            }
            if entry_mask == 0 && len > 0 {
                entry_mask = 1u64 << rng.next_range(0, len - 1);
            }
            tape.entry_mask = entry_mask;

            let expected = oracle_check_reachability(&tape);
            let actual = bp_tcrv_validate_reachability(&tape);
            
            assert_eq!(
                actual == !0u64,
                expected,
                "Reachability mismatch on tape with len={}, entry_mask={:#b}",
                len,
                entry_mask
            );

            let m1 = mutant1_identity_reachability_omission(&tape);
            if (m1 == !0u64) != expected {
                killed_mutant_1 = true;
            }

            let m2 = mutant2_pivot_index_skew(&tape);
            if (m2 == !0u64) != expected {
                killed_mutant_2 = true;
            }

            let m3 = mutant3_loop_redo_admittance_corruption(&tape);
            if (m3 == !0u64) != expected {
                killed_mutant_3 = true;
            }
        }

        assert!(killed_mutant_1, "Mutant 1 (Identity Reachability Omission) survived!");
        assert!(killed_mutant_2, "Mutant 2 (Pivot index Skew) survived!");
        assert!(killed_mutant_3, "Mutant 3 (LoopRedo Admittance Corruption) survived!");
    }

    #[test]
    fn compile_xor_inside_loop_rejected() {
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::XorChoice(vec![
                PowlAstNode::Atom("a"),
                PowlAstNode::Atom("b"),
            ])),
            redo: Box::new(PowlAstNode::Atom("r")),
            max_iters: 3,
        };
        assert!(matches!(
            compile_powl(&ast),
            Err(CompileError::XorInsideLoop { .. })
        ));
    }

    #[test]
    fn compile_loop_inside_xor_accepted() {
        // Loop INSIDE an XorChoice branch — should succeed (loop outside xor is the problem)
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Loop {
                body: Box::new(PowlAstNode::Atom("x")),
                redo: Box::new(PowlAstNode::Atom("y")),
                max_iters: 2,
            },
            PowlAstNode::Atom("z"),
        ]);
        assert!(compile_powl(&ast).is_ok());
    }

    #[test]
    fn kahn_check_rejects_non_loop_cycle() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![(0, 1), (1, 0)],
        };
        assert_eq!(
            compile_powl(&ast),
            Err(CompileError::Cycle),
            "mutual PartialOrder edges must be rejected as a cycle"
        );
    }

    use crate::scheduler::{scheduler_tick, PowlRunState};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_linear_chain_fires_in_topo_order(n in 2usize..8) {
            // Build a Sequence of n Atoms, compile it, then run to completion.
            // Each tick must fire a slot with index >= the last slot fired.
            let atoms: Vec<PowlAstNode<'_>> = (0..n).map(|_| PowlAstNode::Atom("x")).collect();
            let ast = PowlAstNode::Sequence(atoms);
            let tape = compile_powl(&ast).unwrap();

            let mut state = PowlRunState::new(&tape);
            let mut last_fired: u32 = 0;
            let mut first = true;

            for _ in 0..(n + 1) {
                if state.check_mask == 0 { break; }
                let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
                if fs.0 != 0 {
                    let slot = fs.0.trailing_zeros();
                    if first {
                        first = false;
                    } else {
                        prop_assert!(slot >= last_fired,
                            "slot {slot} fired before previous slot {last_fired} (topo order violated)");
                    }
                    last_fired = slot;
                }
            }
        }
    }
}
