//! # Universe64 Contract: AdmissionLifecycle (MaskBank + GeometryCompiler)
//! Plane: D (build-time admission; runtime read-only).
//! Tier: Admission-time only (T3-class; not on hot path).
//! Scope: bounded static arrays; zero heap at runtime.
//! Geometry: MaskBank holds admitted masks; TransitionRegistry holds admitted transitions.
//! Delta: none produced — structural registration only.
//!
//! # Timing contract
//! - **Admission budget:** T3-class (admission happens outside hot loop).
//! - **Runtime read budget:** T0 per lookup.
//! - **Max heap allocations:** 0 at runtime.
//!
//! # Admissibility
//! Admissible_T1: YES (all runtime paths are bounded array reads).
//! CC=1: Absolute branchless logic in runtime paths.

use super::constants::{MAX_WORD_INDEX};
use super::masks::{CellMask, DomainMask, UniverseMask};
use super::scratch::UScope;
use super::index_plane::{IndexPlane, MAX_TRANSITIONS, MAX_BOUNDARIES};

// ---------------------------------------------------------------------------
// Capacity bounds
// ---------------------------------------------------------------------------

/// Maximum admitted masks in MaskBank.
pub const MASK_BANK_CAPACITY: usize = 128;

/// Maximum admitted transitions in TransitionRegistry.
pub const TRANSITION_REGISTRY_CAPACITY: usize = MAX_TRANSITIONS;

/// Maximum boundary entries.
pub const BOUNDARY_REGISTRY_CAPACITY: usize = MAX_BOUNDARIES;

// ---------------------------------------------------------------------------
// MaskKind — variant tag for admitted masks
// ---------------------------------------------------------------------------

/// Variant of an admitted mask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MaskKind {
    Cell = 0,
    Domain = 1,
    Universe = 2,
}

/// An admitted mask entry in the MaskBank.
#[derive(Clone, Copy, Debug)]
pub struct MaskEntry {
    /// Opaque mask id (assigned at registration).
    pub id: u8,
    /// Which scope this mask applies to.
    pub kind: MaskKind,
    /// The raw 64-bit bitmask for Cell/Domain. For Universe masks: primary word mask.
    pub bits: u64,
    /// For Universe masks: which word_idx this mask covers (0 = not applicable).
    pub word_idx: u16,
    /// True if this mask has been validated.
    pub valid: bool,
}

// ---------------------------------------------------------------------------
// MaskBank — bounded registry of admitted masks
// ---------------------------------------------------------------------------

/// Bounded registry of admitted masks.
/// Admission validates scope & coordinate law; runtime paths do not re-validate.
#[derive(Clone, Copy)]
pub struct MaskBank {
    entries: [MaskEntry; MASK_BANK_CAPACITY],
    count: u8,
}

impl MaskBank {
    pub const fn new() -> Self {
        let blank = MaskEntry { id: 0, kind: MaskKind::Cell, bits: 0, word_idx: 0, valid: false };
        Self { entries: [blank; MASK_BANK_CAPACITY], count: 0 }
    }

    /// Admit a CellMask. Returns assigned mask_id, or 255 if full.
    pub fn admit_cell(&mut self, mask: CellMask, word_idx: usize) -> u8 {
        let admit = (self.count as usize) < MASK_BANK_CAPACITY;
        let id = self.count * admit as u8;
        let slot = (self.count as usize) & (MASK_BANK_CAPACITY - 1);
        self.entries[slot] = MaskEntry {
            id,
            kind: MaskKind::Cell,
            bits: mask.0,
            word_idx: (word_idx & MAX_WORD_INDEX) as u16,
            valid: admit,
        };
        self.count = self.count.wrapping_add(admit as u8);
        id | ((!admit as u8) * 255)
    }

    /// Admit a DomainMask. Returns assigned mask_id, or 255 if full.
    pub fn admit_domain(&mut self, mask: DomainMask, domain: usize) -> u8 {
        let word_base = (domain & 63) * 64;
        let admit = (self.count as usize) < MASK_BANK_CAPACITY;
        let id = self.count * admit as u8;
        let slot = (self.count as usize) & (MASK_BANK_CAPACITY - 1);
        self.entries[slot] = MaskEntry {
            id,
            kind: MaskKind::Domain,
            bits: mask.0[0],
            word_idx: word_base as u16,
            valid: admit,
        };
        self.count = self.count.wrapping_add(admit as u8);
        id | ((!admit as u8) * 255)
    }

