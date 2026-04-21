//! # Universe64 Contract: UInstruction (Instruction Decoder)
//! Plane: S-ingress — tiny copyable operators, never carry data payloads.
//! Tier: T0 decode; T1 admission/commit when fired against the data plane.
//! Scope: per-instruction `UScope` declares cell/sparse/domain/full reach.
//! Geometry: 16-byte fixed record; references admitted masks/transitions by id.
//! Delta: not produced by decode; produced by transition kernel after admission.
//!
//! # Timing contract
//! - **T0 primitive budget:** ≤ T0_BUDGET_NS (decode + dispatch class)
//! - **T1 aggregate budget:** ≤ T1_BUDGET_NS (admission + commit)
//! - **Capacity:** Admitted instruction set; ids opaque to runtime
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Fixed-size, branchless decode.
//! CC=1: Absolute branchless logic.

use super::constants::{T0_BUDGET_NS, T1_BUDGET_NS, T2_BUDGET_NS, T3_BUDGET_NS, UINSTRUCTION_GATE};
use super::scratch::UScope;

/// Integrity gate for UInstruction
#[inline(always)]
pub fn uinstruction_phd_gate(val: u64) -> u64 {
    val ^ UINSTRUCTION_GATE
}

/// Execution tier — bounds the WCET of an admitted instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UTier {
    T0 = 0,
    T1 = 1,
    T2 = 2,
    T3 = 3,
    T4 = 4,
}

impl UTier {
    /// Wall-clock budget in nanoseconds for this tier (`T4` returns `u64::MAX`).
    #[inline(always)]
    pub const fn budget_ns(self) -> u64 {
        match self {
            UTier::T0 => T0_BUDGET_NS,
            UTier::T1 => T1_BUDGET_NS,
            UTier::T2 => T2_BUDGET_NS,
            UTier::T3 => T3_BUDGET_NS,
            UTier::T4 => u64::MAX,
        }
    }
}

/// Instruction kind — what motion is being proposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UInstrKind {
    /// Propose coordinate true.
    SetBit = 0,
    /// Propose coordinate false.
    ClearBit = 1,
    /// Apply admitted transition by id.
    ApplyTransition = 2,
    /// Apply bounded sparse mask.
    ApplySparseMask = 3,
    /// Boundary handoff (source→dest).
    ApplyBoundary = 4,
    /// Conformance distance computation.
    Conformance = 5,
    /// Receipt mix (no state mutation).
    Receipt = 6,
    /// Projection update from latest delta.
    Projection = 7,
    /// Deterministic RL update.
    RlUpdate = 8,
}

/// A small, copyable operator. References masks/transitions by opaque id.
/// Fixed shape — must fit cleanly within Scratch Plane staging.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct UInstruction {
    pub kind: UInstrKind,
    pub scope: UScope,
    pub tier: UTier,
    pub _pad: u8,
    /// Opaque admitted-object id (transition / mask / projection).
    pub object_id: u32,
    /// Coordinate or word index, packed; interpretation depends on `kind`.
    pub coord_or_word: u32,
    /// Receipt tag — caller-supplied causal token.
    pub receipt_tag: u32,
}

impl UInstruction {
    /// Construct an instruction with all fields explicit.
    #[inline(always)]
    pub const fn new(
        kind: UInstrKind,
        scope: UScope,
        tier: UTier,
        object_id: u32,
        coord_or_word: u32,
        receipt_tag: u32,
    ) -> Self {
        Self { kind, scope, tier, _pad: 0, object_id, coord_or_word, receipt_tag }
    }

    /// True if this instruction proposes state motion (vs. read-only ops).
    #[inline(always)]
    pub const fn proposes_motion(self) -> bool {
        matches!(
            self.kind,
            UInstrKind::SetBit
                | UInstrKind::ClearBit
                | UInstrKind::ApplyTransition
                | UInstrKind::ApplySparseMask
                | UInstrKind::ApplyBoundary
        )
    }

    /// Wall-clock budget for this instruction's tier.
    #[inline(always)]
    pub const fn budget_ns(self) -> u64 {
        self.tier.budget_ns()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_budgets_monotonic() {
        assert!(UTier::T0.budget_ns() < UTier::T1.budget_ns());
        assert!(UTier::T1.budget_ns() < UTier::T2.budget_ns());
        assert!(UTier::T2.budget_ns() < UTier::T3.budget_ns());
        assert_eq!(UTier::T4.budget_ns(), u64::MAX);
    }

    #[test]
    fn test_proposes_motion_classification() {
        let m = UInstruction::new(
            UInstrKind::SetBit,
            UScope::Cell,
            UTier::T0,
            0,
            0,
            0,
        );
        assert!(m.proposes_motion());

        let r = UInstruction::new(
            UInstrKind::Receipt,
            UScope::Cell,
            UTier::T0,
            0,
            0,
            0,
        );
        assert!(!r.proposes_motion());
    }

    #[test]
    fn test_uinstruction_phd_gate() {
        assert_ne!(uinstruction_phd_gate(0), 0);
        assert_ne!(uinstruction_phd_gate(1), 1);
        assert_ne!(uinstruction_phd_gate(u64::MAX), u64::MAX);
    }
}
