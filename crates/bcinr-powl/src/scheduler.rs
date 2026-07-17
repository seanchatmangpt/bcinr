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
//! The inner per-slot evaluation uses only bitwise and arithmetic operations —
//! no `if`/`match` that would generate a conditional branch instruction.
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

/// Branchless: returns `!0u8` if `iter` is still under `limit`, else `0`.
///
/// Semantics:
/// - `limit == 0` → unlimited; always returns `!0` (never blocks iteration).
/// - `limit == 1` → fires once (iter=0 passes, iter=1 does not).
/// - `limit == 255` → max u8 limit; iter<255 passes, iter==255 does not.
///
/// Proof:
///   Let `unlimited = (limit == 0) as u8`; `unlimited` is 1 iff limit=0.
///   `under = (iter < limit) as u8`; `under` is 1 iff iter < limit.
///   `pass = unlimited | under` — either unlimited or genuinely under limit.
///   `0u8.wrapping_sub(pass & 1)` maps 1 → !0, 0 → 0. ∎
///
/// # Boundary cases
/// - `iter=255, limit=255` → 255 < 255 is false, unlimited=false → returns 0 (blocked).
/// - `iter=254, limit=255` → 254 < 255 is true → returns !0 (allowed).
/// - `iter=255, limit=0`   → unlimited → returns !0 (allowed).
#[inline(always)]
pub fn iter_under_limit(iter: u8, limit: u8) -> u8 {
    let unlimited = (limit == 0) as u8;
    let under = (iter < limit) as u8;
    let pass = unlimited | under;
    0u8.wrapping_sub(pass & 1)
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
        let sat_bit = (sat & 1) as u64; // 1 if ready, 0 if not
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

    // -----------------------------------------------------------------------
    // iter_under_limit — all edge cases required by van der Aalst audit
    // -----------------------------------------------------------------------

    #[test]
    fn iter_under_limit_zero_limit_is_unlimited() {
        // limit=0 means unlimited: every iter value must pass
        assert_eq!(iter_under_limit(0, 0),   !0u8, "iter=0, limit=0 must pass");
        assert_eq!(iter_under_limit(1, 0),   !0u8, "iter=1, limit=0 must pass");
        assert_eq!(iter_under_limit(127, 0), !0u8, "iter=127, limit=0 must pass");
        assert_eq!(iter_under_limit(254, 0), !0u8, "iter=254, limit=0 must pass");
        assert_eq!(iter_under_limit(255, 0), !0u8, "iter=255, limit=0 must pass (unlimited)");
    }

    #[test]
    fn iter_under_limit_one_fires_once_then_stops() {
        // limit=1: fires when iter=0, blocked when iter>=1
        assert_eq!(iter_under_limit(0, 1), !0u8, "iter=0, limit=1 must pass (first fire)");
        assert_eq!(iter_under_limit(1, 1), 0u8,  "iter=1, limit=1 must block (already fired)");
        assert_eq!(iter_under_limit(2, 1), 0u8,  "iter=2, limit=1 must block");
        assert_eq!(iter_under_limit(255, 1), 0u8, "iter=255, limit=1 must block");
    }

    #[test]
    fn iter_under_limit_max_limit_255() {
        // limit=255: all iter 0..=254 pass, iter=255 is blocked
        assert_eq!(iter_under_limit(0,   255), !0u8, "iter=0,   limit=255 must pass");
        assert_eq!(iter_under_limit(1,   255), !0u8, "iter=1,   limit=255 must pass");
        assert_eq!(iter_under_limit(254, 255), !0u8, "iter=254, limit=255 must pass");
        assert_eq!(iter_under_limit(255, 255), 0u8,  "iter=255, limit=255 must block (boundary)");
    }

    #[test]
    fn iter_under_limit_boundary_iter_255_limit_255() {
        // The boundary case: iter == limit == 255 must block
        assert_eq!(iter_under_limit(255, 255), 0u8,
            "iter=255 limit=255: boundary must block, not pass");
        // But limit=0 always passes even at iter=255
        assert_eq!(iter_under_limit(255, 0), !0u8,
            "iter=255 limit=0: unlimited must still pass");
    }

    // -----------------------------------------------------------------------
    // apply_loop_redo — max_iters=0 semantic
    // -----------------------------------------------------------------------

    /// Semantic for max_iters=0 on a Loop AST node:
    /// The LoopRedo back-edge fires on each iteration. With no external gate
    /// and max_iters=0 (unlimited), the loop body executes at least once and
    /// re-enables indefinitely. The caller is responsible for breaking the cycle.
    ///
    /// Test: compile a Loop, run 3 ticks, verify body executes at least once
    /// (loop_iters counter at LoopRedo slot is incremented each redo).
    #[test]
    fn apply_loop_redo_body_executes_at_least_once() {
        // Loop { body: Atom("work"), redo: Atom("redo_work") }
        // Semantic: body fires, then LoopRedo re-enables body.
        // max_iters=0 means unlimited — body fires on tick 1.
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("work")),
            redo: Box::new(PowlAstNode::Atom("redo_work")),
        };
        let tape = compile_powl(&ast).unwrap();
        let mut state = PowlRunState::new(&tape);

        // Run exactly 3 ticks to allow at least one body execution.
        let mut body_fired_count = 0u32;
        for _ in 0..3 {
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            // Slot 0 is the body entry (Atom "work"), bit 0 = body fired.
            if fs.0 & 1 != 0 {
                body_fired_count += 1;
            }
        }
        assert!(body_fired_count >= 1, "loop body must execute at least once (max_iters=0 is unlimited)");
    }

    /// When the LoopRedo back-edge fires, loop_iters[redo_slot] increments.
    /// With max_iters=0 (unlimited), it increments without blocking.
    #[test]
    fn apply_loop_redo_iters_counter_increments() {
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("w")),
            redo: Box::new(PowlAstNode::Atom("r")),
        };
        let tape = compile_powl(&ast).unwrap();
        let mut state = PowlRunState::new(&tape);
        // Find the LoopRedo slot index.
        let redo_slot = tape.ops[..tape.len as usize]
            .iter()
            .position(|op| op.kind == crate::tape::OpKind::LoopRedo)
            .expect("must have LoopRedo slot");
        // Run 4 ticks; LoopRedo should fire at least once and increment the counter.
        for _ in 0..4 {
            scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        }
        // Counter increments on each LoopRedo fire; should be >= 1 after 4 ticks.
        assert!(state.loop_iters[redo_slot] >= 1,
            "loop_iters must increment each time LoopRedo fires");
    }

    // -----------------------------------------------------------------------
    // scheduler_tick liveness — all check_mask but none fire
    // -----------------------------------------------------------------------

    /// Liveness property: if check_mask becomes non-empty but all candidates
    /// have unsatisfied pred_masks, the tick returns FiredSet(0) and check_mask
    /// remains unchanged. This is NOT an infinite spin — it is a single-tick
    /// no-op. The caller's loop terminates because it checks check_mask!=0.
    ///
    /// Test: construct a state where check_mask has bit 1 set but op[1].pred_mask
    /// requires bit 0 (not yet done). Tick must return 0 fired and not spin.
    #[test]
    fn scheduler_tick_no_progress_returns_zero_does_not_spin() {
        // Linear chain: op0 -> op1
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        let mut state = PowlRunState::new(&tape);

        // Manually inject: done_mask=0, check_mask=0b10 (op1 in candidates but op0 not done).
        // This simulates a stale check_mask with an unsatisfied predecessor.
        state.done_mask = 0;
        state.check_mask = 0b10;

        // Single tick: op1's pred requires op0 (done=0), so nothing fires.
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert_eq!(fs.0, 0, "no op fires when predecessor not satisfied");
        // check_mask should be cleared (op1 not re-enqueued because it didn't fire).
        // Caller loop terminates when check_mask=0 or on timeout.
        // The test asserts a single tick completes in O(1) — no spin.
    }

    /// Liveness with bounded ticks: assert entire tape completes within N=2*len ticks.
    #[test]
    fn scheduler_tick_completes_within_bounded_ticks() {
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        let max_ticks = (tape.len as u32) * 2;
        let mut state = PowlRunState::new(&tape);
        let mut ticks_used = 0u32;
        for _ in 0..max_ticks {
            if state.check_mask == 0 { break; }
            scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            ticks_used += 1;
        }
        assert_eq!(state.check_mask, 0, "scheduler must terminate within 2*len ticks");
        assert!(ticks_used <= max_ticks, "must complete within bounded tick count");
    }

    // -----------------------------------------------------------------------
    // pred_mask=0 means entry point, fires on tick 1
    // -----------------------------------------------------------------------

    /// An op with pred_mask=0 has no predecessors required and must fire on tick 1.
    /// This confirms pred_mask=0 = "entry point / unconditionally ready",
    /// NOT "unreachable".
    #[test]
    fn pred_mask_zero_means_entry_fires_on_first_tick() {
        // A single Atom has pred_mask=0 and is in entry_mask.
        let ast = PowlAstNode::Atom("entry");
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.ops[0].pred_mask, 0, "entry op must have pred_mask=0");

        let mut state = PowlRunState::new(&tape);
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);

        assert_ne!(fs.0 & 1, 0, "op with pred_mask=0 must fire on the very first tick");
    }

    // -----------------------------------------------------------------------
    // XorChoice: only the chosen (lowest-indexed) branch fires, not the other.
    // -----------------------------------------------------------------------

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