    /// Look up a mask entry by id.
    #[inline(always)]
    pub fn get(&self, id: u8) -> &MaskEntry {
        &self.entries[id as usize & (MASK_BANK_CAPACITY - 1)]
    }

    /// Number of admitted masks.
    #[inline(always)]
    pub const fn len(&self) -> usize { self.count as usize }

    /// Returns true if no masks have been admitted.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool { self.count == 0 }
}

impl Default for MaskBank {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// TransitionEntry — an admitted transition
// ---------------------------------------------------------------------------

/// A registered, validated transition.
#[derive(Clone, Copy, Debug)]
pub struct TransitionEntry {
    /// Opaque transition id.
    pub id: u16,
    /// Input (prerequisite) mask id.
    pub input_mask_id: u8,
    /// Output (post-fire) mask id.
    pub output_mask_id: u8,
    /// Scope of the transition.
    pub scope: UScope,
    /// Number of valid active word indexes.
    pub active_word_count: u8,
    /// Active word indexes touched by this transition.
    pub active_words: [u16; 16],
    /// Validated at admission time.
    pub valid: bool,
}

impl Default for TransitionEntry {
    fn default() -> Self {
        Self {
            id: 0,
            input_mask_id: 0,
            output_mask_id: 0,
            scope: UScope::Cell,
            active_word_count: 0,
            active_words: [0u16; 16],
            valid: false,
        }
    }
}

// ---------------------------------------------------------------------------
// TransitionRegistry
// ---------------------------------------------------------------------------

/// Bounded registry of admitted transitions.
#[derive(Clone, Copy)]
pub struct TransitionRegistry {
    entries: [TransitionEntry; TRANSITION_REGISTRY_CAPACITY],
    count: u16,
}

impl TransitionRegistry {
    pub const fn new() -> Self {
        Self {
            entries: [TransitionEntry {
                id: 0, input_mask_id: 0, output_mask_id: 0,
                scope: UScope::Cell, active_word_count: 0,
                active_words: [0u16; 16], valid: false,
            }; TRANSITION_REGISTRY_CAPACITY],
            count: 0,
        }
    }

    /// Admit a new transition. Returns assigned id, or u16::MAX if full.
    pub fn admit(
        &mut self,
        input_mask_id: u8,
        output_mask_id: u8,
        scope: UScope,
        active_words: &[u16],
    ) -> u16 {
        let admit = (self.count as usize) < TRANSITION_REGISTRY_CAPACITY;
        let id = self.count * admit as u16;
        let slot = (self.count as usize) & (TRANSITION_REGISTRY_CAPACITY - 1);
        let mut entry = TransitionEntry {
            id,
            input_mask_id,
            output_mask_id,
            scope,
            active_word_count: (active_words.len().min(16)) as u8,
            active_words: [0u16; 16],
            valid: admit,
        };
        let n = active_words.len().min(16);
        let mut i = 0;
        while i < n { entry.active_words[i] = active_words[i]; i += 1; }
        self.entries[slot] = entry;
        self.count = self.count.wrapping_add(admit as u16);
        id | ((!admit as u16) * u16::MAX)
    }

    #[inline(always)]
    pub fn get(&self, id: u16) -> &TransitionEntry {
        &self.entries[id as usize & (TRANSITION_REGISTRY_CAPACITY - 1)]
    }

    #[inline(always)]
    pub const fn len(&self) -> usize { self.count as usize }

