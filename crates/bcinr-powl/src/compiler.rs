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
    Loop {
        body: Box<PowlAstNode<'a>>,
        redo: Box<PowlAstNode<'a>>,
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
    InvalidEdge { from: usize, to: usize, len: usize },
    Cycle,
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
// Recursive compiler
// ---------------------------------------------------------------------------

fn compile_node<'a>(
    node: &'a PowlAstNode<'a>,
    tape: &mut PowlTape,
) -> Result<Segment, CompileError> {
    match node {
        PowlAstNode::Atom(_label) => {
            let idx = tape.alloc(OpKind::Atom).ok_or(CompileError::TapeFull)?;
            let bit = 1u64 << idx;
            Ok(Segment { entries: bit, exits: bit })
        }

        PowlAstNode::Silent => {
            let idx = tape.alloc(OpKind::Silent).ok_or(CompileError::TapeFull)?;
            let bit = 1u64 << idx;
            Ok(Segment { entries: bit, exits: bit })
        }

        PowlAstNode::Sequence(children) => {
            if children.is_empty() {
                return Err(CompileError::EmptySequence);
            }
            let mut seg = compile_node(&children[0], tape)?;
            for child in &children[1..] {
                let next = compile_node(child, tape)?;
                wire(tape, seg.exits, next.entries);
                seg = Segment { entries: seg.entries, exits: next.exits };
            }
            Ok(seg)
        }

        PowlAstNode::PartialOrder { children, edges } => {
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

            // Collect overall entries (children with no incoming edges from siblings)
            // and overall exits (children with no outgoing edges to siblings).
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

        PowlAstNode::XorChoice(branches) => {
            if branches.is_empty() {
                return Err(CompileError::EmptyChoice);
            }

            // Dispatcher slot.
            let dispatch_idx = tape.alloc(OpKind::XorDispatch).ok_or(CompileError::TapeFull)?;
            let dispatch_bit = 1u64 << dispatch_idx;

            // Join slot.
            let join_idx = tape.alloc(OpKind::Join).ok_or(CompileError::TapeFull)?;
            let join_bit = 1u64 << join_idx;

            // Compile each branch.
            let mut branch_entries = 0u64;
            for branch in branches {
                let seg = compile_node(branch, tape)?;
                // Each branch entry requires the dispatcher to have fired.
                wire(tape, dispatch_bit, seg.entries);
                // Each branch exit feeds the join.
                wire(tape, seg.exits, join_bit);
                branch_entries |= seg.entries;
            }

            tape.ops[dispatch_idx as usize].branch_mask = branch_entries;
            tape.ops[dispatch_idx as usize].branch_count = branches.len() as u8;

            // The join only needs *one* branch to complete (XOR semantics).
            // We encode this by leaving pred_mask as the union but the scheduler
            // will use choice_taken to suppress unchosen branches.
            // For pred_mask on the join: it waits for whichever branch was taken.
            // We set the join's pred_mask to the union; the scheduler masks it
            // with (done_mask | !choice_taken_branches) to handle XOR correctly.
            // Simplest correct encoding: join pred_mask = branch_entries (all),
            // scheduler treats done unchosen branches as "virtually done" via
            // choice_taken suppression logic in FiredSet post-processing.
            // See scheduler.rs for the exact protocol.
            tape.ops[join_idx as usize].pred_mask = branch_entries;

            Ok(Segment { entries: dispatch_bit, exits: join_bit })
        }

        PowlAstNode::Loop { body, redo } => {
            // Structure:
            //   [body] → exit decision (silent)
            //     ↓ (exit path, normal)        ↓ (redo path via LoopRedo back-edge)
            //   [after loop]              [redo] → back to body entry
            //
            // We model it as:
            //   body_entry ... body_exit → redo_entry ... redo_exit
            //   redo_exit --(back-edge)-→ body_entry (LoopRedo slot)
            //
            // The body exits are also the loop exits (caller decides to stop looping
            // by not firing the redo path; the scheduler uses choice_taken for this).
            // For simplicity we emit a silent "loop-exit" dispatcher after body,
            // and wire redo exit back to body entry via a LoopRedo slot.

            let body_seg = compile_node(body, tape)?;
            let redo_seg = compile_node(redo, tape)?;

            // Wire body exits → redo entries (the redo path).
            wire(tape, body_seg.exits, redo_seg.entries);

            // Back-edge: redo exits → body entries.
            // We use a LoopRedo slot to mark the back-edge explicitly.
            let back_idx = tape.alloc(OpKind::LoopRedo).ok_or(CompileError::TapeFull)?;
            let back_bit = 1u64 << back_idx;
            wire(tape, redo_seg.exits, back_bit);

            // The back-edge slot's succ_mask points to body entries,
            // but with a back-edge flag (scheduler skips Kahn's check for these).
            tape.ops[back_idx as usize].succ_mask = body_seg.entries;
            // Do NOT set pred_mask on body entries for the back-edge
            // (would create a cycle in Kahn's). The scheduler handles
            // LoopRedo back-edges specially.

            // Loop exits = body exits (the scheduler decides via choice_taken).
            Ok(Segment {
                entries: body_seg.entries,
                exits: body_seg.exits,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Kahn's cycle detection (ignoring LoopRedo back-edges)
// ---------------------------------------------------------------------------

fn kahn_check(tape: &PowlTape) -> Result<(), CompileError> {
    let n = tape.len as usize;
    // Compute in-degree per slot from pred_mask, excluding LoopRedo back-edges.
    // LoopRedo slots are excluded from the DAG check entirely.
    let mut in_deg2 = [0u32; 64];
    for i in 0..n {
        if tape.ops[i].kind == OpKind::LoopRedo {
            continue;
        }
        let mut preds = tape.ops[i].pred_mask;
        while preds != 0 {
            let j = preds.trailing_zeros() as usize;
            if j < n && tape.ops[j].kind != OpKind::LoopRedo {
                in_deg2[i] += 1;
            }
            preds &= preds - 1;
        }
    }

    let mut queue: Vec<usize> = (0..n)
        .filter(|&i| in_deg2[i] == 0 && tape.ops[i].kind != OpKind::LoopRedo)
        .collect();
    let mut visited = 0usize;

    while let Some(u) = queue.pop() {
        visited += 1;
        let mut succs = tape.ops[u].succ_mask;
        while succs != 0 {
            let v = succs.trailing_zeros() as usize;
            if v < n && tape.ops[v].kind != OpKind::LoopRedo {
                in_deg2[v] = in_deg2[v].saturating_sub(1);
                if in_deg2[v] == 0 {
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
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        // slot 0 = a, slot 1 = b
        assert_eq!(tape.len, 2);
        assert_eq!(tape.ops[0].succ_mask, 0b10);
        assert_eq!(tape.ops[1].pred_mask, 0b01);
    }

    #[test]
    fn compile_xor_choice() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("left"),
            PowlAstNode::Atom("right"),
        ]);
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
        assert!(matches!(compile_powl(&ast), Err(CompileError::InvalidEdge { .. })));
    }
}
