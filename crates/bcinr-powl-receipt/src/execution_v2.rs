//! Whole-run receipt and replay verification for executable POWL v2 tapes.
//!
//! A receipt commits to the immutable compiled tape, every admitted firing
//! mask, the final completion mask, and a chained BLAKE3 root. Verification
//! replays the same deterministic stable-maximal scheduler against the same
//! concurrency guards and compares every committed field.

use bcinr_powl::scheduler::StableMaximalSelector;
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
        }
    }
}

impl std::error::Error for PowlV2ReceiptError {}

/// Execute a compiled POWL v2 tape and seal a whole-run receipt.
pub fn execute_and_seal_v2(
    tape: &PowlTape,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> Result<PowlV2ExecutionReceipt, PowlV2ReceiptError> {
    let tape_root = digest_tape(tape);
    let guard_root = digest_guards(guards);
    let mut state = PowlV2RunState::new();
    let mut selector = StableMaximalSelector;
    let mut fired_masks = Vec::new();

    for _ in 0..max_ticks {
        match scheduler_tick_v2(tape, &mut state, &mut selector, guards) {
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
    Ok(PowlV2ExecutionReceipt {
        version: EXECUTION_V2_RECEIPT_VERSION,
        tape_root,
        guard_root,
        fired_masks,
        final_done_mask: state.done_mask,
        tick_count: state.tick,
        chain_root,
    })
}

/// Replay and verify every field of a POWL v2 execution receipt.
pub fn verify_execution_v2(
    receipt: &PowlV2ExecutionReceipt,
    tape: &PowlTape,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> Result<(), PowlV2ReceiptError> {
    if receipt.version != EXECUTION_V2_RECEIPT_VERSION {
        return Err(PowlV2ReceiptError::UnsupportedVersion {
            found: receipt.version,
        });
    }
    let replay = execute_and_seal_v2(tape, guards, max_ticks)?;
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
