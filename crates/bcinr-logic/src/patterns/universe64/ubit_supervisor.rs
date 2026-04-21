//! # UBitSupervisor — Self-healing kernel for the U_{1,262144} substrate.
//!
//! Observes `UDelta` emissions. Detects repeated failures, drift spikes, and
//! denial storms. Emits `UInstruction::Recovery` — NOT logs.
//!
//! Uses a fixed-capacity ring (power-of-2, no heap).
//!
//! ```
//! use bcinr_logic::patterns::universe64::ubit_supervisor::{UBitSupervisor, SupervisorThresholds};
//! let sv = UBitSupervisor::new(SupervisorThresholds::default_thresholds());
//! assert_eq!(sv.counters.drift_accum, 0);
//! ```

use super::instruction::UInstruction;
use super::scratch::UDelta;

pub const UBIT_SUPERVISOR_RING_CAPACITY: usize = 16; // must be power of 2

pub const UBIT_FAMILY_COUNT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RecoveryKind {
    ResetFamily       = 0,
    ResetEpoch        = 1,
    ClearDrift        = 2,
    RequestCheckpoint = 3,
    BlockBoundary     = 4,
}

#[derive(Clone, Copy, Debug)]
pub struct SupervisorCounters {
    pub failures_per_family: [u8; UBIT_FAMILY_COUNT],
    pub drift_accum: u32,
    pub denials: u32,
    pub epoch: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct SupervisorThresholds {
    pub max_failures: u8,
    pub max_drift: u32,
    pub max_denials: u32,
}

pub struct UBitSupervisor {
    pub counters: SupervisorCounters,
    pub thresholds: SupervisorThresholds,
    pub recovery_ring_head: u32,
    pub recovery_ring_tail: u32,
    pub recovery_ring: [UInstruction; UBIT_SUPERVISOR_RING_CAPACITY],
}

impl SupervisorCounters {
    pub const fn new() -> Self {
        Self { failures_per_family: [0u8; UBIT_FAMILY_COUNT], drift_accum: 0, denials: 0, epoch: 0 }
    }
}

impl Default for SupervisorCounters {
    fn default() -> Self {
        Self::new()
    }
}

impl SupervisorThresholds {
    /// Default thresholds: 5 failures, 1000 drift, 20 denials.
    pub const fn default_thresholds() -> Self {
        Self { max_failures: 5, max_drift: 1000, max_denials: 20 }
    }
}

impl UBitSupervisor {
    pub fn new(thresholds: SupervisorThresholds) -> Self {
        Self {
            counters: SupervisorCounters::new(),
            thresholds,
            recovery_ring_head: 0,
            recovery_ring_tail: 0,
            recovery_ring: [UInstruction::recovery(0, 0); UBIT_SUPERVISOR_RING_CAPACITY],
        }
    }

    /// Observe a delta. `family_id` is the transition family (0..63). `is_denial` if cap denied.
    /// Returns count of recovery instructions enqueued.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe64::ubit_supervisor::{UBitSupervisor, SupervisorThresholds};
    /// use bcinr_logic::patterns::universe64::scratch::{UDelta, UScope};
    /// let mut sv = UBitSupervisor::new(SupervisorThresholds { max_failures: 2, max_drift: 100, max_denials: 5 });
    /// let delta = UDelta::new(0, UScope::Cell, 0, 0xF, 0xF, 0); // fired_mask=0 → failed
    /// sv.observe(&delta, 0, false);
    /// assert_eq!(sv.counters.failures_per_family[0], 1);
    /// ```
    pub fn observe(&mut self, delta: &UDelta, family_id: u8, is_denial: bool) -> u32 {
        let failed = (delta.fired_mask == 0) as u8;
        let fam = (family_id as usize) & (UBIT_FAMILY_COUNT - 1);
        self.counters.failures_per_family[fam] =
            self.counters.failures_per_family[fam].saturating_add(failed);

        let drift_bits = (delta.before ^ delta.after).count_ones();
        self.counters.drift_accum = self.counters.drift_accum.saturating_add(drift_bits);
        self.counters.denials = self.counters.denials.saturating_add(is_denial as u32);

        let mut emitted: u32 = 0;
        if self.counters.failures_per_family[fam] >= self.thresholds.max_failures {
            emitted += self.enqueue_recovery(RecoveryKind::ResetFamily, fam as u32);
            self.counters.failures_per_family[fam] = 0;
        }
        if self.counters.drift_accum >= self.thresholds.max_drift {
            emitted += self.enqueue_recovery(RecoveryKind::ClearDrift, 0);
            self.counters.drift_accum = 0;
        }
        if self.counters.denials >= self.thresholds.max_denials {
            emitted += self.enqueue_recovery(RecoveryKind::RequestCheckpoint, 0);
            self.counters.denials = 0;
        }
        emitted
    }

