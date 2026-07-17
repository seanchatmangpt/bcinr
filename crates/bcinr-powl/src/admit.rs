//! admit — O(1) phase admission via branchless DPAG.
//!
//! [`admit`] maps an [`AdmissionContext`] bitfield to a [`ProcessTopology`]
//! without any runtime branches. It uses dynamic thresholds and sign-mask
//! multiplexers to route topologies dynamically.
//!
//! # AdmissionContext bit layout
//!
//! | Bits  | Field          | Range  | Meaning                              |
//! |-------|----------------|--------|--------------------------------------|
//! | 0..3  | tenant_class   | 0..3   | 0=free, 1=standard, 2=enterprise, 3=sovereign |
//! | 4..7  | urgency_tier   | 0..15  | Higher = more urgent                 |
//! | 8..11 | resource_load  | 0..15  | Higher = more saturated              |
//! | 12    | has_sla_token  | 0/1    |                                      |
//! | 15    | is_compensating| 0/1    |                                      |

#![forbid(unsafe_code)]

use core::sync::atomic::{AtomicU64, Ordering};

/// Packed admission context word. See module-level docs for bit layout.
pub type AdmissionContext = u64;

/// Routing topology assigned to an admitted process.
///
/// Variants are ordered by descending priority so that numeric comparison
/// `topology as u8` gives a sensible ordering (0 = highest priority).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessTopology {
    /// Highest priority lane — enterprise/sovereign tenants with SLA token and
    /// sufficient urgency.
    Priority = 0,
    /// Normal execution lane.
    Standard = 1,
    /// Best-effort, low-urgency lane.
    Background = 2,
    /// Isolated lane for overloaded or untrusted contexts.
    Quarantine = 3,
}

/// Runtime admission thresholds adjusted dynamically by the autonomic MAPE-K loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionParameters {
    /// Load level above which all processes are routed to Quarantine (0..15).
    pub load_saturation_threshold: u64,
    /// Minimum urgency required to enter the Priority lane (0..15).
    pub urgency_priority_threshold: u64,
    /// Minimum tenant class required to enter the Priority lane (0..3).
    pub tenant_class_priority_min: u64,
    /// Minimum tenant class required to enter the Standard lane (0..3).
    pub tenant_class_standard_min: u64,
    /// 1 if SLA token is strictly required for the Priority lane; 0 otherwise.
    pub sla_required: u64,
}

/// Thread-safe atomic wrapper for AdmissionParameters.
pub struct AtomicAdmissionParameters {
    pub load_saturation_threshold: AtomicU64,
    pub urgency_priority_threshold: AtomicU64,
    pub tenant_class_priority_min: AtomicU64,
    pub tenant_class_standard_min: AtomicU64,
    pub sla_required: AtomicU64,
}

impl AtomicAdmissionParameters {
    /// Create a new AtomicAdmissionParameters initialized with default values.
    pub const fn new(default: AdmissionParameters) -> Self {
        Self {
            load_saturation_threshold: AtomicU64::new(default.load_saturation_threshold),
            urgency_priority_threshold: AtomicU64::new(default.urgency_priority_threshold),
            tenant_class_priority_min: AtomicU64::new(default.tenant_class_priority_min),
            tenant_class_standard_min: AtomicU64::new(default.tenant_class_standard_min),
            sla_required: AtomicU64::new(default.sla_required),
        }
    }

    /// Atomically load the parameter set.
    pub fn load(&self) -> AdmissionParameters {
        AdmissionParameters {
            load_saturation_threshold: self.load_saturation_threshold.load(Ordering::Acquire),
            urgency_priority_threshold: self.urgency_priority_threshold.load(Ordering::Acquire),
            tenant_class_priority_min: self.tenant_class_priority_min.load(Ordering::Acquire),
            tenant_class_standard_min: self.tenant_class_standard_min.load(Ordering::Acquire),
            sla_required: self.sla_required.load(Ordering::Acquire),
        }
    }

