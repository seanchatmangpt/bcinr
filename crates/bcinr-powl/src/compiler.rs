//! compiler — Compile a POWL AST into a `PowlTape`.
//!
//! # Algorithm
//!
//! The compiler does a single recursive descent over the AST, allocating tape
//! slots as it goes.  Each recursive call returns a `Segment` that describes
//! the entry slot(s) and exit slot(s) of the compiled sub-tree.  The caller
//! wires `pred_mask`/`succ_mask` across the boundary.
//!
//! After compilation a Kahn's-algorithm post-pass verifies there are no
//! non-loop cycles.

use crate::tape::{OpKind, PowlTape};

// ---------------------------------------------------------------------------
// Public AST
// ---------------------------------------------------------------------------

/// Minimal local POWL AST (no external wasm4pm-compat dependency).
pub enum PowlAstNode<'a> {
    /// A named activity.
    Atom(&'a str),
    /// Silent / tau transition.
    Silent,
    /// Sequential composition: execute children left-to-right.
    Sequence(Vec<PowlAstNode<'a>>),
    /// Partial-order: children with explicit dependency edges.
    /// `edges` are `(from_child_idx, to_child_idx)` pairs.
    PartialOrder {
        children: Vec<PowlAstNode<'a>>,
        edges: Vec<(usize, usize)>,
    },
    /// Exclusive choice: exactly one branch executes.
    XorChoice(Vec<PowlAstNode<'a>>),
    /// Loop: `body` executes, then either exits or `redo` executes and loops.
    /// `max_iters` caps the number of redo cycles (0 = unlimited).
    Loop {
        body: Box<PowlAstNode<'a>>,
        redo: Box<PowlAstNode<'a>>,
        max_iters: u8,
    },
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum CompileError {
    TapeFull,
    EmptySequence,
    EmptyChoice,
    EmptyPartialOrder,
    InvalidEdge {
        from: usize,
        to: usize,
        len: usize,
    },
    Cycle,
    /// A non-LoopRedo slot is unreachable from the entry mask.
    Unreachable,
    /// XorChoice nested inside Loop body or redo — unsafe; LoopRedo can
    /// re-enable unchosen XOR branches across iterations.
    XorInsideLoop {
        xor_slot: u8,
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

/// Phase 1 of two-phase Kahn: detect non-loop cycles.
pub fn check_full_graph_acyclic(tape: &PowlTape) -> Result<(), CompileError> {
    let n = tape.len as usize;
    let in_deg = build_in_degrees(tape, n);
    run_kahn_walk(tape, n, in_deg)
}

/// Phase 2 of two-phase Kahn: ensure every non-LoopRedo slot is reachable
/// from at least one entry (entry_mask bit set or transitively via succ_mask).
pub fn check_all_ops_reachable(tape: &PowlTape) -> Result<(), CompileError> {
    let n = tape.len as usize;
    let mut visited = 0u64;
    // BFS from each entry slot.
    let mut queue: Vec<usize> = Vec::new();
    let mut seeds = tape.entry_mask;
    while seeds != 0 {
        let i = seeds.trailing_zeros() as usize;
        seeds &= seeds - 1;
        if i < n && visited & (1u64 << i) == 0 {
            visited |= 1u64 << i;
            queue.push(i);
        }
    }
    while let Some(u) = queue.pop() {
        let mut succs = tape.ops[u].succ_mask;
        while succs != 0 {
            let v = succs.trailing_zeros() as usize;
            succs &= succs - 1;
            if v < n && visited & (1u64 << v) == 0 {
                visited |= 1u64 << v;
                queue.push(v);
            }
        }
    }
    // Every non-LoopRedo slot must be reachable.
    for i in 0..n {
        if tape.ops[i].kind != OpKind::LoopRedo && visited & (1u64 << i) == 0 {
            return Err(CompileError::Unreachable);
        }
    }
    Ok(())
}

fn kahn_check(tape: &PowlTape) -> Result<(), CompileError> {
    check_full_graph_acyclic(tape)?;
    check_all_ops_reachable(tape)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Compile a POWL AST into a `PowlTape`.
pub fn compile_powl(root: &PowlAstNode<'_>) -> Result<PowlTape, CompileError> {
    let mut tape = PowlTape::new();
    let seg = compile_node(root, &mut tape)?;
    tape.entry_mask = seg.entries;
    kahn_check(&tape)?;
    Ok(tape)
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
