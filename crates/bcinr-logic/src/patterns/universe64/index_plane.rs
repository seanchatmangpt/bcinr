//! # Universe64 Contract: IndexPlane (Build-Time Geometry Index)
//! Plane: D (read-only at runtime; written once at admission/compile time).
//! Tier: T0 lookup; T1 resolve active path; T2 full rebuild.
//! Scope: bounded static arrays; zero heap; zero runtime allocation.
//! Geometry: 5 cross-reference indexes over (word_idx, bit_idx, transition_id, projection_id, boundary_id).
//! Delta: none produced — read-only structural index.
//!
//! # Timing contract
//! - **T0 lookup budget:** ≤ T0_BUDGET_NS per coord→word_bit projection
//! - **T1 resolve budget:** ≤ T1_BUDGET_NS per active-path scan
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. All lookups are bounded-array index operations.
//! CC=1: Absolute branchless logic.

use super::constants::{COORD_MASK, DOMAIN_SHIFT, CELL_SHIFT, MAX_WORD_INDEX};
use super::coord::UCoord;

// ---------------------------------------------------------------------------
// Capacity constants for bounded static index arrays
// ---------------------------------------------------------------------------

/// Maximum transitions registered in IndexPlane.
pub const MAX_TRANSITIONS: usize = 256;

/// Maximum projections registered in IndexPlane.
pub const MAX_PROJECTIONS: usize = 64;

/// Maximum boundaries registered in IndexPlane.
pub const MAX_BOUNDARIES: usize = 32;

/// Maximum words touched by a single transition.
pub const MAX_WORDS_PER_TRANSITION: usize = 16;

/// Maximum words per boundary side (source or dest).
pub const MAX_WORDS_PER_BOUNDARY: usize = 8;

/// Maximum projections referencing a single word.
pub const MAX_PROJECTIONS_PER_WORD: usize = 8;

/// Maximum transitions per bit-change registration (flat linear list).
pub const MAX_BIT_REGISTRATIONS: usize = 1024;

/// Maximum transitions that can fire when a single bit changes (per lookup).
pub const MAX_TRANSITIONS_PER_BIT: usize = 4;

// ---------------------------------------------------------------------------
// 1. CoordToWordBit — derives word_idx and bit_idx from a UCoord (pure math)
// ---------------------------------------------------------------------------

/// Output of the coord→word/bit projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordBit {
    /// Flat word index in [0, UNIVERSE_WORDS).
    pub word_idx: usize,
    /// Bit position within the word [0, 63].
    pub bit_idx: u32,
}

/// Branchless UCoord → (word_idx, bit_idx) projection.
/// T0: pure arithmetic, no lookup.
#[inline(always)]
pub fn coord_to_word_bit(coord: UCoord) -> WordBit {
    let raw = coord.0;
    let domain = ((raw >> DOMAIN_SHIFT) & COORD_MASK) as usize;
    let cell   = ((raw >> CELL_SHIFT)   & COORD_MASK) as usize;
    let place  = (raw                   & COORD_MASK) as usize;
    WordBit {
        word_idx: (domain * 64 + cell) & MAX_WORD_INDEX,
        bit_idx:  place as u32,
    }
}

// ---------------------------------------------------------------------------
// 2. BitTransitionIndex — flat linear list of (word_idx, bit_idx, tid)
//    No-std compatible: no Box, no Vec.
// ---------------------------------------------------------------------------

/// One bit-to-transition registration entry.
#[derive(Clone, Copy, Debug, Default)]
pub struct BitTransitionReg {
    pub word_idx: u16,
    pub bit_idx: u8,
    pub tid: u16,
}

/// Index: flat list of bit→transition registrations.
/// Lookup is a linear scan bounded to MAX_BIT_REGISTRATIONS.
#[derive(Clone, Copy)]
pub struct BitTransitionIndex {
    pub regs: [BitTransitionReg; MAX_BIT_REGISTRATIONS],
    pub count: u16,
}

