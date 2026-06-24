//! enterprise — SLA capabilities, saga compensation stacks, op metadata, and
//! branchless graduation evaluation.
//!
//! All hot-path operations are `O(1)` and branch-free. The [`SagaStack`] is a
//! fixed-size LIFO with 32 frames allocated on the stack (no heap). The
//! [`EnterpriseOpMeta`] parallel array is explicitly `repr(C, align(64))` so
//! that each entry occupies exactly one cache line and is never interleaved
//! with the hot tape.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Capability set
// ---------------------------------------------------------------------------

/// A 64-bit bitmask representing a set of granted or required capabilities.
///
/// Each bit corresponds to a capability defined by the platform operator.
/// Bit positions are assigned by convention; this type is intentionally opaque
/// to the admission layer.
pub type CapabilitySet = u64;

/// Branchless capability check.
///
/// Returns `0xFFFF_FFFF_FFFF_FFFF` (all-ones) when `granted` contains every
/// bit set in `required`; returns `0` otherwise.
///
/// # Algorithm
///
/// ```text
/// has  = granted & required
/// xor  = has XOR required          (0 iff has == required)
/// nz   = (xor | xor.wrapping_neg()) >> 63  (1 iff xor != 0)
/// ok   = 1.wrapping_sub(nz)        (1 iff xor == 0)
/// mask = 0u64.wrapping_sub(ok)     (0xFFFF... iff ok==1, 0 otherwise)
/// ```
///
/// No branches, no conditionals — pure arithmetic on the mask calculus
/// pattern established in `bcinr_logic::mask`.
///
/// # Examples
///
/// ```
/// use bcinr_powl::enterprise::capability_mask;
///
/// let granted: u64 = 0b1111;
/// let required: u64 = 0b0101;
/// assert_eq!(capability_mask(granted, required), u64::MAX);
///
/// let required_missing: u64 = 0b1010_0000;
/// assert_eq!(capability_mask(granted, required_missing), 0);
/// ```
#[inline(always)]
pub fn capability_mask(granted: CapabilitySet, required: CapabilitySet) -> u64 {
    let has = granted & required;
    // xor is 0 iff has == required.
    // (xor | xor.wrapping_neg()) >> 63: 1 when xor != 0, 0 when xor == 0.
    // ok: 1 when xor == 0 (all required bits present), 0 otherwise.
    let xor = has ^ required;
    let nonzero = (xor | xor.wrapping_neg()) >> 63;
    let ok = 1u64.wrapping_sub(nonzero);
    0u64.wrapping_sub(ok)
}

// ---------------------------------------------------------------------------
// Saga stack
// ---------------------------------------------------------------------------

/// Fixed-capacity LIFO stack for saga compensation operation indices.
///
/// Stores up to 32 `u16` compensation op indices with no heap allocation.
/// Push beyond capacity silently saturates (the top entry is overwritten);
/// this is intentional — callers that care about overflow should check
/// [`SagaStack::is_full`] before pushing.
#[derive(Debug)]
pub struct SagaStack {
    frames: [u16; 32],
    top: u8,
}

impl SagaStack {
    /// Create an empty [`SagaStack`].
    pub const fn new() -> Self {
        Self {
            frames: [0u16; 32],
            top: 0,
        }
    }

    /// Returns `true` when the stack contains no frames.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.top == 0
    }

    /// Returns `true` when the stack is at capacity (32 frames).
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.top as usize >= 32
    }

    /// Current number of frames on the stack.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.top as usize
    }

    /// Push a compensation op index.
    ///
    /// If the stack is full the push is silently dropped (capacity-saturating
    /// behaviour). The caller is responsible for checking [`SagaStack::is_full`]
    /// in contexts where overflow is a protocol error.
    #[inline(always)]
    pub fn push(&mut self, comp_op_idx: u16) {
        if (self.top as usize) < 32 {
            self.frames[self.top as usize] = comp_op_idx;
            self.top = self.top.saturating_add(1);
        }
    }

    /// Pop the most-recently-pushed compensation op index.
    ///
    /// Returns `None` when the stack is empty.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<u16> {
        if self.top == 0 {
            return None;
        }
        self.top -= 1;
        Some(self.frames[self.top as usize])
    }
}

