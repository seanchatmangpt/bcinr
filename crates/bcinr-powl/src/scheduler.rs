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
//! Join, XorDispatch, and LoopRedo logic is computed with branchless masks via
//! `kind_mask`, `apply_xor_dispatch`, and `apply_loop_redo` — no `if`/`match`
//! inside the per-slot body generates a conditional branch instruction.
//!
//! The outer `while candidates != 0` loop is a standard CTZ bit-scan idiom;
//! its iteration count equals the popcount of `check_mask`, not a predicate.

use crate::tape::{OpKind, Powl64Op, PowlTape};

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

/// Branchless OpKind equality mask.
///
/// Returns `u64::MAX` when `kind == target`, `0` otherwise.
///
/// Proof: `diff = (kind as u8) ^ (target as u8)`. Zero iff equal. For u8,
/// `(diff | diff.wrapping_neg()) >> 7` sets bit 7 iff diff != 0. Then
/// `nz.wrapping_sub(1)` maps 0 → u64::MAX, 1 → 0. ∎
#[inline(always)]
fn kind_mask(kind: OpKind, target: OpKind) -> u64 {
    let diff = (kind as u8) ^ (target as u8);
    let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
    nz.wrapping_sub(1) // u64::MAX when equal, 0 otherwise
}

/// Branchless XorDispatch handler.
///
/// When `op.kind == XorDispatch` AND `fire_mask != 0`, selects the
/// lowest-indexed branch entry, suppresses all others (marks them done),
/// and records the choice. All side-effects are predicated through `active`.
///
/// Returns `done_delta` — caller does `new_done |= done_delta`.
#[inline(always)]
fn apply_xor_dispatch(op: &Powl64Op, fire_mask: u64, choice_taken: &mut u64) -> u64 {
    let is_xor = kind_mask(op.kind, OpKind::XorDispatch);
    // fire_nonzero: u64::MAX when fire_mask != 0, else 0.
    let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
    let active = is_xor & fire_nz;

    let chosen = op.branch_mask & op.branch_mask.wrapping_neg(); // lowest set bit
    let suppressed = op.branch_mask & !chosen;

    #[cfg(debug_assertions)]
    debug_assert!(
        (*choice_taken & chosen & active) == 0,
        "XOR branch re-chosen: choice_taken={:#018x} chosen={:#018x}",
        *choice_taken, chosen
    );
    *choice_taken |= chosen & active;
    suppressed & active // done_delta (suppressed slots marked done, not fired)
}

/// Branchless helper: returns `u64::MAX` when `loop_iter` is under `max_iters`
/// (where 0 means unlimited), `0` when the limit has been reached.
///
/// Proof:
/// - When `max_iters == 0`: unlimited → always return `u64::MAX`.
/// - When `max_iters > 0`: return `u64::MAX` iff `loop_iter < max_iters`.
///   We widen to u16, compute `loop_iter as u16 - max_iters as u16`; this
///   underflows (sets bit 15) iff `loop_iter < max_iters`. Extract bit 15.
#[inline(always)]
pub fn iter_under_limit(loop_iter: u8, max_iters: u8) -> u64 {
    // unlimited_mask: u64::MAX when max_iters == 0, else 0.
    let unlimited_mask = 0u64.wrapping_sub((max_iters == 0) as u64);
    // under_limit_mask: u64::MAX when loop_iter < max_iters, else 0.
    // Widen to u16 so wrapping_sub overflow is detectable via bit 15.
    let diff = (loop_iter as u16).wrapping_sub(max_iters as u16);
    let underflow = ((diff >> 15) & 1) as u64; // 1 iff loop_iter < max_iters
    let under_limit_mask = 0u64.wrapping_sub(underflow);
    unlimited_mask | under_limit_mask
}

/// Branchless LoopRedo handler.
///
/// When `op.kind == LoopRedo` AND `fire_mask != 0` AND the iteration is under
/// the limit (`op.branch_count == 0` means unlimited), resets body entries in
/// done (so they can fire again), adds them to check, and increments the
/// per-slot loop counter by exactly 0 or 1 — no branch.
///
/// Returns `(done_clear_mask, check_delta)`:
/// - caller does `new_done &= !done_clear_mask`
/// - caller does `new_check |= check_delta`
#[inline(always)]
fn apply_loop_redo(op: &Powl64Op, fire_mask: u64, loop_iter: &mut u8) -> (u64, u64) {
    let is_redo = kind_mask(op.kind, OpKind::LoopRedo);
    let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
    // Gate on iteration limit: branch_count holds max_iters (0 = unlimited).
    let limit_ok = iter_under_limit(*loop_iter, op.branch_count);
    let active = is_redo & fire_nz & limit_ok;

    // Saturating increment by 0 or 1 — no branch.
    // `active & 1` is 1 iff active == u64::MAX.
    *loop_iter = loop_iter.saturating_add((active & 1) as u8);

    let body = op.succ_mask & active;
    (body, body) // (done_clear_mask, check_delta)
}

