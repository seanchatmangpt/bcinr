//! scheduler — Branchless SWAR scheduling loop for POWL v2.
//!
//! # Protocol
//!
//! Each call to `scheduler_tick` advances the run-state by one tick:
//! 1. For each slot `i` in `check_mask`, compute `pred_sat` branchlessly.
//! 2. Derive a `fire_mask` word using `wrapping_sub(0, pred_sat & available & gate)`.
//! 3. Update `done_mask`, `active_mask`, `check_mask` from fired slots.
//! 4. XorDispatch slots pick one branch; LoopRedo slots re-enable the body.
//!
//! # Branchless invariant
//!
//! The pred-sat core (computing `pred_satisfied` and `fire_mask`) uses only
//! bitwise and arithmetic operations — no conditional branch instructions.
//! Op-kind dispatch (`XorDispatch`, `LoopRedo`, `Join`) uses conditional
//! branches on the cold side-effect path, not in the pred-sat hot path.
//! The outer `while bits != 0` loop is a standard bit-scan idiom whose
//! iteration count is bounded by the number of bits set, not by a predicate.

use crate::tape::{OpKind, PowlTape};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Mutable run-state for a POWL tape execution.
#[repr(C, align(8))]
pub struct PowlRunState {
    /// Bitmask of slots that have completed.
    pub done_mask: u64,
    /// Bitmask of slots that are currently firing (in-progress this tick).
    pub active_mask: u64,
    /// Bitmask of slots whose readiness should be checked next tick.
    pub check_mask: u64,
    /// For XorChoice: bitmask of slots in the *chosen* branch (others suppressed).
    pub choice_taken: u64,
    /// Per-slot loop iteration counter (saturates at 255).
    pub loop_iters: [u8; 64],
    /// Logical tick counter.
    pub tick: u32,
    _pad: [u8; 4],
}

impl PowlRunState {
    /// Construct initial state for a tape, seeding `check_mask` from its `entry_mask`.
    pub fn new(tape: &PowlTape) -> Self {
        Self {
            done_mask: 0,
            active_mask: 0,
            check_mask: tape.entry_mask,
            choice_taken: 0,
            loop_iters: [0u8; 64],
            tick: 0,
            _pad: [0u8; 4],
        }
    }
}

/// Bitmask of slots that fired during a single `scheduler_tick` call.
pub struct FiredSet(pub u64);

// ---------------------------------------------------------------------------
// Branchless helpers
// ---------------------------------------------------------------------------

/// Branchless: `!0u64` if all bits in `required` are set in `done`, else `0`.
///
/// Proof: `unmet = required & !done` is zero iff every required bit is done.
/// `(unmet == 0) as u64` ∈ {0, 1}; `wrapping_neg` maps 1 → !0, 0 → 0.
#[inline(always)]
fn pred_satisfied(done: u64, required: u64) -> u64 {
    let unmet = required & !done;
    0u64.wrapping_sub((unmet == 0) as u64)
}

// ---------------------------------------------------------------------------
// Main tick function
// ---------------------------------------------------------------------------

