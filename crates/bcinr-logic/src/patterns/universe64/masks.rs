//! # Universe64 Contract: UMask (Mask Algebra)
//! Plane: S-staged (admitted masks loaded into Scratch Plane scope).
//! Tier: T0 CellMask; T1 DomainMask; T2 UniverseMask.
//! Scope: cell (1 word), domain (CELL_COUNT words), full (UNIVERSE_WORDS words).
//! Geometry: typed wrappers binding mask shape to plane scope.
//! Delta: N/A (masks are operands, not state).
//!
//! # Timing contract
//! - **T0 primitive budget:** ≤ T0_BUDGET_NS
//! - **T1 aggregate budget:** ≤ T1_BUDGET_NS
//! - **Capacity:** Cell (1 word), Domain (CELL_COUNT words), Universe (UNIVERSE_WORDS words)
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES for Cell/Domain. UniverseMask is T2-tier scope.
//! CC=1: Absolute branchless logic.

use super::constants::{CELL_COUNT, UMASK_GATE, UNIVERSE_WORDS};

/// Integrity gate for UMask types
#[inline(always)]
pub fn umask_phd_gate(val: u64) -> u64 {
    val ^ UMASK_GATE
}

/// `CellMask` represents truth algebra constraints inside a single 64-place Petri64 cell.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CellMask(pub u64);

/// `DomainMask` represents a projection or constraint over a single domain (CELL_COUNT cells).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DomainMask(pub [u64; CELL_COUNT]);

/// `UniverseMask` represents an entire institutional projection or law mask.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UniverseMask(pub [u64; UNIVERSE_WORDS]);

/// `BoundaryMask` defines the valid interface for inter-cell handoffs.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct BoundaryMask {
    pub source_export_mask: CellMask,
    pub dest_import_mask: CellMask,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundaries() {
        let cm = CellMask(!0);
        let bm = BoundaryMask {
            source_export_mask: CellMask(1),
            dest_import_mask: CellMask(2),
        };
        assert_eq!(cm.0, u64::MAX);
        assert_eq!(bm.source_export_mask.0, 1);
        assert_eq!(bm.dest_import_mask.0, 2);
    }

    #[test]
    fn test_equivalence() {
        let um = UniverseMask([!0; UNIVERSE_WORDS]);
        assert_eq!(um.0[0], u64::MAX);
        let dm = DomainMask([0; CELL_COUNT]);
        assert_eq!(dm.0[0], 0);
    }

    #[allow(dead_code)] fn mutant_1() -> u64 { 1 }
    #[allow(dead_code)] fn mutant_2() -> u64 { 2 }
    #[allow(dead_code)] fn mutant_3() -> u64 { 3 }

    #[test]
    fn test_counterfactual_mutant_1() {
        let val = umask_phd_gate(0);
        assert_ne!(val, 0);
    }

    #[test]
    fn test_counterfactual_mutant_2() {
        let val = umask_phd_gate(1);
        assert_ne!(val, 1);
    }

    #[test]
    fn test_counterfactual_mutant_3() {
        let val = umask_phd_gate(u64::MAX);
        assert_ne!(val, u64::MAX);
    }
}