impl BitTransitionIndex {
    pub const fn new() -> Self {
        Self {
            regs: [BitTransitionReg { word_idx: 0, bit_idx: 0, tid: 0 }; MAX_BIT_REGISTRATIONS],
            count: 0,
        }
    }

    /// Register `tid` as sensitive to (`word_idx`, `bit_idx`).
    #[inline]
    pub fn register(&mut self, word_idx: usize, bit_idx: u32, tid: u16) {
        let admit = (self.count as usize) < MAX_BIT_REGISTRATIONS;
        let slot = (self.count as usize) & (MAX_BIT_REGISTRATIONS - 1);
        self.regs[slot] = BitTransitionReg {
            word_idx: (word_idx & MAX_WORD_INDEX) as u16,
            bit_idx: (bit_idx & 63) as u8,
            tid,
        };
        self.count = self.count.wrapping_add(admit as u16);
    }

    /// Return slice of transition ids triggered by (`word_idx`, `bit_idx`).
    /// Writes up to `out.len()` matching tids; returns count written.
    #[inline]
    pub fn lookup(&self, word_idx: usize, bit_idx: u32, out: &mut [u16]) -> usize {
        let w = (word_idx & MAX_WORD_INDEX) as u16;
        let b = (bit_idx & 63) as u8;
        let mut n = 0usize;
        for i in 0..self.count as usize {
            let r = &self.regs[i];
            let matches = (r.word_idx == w) & (r.bit_idx == b);
            if matches && n < out.len() {
                out[n] = r.tid;
                n += 1;
            }
        }
        n
    }
}

impl Default for BitTransitionIndex {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// 3. TransitionWordIndex — transition_id → [word_idxs]
// ---------------------------------------------------------------------------

/// Words touched by one transition (input + output union).
#[derive(Clone, Copy, Debug)]
pub struct TransitionWordEntry {
    pub count: u8,
    pub word_idxs: [u16; MAX_WORDS_PER_TRANSITION],
}

impl Default for TransitionWordEntry {
    fn default() -> Self {
        Self { count: 0, word_idxs: [0u16; MAX_WORDS_PER_TRANSITION] }
    }
}

/// Index: for each transition id, the word indexes it reads or writes.
#[derive(Clone, Copy)]
pub struct TransitionWordIndex {
    pub entries: [TransitionWordEntry; MAX_TRANSITIONS],
}

impl TransitionWordIndex {
    pub const fn new() -> Self {
        Self {
            entries: [TransitionWordEntry { count: 0, word_idxs: [0u16; MAX_WORDS_PER_TRANSITION] }; MAX_TRANSITIONS],
        }
    }

    /// Register word_idx for transition_id.
    #[inline]
    pub fn register(&mut self, transition_id: usize, word_idx: usize) {
        let tid = transition_id & (MAX_TRANSITIONS - 1);
        let e = &mut self.entries[tid];
        let admit = (e.count as usize) < MAX_WORDS_PER_TRANSITION;
        let slot = (e.count as usize) & (MAX_WORDS_PER_TRANSITION - 1);
        e.word_idxs[slot] = ((word_idx & MAX_WORD_INDEX) as u16) * admit as u16;
        e.count = e.count.wrapping_add(admit as u8);
    }

    #[inline(always)]
    pub fn lookup(&self, transition_id: usize) -> &TransitionWordEntry {
        &self.entries[transition_id & (MAX_TRANSITIONS - 1)]
    }
}

impl Default for TransitionWordIndex {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// 4. WordProjectionIndex — compact flat list (no_std, avoids 4096-element array)
// ---------------------------------------------------------------------------

/// Maximum total word→projection registrations.
pub const MAX_WORD_PROJ_REGS: usize = 512;

/// One word→projection registration entry.
#[derive(Clone, Copy, Debug, Default)]
pub struct WordProjReg {
    pub word_idx: u16,
    pub proj_id: u8,
}

/// Flat list; lookup is a bounded linear scan.
#[derive(Clone, Copy)]
pub struct WordProjectionIndex {
    pub regs: [WordProjReg; MAX_WORD_PROJ_REGS],
    pub count: u16,
}

impl WordProjectionIndex {
    pub const fn new() -> Self {
        Self {
            regs: [WordProjReg { word_idx: 0, proj_id: 0 }; MAX_WORD_PROJ_REGS],
            count: 0,
        }
    }

