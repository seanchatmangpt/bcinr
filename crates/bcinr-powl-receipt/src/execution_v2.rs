//! Whole-run receipt and replay verification for executable POWL v2 tapes.
//!
//! A receipt commits to the immutable compiled tape, every admitted firing
//! mask, the final completion mask, and a chained BLAKE3 root. Verification
//! replays the same deterministic stable-maximal scheduler against the same
//! concurrency guards and compares every committed field.

use bcinr_powl::scheduler::{ConcurrencySelector, StableMaximalSelector};
use bcinr_powl::scheduler_v2::{scheduler_tick_v2, PowlV2RunState, PowlV2TickOutcome};
use bcinr_powl::tape::v2::{ConcurrencyGuardTable, PowlTape};
use serde::{Deserialize, Serialize};

/// Receipt format version for the POWL v2 execution rail.
pub const EXECUTION_V2_RECEIPT_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowlV2ExecutionReceipt {
    pub version: u16,
    pub tape_root: String,
    pub guard_root: String,
    pub fired_masks: Vec<u64>,
    pub final_done_mask: u64,
    pub tick_count: u32,
    pub chain_root: String,
    /// Operations blocked by external constraints (resource conflicts, etc.)
    pub final_blocked_mask: u64,
    /// Operations refused by admission gates
    pub final_refused_mask: u64,
    /// Operations that timed out (exceeded iteration/deadline limits)
    pub final_timed_out_mask: u64,
    /// Digest of reasons for blocked/refused/timed_out operations
    pub reasons_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowlV2ReceiptError {
    Deadlock { remaining_mask: u64 },
    TickBoundExceeded { limit: u32, remaining_mask: u64 },
    TapeRootMismatch,
    GuardRootMismatch,
    FiredTraceMismatch,
    FinalStateMismatch,
    ChainRootMismatch,
    UnsupportedVersion { found: u16 },
    BlockedMaskMismatch { expected: u64, found: u64 },
    RefusedMaskMismatch { expected: u64, found: u64 },
    TimedOutMaskMismatch { expected: u64, found: u64 },
    ReasonsDigestMismatch,
}

impl std::fmt::Display for PowlV2ReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deadlock { remaining_mask } => {
                write!(
                    f,
                    "POWL v2 execution deadlocked with mask {remaining_mask:#x}"
                )
            }
            Self::TickBoundExceeded {
                limit,
                remaining_mask,
            } => write!(
                f,
                "POWL v2 execution exceeded {limit} ticks with mask {remaining_mask:#x}"
            ),
            Self::TapeRootMismatch => write!(f, "POWL v2 tape root mismatch"),
            Self::GuardRootMismatch => write!(f, "POWL v2 guard root mismatch"),
            Self::FiredTraceMismatch => write!(f, "POWL v2 fired-mask trace mismatch"),
            Self::FinalStateMismatch => write!(f, "POWL v2 final state mismatch"),
            Self::ChainRootMismatch => write!(f, "POWL v2 chain root mismatch"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported POWL v2 receipt version {found}")
            }
            Self::BlockedMaskMismatch { expected, found } => {
                write!(
                    f,
                    "POWL v2 blocked mask mismatch: expected {expected:#x}, found {found:#x}"
                )
            }
            Self::RefusedMaskMismatch { expected, found } => {
                write!(
                    f,
                    "POWL v2 refused mask mismatch: expected {expected:#x}, found {found:#x}"
                )
            }
            Self::TimedOutMaskMismatch { expected, found } => {
                write!(
                    f,
                    "POWL v2 timed out mask mismatch: expected {expected:#x}, found {found:#x}"
                )
            }
            Self::ReasonsDigestMismatch => write!(f, "POWL v2 reasons digest mismatch"),
        }
    }
}

impl std::error::Error for PowlV2ReceiptError {}

