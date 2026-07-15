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
use bcinr_powl::scheduler::{
    scheduler_tick, scheduler_tick_guarded, ConcurrencySelector, PowlRunState,
};
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

/// Canonical content digest of a [`ConcurrencyGuardTable`]: hashes
/// `nonfaces` in the table's own (deterministic, insertion-ordered `Vec`)
/// order — each nonface's `members` `EventSet` and `witness_digest`.
///
/// This exists so an [`ExecutionReceipt`] can *commit to which guard table
/// its `fired` set was actually checked against*, not merely record that
/// *some* table's `admits` call returned `true`. Before this, `guards` was
/// a live argument to [`seal_execution_receipt`]/[`verify_execution_receipt`]
/// that never left a trace in the receipt itself: two calls to
/// `seal_execution_receipt` with the same `fired`/`completed_after` but
/// different `guards` (e.g. a real compiled table vs.
/// `ConcurrencyGuardTable::empty()`) produced byte-identical receipts, and
/// `verify_execution_receipt` would happily "verify" a receipt sealed
/// against a weak table by re-checking it against that same weak table —
/// the receipt carried no evidence of *which* table was used, so nothing
/// stopped a caller from substituting a different/weaker one at both ends.
/// Folding `digest_guard_table(guards)` into the receipt's hash chain (see
/// [`canonical_bytes`]) makes the guard table's content part of what the
/// hash attests to: verifying with a table whose content differs from the
/// one used at sealing time now fails loudly (see
/// [`ExecutionIntegrityError::GuardsMismatch`]) instead of silently
/// succeeding.
///
/// This does **not** prove the supplied `guards` is *the* table that
/// actually corresponds to `powl_model_digest`/`compiled_digest` — that
/// would require a compiled-model -> guard-table bridge this phase does
/// not have (see this module's own doc comment: the legacy tape
/// `scheduler_tick_guarded` runs on and the v2 `PowlModel` /
/// `ConcurrencyGuardTable` pipeline are not bridged in `bcinr-powl` yet).
/// A verifier still has to obtain the *correct* table for a claimed model
/// out of band; what this digest buys is that once they have it, a
/// mismatched or substituted table is now detectable from the receipt
/// alone, rather than invisible.
///
/// # Complexity
/// O(`guards.nonfaces.len()`).
pub fn digest_guard_table(guards: &ConcurrencyGuardTable) -> Digest {
    let mut buf = Vec::with_capacity(4 + guards.nonfaces.len() * 40);
    buf.extend_from_slice(&(guards.nonfaces.len() as u32).to_le_bytes());
    for nf in &guards.nonfaces {
        push_event_set(&mut buf, &nf.members);
        buf.extend_from_slice(nf.witness_digest.as_bytes());
    }
    Digest::hash(&buf)
}

// ---------------------------------------------------------------------------
// ExecutionReceipt
// ---------------------------------------------------------------------------

/// A receipt attesting to one scheduler tick's real firing decision.
///
/// `guards_digest` commits to the exact [`ConcurrencyGuardTable`] content
/// `fired` was checked against (see [`digest_guard_table`]) — it is what
/// closes the "seal against a weak table, verify against that same weak
/// table, get `Ok`" gap: a verifier that supplies a table with different
/// content than what was used at sealing time now gets
/// [`ExecutionIntegrityError::GuardsMismatch`] instead of a silent pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionReceipt {
    pub powl_model_digest: Digest,
    pub compiled_digest: Digest,
    pub tick: u32,
    pub scheduler_decision_digest: Digest,
    pub fired: EventSet,
    pub completed_after: EventSet,
    pub guards_digest: Digest,
    pub prior_hash: Digest,
    pub hash: Digest,
}