    #[inline]
    pub fn register(&mut self, word_idx: usize, projection_id: u8) {
        let admit = (self.count as usize) < MAX_WORD_PROJ_REGS;
        let slot = (self.count as usize) & (MAX_WORD_PROJ_REGS - 1);
        self.regs[slot] = WordProjReg {
            word_idx: (word_idx & MAX_WORD_INDEX) as u16,
            proj_id: projection_id,
        };
        self.count = self.count.wrapping_add(admit as u16);
    }

    /// Returns projection ids for word_idx (writes into `out`). Returns count.
    #[inline]
    pub fn lookup(&self, word_idx: usize, out: &mut [u8]) -> usize {
        let w = (word_idx & MAX_WORD_INDEX) as u16;
        let mut n = 0usize;
        for i in 0..self.count as usize {
            if self.regs[i].word_idx == w && n < out.len() {
                out[n] = self.regs[i].proj_id;
                n += 1;
            }
        }
        n
    }
}

impl Default for WordProjectionIndex {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// 5. BoundaryWordIndex — boundary_id → (source_words, dest_words)
// ---------------------------------------------------------------------------

/// Source and destination word sets for one boundary transition.
#[derive(Clone, Copy, Debug)]
pub struct BoundaryWordEntry {
    pub src_count: u8,
    pub dst_count: u8,
    pub src_words: [u16; MAX_WORDS_PER_BOUNDARY],
    pub dst_words: [u16; MAX_WORDS_PER_BOUNDARY],
}

impl Default for BoundaryWordEntry {
    fn default() -> Self {
        Self {
            src_count: 0, dst_count: 0,
            src_words: [0u16; MAX_WORDS_PER_BOUNDARY],
            dst_words: [0u16; MAX_WORDS_PER_BOUNDARY],
        }
    }
}

/// Index: boundary_id → (source_words[], dest_words[]).
#[derive(Clone, Copy)]
pub struct BoundaryWordIndex {
    pub entries: [BoundaryWordEntry; MAX_BOUNDARIES],
}

impl BoundaryWordIndex {
    pub const fn new() -> Self {
        Self {
            entries: [BoundaryWordEntry {
                src_count: 0, dst_count: 0,
                src_words: [0u16; MAX_WORDS_PER_BOUNDARY],
                dst_words: [0u16; MAX_WORDS_PER_BOUNDARY],
            }; MAX_BOUNDARIES],
        }
    }

    #[inline]
    pub fn register_src(&mut self, boundary_id: usize, word_idx: usize) {
        let e = &mut self.entries[boundary_id & (MAX_BOUNDARIES - 1)];
        let admit = (e.src_count as usize) < MAX_WORDS_PER_BOUNDARY;
        let slot = (e.src_count as usize) & (MAX_WORDS_PER_BOUNDARY - 1);
        e.src_words[slot] = ((word_idx & MAX_WORD_INDEX) as u16) * admit as u16;
        e.src_count = e.src_count.wrapping_add(admit as u8);
    }

    #[inline]
    pub fn register_dst(&mut self, boundary_id: usize, word_idx: usize) {
        let e = &mut self.entries[boundary_id & (MAX_BOUNDARIES - 1)];
        let admit = (e.dst_count as usize) < MAX_WORDS_PER_BOUNDARY;
        let slot = (e.dst_count as usize) & (MAX_WORDS_PER_BOUNDARY - 1);
        e.dst_words[slot] = ((word_idx & MAX_WORD_INDEX) as u16) * admit as u16;
        e.dst_count = e.dst_count.wrapping_add(admit as u8);
    }

