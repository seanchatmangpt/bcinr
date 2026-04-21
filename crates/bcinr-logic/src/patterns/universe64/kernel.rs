//! # UniverseOS Kernel: The Execution Loop
//!
//! Plane: S (Scratch) + D (Data)
//! Tier: T1/T2 Execution Loop
//! Scope: Bounded 64 KiB L1 envelope execution.
//! Geometry: State transforms over the 32 KiB `UniverseBlock`.
//!
//! # Timing contract
//! - **T1 aggregate budget:** ≤ 200 ns per cell update.
//! - **Capacity:** 64 active words per transaction maximum.
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. All execution relies on O(1) cell modifications or O(64) domain loops.
//! CC=1: Branchless execution of transitions.

use super::block::{UniverseBlock, UniverseDelta};
use super::constants::{UKERNEL_GATE, UNIVERSE_WORDS, MAX_WORD_INDEX};
use super::instruction::{UInstruction, UInstrKind};
use super::masks::CellMask;
use super::receipt::{new_receipt, receipt_mix_transition, TransitionReceipt};
use super::scratch::{ActiveWordSet, UDelta, UScope, UniverseScratch};
use super::transition::cell_try_fire;

/// Integrity gate for UniverseKernel
#[inline(always)]
pub fn ukernel_phd_gate(val: u64) -> u64 {
    val ^ UKERNEL_GATE
}

/// The Execution Kernel of UniverseOS.
///
/// Contains exactly one 32 KiB Data Plane (`UniverseBlock`) and one 32 KiB Scratch Plane (`UniverseScratch`),
/// creating the 64 KiB L1D-class execution envelope.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct UniverseKernel {
    /// Plane D: The canonical truth geometry.
    pub data_plane: UniverseBlock,
    /// Plane S: The bounded motion workspace.
    pub scratch_plane: UniverseScratch,
    /// The rolling execution manifest (receipt).
    pub receipt: TransitionReceipt,
}

impl UniverseKernel {
    /// Boots a new UniverseOS instance with empty planes and a fresh receipt.
    #[inline(always)]
    pub fn boot() -> Self {
        Self {
            data_plane: UniverseBlock::new(),
            scratch_plane: UniverseScratch::new(),
            receipt: new_receipt(),
        }
    }

    /// Ticks the UniverseOS by proposing a `UInstruction` against the resident `UniverseBlock`.
    ///
    /// This function applies the fundamental loop:
    /// 1. Decode the instruction.
    /// 2. Evaluate admissibility (read data + masks).
    /// 3. Transition the data plane (branchless commit).
    /// 4. Update the rolling receipt.
    /// 5. Emit a compact `UDelta`.
    #[inline(always)]
    pub fn tick(&mut self, instruction: UInstruction) -> UDelta {
        self.scratch_plane.reset_cycle();
        
        let op = instruction.kind();
        let coord = instruction.target_coord();
        let payload = instruction.payload();
        let word_idx = coord.flat_cell_index() & MAX_WORD_INDEX;
        
        // Stage the before-state in the scratch plane.
        let current_cell = self.data_plane.state[word_idx];
        self.scratch_plane.before[word_idx] = current_cell;
        self.scratch_plane.active.push(word_idx);
        
        // Compute truth algebra.
        let (next_cell, success_mask, scope) = match op {
            UInstrKind::CellTransition => {
                // In a real system, the input/output masks would be fetched from an admitted MaskBank via `payload`.
                // For this microkernel, we simulate it via fixed masks provided in the payload (split 16/16).
                let i_mask = ((payload >> 16) & 0xFFFF) as u64;
                let o_mask = (payload & 0xFFFF) as u64;
                let (next, success) = cell_try_fire(current_cell, i_mask, o_mask);
                (next, success, UScope::Cell)
            }
            UInstrKind::CellMaskUpdate => {
                // A direct mask application (e.g. forced transition or boundary ingress).
                let apply_mask = payload as u64;
                let next = current_cell ^ apply_mask; // flip the bits
                (next, !0u64, UScope::Cell) // Always succeeds
            }
            _ => {
                // Unsupported or NO-OP instruction defaults to rejection.
                (current_cell, 0u64, UScope::Cell)
            }
        };

        // Stage post-image and conditionally commit to the data plane.
        self.scratch_plane.after[word_idx] = next_cell;
        self.data_plane.state[word_idx] = next_cell;

        // Mix the receipt.
        self.receipt = TransitionReceipt(receipt_mix_transition(
            self.receipt.value(),
            coord,
            success_mask,
        ));

        // Emit the delta.
        UDelta::new(
            word_idx,
            scope,
            instruction.id(),
            current_cell,
            next_cell,
            success_mask,
        )
    }
}

impl Default for UniverseKernel {
    fn default() -> Self {
        Self::boot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::universe64::coord::UCoord;
    use crate::patterns::universe64::instruction::UTier;

    #[test]
    fn test_ukernel_phd_gate() {
        assert_ne!(ukernel_phd_gate(0), 0);
    }

    #[test]
    fn test_kernel_tick_transition() {
        let mut kernel = UniverseKernel::boot();
        let coord = UCoord::new(0, 0, 0);
        
        // Initial state is 0. Give it a token at bit 1 via direct mask update.
        let init_instr = UInstruction::new(UInstrKind::CellMaskUpdate, UTier::T1, coord, 2);
        let init_delta = kernel.tick(init_instr);
        assert_eq!(init_delta.changed_mask(), 2);
        assert_eq!(kernel.data_plane.get_cell(0), 2);
        
        // Transition: requires bit 1 (2), outputs bit 2 (4).
        let payload = (2 << 16) | 4; 
        let instr = UInstruction::new(UInstrKind::CellTransition, UTier::T1, coord, payload);
        let delta = kernel.tick(instr);
        
        // Success mask should be all 1s.
        assert_eq!(delta.fired_mask, !0);
        // Before was 2, after is 4. Changed mask is 6 (2 ^ 4).
        assert_eq!(delta.changed_mask(), 6);
        assert_eq!(kernel.data_plane.get_cell(0), 4);
    }
    
    #[test]
    fn test_kernel_tick_rejected_transition() {
        let mut kernel = UniverseKernel::boot();
        let coord = UCoord::new(0, 0, 0);
        
        // Transition: requires bit 1 (2), but state is 0.
        let payload = (2 << 16) | 4; 
        let instr = UInstruction::new(UInstrKind::CellTransition, UTier::T1, coord, payload);
        let delta = kernel.tick(instr);
        
        // Success mask should be 0.
        assert_eq!(delta.fired_mask, 0);
        assert_eq!(delta.changed_mask(), 0);
        assert_eq!(kernel.data_plane.get_cell(0), 0);
    }
}
