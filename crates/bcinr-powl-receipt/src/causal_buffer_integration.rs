//! Causal Order Buffer Integration (Iteration 8)
//!
//! Assembles the final zero-allocation deterministic causal order buffers
//! by integrating the branchless MPMC ring buffer with OCEL Causal Frames.

use bcinr_logic::algorithms::branchless_ring_buffer_mpmc::branchless_ring_buffer_mpmc;
use crate::causal_receipt::{OcelCausalFrame, OcelCausalReceipt};

/// A `#![no_std]` compliant, branchless causal order buffer.
pub struct CausalOrderBuffer<const N: usize> {
    pub frames: [OcelCausalFrame; N],
    pub receipt: OcelCausalReceipt,
    pub head: u64,
}

impl<const N: usize> CausalOrderBuffer<N> {
    /// Initializes a new zero-allocation buffer.
    pub fn new(run_id: [u8; 32], empty_frame: OcelCausalFrame) -> Self {
        Self {
            frames: [empty_frame; N],
            receipt: OcelCausalReceipt::genesis(run_id),
            head: 0,
        }
    }

    /// Pushes a frame deterministically without branching.
    /// `N` must be a power of two.
    pub fn push(&mut self, mut frame: OcelCausalFrame) {
        // Causal ordering: capture the current receipt hash.
        frame.prior_hash = self.receipt.chain_hash;
        
        // Chain the receipt branchlessly.
        self.receipt.chain(&frame);
        
        // Branchless ring buffer offset masking (N must be a power of 2, so N-1 is a mask).
        let mask = (N as u64).wrapping_sub(1);
        let index = branchless_ring_buffer_mpmc(self.head, mask) as usize;
        
        self.frames[index] = frame;
        self.head = self.head.wrapping_add(1);
    }
}