    /// Atomically store the parameter set.
    pub fn store(&self, params: AdmissionParameters) {
        self.load_saturation_threshold.store(params.load_saturation_threshold, Ordering::Release);
        self.urgency_priority_threshold.store(params.urgency_priority_threshold, Ordering::Release);
        self.tenant_class_priority_min.store(params.tenant_class_priority_min, Ordering::Release);
        self.tenant_class_standard_min.store(params.tenant_class_standard_min, Ordering::Release);
        self.sla_required.store(params.sla_required, Ordering::Release);
    }
}

/// Default admission parameters matching the behavior of the legacy static LUT.
pub const DEFAULT_PARAMETERS: AdmissionParameters = AdmissionParameters {
    load_saturation_threshold: 15,
    urgency_priority_threshold: 8,
    tenant_class_priority_min: 2,
    tenant_class_standard_min: 1,
    sla_required: 1,
};

/// Global static admission parameters, initialized to DEFAULT_PARAMETERS.
pub static GLOBAL_ADMISSION_PARAMETERS: AtomicAdmissionParameters =
    AtomicAdmissionParameters::new(DEFAULT_PARAMETERS);

// ---------------------------------------------------------------------------
// Field extraction helpers (const-friendly)
// ---------------------------------------------------------------------------

/// Extract `tenant_class` from bits [0..3].
#[inline(always)]
pub const fn tenant_class(ctx: u64) -> u8 {
    (ctx & 0xF) as u8
}

/// Extract `urgency_tier` from bits [4..7].
#[inline(always)]
pub const fn urgency_tier(ctx: u64) -> u8 {
    ((ctx >> 4) & 0xF) as u8
}

/// Extract `resource_load` from bits [8..11].
#[inline(always)]
pub const fn resource_load(ctx: u64) -> u8 {
    ((ctx >> 8) & 0xF) as u8
}

/// Extract `has_sla_token` from bit [12].
#[inline(always)]
pub const fn has_sla_token(ctx: u64) -> u8 {
    ((ctx >> 12) & 0x1) as u8
}

// ---------------------------------------------------------------------------
// Branchless primitives
// ---------------------------------------------------------------------------

/// Returns `!0` (all bits set) if `x >= y`, and `0` if `x < y`.
/// Inputs should be within bounded range to prevent signed overflow.
#[inline(always)]
pub const fn ge_mask(x: u64, y: u64) -> u64 {
    let diff = (y as i64).wrapping_sub(x as i64).wrapping_sub(1);
    (diff >> 63) as u64
}

/// Branchless multiplexer selector.
#[inline(always)]
pub const fn select(mask: u64, active: u64, fallback: u64) -> u64 {
    (mask & active) | (!mask & fallback)
}

// ---------------------------------------------------------------------------
// Public admission functions
// ---------------------------------------------------------------------------

/// Admit a process context to its routing topology using the global thresholds.
///
/// This function is `O(1)` and branch-free at runtime.
///
/// # Examples
///
/// ```
/// use bcinr_powl::admit::{AdmissionContext, ProcessTopology, admit};
///
/// // Enterprise tenant (class=2), urgency=12, no load, SLA token set.
/// let ctx: AdmissionContext = 0b0001_0000_1100_0010; // tc=2,urg=12,load=0,sla=1
/// assert_eq!(admit(ctx), ProcessTopology::Priority);
/// ```
#[inline(always)]
pub fn admit(ctx: AdmissionContext) -> ProcessTopology {
    admit_dpag(ctx, &GLOBAL_ADMISSION_PARAMETERS.load())
}

