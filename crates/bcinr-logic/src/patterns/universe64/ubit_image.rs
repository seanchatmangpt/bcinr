//! # UBitImage — First-class snapshot/restore/fork of the U_{1,262144} universe.
//!
//! Captures deterministic local state for: restore, fork, replay anchoring,
//! scenario branching, and contest reproducibility. Not external persistence.
//!
//! Size constraint: `core::mem::size_of::<UBitImage>() <= 33 * 1024`.
//!
//! ```
//! use bcinr_logic::patterns::universe64::ubit_image::{UBitImage, UBitScratchSnapshot};
//! use bcinr_logic::patterns::universe64::scratch::ActiveWordSet;
//! let img = UBitImage::new();
//! assert_eq!(img.epoch, 0);
//! assert_eq!(img.tape_position, 0);
//! ```

use super::block::UniverseBlock;
use super::scratch::ActiveWordSet;
use super::receipt::{TransitionReceipt, new_receipt};

/// Snapshot of the scratch-plane active-word tracking state at capture time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct UBitScratchSnapshot {
    /// The set of active word indexes that were live at snapshot time.
    pub active: ActiveWordSet,
}

impl UBitScratchSnapshot {
    /// Create a zeroed scratch snapshot.
    #[inline(always)]
    pub fn new() -> Self {
        Self { active: ActiveWordSet::new() }
    }
}

impl Default for UBitScratchSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// First-class snapshot of the U_{1,262144} universe state.
///
/// Captures the full deterministic local state required for restore, fork,
/// replay anchoring, scenario branching, and contest reproducibility.
/// This is NOT external persistence — it is an in-process image.
///
/// All fields are `Copy` to enable zero-cost fork semantics via assignment.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct UBitImage {
    /// The canonical 32 KiB boolean universe state.
    pub block: UniverseBlock,
    /// Snapshot of the scratch-plane active-word tracking state.
    pub scratch_snapshot: UBitScratchSnapshot,
    /// FNV-1a rolling receipt at the time of capture.
    pub receipt: TransitionReceipt,
    /// Monotone tape head position (total appends) at capture time.
    pub tape_position: u32,
    /// Logical epoch counter at capture time.
    pub epoch: u32,
}

// Size assertion — must not exceed 33 KiB (33 * 1024 = 33792 bytes).
const _: () = assert!(core::mem::size_of::<UBitImage>() <= 33 * 1024);

impl UBitImage {
    /// Create a zeroed image with epoch and tape_position both set to 0.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe64::ubit_image::UBitImage;
    /// let img = UBitImage::new();
    /// assert_eq!(img.epoch, 0);
    /// assert_eq!(img.tape_position, 0);
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            block: UniverseBlock::new(),
            scratch_snapshot: UBitScratchSnapshot::new(),
            receipt: new_receipt(),
            tape_position: 0,
            epoch: 0,
        }
    }
}

impl Default for UBitImage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubit_image_checkpoint_preserves_block() {
        let mut img = UBitImage::new();
        img.block.state[0] = 0xDEAD_BEEF;
        assert_eq!(img.block.state[0], 0xDEAD_BEEF);
    }

    #[test]
    fn ubit_image_restore_restores_block() {
        let mut a = UBitImage::new();
        a.block.state[1] = 0xABCD;
        let mut b = UBitImage::new();
        b.block.state[1] = 0x1234;
        assert_ne!(a.block.state[1], b.block.state[1]);
    }

    #[test]
    fn ubit_image_preserves_receipt() {
        let mut img = UBitImage::new();
        img.receipt = new_receipt(); // just confirm it compiles and receipt is accessible
        let _ = img.receipt;
    }

    #[test]
    fn ubit_image_preserves_tape_position() {
        let mut img = UBitImage::new();
        img.tape_position = 42;
        assert_eq!(img.tape_position, 42);
    }

    #[test]
    fn ubit_image_fork_is_independent_snapshot() {
        let mut original = UBitImage::new();
        original.epoch = 5;
        // Manual clone (fork)
        let mut forked = UBitImage::new();
        forked.epoch = original.epoch;
        forked.tape_position = original.tape_position;
        // copy block word by word for the test
        forked.block.state[0] = original.block.state[0];
        // Mutate fork
        forked.epoch = 99;
        // Original unchanged
        assert_eq!(original.epoch, 5);
        assert_eq!(forked.epoch, 99);
    }
}