impl Default for SagaStack {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SagaRole
// ---------------------------------------------------------------------------

/// The saga participation role of an op in the workflow.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SagaRole {
    /// Op participates in no saga.
    None = 0,
    /// Op is the root (initiating) participant of a saga.
    Root = 1,
    /// Op is a forward (normal-path) participant of a saga.
    Forward = 2,
    /// Op is a compensating participant — executed on rollback.
    Compensator = 3,
}

// ---------------------------------------------------------------------------
// EnterpriseOpMeta
// ---------------------------------------------------------------------------

/// Per-op metadata for enterprise scheduling — stored in a parallel array
/// that is **never placed on the hot tape**.
///
/// The struct is `repr(C, align(64))` to guarantee exactly one cache-line per
/// entry (64 bytes). The `_pad` field enforces the size invariant at compile
/// time via the `const` assertion below.
#[repr(C, align(64))]
pub struct EnterpriseOpMeta {
    /// Absolute deadline in nanoseconds (monotonic clock).
    pub deadline_ns: u64,
    /// Capability bits that must be present in the executing tenant's grant.
    pub required_caps: CapabilitySet,
    /// Index of the compensating op in the saga, or `u16::MAX` if none.
    pub comp_op_idx: u16,
    /// Saga role of this op.
    pub saga_role: SagaRole,
    /// SLA tier classification (0 = best-effort, 255 = highest).
    pub sla_tier: u8,
    _pad: [u8; 37],
}

// Compile-time size assertion: must be exactly 64 bytes (one cache line).
const _: () = assert!(
    core::mem::size_of::<EnterpriseOpMeta>() == 64,
    "EnterpriseOpMeta must be exactly 64 bytes (one cache line)"
);

impl EnterpriseOpMeta {
    /// Construct a zeroed entry with the given fields.
    pub const fn new(
        deadline_ns: u64,
        required_caps: CapabilitySet,
        comp_op_idx: u16,
        saga_role: SagaRole,
        sla_tier: u8,
    ) -> Self {
        Self {
            deadline_ns,
            required_caps,
            comp_op_idx,
            saga_role,
            sla_tier,
            _pad: [0u8; 37],
        }
    }
}

// ---------------------------------------------------------------------------
// Graduation evaluation
// ---------------------------------------------------------------------------

/// Bitmask bits returned by [`evaluate_graduation`].
pub mod graduation {
    /// Bit 0: process mining discovery pass required (order violations detected).
    pub const NEEDS_DISCOVERY: u64 = 1 << 0;
    /// Bit 1: conformance check required (SLA breaches observed).
    pub const NEEDS_CONFORMANCE: u64 = 1 << 1;
    /// Bit 2: replay validation required (watchdog trips detected).
    pub const NEEDS_REPLAY: u64 = 1 << 2;
    /// Bit 3: receipt audit required (compensations were executed).
    pub const NEEDS_RECEIPTS: u64 = 1 << 3;
    /// Bit 4: benchmark regression required (sufficient instance volume).
    pub const NEEDS_BENCHMARK: u64 = 1 << 4;
}