/// Admit a process context dynamically and branchlessly based on runtime parameters.
///
/// This implementation guarantees CC=1, 0 heap allocations, and zero branching.
pub fn admit_dpag(ctx: AdmissionContext, params: &AdmissionParameters) -> ProcessTopology {
    // Extract fields
    let c = (ctx & 0xF) as u64;
    let u = ((ctx >> 4) & 0xF) as u64;
    let l = ((ctx >> 8) & 0xF) as u64;
    let s = ((ctx >> 12) & 0x1) as u64;

    // 1. Quarantine evaluation: load >= load_saturation_threshold
    let q_mask = ge_mask(l, params.load_saturation_threshold);

    // 2. Priority evaluation: tc >= tc_priority_min && urgency >= urgency_priority_threshold && sla_ok
    let tc_pri_ok = ge_mask(c, params.tenant_class_priority_min);
    let urg_ok = ge_mask(u, params.urgency_priority_threshold);
    
    // SLA token check: if sla_required is active, has_sla_token must be 1.
    // Bitwise equivalent: (!sla_required_mask) | has_sla_token_mask
    let sla_req_mask = 0u64.wrapping_sub(params.sla_required);
    let sla_has_mask = 0u64.wrapping_sub(s);
    let sla_ok = (!sla_req_mask) | sla_has_mask;

    let p_mask = tc_pri_ok & urg_ok & sla_ok;

    // 3. Standard evaluation: tc >= tenant_class_standard_min
    let s_mask = ge_mask(c, params.tenant_class_standard_min);

    // Discriminant constants mapping directly to enum discriminants
    let topo_q = ProcessTopology::Quarantine as u64;   // 3
    let topo_p = ProcessTopology::Priority as u64;     // 0
    let topo_s = ProcessTopology::Standard as u64;     // 1
    let topo_bg = ProcessTopology::Background as u64;   // 2

    // Apply sequential sign-mask multiplexing (simulating an if-else chain)
    let v1 = select(s_mask, topo_s, topo_bg);
    let v2 = select(p_mask, topo_p, v1);
    let v_final = select(q_mask, topo_q, v2);

    // Map discriminant back to the ProcessTopology enum branchlessly.
    // Avoids branch-bearing match statements by indexing a tiny stack array.
    const TOPOLOGIES: [ProcessTopology; 4] = [
        ProcessTopology::Priority,
        ProcessTopology::Standard,
        ProcessTopology::Background,
        ProcessTopology::Quarantine,
    ];

    TOPOLOGIES[(v_final & 3) as usize]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Typed refusal codes for contract verification in tests.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum StabilityRefusal {
        ContractViolation,
    }

    /// Construct an AdmissionContext from its fields.
    const fn make_ctx(
        tenant_class: u64,
        urgency_tier: u64,
        resource_load: u64,
        has_sla_token: u64,
        is_compensating: u64,
    ) -> AdmissionContext {
        (tenant_class & 0xF)
            | ((urgency_tier & 0xF) << 4)
            | ((resource_load & 0xF) << 8)
            | ((has_sla_token & 0x1) << 12)
            | ((is_compensating & 0x1) << 15)
    }

    /// Independent branching reference oracle.
    fn oracle_admit(ctx: AdmissionContext, params: &AdmissionParameters) -> ProcessTopology {
        let c = (ctx & 0xF) as u64;
        let u = ((ctx >> 4) & 0xF) as u64;
        let l = ((ctx >> 8) & 0xF) as u64;
        let s = ((ctx >> 12) & 0x1) as u64;

        if l >= params.load_saturation_threshold {
            ProcessTopology::Quarantine
        } else if c >= params.tenant_class_priority_min
            && s >= params.sla_required
            && u >= params.urgency_priority_threshold
        {
            ProcessTopology::Priority
        } else if c >= params.tenant_class_standard_min {
            ProcessTopology::Standard
        } else {
            ProcessTopology::Background
        }
    }

    #[test]
    fn enterprise_with_sla_is_priority() {
        let ctx = make_ctx(2, 12, 0, 1, 0);
        assert_eq!(admit(ctx), ProcessTopology::Priority);
    }

    #[test]
    fn sovereign_with_sla_is_priority() {
        let ctx = make_ctx(3, 8, 0, 1, 0);
        assert_eq!(admit(ctx), ProcessTopology::Priority);
    }

    #[test]
    fn resource_load_15_is_quarantine() {
        let ctx = make_ctx(2, 12, 15, 1, 0);
        assert_eq!(admit(ctx), ProcessTopology::Quarantine);
    }

    #[test]
    fn free_tenant_no_sla_is_background() {
        let ctx = make_ctx(0, 0, 0, 0, 0);
        assert_eq!(admit(ctx), ProcessTopology::Background);
    }

    #[test]
    fn standard_tenant_is_standard() {
        let ctx = make_ctx(1, 4, 0, 0, 0);
        assert_eq!(admit(ctx), ProcessTopology::Standard);
    }

    #[test]
    fn enterprise_without_sla_is_standard() {
        let ctx = make_ctx(2, 12, 0, 0, 0);
        assert_eq!(admit(ctx), ProcessTopology::Standard);
    }

    #[test]
    fn enterprise_with_sla_low_urgency_is_standard() {
        let ctx = make_ctx(2, 6, 0, 1, 0);
        assert_eq!(admit(ctx), ProcessTopology::Standard);
    }

    #[test]
    fn quarantine_ignores_compensating_flag() {
        let ctx = make_ctx(3, 15, 15, 1, 1);
        assert_eq!(admit(ctx), ProcessTopology::Quarantine);
    }

    #[test]
    fn exhaustive_differential_testing() {
        // We select representative parameter sets to cover boundary values and transitions.
        let load_saturation_vals = [0, 1, 7, 14, 15];
        let urgency_priority_vals = [0, 1, 7, 8, 14, 15];
        let tenant_class_pri_vals = [0, 1, 2, 3];
        let tenant_class_std_vals = [0, 1, 2, 3];
        let sla_required_vals = [0, 1];

        // 8192 context configurations
        for c in 0..16 {
            for u in 0..16 {
                for l in 0..16 {
                    for s in 0..2 {
                        let ctx = make_ctx(c, u, l, s, 0);
                        
                        for &load_sat in &load_saturation_vals {
                            for &urg_pri in &urgency_priority_vals {
                                for &tc_pri in &tenant_class_pri_vals {
                                    for &tc_std in &tenant_class_std_vals {
                                        for &sla_req in &sla_required_vals {
                                            let params = AdmissionParameters {
                                                load_saturation_threshold: load_sat,
                                                urgency_priority_threshold: urg_pri,
                                                tenant_class_priority_min: tc_pri,
                                                tenant_class_standard_min: tc_std,
                                                sla_required: sla_req,
                                            };
                                            let got = admit_dpag(ctx, &params);
                                            let expected = oracle_admit(ctx, &params);
                                            assert_eq!(
                                                got, expected,
                                                "Mismatch for ctx (tc={}, urg={}, load={}, sla={}) with params {:?}",
                                                c, u, l, s, params
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Hostile Mutant Implementations & Verification ---

    // Mutant 1 (Sign Shift Omission): arithmetic right shift by 62 instead of 63.
    const fn ge_mask_mutant_1(x: u64, y: u64) -> u64 {
        let diff = (y as i64).wrapping_sub(x as i64).wrapping_sub(1);
        (diff as u64 >> 62) as u64
    }

    fn admit_dpag_mutant_1(ctx: AdmissionContext, params: &AdmissionParameters) -> ProcessTopology {
        let c = (ctx & 0xF) as u64;
        let u = ((ctx >> 4) & 0xF) as u64;
        let l = ((ctx >> 8) & 0xF) as u64;
        let s = ((ctx >> 12) & 0x1) as u64;

        let q_mask = ge_mask_mutant_1(l, params.load_saturation_threshold);
        let tc_pri_ok = ge_mask_mutant_1(c, params.tenant_class_priority_min);
        let urg_ok = ge_mask_mutant_1(u, params.urgency_priority_threshold);
        let sla_req_mask = 0u64.wrapping_sub(params.sla_required);
        let sla_has_mask = 0u64.wrapping_sub(s);
        let sla_ok = (!sla_req_mask) | sla_has_mask;
        let p_mask = tc_pri_ok & urg_ok & sla_ok;
        let s_mask = ge_mask_mutant_1(c, params.tenant_class_standard_min);

        let topo_q = ProcessTopology::Quarantine as u64;
        let topo_p = ProcessTopology::Priority as u64;
        let topo_s = ProcessTopology::Standard as u64;
        let topo_bg = ProcessTopology::Background as u64;

        let v1 = select(s_mask, topo_s, topo_bg);
        let v2 = select(p_mask, topo_p, v1);
        let v_final = select(q_mask, topo_q, v2);

        const TOPOLOGIES: [ProcessTopology; 4] = [
            ProcessTopology::Priority,
            ProcessTopology::Standard,
            ProcessTopology::Background,
            ProcessTopology::Quarantine,
        ];
        TOPOLOGIES[(v_final & 3) as usize]
    }

    // Mutant 2 (Priority Bypass / Order Inversion): swapped Quarantine and Priority sequence
    fn admit_dpag_mutant_2(ctx: AdmissionContext, params: &AdmissionParameters) -> ProcessTopology {
        let c = (ctx & 0xF) as u64;
        let u = ((ctx >> 4) & 0xF) as u64;
        let l = ((ctx >> 8) & 0xF) as u64;
        let s = ((ctx >> 12) & 0x1) as u64;

        let q_mask = ge_mask(l, params.load_saturation_threshold);
        let tc_pri_ok = ge_mask(c, params.tenant_class_priority_min);
        let urg_ok = ge_mask(u, params.urgency_priority_threshold);
        let sla_req_mask = 0u64.wrapping_sub(params.sla_required);
        let sla_has_mask = 0u64.wrapping_sub(s);
        let sla_ok = (!sla_req_mask) | sla_has_mask;
        let p_mask = tc_pri_ok & urg_ok & sla_ok;
        let s_mask = ge_mask(c, params.tenant_class_standard_min);

        let topo_q = ProcessTopology::Quarantine as u64;
        let topo_p = ProcessTopology::Priority as u64;
        let topo_s = ProcessTopology::Standard as u64;
        let topo_bg = ProcessTopology::Background as u64;

        // Swapped quarantine and priority sequence order (Quarantine selected before Standard, but after Priority)
        let v1 = select(s_mask, topo_s, topo_bg);
        let v2 = select(q_mask, topo_q, v1);
        let v_final = select(p_mask, topo_p, v2);

        const TOPOLOGIES: [ProcessTopology; 4] = [
            ProcessTopology::Priority,
            ProcessTopology::Standard,
            ProcessTopology::Background,
            ProcessTopology::Quarantine,
        ];
        TOPOLOGIES[(v_final & 3) as usize]
    }

    // Mutant 3 (Off-by-One Comparison Offset): dropping the -1 in ge_mask
    const fn ge_mask_mutant_3(x: u64, y: u64) -> u64 {
        let diff = (y as i64).wrapping_sub(x as i64); // -1 dropped
        (diff >> 63) as u64
    }

    fn admit_dpag_mutant_3(ctx: AdmissionContext, params: &AdmissionParameters) -> ProcessTopology {
        let c = (ctx & 0xF) as u64;
        let u = ((ctx >> 4) & 0xF) as u64;
        let l = ((ctx >> 8) & 0xF) as u64;
        let s = ((ctx >> 12) & 0x1) as u64;

        let q_mask = ge_mask_mutant_3(l, params.load_saturation_threshold);
        let tc_pri_ok = ge_mask_mutant_3(c, params.tenant_class_priority_min);
        let urg_ok = ge_mask_mutant_3(u, params.urgency_priority_threshold);
        let sla_req_mask = 0u64.wrapping_sub(params.sla_required);
        let sla_has_mask = 0u64.wrapping_sub(s);
        let sla_ok = (!sla_req_mask) | sla_has_mask;
        let p_mask = tc_pri_ok & urg_ok & sla_ok;
        let s_mask = ge_mask_mutant_3(c, params.tenant_class_standard_min);

        let topo_q = ProcessTopology::Quarantine as u64;
        let topo_p = ProcessTopology::Priority as u64;
        let topo_s = ProcessTopology::Standard as u64;
        let topo_bg = ProcessTopology::Background as u64;

        let v1 = select(s_mask, topo_s, topo_bg);
        let v2 = select(p_mask, topo_p, v1);
        let v_final = select(q_mask, topo_q, v2);

        const TOPOLOGIES: [ProcessTopology; 4] = [
            ProcessTopology::Priority,
            ProcessTopology::Standard,
            ProcessTopology::Background,
            ProcessTopology::Quarantine,
        ];
        TOPOLOGIES[(v_final & 3) as usize]
    }

    fn verify_mutant_failure<F>(mutant_admit: F) -> Result<(), StabilityRefusal>
    where
        F: Fn(AdmissionContext, &AdmissionParameters) -> ProcessTopology,
    {
        // Check if the mutant behaves differently from the oracle.
        let load_saturation_vals = [0, 1, 7, 14, 15, (1u64 << 62) + 1];
        let urgency_priority_vals = [0, 1, 7, 8, 14, 15];
        let tenant_class_pri_vals = [0, 1, 2, 3];
        let tenant_class_std_vals = [0, 1, 2, 3];
        let sla_required_vals = [0, 1];

        for c in 0..16 {
            for u in 0..16 {
                for l in 0..16 {
                    for s in 0..2 {
                        let ctx = make_ctx(c, u, l, s, 0);
                        for &load_sat in &load_saturation_vals {
                            for &urg_pri in &urgency_priority_vals {
                                for &tc_pri in &tenant_class_pri_vals {
                                    for &tc_std in &tenant_class_std_vals {
                                        for &sla_req in &sla_required_vals {
                                            let params = AdmissionParameters {
                                                load_saturation_threshold: load_sat,
                                                urgency_priority_threshold: urg_pri,
                                                tenant_class_priority_min: tc_pri,
                                                tenant_class_standard_min: tc_std,
                                                sla_required: sla_req,
                                            };
                                            let got = mutant_admit(ctx, &params);
                                            let expected = oracle_admit(ctx, &params);
                                            if got != expected {
                                                // Success: the mutant was detected (killed) by mismatching the oracle!
                                                return Err(StabilityRefusal::ContractViolation);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn kill_mutant_1_sign_shift_omission() {
        let result = verify_mutant_failure(admit_dpag_mutant_1);
        assert_eq!(result, Err(StabilityRefusal::ContractViolation));
    }

    #[test]
    fn kill_mutant_2_priority_bypass() {
        let result = verify_mutant_failure(admit_dpag_mutant_2);
        assert_eq!(result, Err(StabilityRefusal::ContractViolation));
    }

    #[test]
    fn kill_mutant_3_off_by_one_offset() {
        let result = verify_mutant_failure(admit_dpag_mutant_3);
        assert_eq!(result, Err(StabilityRefusal::ContractViolation));
    }

    // ---------------------------------------------------------------------------
    // Proptests
    // ---------------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_admit_lut_exhaustive(ctx: u64) {
            // Ensure admit function with global parameters never panics and returns a valid discriminant
            let topo = admit(ctx);
            let disc = topo as u8;
            prop_assert!(disc <= 3,
                "admit({:#018x}) returned invalid discriminant {disc}", ctx);
        }

        #[test]
        fn prop_admit_dpag_matches_oracle(
            ctx: u64,
            load_sat in 0u64..16,
            urg_pri in 0u64..16,
            tc_pri in 0u64..4,
            tc_std in 0u64..4,
            sla_req in 0u64..2,
        ) {
            let params = AdmissionParameters {
                load_saturation_threshold: load_sat,
                urgency_priority_threshold: urg_pri,
                tenant_class_priority_min: tc_pri,
                tenant_class_standard_min: tc_std,
                sla_required: sla_req,
            };
            let got = admit_dpag(ctx, &params);
            let expected = oracle_admit(ctx, &params);
            prop_assert_eq!(got, expected);
        }
    }
}