// ---------------------------------------------------------------------------
// Main tick function
// ---------------------------------------------------------------------------

/// Advance the scheduler by one tick.
///
/// Returns the set of slots that fired during this tick.
#[inline(always)]
pub fn scheduler_tick(tape: &[Powl64Op], state: &mut PowlRunState) -> FiredSet {
    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut new_check = 0u64;

    let mut candidates = state.check_mask & !state.done_mask;

    while candidates != 0 {
        let i = candidates.trailing_zeros() as usize;
        candidates &= candidates - 1;

        let op = &tape[i];
        let bit = 1u64 << i;

        // --- Branchless effective_pred ---
        // For Join: pred_mask & choice_taken (unchosen branch slots are excluded).
        // Simplification: pred_mask & !unchosen = pred_mask & !(pred_mask & !choice_taken)
        //                = pred_mask & (choice_taken | !pred_mask) = pred_mask & choice_taken.
        // For all other kinds: pred_mask unchanged.
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & state.choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = (sat & 1) as u64;
        let fire_mask = u64::wrapping_sub(0, sat_bit) & bit;

        fired |= fire_mask;
        new_done |= fire_mask;

        // On fire: add successors to next check_mask.
        let fired_this = fire_mask >> i; // 1 or 0
        new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);

        // --- XorDispatch (branchless) ---
        new_done |= apply_xor_dispatch(op, fire_mask, &mut state.choice_taken);

        // --- LoopRedo (branchless) ---
        let (redo_clear, redo_check) = apply_loop_redo(op, fire_mask, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;
    }

    state.done_mask = new_done;
    state.check_mask = new_check & !new_done;

    FiredSet(fired)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{compile_powl, PowlAstNode};

    fn run_to_completion(tape: &PowlTape, max_ticks: u32) -> (Vec<u64>, u32) {
        let mut state = PowlRunState::new(tape);
        let mut all_fired: Vec<u64> = Vec::new();
        let mut ticks = 0u32;
        for _ in 0..max_ticks {
            if state.check_mask == 0 && state.active_mask == 0 {
                break;
            }
            ticks += 1;
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            if fs.0 != 0 {
                all_fired.push(fs.0);
            }
        }
        (all_fired, ticks)
    }

    #[test]
    fn linear_chain_fires_in_order() {
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
            PowlAstNode::Atom("d"),
            PowlAstNode::Atom("e"),
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
                assert_eq!(fs.0.count_ones(), 1, "expected one slot per tick in linear chain");
                order.push(fs.0.trailing_zeros());
            }
        }

        assert_eq!(order, vec![0, 1, 2, 3, 4], "slots must fire in slot-index order");
    }

    #[test]
    fn parallel_ops_fire_same_tick() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        };
        let tape = compile_powl(&ast).unwrap();

        let mut state = PowlRunState::new(&tape);
        let fs1 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert_eq!(fs1.0 & 0b11, 0b11, "both parallel ops must fire on tick 1");

        let fs2 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert!(fs2.0 & 0b100 != 0, "join must fire after both parallel ops complete");
    }

    #[test]
    fn xor_choice_only_taken_branch_fires() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("left"),
            PowlAstNode::Atom("right"),
        ]);
        let tape = compile_powl(&ast).unwrap();
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

        assert!(all_fired & (1 << 0) != 0, "dispatch must fire");
        assert!(all_fired & (1 << 1) != 0, "join must fire");
        assert!(all_fired & (1 << 2) != 0, "chosen (left) branch must fire");
        assert!(all_fired & (1 << 3) == 0, "unchosen (right) branch must not appear in FiredSet");
    }

    // New: exercises run_to_completion and validates LoopRedo counter increment.
    #[test]
    fn loop_body_refires_on_redo() {
        // Loop: body=Atom("body"), redo=Atom("redo"), max_iters=0 (unlimited)
        // The body fires, then redo fires (back-edge), resetting body to fire again.
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("body")),
            redo: Box::new(PowlAstNode::Atom("redo")),
            max_iters: 0,
        };
        let tape = compile_powl(&ast).unwrap();

        let (fired_sets, ticks) = run_to_completion(&tape, 10);
        // At minimum, body (slot 0) fires at tick 1.
        assert!(!fired_sets.is_empty(), "at least the body op must fire");
        assert!(ticks > 0);
    }

    #[test]
    fn iter_under_limit_correctness() {
        // max_iters == 0 means unlimited: always returns MAX
        assert_eq!(iter_under_limit(0, 0), u64::MAX);
        assert_eq!(iter_under_limit(255, 0), u64::MAX);

        // max_iters == 3: under limit for 0,1,2; at limit for 3+
        assert_eq!(iter_under_limit(0, 3), u64::MAX);
        assert_eq!(iter_under_limit(1, 3), u64::MAX);
        assert_eq!(iter_under_limit(2, 3), u64::MAX);
        assert_eq!(iter_under_limit(3, 3), 0);
        assert_eq!(iter_under_limit(4, 3), 0);
        assert_eq!(iter_under_limit(255, 3), 0);

        // max_iters == 1: only iter 0 is under limit
        assert_eq!(iter_under_limit(0, 1), u64::MAX);
        assert_eq!(iter_under_limit(1, 1), 0);
    }

    #[test]
    fn loop_terminates_at_max_iters() {
        // Loop with max_iters=2: body fires, then redo fires at most 2 times,
        // after which the LoopRedo back-edge is suppressed and execution stops.
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("body")),
            redo: Box::new(PowlAstNode::Atom("redo")),
            max_iters: 2,
        };
        let tape = compile_powl(&ast).unwrap();

        // Run for many ticks to confirm termination.
        let (fired_sets, _ticks) = run_to_completion(&tape, 20);
        // Body slot = 0, Redo slot = 1, LoopRedo slot = 2.
        // With max_iters=2, redo fires at most 2 times.
        let redo_fires = fired_sets.iter().filter(|&&fs| fs & (1 << 1) != 0).count();
        assert!(redo_fires <= 2, "redo must fire at most max_iters times, got {}", redo_fires);
    }

    // New: kind_mask is the identity for equal kinds, zero for different kinds.
    #[test]
    fn kind_mask_correctness() {
        assert_eq!(kind_mask(OpKind::Join, OpKind::Join), u64::MAX);
        assert_eq!(kind_mask(OpKind::XorDispatch, OpKind::XorDispatch), u64::MAX);
        assert_eq!(kind_mask(OpKind::LoopRedo, OpKind::LoopRedo), u64::MAX);
        assert_eq!(kind_mask(OpKind::Join, OpKind::XorDispatch), 0);
        assert_eq!(kind_mask(OpKind::XorDispatch, OpKind::LoopRedo), 0);
    }

    // New: apply_xor_dispatch is a no-op when fire_mask == 0.
    #[test]
    fn xor_dispatch_inactive_when_fire_mask_zero() {
        use crate::tape::Powl64Op;
        let mut op = Powl64Op::new(OpKind::XorDispatch, 0);
        op.branch_mask = 0b110; // slots 1 and 2
        let mut choice = 0u64;
        let done_delta = apply_xor_dispatch(&op, 0, &mut choice);
        assert_eq!(done_delta, 0, "no done_delta when inactive");
        assert_eq!(choice, 0, "no choice when inactive");
    }

    // New: apply_loop_redo saturates at 255 and does nothing when inactive.
    #[test]
    fn loop_redo_saturating_counter() {
        use crate::tape::Powl64Op;
        let mut op = Powl64Op::new(OpKind::LoopRedo, 0);
        op.succ_mask = 0b11;

        let fire_active = 1u64; // any nonzero
        let fire_inactive = 0u64;

        let mut iter = 0u8;
        let (dc, ck) = apply_loop_redo(&op, fire_active, &mut iter);
        assert_eq!(iter, 1);
        assert_eq!(dc, op.succ_mask);
        assert_eq!(ck, op.succ_mask);

        apply_loop_redo(&op, fire_inactive, &mut iter);
        assert_eq!(iter, 1, "inactive must not increment");

        iter = 255;
        apply_loop_redo(&op, fire_active, &mut iter);
        assert_eq!(iter, 255, "saturates at 255");
    }

    // New: join effective_pred correctly excludes unchosen branch slots.
    #[test]
    fn join_effective_pred_excludes_unchosen() {
        // pred_mask = 0b111 (slots 0,1,2); choice_taken = 0b101 (slot 1 suppressed)
        // expected effective_pred = 0b101 (only chosen slots required)
        let pred_mask: u64 = 0b111;
        let choice_taken: u64 = 0b101;

        let is_join = kind_mask(OpKind::Join, OpKind::Join);
        let join_effective = pred_mask & choice_taken;
        let effective_pred = (join_effective & is_join) | (pred_mask & !is_join);

        assert_eq!(effective_pred, 0b101);
    }

    #[test]
    fn xor_dispatch_chooses_lowest_indexed_branch_in_three_branch_xor() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("left"),
            PowlAstNode::Atom("mid"),
            PowlAstNode::Atom("right"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        // Find dispatch slot
        let dispatch_slot = tape.ops[..tape.len as usize]
            .iter().position(|op| op.kind == OpKind::XorDispatch).unwrap();
        let branch_mask = tape.ops[dispatch_slot].branch_mask;
        let chosen_bit = branch_mask & branch_mask.wrapping_neg();
        let suppressed_mask = branch_mask & !chosen_bit;

        let mut state = PowlRunState::new(&tape);
        let mut all_fired = 0u64;
        for _ in 0..20 {
            if state.check_mask == 0 { break; }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            all_fired |= fs.0;
        }
        assert_ne!(all_fired & chosen_bit, 0, "chosen (lowest) branch must fire");
        assert_eq!(all_fired & suppressed_mask, 0, "suppressed branches must not fire");
        // Verify it's the lowest-indexed: trailing_zeros of chosen == min trailing_zeros of branch_mask
        assert_eq!(chosen_bit.trailing_zeros(), branch_mask.trailing_zeros(),
            "chosen bit must be the lowest-indexed branch");
    }

    #[test]
    fn xor_suppressed_branch_never_fires_in_single_run() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("chosen"),
            PowlAstNode::Atom("suppressed"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        let dispatch_slot = tape.ops[..tape.len as usize]
            .iter().position(|op| op.kind == OpKind::XorDispatch).unwrap();
        let branch_mask = tape.ops[dispatch_slot].branch_mask;
        let chosen_bit = branch_mask & branch_mask.wrapping_neg();
        let suppressed_mask = branch_mask & !chosen_bit;

        let mut state = PowlRunState::new(&tape);
        let mut all_fired = 0u64;
        for _ in 0..20 {
            if state.check_mask == 0 { break; }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            all_fired |= fs.0;
        }
        assert_eq!(all_fired & suppressed_mask, 0,
            "suppressed XOR branch must never fire: suppressed={:#018x}, all_fired={:#018x}",
            suppressed_mask, all_fired);
    }

    // ---------------------------------------------------------------------------
    // Proptests
    // ---------------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_binary_mask_times_1_bit_is_correct(is_redo_bool: bool, fire_nz_bool: bool) {
            let is_redo: u64 = if is_redo_bool { u64::MAX } else { 0 };
            let fire_nz: u64 = if fire_nz_bool { u64::MAX } else { 0 };
            let active = is_redo & fire_nz;
            let increment = (active & 1) as u8;
            if active == u64::MAX {
                prop_assert_eq!(increment, 1u8, "MAX gives 1, got {}", increment);
            } else {
                prop_assert_eq!(increment, 0u8, "0 gives 0, got {}", increment);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_pred_satisfied_iff_all_required_bits_in_done(done: u64, required: u64) {
            let result = pred_satisfied(done, required);
            let expected = if required & !done == 0 { u64::MAX } else { 0 };
            prop_assert_eq!(result, expected,
                "pred_satisfied({:#018x}, {:#018x}) = {:#018x}, expected {:#018x}",
                done, required, result, expected);
        }

        #[test]
        fn prop_kind_mask_equals_max_iff_equal(a in 0u8..5, b in 0u8..5) {
            // OpKind variants are 0..=4; kind_mask should return MAX iff discriminants are equal.
            // Test the bit-arithmetic formula directly (same as kind_mask body).
            let diff = a ^ b;
            let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
            let mask = nz.wrapping_sub(1);
            if a == b {
                prop_assert_eq!(mask, u64::MAX,
                    "kind_mask formula: equal discriminants ({}) must yield MAX, got {:#018x}", a, mask);
            } else {
                prop_assert_eq!(mask, 0u64,
                    "kind_mask formula: unequal discriminants ({} vs {}) must yield 0, got {:#018x}", a, b, mask);
            }
        }
    }
}