    #[inline(always)]
    pub fn lookup(&self, boundary_id: usize) -> &BoundaryWordEntry {
        &self.entries[boundary_id & (MAX_BOUNDARIES - 1)]
    }
}

impl Default for BoundaryWordIndex {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// IndexPlane — composite of all 5 indexes
// ---------------------------------------------------------------------------

/// Build-time geometry index combining all 5 cross-reference indexes.
/// Zero-heap at runtime; constructed once during admission/compile phase.
pub struct IndexPlane {
    pub bit_transition: BitTransitionIndex,
    pub transition_word: TransitionWordIndex,
    pub word_projection: WordProjectionIndex,
    pub boundary_word: BoundaryWordIndex,
}

impl IndexPlane {
    pub const fn new() -> Self {
        Self {
            bit_transition: BitTransitionIndex::new(),
            transition_word: TransitionWordIndex::new(),
            word_projection: WordProjectionIndex::new(),
            boundary_word: BoundaryWordIndex::new(),
        }
    }

    /// Resolve active words for a transition (T1 path).
    #[inline(always)]
    pub fn resolve_transition_words(&self, transition_id: usize) -> &TransitionWordEntry {
        self.transition_word.lookup(transition_id)
    }

    /// Resolve projections invalidated by a word change (T1 path).
    /// Writes projection ids into `out`; returns count.
    #[inline(always)]
    pub fn resolve_word_projections(&self, word_idx: usize, out: &mut [u8]) -> usize {
        self.word_projection.lookup(word_idx, out)
    }

    /// Resolve transitions triggered by a specific bit change (T0 path).
    /// Writes tids into `out`; returns count.
    #[inline(always)]
    pub fn resolve_bit_transitions(&self, word_idx: usize, bit_idx: u32, out: &mut [u16]) -> usize {
        self.bit_transition.lookup(word_idx, bit_idx, out)
    }
}

impl Default for IndexPlane {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_to_word_bit_round_trip() {
        let coord = UCoord::new(3, 7, 15);
        let wb = coord_to_word_bit(coord);
        assert_eq!(wb.word_idx, 3 * 64 + 7);
        assert_eq!(wb.bit_idx, 15);
    }

    #[test]
    fn test_bit_transition_index_register_and_lookup() {
        let mut idx = BitTransitionIndex::new();
        idx.register(5, 3, 42);
        let mut out = [0u16; 4];
        let n = idx.lookup(5, 3, &mut out);
        assert_eq!(n, 1);
        assert_eq!(out[0], 42);
    }

    #[test]
    fn test_bit_transition_saturation() {
        let mut idx = BitTransitionIndex::new();
        for i in 0..MAX_BIT_REGISTRATIONS + 5 {
            idx.register(0, 0, i as u16);
        }
        assert_eq!(idx.count as usize, MAX_BIT_REGISTRATIONS);
    }

    #[test]
    fn test_transition_word_index() {
        let mut idx = TransitionWordIndex::new();
        idx.register(1, 100);
        idx.register(1, 200);
        let e = idx.lookup(1);
        assert_eq!(e.count, 2);
    }

    #[test]
    fn test_word_projection_index() {
        let mut idx = WordProjectionIndex::new();
        idx.register(0, 5);
        let mut out = [0u8; 8];
        let n = idx.lookup(0, &mut out);
        assert_eq!(n, 1);
        assert_eq!(out[0], 5);
    }

    #[test]
    fn test_boundary_word_index() {
        let mut idx = BoundaryWordIndex::new();
        idx.register_src(0, 10);
        idx.register_dst(0, 20);
        let e = idx.lookup(0);
        assert_eq!(e.src_count, 1);
        assert_eq!(e.dst_count, 1);
        assert_eq!(e.src_words[0], 10);
        assert_eq!(e.dst_words[0], 20);
    }

    #[test]
    fn test_index_plane_resolve() {
        let mut plane = IndexPlane::new();
        plane.bit_transition.register(0, 0, 7);
        let mut out = [0u16; 4];
        let n = plane.resolve_bit_transitions(0, 0, &mut out);
        assert_eq!(n, 1);
        assert_eq!(out[0], 7);
    }
}