/// Why an [`ExecutionReceipt`] was refused (at sealing time, by
/// [`seal_execution_receipt`]/[`tick_and_seal_execution_receipt`]) or found
/// invalid (after the fact, by [`verify_execution_receipt`]).
///
/// This is the "execution integrity" half of the `projection integrity !=
/// execution integrity` distinction: a [`crate::projection::ProjectionReceipt`]
/// attests that a *compilation* step preserved semantics; this attests that
/// a *firing decision* was actually admissible under the compiled
/// [`ConcurrencyGuardTable`] and that the receipt recording it is exactly
/// what [`seal_execution_receipt`] would have produced — not merely an
/// internally-consistent hash chain (digest equality is not semantic
/// equivalence: a hand-fabricated receipt can hash-chain perfectly while
/// still claiming an inadmissible `fired` set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionIntegrityError {
    /// `fired` contains one of `guards`'s minimal nonfaces as a subset —
    /// this `EventSet` was never jointly executable, regardless of what the
    /// rest of the receipt claims. See
    /// [`bcinr_powl::tape::v2::ConcurrencyGuardTable::admits`].
    InadmissibleFiredSet { fired: EventSet },
    /// The receipt's own `hash` does not equal
    /// `fold(receipt.prior_hash, canonical_bytes_of(receipt's other fields))`
    /// — the receipt was tampered with, mis-chained, or hand-assembled via
    /// struct-literal syntax rather than produced by
    /// [`seal_execution_receipt`].
    HashMismatch { expected: Digest, found: Digest },
    /// `guards`'s content digest (see [`digest_guard_table`]) does not
    /// match `receipt.guards_digest` — the table passed to
    /// [`verify_execution_receipt`] is not the same table (by content) that
    /// [`seal_execution_receipt`] checked `fired` against. This is the
    /// direct defense against "seal against table A, verify against a
    /// different table B that happens to also admit `fired`": verification
    /// now requires the *same* table content, not merely *some* table that
    /// agrees. It does not, by itself, prove `guards` is the table that
    /// actually corresponds to `powl_model_digest`/`compiled_digest` — see
    /// [`digest_guard_table`]'s doc comment for that residual gap.
    GuardsMismatch { expected: Digest, found: Digest },
}

/// Canonical byte encoding shared by [`seal_execution_receipt`] (which
/// folds it against `prior_hash` to produce `hash`) and
/// [`verify_execution_receipt`] (which recomputes it to check `hash` wasn't
/// tampered with) — kept as one function so the two can never drift apart.
#[allow(clippy::too_many_arguments)]
fn canonical_bytes(
    powl_model_digest: Digest,
    compiled_digest: Digest,
    tick: u32,
    scheduler_decision_digest: Digest,
    fired: &EventSet,
    completed_after: &EventSet,
    guards_digest: Digest,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32 + 32 + 4 + 32 + 8 + 8 + 32);
    buf.extend_from_slice(powl_model_digest.as_bytes());
    buf.extend_from_slice(compiled_digest.as_bytes());
    buf.extend_from_slice(&tick.to_le_bytes());
    buf.extend_from_slice(scheduler_decision_digest.as_bytes());
    push_event_set(&mut buf, fired);
    push_event_set(&mut buf, completed_after);
    buf.extend_from_slice(guards_digest.as_bytes());
    buf
}

/// Seal an [`ExecutionReceipt`] from already-computed evidence, refusing to
/// produce one if `fired` is not admissible under `guards` — i.e. `fired`
/// contains one of `guards`'s minimal nonfaces as a subset (see
/// [`bcinr_powl::tape::v2::ConcurrencyGuardTable::admits`]). This is the
/// enforcement point: a caller cannot obtain a well-formed `ExecutionReceipt`
/// for an inadmissible `fired` set through this function, closing the gap
/// where `fired` was previously accepted as a trusted argument with nothing
/// to validate it against.
///
/// Pure otherwise — does not itself run the scheduler (see
/// [`tick_and_seal_execution_receipt`] for the real-scheduler wiring that
/// produces `fired`/`completed_after`/`scheduler_decision_digest`, and
/// which passes the same `guards` the real `ConcurrencySelector` already
/// gated firing against, so in normal use this check should never trip —
/// it exists to reject a receipt built any other way, e.g. by hand).
///
/// # Complexity
/// O(`guards.nonfaces.len()`), dominated by `ConcurrencyGuardTable::admits`.
#[allow(clippy::too_many_arguments)]
pub fn seal_execution_receipt(
    prior_hash: Digest,
    powl_model_digest: Digest,
    compiled_digest: Digest,
    tick: u32,
    scheduler_decision_digest: Digest,
    fired: EventSet,
    completed_after: EventSet,
    guards: &ConcurrencyGuardTable,
) -> Result<ExecutionReceipt, ExecutionIntegrityError> {
    if !guards.admits(&fired) {
        return Err(ExecutionIntegrityError::InadmissibleFiredSet { fired });
    }

    let guards_digest = digest_guard_table(guards);
    let buf = canonical_bytes(
        powl_model_digest,
        compiled_digest,
        tick,
        scheduler_decision_digest,
        &fired,
        &completed_after,
        guards_digest,
    );
    let hash = fold(&prior_hash, &buf);

    Ok(ExecutionReceipt {
        powl_model_digest,
        compiled_digest,
        tick,
        scheduler_decision_digest,
        fired,
        completed_after,
        guards_digest,
        prior_hash,
        hash,
    })
}