/// Branchless graduation evaluation.
///
/// Returns a bitmask indicating which post-manufacturing validation passes are
/// required for this instance set.  Each signal is derived independently and
/// combined with bitwise OR — no conditional branches, constant latency.
///
/// # Bit assignments
///
/// | Bit | Constant                          | Trigger condition                    |
/// |-----|-----------------------------------|--------------------------------------|
/// | 0   | [`graduation::NEEDS_DISCOVERY`]   | `order_violations > 0`               |
/// | 1   | [`graduation::NEEDS_CONFORMANCE`] | `sla_breaches > 0`                   |
/// | 2   | [`graduation::NEEDS_REPLAY`]      | `watchdog_trips > 0`                 |
/// | 3   | [`graduation::NEEDS_RECEIPTS`]    | `compensation_count > 0`             |
/// | 4   | [`graduation::NEEDS_BENCHMARK`]   | `instance_count >= 1_000`            |
///
/// # Algorithm
///
/// For each u32 counter n, the expression (n | n.wrapping_neg()) >> 31
/// produces 1 when n > 0 and 0 when n == 0 — branchless, single
/// instruction on most ISAs (OR + NEG + SHR).
///
/// # Examples
///
/// ```
/// use bcinr_powl::enterprise::{evaluate_graduation, graduation};
///
/// let bits = evaluate_graduation(1, 0, 0, 0, 0);
/// assert_ne!(bits & graduation::NEEDS_DISCOVERY, 0);
/// assert_eq!(bits & graduation::NEEDS_CONFORMANCE, 0);
/// ```
#[inline(always)]
pub fn evaluate_graduation(
    order_violations: u32,
    sla_breaches: u32,
    watchdog_trips: u32,
    compensation_count: u32,
    instance_count: u64,
) -> u64 {
    // Branchless nonzero: (n | n.wrapping_neg()) >> 31 == 1 iff n > 0.
    let nonzero_u32 = |n: u32| -> u64 { ((n | n.wrapping_neg()) >> 31) as u64 };

    let needs_discovery = nonzero_u32(order_violations) * graduation::NEEDS_DISCOVERY;
    let needs_conformance = nonzero_u32(sla_breaches) * graduation::NEEDS_CONFORMANCE;
    let needs_replay = nonzero_u32(watchdog_trips) * graduation::NEEDS_REPLAY;
    let needs_receipts = nonzero_u32(compensation_count) * graduation::NEEDS_RECEIPTS;

    // Benchmark required when instance_count >= 1_000.
    let bench_flag = (instance_count >= 1_000) as u64 * graduation::NEEDS_BENCHMARK;

    needs_discovery | needs_conformance | needs_replay | needs_receipts | bench_flag
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{graduation, *};

    // -----------------------------------------------------------------------
    // capability_mask
    // -----------------------------------------------------------------------

    #[test]
    fn capability_mask_exact_match_returns_all_ones() {
        let granted: u64 = 0b1111_0000;
        let required: u64 = 0b1111_0000;
        assert_eq!(capability_mask(granted, required), u64::MAX);
    }

    #[test]
    fn capability_mask_superset_returns_all_ones() {
        let granted: u64 = 0b1111_1111;
        let required: u64 = 0b0000_1111;
        assert_eq!(capability_mask(granted, required), u64::MAX);
    }

    #[test]
    fn capability_mask_missing_bit_returns_zero() {
        let granted: u64 = 0b0101;
        let required: u64 = 0b0111; // bit 1 missing in granted
        assert_eq!(capability_mask(granted, required), 0);
    }

    #[test]
    fn capability_mask_empty_required_always_succeeds() {
        // Every set of capabilities satisfies empty requirements.
        assert_eq!(capability_mask(0, 0), u64::MAX);
        assert_eq!(capability_mask(u64::MAX, 0), u64::MAX);
    }

    #[test]
    fn capability_mask_disjoint_returns_zero() {
        assert_eq!(capability_mask(0b1010, 0b0101), 0);
    }

    // -----------------------------------------------------------------------
    // SagaStack
    // -----------------------------------------------------------------------

    #[test]
    fn saga_stack_empty_on_construction() {
        let s = SagaStack::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn saga_stack_push_pop_lifo_order() {
        let mut s = SagaStack::new();
        s.push(10);
        s.push(20);
        s.push(30);
        assert_eq!(s.len(), 3);
        assert_eq!(s.pop(), Some(30));
        assert_eq!(s.pop(), Some(20));
        assert_eq!(s.pop(), Some(10));
        assert_eq!(s.pop(), None);
        assert!(s.is_empty());
    }

    #[test]
    fn saga_stack_pop_empty_returns_none() {
        let mut s = SagaStack::new();
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn saga_stack_saturates_at_32_frames() {
        let mut s = SagaStack::new();
        for i in 0..32u16 {
            s.push(i);
        }
        assert!(s.is_full());
        assert_eq!(s.len(), 32);
        // Push beyond capacity — must not panic, silently dropped.
        s.push(999);
        assert_eq!(s.len(), 32);
        // LIFO still correct for the last valid frame.
        assert_eq!(s.pop(), Some(31));
    }

    // -----------------------------------------------------------------------
    // EnterpriseOpMeta size
    // -----------------------------------------------------------------------

    #[test]
    fn enterprise_op_meta_is_one_cache_line() {
        assert_eq!(core::mem::size_of::<EnterpriseOpMeta>(), 64);
        assert_eq!(core::mem::align_of::<EnterpriseOpMeta>(), 64);
    }

    // -----------------------------------------------------------------------
    // evaluate_graduation
    // -----------------------------------------------------------------------

    #[test]
    fn graduation_sets_discovery_on_order_violations() {
        let bits = evaluate_graduation(1, 0, 0, 0, 0);
        assert_ne!(bits & graduation::NEEDS_DISCOVERY, 0);
        assert_eq!(bits & graduation::NEEDS_CONFORMANCE, 0);
        assert_eq!(bits & graduation::NEEDS_REPLAY, 0);
        assert_eq!(bits & graduation::NEEDS_RECEIPTS, 0);
    }

    #[test]
    fn graduation_no_flags_on_clean_run() {
        let bits = evaluate_graduation(0, 0, 0, 0, 0);
        assert_eq!(bits, 0);
    }

    #[test]
    fn graduation_conformance_on_sla_breaches() {
        let bits = evaluate_graduation(0, 3, 0, 0, 0);
        assert_ne!(bits & graduation::NEEDS_CONFORMANCE, 0);
        assert_eq!(bits & graduation::NEEDS_DISCOVERY, 0);
    }

    #[test]
    fn graduation_replay_on_watchdog_trips() {
        let bits = evaluate_graduation(0, 0, 1, 0, 0);
        assert_ne!(bits & graduation::NEEDS_REPLAY, 0);
    }

    #[test]
    fn graduation_receipts_on_compensations() {
        let bits = evaluate_graduation(0, 0, 0, 5, 0);
        assert_ne!(bits & graduation::NEEDS_RECEIPTS, 0);
    }

    #[test]
    fn graduation_benchmark_at_1000_instances() {
        let bits_below = evaluate_graduation(0, 0, 0, 0, 999);
        assert_eq!(bits_below & graduation::NEEDS_BENCHMARK, 0);

        let bits_at = evaluate_graduation(0, 0, 0, 0, 1_000);
        assert_ne!(bits_at & graduation::NEEDS_BENCHMARK, 0);

        let bits_above = evaluate_graduation(0, 0, 0, 0, 10_000);
        assert_ne!(bits_above & graduation::NEEDS_BENCHMARK, 0);
    }

    #[test]
    fn graduation_all_flags_set_on_fully_degraded_run() {
        let bits = evaluate_graduation(1, 1, 1, 1, 1_000);
        assert_ne!(bits & graduation::NEEDS_DISCOVERY, 0);
        assert_ne!(bits & graduation::NEEDS_CONFORMANCE, 0);
        assert_ne!(bits & graduation::NEEDS_REPLAY, 0);
        assert_ne!(bits & graduation::NEEDS_RECEIPTS, 0);
        assert_ne!(bits & graduation::NEEDS_BENCHMARK, 0);
    }

    #[test]
    fn graduation_discovery_not_set_when_zero_violations() {
        let bits = evaluate_graduation(0, 1, 1, 1, 0);
        assert_eq!(bits & graduation::NEEDS_DISCOVERY, 0);
    }

    #[test]
    fn capability_mask_high_bit_xor_returns_zero() {
        // Regression for wrapping_sub bug: xor = 2^63 would previously give
        // (2^63).wrapping_sub(1) >> 63 == 0, falsely reporting success.
        let granted: u64  = 0;
        let required: u64 = 1u64 << 63;
        assert_eq!(capability_mask(granted, required), 0,
            "missing high-bit cap must return 0");
    }

    #[test]
    fn graduation_nonzero_u32_max_value() {
        // Regression for wrapping_sub bug: n = u32::MAX was wrong.
        let bits = evaluate_graduation(u32::MAX, 0, 0, 0, 0);
        assert_ne!(bits & graduation::NEEDS_DISCOVERY, 0,
            "u32::MAX order_violations must set NEEDS_DISCOVERY");
    }

    #[test]
    fn graduation_benchmark_near_u64_max() {
        // Regression for bench_flag overflow: instance_count near u64::MAX.
        let bits = evaluate_graduation(0, 0, 0, 0, u64::MAX);
        assert_ne!(bits & graduation::NEEDS_BENCHMARK, 0,
            "u64::MAX instances must set NEEDS_BENCHMARK");
    }
}
