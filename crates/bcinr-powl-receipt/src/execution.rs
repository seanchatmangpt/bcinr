//! `ExecutionReceipt` — a receipt attesting to one real
//! `bcinr_powl::scheduler::scheduler_tick_guarded` decision: which of a
//! tick's ready ops the `ConcurrencySelector` actually admitted, and which
//! ops were complete afterward.
//!
//! # Which tape this attests to (read before wiring a caller)
//!
//! `bcinr-powl`'s scheduler (`scheduler_tick`/`scheduler_tick_guarded`/
//! `PowlRunState`, all in `bcinr_powl::scheduler`) operates exclusively on
//! the **legacy** `bcinr_powl::tape::{Powl64Op, PowlTape}` pair (the
//! `OpKind::{Atom,Silent,XorDispatch,Join,LoopRedo}` shape, built by
//! `bcinr_powl::compiler::compile_powl` from a `PowlAstNode`). It does
//! **not** operate on the newer `bcinr_powl::tape::v2::{Powl64Op, PowlTape}`
//! pair that `bcinr_powl::compiler::v2::compile_powl_v2` produces from a
//! `PowlModel` (the type `crate::projection::ProjectionReceipt` is about) —
//! the two `Powl64Op` shapes are structurally different (the legacy op has
//! `branch_mask`/`kind`/`index`/`branch_count`; the v2 op has
//! `op_kind`/`choice_group`/`depth`/`fan_out`/`ctrl`), and as of the
//! `bcinr-powl` phase this crate depends on, **no bridge from v2 to legacy
//! exists anywhere in `bcinr-powl`** (confirmed by grep — see this crate's
//! phase report). So `compiled_digest` here is computed over whichever
//! legacy `PowlTape` was actually handed to `scheduler_tick_guarded` — real,
//! not a placeholder — but it is honestly *not yet* wired to a
//! `ProjectionReceipt`'s `powl_model_digest`/`CompiledPowlV2` in this phase.
//! `powl_model_digest` is carried as its own field precisely so a future
//! phase that closes this gap can populate it from a real `PowlModel`
//! without changing this receipt's shape.

use bcinr_mfw_ir::{Digest, EventSet};
use bcinr_powl::scheduler::{scheduler_tick, scheduler_tick_guarded, ConcurrencySelector, PowlRunState};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use bcinr_powl::tape::Powl64Op as LegacyPowl64Op;
use bcinr_powl::tape::PowlTape as LegacyPowlTape;

use crate::chain::fold;

// ---------------------------------------------------------------------------
// Canonical digest helpers
// ---------------------------------------------------------------------------

/// Canonical digest over a legacy `PowlTape` — the exact tape shape
/// `scheduler_tick`/`scheduler_tick_guarded` execute (see module docs).
pub fn digest_legacy_tape(tape: &LegacyPowlTape) -> Digest {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(tape.len as u32).to_le_bytes());
    buf.extend_from_slice(&tape.entry_mask.to_le_bytes());
    for op in &tape.ops[..tape.len as usize] {
        buf.extend_from_slice(&op.pred_mask.to_le_bytes());
        buf.extend_from_slice(&op.succ_mask.to_le_bytes());
        buf.extend_from_slice(&op.branch_mask.to_le_bytes());
        buf.push(op.kind as u8);
        buf.push(op.index);
        buf.push(op.branch_count);
    }
    Digest::hash(&buf)
}

fn push_event_set(buf: &mut Vec<u8>, es: &EventSet) {
    let members: Vec<usize> = es.iter_stable().collect();
    buf.extend_from_slice(&(members.len() as u32).to_le_bytes());
    for m in members {
        buf.extend_from_slice(&(m as u32).to_le_bytes());
    }
}

/// Convert a `u64` tape-slot bitmask into an `EventSet`. `bcinr_powl`'s own
/// equivalent (`scheduler::mask_to_event_set`) is private to that crate, so
/// this is a from-scratch, structurally-equivalent reimplementation over
/// `EventSet`'s public `insert` — not a call into private bcinr-powl code.
fn mask_to_event_set(mask: u64) -> EventSet {
    let mut set = EventSet::empty();
    let mut bits = mask;
    while bits != 0 {
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        set.insert(i);
    }
    set
}