/// Verify a previously-sealed [`ExecutionReceipt`] against `guards`: confirm
/// `fired` is (still) admissible, and that `hash` genuinely equals
/// `fold(receipt.prior_hash, canonical_bytes)` recomputed from the receipt's
/// own fields — i.e. the receipt is both execution-admissible and was not
/// hand-assembled or tampered with after sealing (struct-literal
/// construction can set every field to any value, since none are private —
/// this recomputation is what actually pins `hash` to the fields it claims
/// to attest to).
///
/// This does **not** re-verify that `prior_hash` matches an external chain
/// position — that is a chain-level concern for the receipt's caller, which
/// already carries `prior_hash` as its own linkage field.
///
/// # Complexity
/// O(`guards.nonfaces.len()` + `receipt.fired.len()` +
/// `receipt.completed_after.len()`).
pub fn verify_execution_receipt(
    receipt: &ExecutionReceipt,
    guards: &ConcurrencyGuardTable,
) -> Result<(), ExecutionIntegrityError> {
    if !guards.admits(&receipt.fired) {
        return Err(ExecutionIntegrityError::InadmissibleFiredSet {
            fired: receipt.fired,
        });
    }

    let guards_digest = digest_guard_table(guards);
    if guards_digest != receipt.guards_digest {
        return Err(ExecutionIntegrityError::GuardsMismatch {
            expected: receipt.guards_digest,
            found: guards_digest,
        });
    }

    let buf = canonical_bytes(
        receipt.powl_model_digest,
        receipt.compiled_digest,
        receipt.tick,
        receipt.scheduler_decision_digest,
        &receipt.fired,
        &receipt.completed_after,
        receipt.guards_digest,
    );
    let expected = fold(&receipt.prior_hash, &buf);
    if expected != receipt.hash {
        return Err(ExecutionIntegrityError::HashMismatch {
            expected,
            found: receipt.hash,
        });
    }

    Ok(())
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
///
/// Returns [`ExecutionIntegrityError::InadmissibleFiredSet`] if
/// `scheduler_tick_guarded`'s real output somehow fails `guards.admits` —
/// this should never happen given `ConcurrencySelector::select`/
/// `select_checked` already gate every candidate against the same `guards`
/// before it fires (see `bcinr_powl::scheduler`), but the check is not
/// skipped on that assumption: if the selector's own gating ever regressed,
/// this is the backstop that keeps a bad firing decision from being sealed
/// into a receipt at all, rather than silently trusting the assumption.
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
) -> Result<ExecutionReceipt, ExecutionIntegrityError> {
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
        guards,
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
        let guards = ConcurrencyGuardTable::empty();
        let r1 = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            fired,
            completed_after,
            &guards,
        )
        .unwrap();
        let r2 = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            fired,
            completed_after,
            &guards,
        )
        .unwrap();
        assert_eq!(r1.hash, r2.hash);

        let r3 = seal_execution_receipt(
            r1.hash,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            2,
            Digest::hash(b"decision-2"),
            fired,
            completed_after,
            &guards,
        )
        .unwrap();
        assert_ne!(r3.hash, r1.hash);
        assert_eq!(r3.prior_hash, r1.hash);
    }

    #[test]
    fn different_fired_sets_produce_different_hashes() {
        let guards = ConcurrencyGuardTable::empty();
        let base = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            EventSet::empty().with(0),
            EventSet::empty().with(0),
            &guards,
        )
        .unwrap();
        let different_fired = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            EventSet::empty().with(1),
            EventSet::empty().with(0),
            &guards,
        )
        .unwrap();
        assert_ne!(base.hash, different_fired.hash);
    }

    #[test]
    fn seal_execution_receipt_refuses_an_inadmissible_fired_set() {
        // The exact adversarial fixture from
        // `bcinr-pddl/tests/mfw_capacity2_fixture.rs`'s
        // `link8b_...`: a hand-fabricated "all three fired" claim, against
        // a guard table whose one minimal nonface is exactly that triple.
        // Before this fix, `seal_execution_receipt` had no `guards`
        // parameter at all and accepted this unconditionally.
        let guards = ConcurrencyGuardTable {
            nonfaces: vec![CompiledNonFace {
                members: EventSet::empty().with(0).with(1).with(2),
                witness_digest: Digest::hash(b"capacity2-conflict"),
            }],
        };
        let fabricated_fired = EventSet::empty().with(0).with(1).with(2);
        let result = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"fabricated-decision"),
            fabricated_fired,
            fabricated_fired,
            &guards,
        );
        assert_eq!(
            result,
            Err(ExecutionIntegrityError::InadmissibleFiredSet {
                fired: fabricated_fired
            })
        );
    }

    #[test]
    fn verify_execution_receipt_accepts_a_genuine_receipt() {
        let guards = ConcurrencyGuardTable::empty();
        let receipt = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            EventSet::empty().with(0),
            EventSet::empty().with(0),
            &guards,
        )
        .unwrap();
        assert_eq!(verify_execution_receipt(&receipt, &guards), Ok(()));
    }

    #[test]
    fn verify_execution_receipt_rejects_a_receipt_hand_assembled_via_struct_literal() {
        // `ExecutionReceipt`'s fields are all `pub`, so nothing stops a
        // caller from constructing one directly instead of going through
        // `seal_execution_receipt` — this is exactly the "internally
        // consistent but never actually sealed" fabrication the gap report
        // describes. `verify_execution_receipt` must catch it because the
        // claimed `hash` does not match what folding the other fields would
        // actually produce.
        let guards = ConcurrencyGuardTable::empty();
        let fired = EventSet::empty().with(0).with(1).with(2);
        let fabricated = ExecutionReceipt {
            powl_model_digest: Digest::hash(b"model"),
            compiled_digest: Digest::hash(b"compiled"),
            tick: 1,
            scheduler_decision_digest: Digest::hash(b"decision"),
            fired,
            completed_after: fired,
            guards_digest: digest_guard_table(&guards),
            prior_hash: Digest::ZERO,
            hash: Digest::hash(b"not-actually-derived-from-the-other-fields"),
        };
        match verify_execution_receipt(&fabricated, &guards) {
            Err(ExecutionIntegrityError::HashMismatch { found, .. }) => {
                assert_eq!(found, fabricated.hash);
            }
            other => panic!("expected HashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn verify_execution_receipt_rejects_an_inadmissible_fired_set_even_with_a_consistent_hash() {
        // A receipt can be perfectly self-consistent (hash genuinely folds
        // its own fields) while still claiming an execution that was never
        // admissible — this is the case `seal_execution_receipt` itself
        // prevents, but `verify_execution_receipt` must independently catch
        // it too, since a receipt could reach a verifier via any channel
        // (not just this crate's own sealing function), and a stale
        // `guards` mismatch (e.g. re-verifying against a different/updated
        // concurrency complex) is exactly when this check earns its keep.
        let empty_guards = ConcurrencyGuardTable::empty();
        let fired = EventSet::empty().with(0).with(1).with(2);
        let receipt = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            fired,
            fired,
            &empty_guards,
        )
        .unwrap();

        let stricter_guards = ConcurrencyGuardTable {
            nonfaces: vec![CompiledNonFace {
                members: fired,
                witness_digest: Digest::hash(b"conflict"),
            }],
        };
        match verify_execution_receipt(&receipt, &stricter_guards) {
            Err(ExecutionIntegrityError::InadmissibleFiredSet { fired: f }) => {
                assert_eq!(f, fired);
            }
            other => panic!("expected InadmissibleFiredSet, got {other:?}"),
        }
    }

    #[test]
    fn verify_execution_receipt_rejects_a_different_guard_table_that_still_admits_fired() {
        // The exact substitution the gap report describes: `fired` is
        // admissible under *both* tables (so an `admits`-only check cannot
        // tell them apart), but table_b is not the table
        // `seal_execution_receipt` actually checked `fired` against.
        // Before `guards_digest` was folded into the receipt, this
        // substitution was undetectable: `verify_execution_receipt(&receipt,
        // &table_b)` returned `Ok` purely because table_b's `admits` call
        // also happened to agree, not because table_b was the table
        // sealing used.
        let fired = EventSet::empty().with(0).with(1);
        let table_a = ConcurrencyGuardTable::empty();
        let table_b = ConcurrencyGuardTable {
            nonfaces: vec![CompiledNonFace {
                members: EventSet::empty().with(5).with(6), // disjoint from `fired`
                witness_digest: Digest::hash(b"unrelated-conflict"),
            }],
        };
        assert!(table_a.admits(&fired));
        assert!(table_b.admits(&fired)); // both admit -- admits() alone can't tell them apart

        let receipt = seal_execution_receipt(
            Digest::ZERO,
            Digest::hash(b"model"),
            Digest::hash(b"compiled"),
            1,
            Digest::hash(b"decision"),
            fired,
            fired,
            &table_a,
        )
        .unwrap();

        // Verifying against the real table used at sealing time succeeds.
        assert_eq!(verify_execution_receipt(&receipt, &table_a), Ok(()));

        // Verifying against a *different* table -- one that also admits
        // `fired`, so an `admits`-only check would have accepted it -- must
        // now fail, because its content digest does not match what sealing
        // actually used.
        match verify_execution_receipt(&receipt, &table_b) {
            Err(ExecutionIntegrityError::GuardsMismatch { expected, found }) => {
                assert_eq!(expected, digest_guard_table(&table_a));
                assert_eq!(found, digest_guard_table(&table_b));
            }
            other => panic!("expected GuardsMismatch, got {other:?}"),
        }
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
        )
        .expect(
            "the real scheduler's own firing decision must be admissible under the same guards",
        );
        assert_eq!(verify_execution_receipt(&receipt1, &guards), Ok(()));

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
        )
        .expect(
            "the real scheduler's own firing decision must be admissible under the same guards",
        );
        assert_eq!(verify_execution_receipt(&receipt2, &guards), Ok(()));
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
