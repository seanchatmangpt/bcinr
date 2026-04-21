//! # Universe64 Contract: DeltaTape (Bounded Append-Only Ring)
//! Plane: S — scratch-resident delta history; not canonical state.
//! Tier: T0 per-append; T1 replay scan.
//! Scope: bounded ring of DELTA_TAPE_CAPACITY UDelta records; no heap.
//! Geometry: append-only logical tape; physical ring-buffer with wrapping position.
//! Delta: stores UDelta records; does not emit new ones.
//!
//! # Timing contract
//! - **T0 append:** ≤ T0_BUDGET_NS (single slot write)
//! - **T1 replay scan:** ≤ T1_BUDGET_NS (up to 64 replays)
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Ring-buffer index arithmetic; branchless slot masking.
//! CC=1: Absolute branchless logic.

use super::scratch::{UDelta, UScope};

/// Ring capacity for the delta tape.
pub const DELTA_TAPE_CAPACITY: usize = 256;

/// Logical tape position type.
pub type TapePosition = u32;

/// Bounded append-only ring buffer of UDelta records.
/// When full, oldest records are overwritten (ring semantics).
#[derive(Clone, Copy)]
pub struct DeltaTape {
    /// Physical ring storage.
    entries: [UDelta; DELTA_TAPE_CAPACITY],
    /// Logical append position (monotone, wraps mod DELTA_TAPE_CAPACITY for physical slot).
    head: u32,
    /// Number of valid entries (saturates at DELTA_TAPE_CAPACITY).
    fill: u32,
}

impl DeltaTape {
    pub const fn new() -> Self {
        let blank = UDelta { word_idx: 0, scope: UScope::Cell, instr_id: 0, before: 0, after: 0, fired_mask: 0 };
        Self {
            entries: [blank; DELTA_TAPE_CAPACITY],
            head: 0,
            fill: 0,
        }
    }

    /// Append a delta. Overwrites oldest entry if ring is full.
    #[inline(always)]
    pub fn append(&mut self, delta: UDelta) -> TapePosition {
        let slot = (self.head as usize) & (DELTA_TAPE_CAPACITY - 1);
        self.entries[slot] = delta;
        let pos = self.head;
        self.head = self.head.wrapping_add(1);
        self.fill = self.fill.saturating_add(1).min(DELTA_TAPE_CAPACITY as u32);
        pos
    }

    /// Number of valid records (up to DELTA_TAPE_CAPACITY).
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.fill as usize
    }

    /// True if no records present.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.fill == 0
    }

    /// Monotone logical head position (total appends, wraps at u32::MAX).
    #[inline(always)]
    pub const fn tape_position(&self) -> TapePosition {
        self.head
    }

    /// Read entry at logical position `pos`.
    /// Returns None if pos is outside the current window.
    #[inline(always)]
    pub fn read(&self, pos: TapePosition) -> Option<&UDelta> {
        // Valid window: [head - fill, head)
        let fill = self.fill;
        let start = self.head.wrapping_sub(fill);
        let in_range = (pos.wrapping_sub(start) < fill) as u32;
        let slot = (pos as usize) & (DELTA_TAPE_CAPACITY - 1);
        if in_range != 0 { Some(&self.entries[slot]) } else { None }
    }

    /// Replay deltas from logical position `from` up to head, calling `f` on each.
    /// Calls are in chronological order (oldest first).
    pub fn replay_from<F: FnMut(&UDelta)>(&self, from: TapePosition, mut f: F) {
        let fill = self.fill;
        let start = self.head.wrapping_sub(fill);
        // Clamp `from` to start of valid window.
        let replay_start = if from.wrapping_sub(start) < fill { from } else { start };
        let count = self.head.wrapping_sub(replay_start);
        for i in 0..count {
            let pos = replay_start.wrapping_add(i);
            let slot = (pos as usize) & (DELTA_TAPE_CAPACITY - 1);
            f(&self.entries[slot]);
        }
    }

    /// Reset tape (clear all entries).
    #[inline(always)]
    pub fn reset(&mut self) {
        self.head = 0;
        self.fill = 0;
    }
}

impl Default for DeltaTape {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_delta(word_idx: usize, before: u64, after: u64) -> UDelta {
        UDelta::new(word_idx, UScope::Cell, 0, before, after, !0)
    }

    #[test]
    fn test_append_and_read() {
        let mut tape = DeltaTape::new();
        let pos = tape.append(make_delta(5, 0, 0xFF));
        assert_eq!(pos, 0);
        assert_eq!(tape.len(), 1);
        let d = tape.read(0).unwrap();
        assert_eq!(d.word_idx, 5);
        assert_eq!(d.after, 0xFF);
    }

    #[test]
    fn test_tape_position_increments() {
        let mut tape = DeltaTape::new();
        tape.append(make_delta(0, 0, 1));
        tape.append(make_delta(1, 0, 2));
        assert_eq!(tape.tape_position(), 2);
    }

    #[test]
    fn test_ring_overflow() {
        let mut tape = DeltaTape::new();
        for i in 0..(DELTA_TAPE_CAPACITY + 10) {
            tape.append(make_delta(0, 0, i as u64));
        }
        assert_eq!(tape.len(), DELTA_TAPE_CAPACITY);
    }

    #[test]
    fn test_replay_from() {
        let mut tape = DeltaTape::new();
        tape.append(make_delta(0, 0, 1));
        tape.append(make_delta(1, 0, 2));
        tape.append(make_delta(2, 0, 3));
        let mut replayed = [0u64; 8];
        let mut n = 0usize;
        tape.replay_from(1, |d| { if n < 8 { replayed[n] = d.after; n += 1; } });
        assert_eq!(&replayed[..n], &[2, 3]);
    }

    #[test]
    fn test_read_out_of_range() {
        let tape = DeltaTape::new();
        assert!(tape.read(0).is_none());
    }
}