// ---------------------------------------------------------------------------
// ExecutionReceipt
// ---------------------------------------------------------------------------

/// A receipt attesting to one scheduler tick's real firing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionReceipt {
    pub powl_model_digest: Digest,
    pub compiled_digest: Digest,
    pub tick: u32,
    pub scheduler_decision_digest: Digest,
    pub fired: EventSet,
    pub completed_after: EventSet,
    pub prior_hash: Digest,
    pub hash: Digest,
}

/// Seal an [`ExecutionReceipt`] from already-computed evidence. Pure
/// function — does not itself run the scheduler (see
/// [`tick_and_seal_execution_receipt`] for the real-scheduler wiring that
/// produces `fired`/`completed_after`/`scheduler_decision_digest`).
pub fn seal_execution_receipt(
    prior_hash: Digest,
    powl_model_digest: Digest,
    compiled_digest: Digest,
    tick: u32,
    scheduler_decision_digest: Digest,
    fired: EventSet,
    completed_after: EventSet,
) -> ExecutionReceipt {
    let mut buf = Vec::with_capacity(32 + 32 + 4 + 32 + 8 + 8);
    buf.extend_from_slice(powl_model_digest.as_bytes());
    buf.extend_from_slice(compiled_digest.as_bytes());
    buf.extend_from_slice(&tick.to_le_bytes());
    buf.extend_from_slice(scheduler_decision_digest.as_bytes());
    push_event_set(&mut buf, &fired);
    push_event_set(&mut buf, &completed_after);

    let hash = fold(&prior_hash, &buf);

    ExecutionReceipt {
        powl_model_digest,
        compiled_digest,
        tick,
        scheduler_decision_digest,
        fired,
        completed_after,
        prior_hash,
        hash,
    }
}

