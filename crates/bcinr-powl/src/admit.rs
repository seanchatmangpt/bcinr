//! admit — O(1) phase admission via branchless LUT dispatch.
//!
//! [`admit`] maps an [`AdmissionContext`] bitfield to a [`ProcessTopology`]
//! without any runtime branches. The 8-bit LUT key is derived from the context
//! word in a single shift/mask operation; the LUT itself is built at compile
//! time via a `const fn`.
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

#[forbid(unsafe_code)]

/// Packed admission context word.  See module-level docs for bit layout.
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

// ---------------------------------------------------------------------------
// LUT key extraction helpers (const-friendly)
// ---------------------------------------------------------------------------

/// Extract `tenant_class` from bits [0..3].
#[inline(always)]
const fn tenant_class(ctx: u64) -> u8 {
    (ctx & 0xF) as u8
}

/// Extract `urgency_tier` from bits [4..7].
#[inline(always)]
const fn urgency_tier(ctx: u64) -> u8 {
    ((ctx >> 4) & 0xF) as u8
}

/// Extract `resource_load` from bits [8..11].
#[inline(always)]
const fn resource_load(ctx: u64) -> u8 {
    ((ctx >> 8) & 0xF) as u8
}

/// Extract `has_sla_token` from bit [12].
#[inline(always)]
const fn has_sla_token(ctx: u64) -> u8 {
    ((ctx >> 12) & 0x1) as u8
}

/// Build the 8-bit LUT key used to index [`TOPOLOGY_LUT`].
///
/// Key layout (8 bits):
/// - bit 7: resource_load == 15 (saturated)
/// - bits 6..5: tenant_class (0..3, clamped to 2 bits)
/// - bit 4: has_sla_token
/// - bits 3..0: urgency_tier >> 1  (4 urgency buckets of width 2)
///
/// This encoding collapses the full context into 256 entries while preserving
/// all policy-relevant distinctions.
#[inline(always)]
const fn lut_key(ctx: u64) -> u8 {
    let saturated = ((resource_load(ctx) == 15) as u8) << 7;
    let tc = (tenant_class(ctx) & 0x3) << 5;
    let sla = has_sla_token(ctx) << 4;
    let urg = (urgency_tier(ctx) >> 1) & 0xF;
    saturated | tc | sla | urg
}

// ---------------------------------------------------------------------------
// Compile-time LUT construction
// ---------------------------------------------------------------------------

/// The admission LUT — 256 entries, one per possible 8-bit LUT key.
pub static TOPOLOGY_LUT: [ProcessTopology; 256] = build_topology_lut();

/// Build the 256-entry topology LUT at compile time.
///
/// Policy rules (evaluated in priority order):
/// 1. resource_load == 15 (bit 7 set)  → Quarantine, unconditionally.
/// 2. tenant_class >= 2 (enterprise/sovereign) AND has_sla_token AND urgency >= 8
///    → Priority.
/// 3. tenant_class >= 1 (standard/enterprise/sovereign)                → Standard.
/// 4. Otherwise                                                         → Background.
const fn build_topology_lut() -> [ProcessTopology; 256] {
    let mut lut = [ProcessTopology::Background; 256];
    let mut key: usize = 0;
    while key < 256 {
        let k = key as u8;
        let saturated = (k >> 7) & 1;
        let tc = (k >> 5) & 0x3;
        let sla = (k >> 4) & 0x1;
        let urg_bucket = k & 0xF; // urgency_tier >> 1; 4 means original tier >= 8

        lut[key] = if saturated == 1 {
            ProcessTopology::Quarantine
        } else if tc >= 2 && sla == 1 && urg_bucket >= 4 {
            // enterprise or sovereign + SLA token + urgency_tier >= 8
            ProcessTopology::Priority
        } else if tc >= 1 {
            ProcessTopology::Standard
        } else {
            ProcessTopology::Background
        };

        key += 1;
    }
    lut
}

// ---------------------------------------------------------------------------
// Public admission function
// ---------------------------------------------------------------------------

/// Admit a process context to its routing topology.
///
/// This function is `O(1)` and branch-free at runtime: it reduces the
/// [`AdmissionContext`] word to an 8-bit key and performs a single array
/// index into [`TOPOLOGY_LUT`].
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
    TOPOLOGY_LUT[lut_key(ctx) as usize]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn enterprise_with_sla_is_priority() {
        // tenant_class=2 (enterprise), urgency=12, load=0, sla=1
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
        // Even an enterprise+SLA+high-urgency context must be quarantined when
        // resource_load is fully saturated.
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
        // urgency_tier=6 → urg_bucket=3 < 4, so not Priority
        let ctx = make_ctx(2, 6, 0, 1, 0);
        assert_eq!(admit(ctx), ProcessTopology::Standard);
    }

    #[test]
    fn quarantine_ignores_compensating_flag() {
        let ctx = make_ctx(3, 15, 15, 1, 1);
        assert_eq!(admit(ctx), ProcessTopology::Quarantine);
    }

    #[test]
    fn lut_has_no_uninitialised_gaps() {
        // Smoke test: every entry is a valid discriminant (the compiler
        // enforces this via the enum repr, but we verify the cast round-trips).
        for entry in TOPOLOGY_LUT.iter() {
            let disc = *entry as u8;
            assert!(disc <= 3, "invalid topology discriminant {disc}");
        }
    }
}