/// Advance the scheduler by one tick.
///
/// Returns the set of slots that fired during this tick.
#[inline(always)]
pub fn scheduler_tick(tape: &[crate::tape::Powl64Op], state: &mut PowlRunState) -> FiredSet {

    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut new_check = 0u64;

    // Iterate over slots that are candidates this tick.
    // active_mask is zero in synchronous execution; keep it for async future use
    // but skip the AND to avoid the load — uncomment if async dispatch is added.
    let mut candidates = state.check_mask & !state.done_mask;

    while candidates != 0 {
        let i = candidates.trailing_zeros() as usize;
        candidates &= candidates - 1;

        let op = &tape[i];
        let bit = 1u64 << i;

        // ------------------------------------------------------------------
        // Branchless pred-sat computation.
        // ------------------------------------------------------------------
        let effective_pred = match op.kind {
            OpKind::Join => {
                // For XOR-join: slots in pred_mask that were NOT chosen are
                // virtually done. Mask them out of the requirement.
                // choice_taken encodes which branch slots are live; unchosen
                // branch slots (in pred_mask but not in choice_taken) are skipped.
                let unchosen = op.pred_mask & !state.choice_taken;
                op.pred_mask & !unchosen
            }
            _ => op.pred_mask,
        };

        let sat = pred_satisfied(new_done, effective_pred);

        // fire_mask: !0 if satisfied, 0 otherwise — then mask to single bit.
        // wrapping_sub(0, x) where x ∈ {0,1} gives {0, !0}.
        // sat is already !0 or 0, so we AND with 1 to get a single-bit scalar.
        let sat_bit = sat & 1; // 1 if ready, 0 if not
        let fire_mask = u64::wrapping_sub(0, sat_bit) & bit;

        // Accumulate into fired only if fire_mask is nonzero.
        fired |= fire_mask;
        new_done |= fire_mask;

        // On fire: add successors to next check_mask.
        let fired_this = fire_mask >> i; // 1 or 0
        // Branchless: multiply succ_mask by fired_this.
        let succ_contrib = op.succ_mask & u64::wrapping_sub(0, fired_this);
        new_check |= succ_contrib;

        // Handle XorDispatch: pick the lowest-indexed branch (deterministic).
        // We encode the choice into choice_taken so the join knows which path
        // is live.  In a real system the caller provides the choice; here we
        // pick lowest bit (priority-based) for determinism.
        if op.kind == OpKind::XorDispatch && fire_mask != 0 {
            // Choose the lowest-indexed branch entry.
            let chosen_entry = op.branch_mask & op.branch_mask.wrapping_neg();
            // All branch entries except the chosen one are "virtually done"
            // (suppressed) — remove them from check and mark in done.
            let suppressed = op.branch_mask & !chosen_entry;
            new_done |= suppressed;
            fired |= 0; // suppressed slots do NOT appear in FiredSet
            state.choice_taken |= chosen_entry;
            // The join's effective pred will exclude suppressed slots.
        }

        // Handle LoopRedo back-edge: re-enable body entries.
        if op.kind == OpKind::LoopRedo && fire_mask != 0 {
            // Reset done for body entries so they can fire again.
            let body_entries = op.succ_mask;
            new_done &= !body_entries;
            new_check |= body_entries;
            // Increment loop iteration counter.
            let iter = &mut state.loop_iters[i];
            *iter = iter.saturating_add(1);
        }
    }

    state.done_mask = new_done;
    state.check_mask = new_check & !new_done;
    // tick is incremented by the caller if needed; omitting saves a store in hot path.

    FiredSet(fired)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{compile_powl, PowlAstNode};

    #[allow(dead_code)]
    fn run_to_completion(tape: &PowlTape, max_ticks: u32) -> (Vec<u64>, u32) {
        let mut state = PowlRunState::new(tape);
        let mut all_fired: Vec<u64> = Vec::new();
        for _ in 0..max_ticks {
            if state.check_mask == 0 && state.active_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            if fs.0 != 0 {
                all_fired.push(fs.0);
            }
        }
        let ticks = state.tick;
        (all_fired, ticks)
    }

    /// Linear chain of 5 ops: each fires after the previous, in strict order.
    #[test]
    fn linear_chain_fires_in_order() {
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"), // slot 0
            PowlAstNode::Atom("b"), // slot 1
            PowlAstNode::Atom("c"), // slot 2
            PowlAstNode::Atom("d"), // slot 3
            PowlAstNode::Atom("e"), // slot 4
        ]);
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.len, 5);

        let mut state = PowlRunState::new(&tape);
        let mut order: Vec<u32> = Vec::new();

        for _ in 0..10 {
            if state.check_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            if fs.0 != 0 {
                // Each tick should fire exactly one slot.
                assert_eq!(fs.0.count_ones(), 1, "expected one slot per tick in linear chain");
                order.push(fs.0.trailing_zeros());
            }
        }

        assert_eq!(order, vec![0, 1, 2, 3, 4], "slots must fire in slot-index order");
    }

    /// Two parallel ops (no deps between them) both fire on the same tick.
    #[test]
    fn parallel_ops_fire_same_tick() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        };
        let tape = compile_powl(&ast).unwrap();
        // slots: a=0, b=1, join=2

        let mut state = PowlRunState::new(&tape);

        // Tick 1: both a and b should fire simultaneously.
        let fs1 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        // Both slot 0 and slot 1 fire.
        assert_eq!(fs1.0 & 0b11, 0b11, "both parallel ops must fire on tick 1");

        // Tick 2: join fires.
        let fs2 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert!(fs2.0 & 0b100 != 0, "join must fire after both parallel ops complete");
    }

    /// XorChoice: only the chosen (lowest-indexed) branch fires, not the other.
    #[test]
    fn xor_choice_only_taken_branch_fires() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("left"),  // chosen (lower index)
            PowlAstNode::Atom("right"), // suppressed
        ]);
        let tape = compile_powl(&ast).unwrap();
        // dispatch=0, join=1, left=2, right=3
        assert_eq!(tape.len, 4);

        let mut state = PowlRunState::new(&tape);
        let mut all_fired = 0u64;

        for _ in 0..5 {
            if state.check_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            all_fired |= fs.0;
        }

        // Dispatch (slot 0) and join (slot 1) must have fired.
        assert!(all_fired & (1 << 0) != 0, "dispatch must fire");
        assert!(all_fired & (1 << 1) != 0, "join must fire");

        // Left branch (slot 2) must have fired.
        assert!(all_fired & (1 << 2) != 0, "chosen (left) branch must fire");

        // Right branch (slot 3) must NOT have fired via FiredSet
        // (it is suppressed, not a genuine fire).
        assert!(all_fired & (1 << 3) == 0, "unchosen (right) branch must not appear in FiredSet");
    }
}