/// Execute a compiled POWL v2 tape and seal a whole-run receipt, using the
/// default deterministic [`StableMaximalSelector`].
///
/// Delegates to [`execute_and_seal_v2_with_selector`] -- see that function
/// for a version parameterized over [`ConcurrencySelector`] (e.g.
/// `PriorityCapacitySelector` from BCINR-CMCA-E, or `CapacityBoundedSelector`
/// from BCINR-SCHED-002). This wrapper exists so every pre-existing call
/// site keeps its exact prior behavior unchanged.
pub fn execute_and_seal_v2(
    tape: &PowlTape,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> Result<PowlV2ExecutionReceipt, PowlV2ReceiptError> {
    let mut selector = StableMaximalSelector;
    execute_and_seal_v2_with_selector(tape, &mut selector, guards, max_ticks)
}

/// Execute a compiled POWL v2 tape and seal a whole-run receipt, using a
/// caller-supplied [`ConcurrencySelector`].
///
/// # BCINR-CMCA-F: production-reachability boundary
///
/// This is the entry point that lets a non-default selector (e.g. a real
/// CMCA-priority-ordered `PriorityCapacitySelector`) actually seal a receipt
/// -- `execute_and_seal_v2` alone hard-codes `StableMaximalSelector` and
/// cannot be handed a different one. `verify_execution_v2_with_selector`
/// must be given the *same* selector type/state to replay correctly; mixing
/// selectors between seal and verify will produce a genuine
/// `FiredTraceMismatch`, since fired-mask order is selector-dependent.
pub fn execute_and_seal_v2_with_selector<S: ConcurrencySelector>(
    tape: &PowlTape,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> Result<PowlV2ExecutionReceipt, PowlV2ReceiptError> {
    let tape_root = digest_tape(tape);
    let guard_root = digest_guards(guards);
    let mut state = PowlV2RunState::new();
    let mut fired_masks = Vec::new();

    for _ in 0..max_ticks {
        match scheduler_tick_v2(tape, &mut state, selector, guards) {
            PowlV2TickOutcome::Fired(mask) => fired_masks.push(mask),
            PowlV2TickOutcome::Complete => break,
            PowlV2TickOutcome::Deadlock { remaining_mask } => {
                return Err(PowlV2ReceiptError::Deadlock { remaining_mask });
            }
        }
        if state.is_complete(tape) {
            break;
        }
    }

    if !state.is_complete(tape) {
        return Err(PowlV2ReceiptError::TickBoundExceeded {
            limit: max_ticks,
            remaining_mask: valid_mask(tape.len) & !state.done_mask,
        });
    }

    let chain_root = digest_chain(&tape_root, &guard_root, &fired_masks, state.done_mask);

    // Compute reasons digest (currently empty since blocked/refused/timed_out tracking
    // is populated by scheduler_tick_with_resources, not standard scheduler_tick_v2)
    let reasons_digest = digest_reasons(&[], &[], &[]);

    Ok(PowlV2ExecutionReceipt {
        version: EXECUTION_V2_RECEIPT_VERSION,
        tape_root,
        guard_root,
        fired_masks,
        final_done_mask: state.done_mask,
        tick_count: state.tick,
        chain_root,
        final_blocked_mask: 0,
        final_refused_mask: 0,
        final_timed_out_mask: 0,
        reasons_digest,
    })
}

/// Replay and verify every field of a POWL v2 execution receipt, using the
/// default deterministic [`StableMaximalSelector`].
///
/// Delegates to [`verify_execution_v2_with_selector`] -- see that function,
/// and [`execute_and_seal_v2_with_selector`]'s doc comment, for verifying a
/// receipt sealed with a non-default selector.
pub fn verify_execution_v2(
    receipt: &PowlV2ExecutionReceipt,
    tape: &PowlTape,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> Result<(), PowlV2ReceiptError> {
    let mut selector = StableMaximalSelector;
    verify_execution_v2_with_selector(receipt, tape, &mut selector, guards, max_ticks)
}

/// Replay and verify every field of a POWL v2 execution receipt, using a
/// caller-supplied [`ConcurrencySelector`]. Must be the same selector
/// type/construction used to seal `receipt`, or replay will produce a
/// genuine `FiredTraceMismatch` (fired-mask order is selector-dependent).
pub fn verify_execution_v2_with_selector<S: ConcurrencySelector>(
    receipt: &PowlV2ExecutionReceipt,
    tape: &PowlTape,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> Result<(), PowlV2ReceiptError> {
    if receipt.version != EXECUTION_V2_RECEIPT_VERSION {
        return Err(PowlV2ReceiptError::UnsupportedVersion {
            found: receipt.version,
        });
    }
    let replay = execute_and_seal_v2_with_selector(tape, selector, guards, max_ticks)?;
    if replay.tape_root != receipt.tape_root {
        return Err(PowlV2ReceiptError::TapeRootMismatch);
    }
    if replay.guard_root != receipt.guard_root {
        return Err(PowlV2ReceiptError::GuardRootMismatch);
    }
    if replay.fired_masks != receipt.fired_masks || replay.tick_count != receipt.tick_count {
        return Err(PowlV2ReceiptError::FiredTraceMismatch);
    }
    if replay.final_done_mask != receipt.final_done_mask {
        return Err(PowlV2ReceiptError::FinalStateMismatch);
    }
    if replay.chain_root != receipt.chain_root {
        return Err(PowlV2ReceiptError::ChainRootMismatch);
    }
    if replay.final_blocked_mask != receipt.final_blocked_mask {
        return Err(PowlV2ReceiptError::BlockedMaskMismatch {
            expected: receipt.final_blocked_mask,
            found: replay.final_blocked_mask,
        });
    }
    if replay.final_refused_mask != receipt.final_refused_mask {
        return Err(PowlV2ReceiptError::RefusedMaskMismatch {
            expected: receipt.final_refused_mask,
            found: replay.final_refused_mask,
        });
    }
    if replay.final_timed_out_mask != receipt.final_timed_out_mask {
        return Err(PowlV2ReceiptError::TimedOutMaskMismatch {
            expected: receipt.final_timed_out_mask,
            found: replay.final_timed_out_mask,
        });
    }
    if replay.reasons_digest != receipt.reasons_digest {
        return Err(PowlV2ReceiptError::ReasonsDigestMismatch);
    }
    Ok(())
}

/// Deterministic BLAKE3 commitment to the compiled tape, including labels.
pub fn digest_tape(tape: &PowlTape) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:powl-v2:tape:v1");
    hasher.update(&[tape.len, tape.entry_op, tape.exit_op]);
    for index in 0..tape.len as usize {
        let op = &tape.ops[index];
        hasher.update(&op.pred_mask.to_le_bytes());
        hasher.update(&op.succ_mask.to_le_bytes());
        hasher.update(&op.ctrl.to_le_bytes());
        hasher.update(&[op.op_kind as u8, op.choice_group, op.depth, op.fan_out]);
    }
    hasher.update(&tape.label_slab.len.to_le_bytes());
    hasher.update(&tape.label_slab.data[..tape.label_slab.len as usize]);
    hasher.finalize().to_hex().to_string()
}

/// Deterministic commitment to compiled minimal nonfaces.
pub fn digest_guards(guards: &ConcurrencyGuardTable) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:powl-v2:guards:v1");
    hasher.update(&(guards.nonfaces.len() as u64).to_le_bytes());
    for nonface in &guards.nonfaces {
        for member in nonface.members.iter_stable() {
            hasher.update(&(member as u64).to_le_bytes());
        }
        hasher.update(&[0xff]);
        hasher.update(nonface.witness_digest.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

/// Deterministic commitment to blocked/refused/timed_out reasons.
fn digest_reasons(
    blocked_reasons: &[(usize, String)],
    refused_reasons: &[(usize, String)],
    timed_out_reasons: &[(usize, String)],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:powl-v2:reasons:v1");

    // Digest blocked reasons
    hasher.update(&(blocked_reasons.len() as u64).to_le_bytes());
    for (idx, reason) in blocked_reasons {
        hasher.update(&(*idx as u64).to_le_bytes());
        hasher.update(reason.as_bytes());
        hasher.update(&[0xff]); // separator
    }

    // Digest refused reasons
    hasher.update(&(refused_reasons.len() as u64).to_le_bytes());
    for (idx, reason) in refused_reasons {
        hasher.update(&(*idx as u64).to_le_bytes());
        hasher.update(reason.as_bytes());
        hasher.update(&[0xff]); // separator
    }

    // Digest timed_out reasons
    hasher.update(&(timed_out_reasons.len() as u64).to_le_bytes());
    for (idx, reason) in timed_out_reasons {
        hasher.update(&(*idx as u64).to_le_bytes());
        hasher.update(reason.as_bytes());
        hasher.update(&[0xff]); // separator
    }

    hasher.finalize().to_hex().to_string()
}

fn digest_chain(
    tape_root: &str,
    guard_root: &str,
    fired_masks: &[u64],
    final_done_mask: u64,
) -> String {
    let mut chain = blake3::hash(b"bcinr:powl-v2:execution:v1");
    for (tick, mask) in fired_masks.iter().enumerate() {
        let mut hasher = blake3::Hasher::new();
        hasher.update(chain.as_bytes());
        hasher.update(tape_root.as_bytes());
        hasher.update(guard_root.as_bytes());
        hasher.update(&(tick as u64).to_le_bytes());
        hasher.update(&mask.to_le_bytes());
        chain = hasher.finalize();
    }
    let mut final_hasher = blake3::Hasher::new();
    final_hasher.update(chain.as_bytes());
    final_hasher.update(&final_done_mask.to_le_bytes());
    final_hasher.finalize().to_hex().to_string()
}

const fn valid_mask(len: u8) -> u64 {
    if len == 0 {
        0
    } else if len >= 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}

#[cfg(test)]
mod tests {
    use bcinr_powl::powl2::{compile_powl2, LowestIndexPolicy, Powl2Model};

    use super::*;

    fn compiled() -> bcinr_powl::powl2::CompiledPowl2 {
        compile_powl2(
            &Powl2Model::Sequence(vec![
                Powl2Model::PartialOrder {
                    children: vec![
                        Powl2Model::Activity("a".into()),
                        Powl2Model::Activity("b".into()),
                    ],
                    edges: vec![],
                },
                Powl2Model::Activity("c".into()),
            ]),
            &mut LowestIndexPolicy,
        )
        .unwrap()
    }

    #[test]
    fn whole_run_receipt_replays_exactly() {
        let compiled = compiled();
        let guards = ConcurrencyGuardTable::empty();
        let receipt = execute_and_seal_v2(&compiled.tape, &guards, 8).unwrap();
        assert_eq!(receipt.fired_masks, vec![0b1000, 0b0011, 0b0100, 0b1_0000]);
        assert_eq!(receipt.tick_count, 4);
        verify_execution_v2(&receipt, &compiled.tape, &guards, 8).unwrap();
    }

    #[test]
    fn tampered_firing_trace_is_refused() {
        let compiled = compiled();
        let guards = ConcurrencyGuardTable::empty();
        let mut receipt = execute_and_seal_v2(&compiled.tape, &guards, 8).unwrap();
        receipt.fired_masks[1] ^= 1;
        assert_eq!(
            verify_execution_v2(&receipt, &compiled.tape, &guards, 8),
            Err(PowlV2ReceiptError::FiredTraceMismatch)
        );
    }

    #[test]
    fn insufficient_tick_bound_cannot_emit_a_receipt() {
        let compiled = compiled();
        assert!(matches!(
            execute_and_seal_v2(&compiled.tape, &ConcurrencyGuardTable::empty(), 1),
            Err(PowlV2ReceiptError::TickBoundExceeded { .. })
        ));
    }
}