/// Run one **real** `scheduler_tick_guarded` tick against `state`, and seal
/// an [`ExecutionReceipt`] from what actually happened.
///
/// This genuinely calls `bcinr-powl`'s Phase-2b scheduler machinery:
/// - a dry-run preview (`scheduler_tick` on a clone) establishes the tick's
///   ready set,
/// - the real `scheduler_tick_guarded(tape, state, selector, guards)` call
///   mutates `state` and returns the real `FiredSet`,
/// - `fired`/`completed_after` are read back from that call's actual
///   output/`state.done_mask` — never placeholders,
/// - `scheduler_decision_digest` is a digest over `(tick, ready_mask,
///   fired_mask)`, i.e. exactly what the selector was offered versus what it
///   admitted this tick.
#[allow(clippy::too_many_arguments)]
pub fn tick_and_seal_execution_receipt<S: ConcurrencySelector>(
    tape: &[LegacyPowl64Op],
    state: &mut PowlRunState,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
    tick: u32,
    prior_hash: Digest,
    powl_model_digest: Digest,
    compiled_digest: Digest,
) -> ExecutionReceipt {
    let mut preview = state.clone();
    let would_fire = scheduler_tick(tape, &mut preview);
    let ready_mask = would_fire.0;

    let fired_set = scheduler_tick_guarded(tape, state, selector, guards);
    let fired_mask = fired_set.0;

    let fired = mask_to_event_set(fired_mask);
    let completed_after = mask_to_event_set(state.done_mask);

    let mut decision_buf = Vec::with_capacity(20);
    decision_buf.extend_from_slice(&tick.to_le_bytes());
    decision_buf.extend_from_slice(&ready_mask.to_le_bytes());
    decision_buf.extend_from_slice(&fired_mask.to_le_bytes());
    let scheduler_decision_digest = Digest::hash(&decision_buf);

    seal_execution_receipt(
        prior_hash,
        powl_model_digest,
        compiled_digest,
        tick,
        scheduler_decision_digest,
        fired,
        completed_after,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bcinr_powl::compiler::{compile_powl, PowlAstNode};
    use bcinr_powl::scheduler::StableMaximalSelector;
    use bcinr_powl::tape::v2::CompiledNonFace;

    #[test]
    fn seal_execution_receipt_is_deterministic_and_chains() {
        let fired = EventSet::empty().with(0);
        let completed_after = EventSet::empty().with(0);
        let r1 = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            fired,
            completed_after,
        );
        let r2 = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            fired,
            completed_after,
        );
        assert_eq!(r1.hash, r2.hash);

        let r3 = seal_execution_receipt(
            r1.hash,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            2,
            Digest::hash(b"decision-2"),
            fired,
            completed_after,
        );
        assert_ne!(r3.hash, r1.hash);
        assert_eq!(r3.prior_hash, r1.hash);
    }

    #[test]
    fn different_fired_sets_produce_different_hashes() {
        let base = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            EventSet::empty().with(0),
            EventSet::empty().with(0),
        );
        let different_fired = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            EventSet::empty().with(1),
            EventSet::empty().with(0),
        );
        assert_ne!(base.hash, different_fired.hash);
    }

    /// Real end-to-end wiring: two parallel activities, a concurrency guard
    /// forbidding them from firing together (mirroring bcinr-powl's own
    /// `nonempty_guard_defers_a_forbidden_pair_but_both_eventually_fire`
    /// fixture exactly, for consistency), run through the real
    /// `scheduler_tick_guarded` + `StableMaximalSelector`. Asserts the
    /// sealed receipt's `fired` genuinely reflects the guard deferring one
    /// of the two ops on tick 1 -- not a placeholder EventSet.
    #[test]
    fn tick_and_seal_reflects_a_real_guarded_deferral() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        };
        let tape = compile_powl(&ast).unwrap();
        let compiled_digest = digest_legacy_tape(&tape);

        let guards = ConcurrencyGuardTable {
            nonfaces: vec![CompiledNonFace {
                members: EventSet::empty().with(0).with(1),
                witness_digest: Digest::hash(b"a-b-conflict"),
            }],
        };
        let mut state = PowlRunState::new(&tape);
        let mut selector = StableMaximalSelector;

        let receipt1 = tick_and_seal_execution_receipt(
            &tape.ops[..tape.len as usize],
            &mut state,
            &mut selector,
            &guards,
            1,
            Digest::ZERO,
            Digest::hash(b"powl-model"),
            compiled_digest,
        );

        assert_eq!(
            receipt1.fired.len(),
            1,
            "guard must defer exactly one of the two conflicting ops on tick 1"
        );
        assert!(
            receipt1.fired.contains(0) || receipt1.fired.contains(1),
            "the one op that fired must be a or b"
        );
        assert_eq!(receipt1.completed_after, receipt1.fired);
        assert_eq!(receipt1.tick, 1);
        assert_ne!(receipt1.hash, Digest::ZERO);

        // Tick 2: the deferred op must now be able to fire (the guard
        // defers, it never permanently starves -- proving this receipt
        // reflects real, evolving scheduler state across ticks). The
        // `PartialOrder{a, b}` AST compiles to 3 slots (a, b, and a join
        // that waits on both) -- once the second of {a, b} fires this tick,
        // the join's precondition is satisfied within the same
        // `scheduler_tick` call (candidates are processed by ascending bit
        // index within one tick, so a later-indexed join sees the
        // just-updated `done` mask -- real branchless combinational
        // propagation, not a placeholder), so `completed_after` reaches all
        // 3 slots by tick 2, not just 2.
        let receipt2 = tick_and_seal_execution_receipt(
            &tape.ops[..tape.len as usize],
            &mut state,
            &mut selector,
            &guards,
            2,
            receipt1.hash,
            Digest::hash(b"powl-model"),
            compiled_digest,
        );
        assert_eq!(
            receipt2.fired.len(),
            2,
            "the deferred op and the now-satisfied join must both fire on tick 2"
        );
        assert_eq!(
            receipt2.completed_after.len(),
            3,
            "a, b, and the join must all be complete after tick 2"
        );
        assert!(state.check_mask == 0, "run must terminate by tick 2");
        assert_ne!(receipt2.hash, receipt1.hash);
        assert_eq!(receipt2.prior_hash, receipt1.hash);
    }
}