    /// Returns true if no transitions have been admitted.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool { self.count == 0 }
}

impl Default for TransitionRegistry {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// MaskVerifier — validates a mask at admission time
// ---------------------------------------------------------------------------

/// Outcome of mask verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VerifyResult {
    Ok = 0,
    OutOfBounds = 1,
    ZeroMask = 2,
    ScopeConflict = 3,
}

/// Verify a cell-scope mask (branchless — returns result code).
pub fn verify_cell_mask(mask: CellMask, word_idx: usize) -> VerifyResult {
    let in_bounds = ((word_idx <= MAX_WORD_INDEX) as u8).wrapping_mul(1);
    let nonzero = ((mask.0 != 0) as u8).wrapping_mul(1);
    let ok = in_bounds & nonzero;
    // Branchless select: ok → Ok, !in_bounds → OutOfBounds, else ZeroMask
    let oob = VerifyResult::OutOfBounds as u8 * (1 - in_bounds);
    let zero = VerifyResult::ZeroMask as u8 * in_bounds * (1 - nonzero);
    let code = oob | zero; // exactly one is non-zero when not ok
    // ok → 0, not-ok → non-zero code
    let _ = ok;
    match code {
        0 => VerifyResult::Ok,
        1 => VerifyResult::OutOfBounds,
        2 => VerifyResult::ZeroMask,
        _ => VerifyResult::ScopeConflict,
    }
}

/// Verify a universe-scope mask (word_idx must be < UNIVERSE_WORDS).
pub fn verify_universe_mask(mask: &UniverseMask) -> VerifyResult {
    // Universe masks are always valid structurally if they contain at least one set bit.
    let any_set: u64 = mask.0.iter().fold(0u64, |acc, &w| acc | w);
    let nonzero = (any_set != 0) as u8;
    match nonzero {
        0 => VerifyResult::ZeroMask,
        _ => VerifyResult::Ok,
    }
}

// ---------------------------------------------------------------------------
// TransitionVerifier
// ---------------------------------------------------------------------------

/// Verify a transition entry against the MaskBank.
pub fn verify_transition(entry: &TransitionEntry, bank: &MaskBank) -> VerifyResult {
    let in_mask = bank.get(entry.input_mask_id);
    let out_mask = bank.get(entry.output_mask_id);
    let both_valid = (in_mask.valid & out_mask.valid) as u8;
    let has_words = (entry.active_word_count > 0) as u8;
    let ok = both_valid & has_words;
    match ok {
        0 => VerifyResult::ScopeConflict,
        _ => VerifyResult::Ok,
    }
}

// ---------------------------------------------------------------------------
// GeometryCompiler — builds IndexPlane from registered masks/transitions
// ---------------------------------------------------------------------------

/// Compiles geometry indexes from MaskBank + TransitionRegistry.
/// Called once at admission time; result is the immutable IndexPlane.
pub struct GeometryCompiler {
    pub mask_bank: MaskBank,
    pub transition_registry: TransitionRegistry,
}

impl GeometryCompiler {
    pub fn new() -> Self {
        Self {
            mask_bank: MaskBank::new(),
            transition_registry: TransitionRegistry::new(),
        }
    }

    /// Compile all registered transitions into an IndexPlane.
    pub fn compile(&self) -> IndexPlane {
        let mut plane = IndexPlane::new();
        let n = self.transition_registry.len();
        for i in 0..n {
            let t = self.transition_registry.get(i as u16);
            if !t.valid { continue; }
            let tid = t.id as usize;
            let nw = t.active_word_count as usize;
            for j in 0..nw {
                let widx = t.active_words[j] as usize;
                plane.transition_word.register(tid, widx);
                // Register all 64 bit positions for this word as sensitive to this transition.
                for bit in 0u32..64 {
                    plane.bit_transition.register(widx, bit, t.id);
                }
            }
        }
        plane
    }
}

impl Default for GeometryCompiler {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::masks::CellMask;

    #[test]
    fn test_mask_bank_admit_cell() {
        let mut bank = MaskBank::new();
        let id = bank.admit_cell(CellMask(0xFF), 5);
        assert_eq!(id, 0);
        assert_eq!(bank.len(), 1);
        let e = bank.get(0);
        assert!(e.valid);
        assert_eq!(e.bits, 0xFF);
    }

    #[test]
    fn test_transition_registry_admit() {
        let mut reg = TransitionRegistry::new();
        let id = reg.admit(0, 1, UScope::Sparse, &[10, 20]);
        assert_eq!(id, 0);
        assert_eq!(reg.len(), 1);
        let e = reg.get(0);
        assert!(e.valid);
        assert_eq!(e.active_words[0], 10);
    }

    #[test]
    fn test_verify_cell_mask_ok() {
        let r = verify_cell_mask(CellMask(0b1010), 0);
        assert_eq!(r, VerifyResult::Ok);
    }

    #[test]
    fn test_verify_cell_mask_zero() {
        let r = verify_cell_mask(CellMask(0), 0);
        assert_eq!(r, VerifyResult::ZeroMask);
    }

    #[test]
    fn test_geometry_compiler_compile() {
        let mut gc = GeometryCompiler::new();
        gc.mask_bank.admit_cell(CellMask(0xFF), 10);
        gc.mask_bank.admit_cell(CellMask(0x0F), 10);
        gc.transition_registry.admit(0, 1, UScope::Cell, &[10]);
        let plane = gc.compile();
        let e = plane.transition_word.lookup(0);
        assert_eq!(e.count, 1);
    }
}