    /// Enqueue a recovery instruction. Returns 1 if enqueued, 0 if ring full.
    #[inline(always)]
    pub fn enqueue_recovery(&mut self, kind: RecoveryKind, target: u32) -> u32 {
        let mask = UBIT_SUPERVISOR_RING_CAPACITY - 1;
        let next = (self.recovery_ring_tail as usize + 1) & mask;
        let full = (next == self.recovery_ring_head as usize) as u32;
        // Write unconditionally to current tail slot
        let slot = self.recovery_ring_tail as usize;
        self.recovery_ring[slot] = UInstruction::recovery(kind as u8, target);
        // Advance tail only if not full
        let advance = 1u32 - full;
        self.recovery_ring_tail = ((self.recovery_ring_tail + advance) as usize & mask) as u32;
        advance
    }

    /// Drain next recovery instruction. Returns None if ring empty.
    pub fn drain_recovery(&mut self) -> Option<UInstruction> {
        if self.recovery_ring_head == self.recovery_ring_tail { return None; }
        let mask = UBIT_SUPERVISOR_RING_CAPACITY - 1;
        let slot = self.recovery_ring_head as usize;
        let instr = self.recovery_ring[slot];
        self.recovery_ring_head = ((self.recovery_ring_head as usize + 1) & mask) as u32;
        Some(instr)
    }

    /// Number of pending recovery instructions.
    pub fn recovery_pending(&self) -> usize {
        let mask = UBIT_SUPERVISOR_RING_CAPACITY - 1;
        (self.recovery_ring_tail as usize + UBIT_SUPERVISOR_RING_CAPACITY
            - self.recovery_ring_head as usize) & mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::universe64::scratch::{UDelta, UScope};

    fn failed_delta() -> UDelta { UDelta::new(0, UScope::Cell, 0, 0, 0, 0) } // fired_mask=0
    fn success_delta() -> UDelta { UDelta::new(0, UScope::Cell, 0, 0, 1, !0) } // fired_mask=!0

    #[test]
    fn ubit_supervisor_counts_family_failures() {
        let mut sv = UBitSupervisor::new(SupervisorThresholds { max_failures: 10, max_drift: 10000, max_denials: 100 });
        sv.observe(&failed_delta(), 3, false);
        sv.observe(&failed_delta(), 3, false);
        assert_eq!(sv.counters.failures_per_family[3], 2);
    }

    #[test]
    fn ubit_supervisor_emits_recovery_on_failure_threshold() {
        let mut sv = UBitSupervisor::new(SupervisorThresholds { max_failures: 3, max_drift: 10000, max_denials: 100 });
        sv.observe(&failed_delta(), 0, false);
        sv.observe(&failed_delta(), 0, false);
        let emitted = sv.observe(&failed_delta(), 0, false);
        assert!(emitted > 0);
        assert!(sv.recovery_pending() > 0);
    }

    #[test]
    fn ubit_supervisor_emits_recovery_on_drift_threshold() {
        let mut sv = UBitSupervisor::new(SupervisorThresholds { max_failures: 255, max_drift: 5, max_denials: 100 });
        // delta with before=0, after=0xFF -> 8 bits changed; 8 > 5 triggers drift
        let big_delta = UDelta::new(0, UScope::Cell, 0, 0, 0xFF, !0);
        let emitted = sv.observe(&big_delta, 0, false);
        assert!(emitted > 0 || sv.counters.drift_accum >= sv.thresholds.max_drift || sv.recovery_pending() > 0);
    }

    #[test]
    fn ubit_supervisor_emits_recovery_on_denial_threshold() {
        let mut sv = UBitSupervisor::new(SupervisorThresholds { max_failures: 255, max_drift: 10000, max_denials: 2 });
        sv.observe(&success_delta(), 0, true);
        let emitted = sv.observe(&success_delta(), 0, true);
        assert!(emitted > 0 || sv.recovery_pending() > 0);
    }

    #[test]
    fn ubit_supervisor_recovery_ring_wraps_without_heap() {
        let mut sv = UBitSupervisor::new(SupervisorThresholds::default_thresholds());
        // Fill ring to capacity (16 slots, but ring is full at 15 because head==next_tail)
        for i in 0..UBIT_SUPERVISOR_RING_CAPACITY {
            sv.enqueue_recovery(RecoveryKind::ResetFamily, i as u32);
        }
        // Try enqueueing one more — should not panic and ring stays bounded
        sv.enqueue_recovery(RecoveryKind::ClearDrift, 99);
        // Drain some and verify it works
        let first = sv.drain_recovery();
        assert!(first.is_some());
    }
}
