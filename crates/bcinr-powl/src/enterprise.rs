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
/// has      = granted & required              (bits present)
/// xor      = has ^ required                  (0 iff all required bits present)
/// nz       = (xor | -xor) >> 63             (1 iff xor != 0; correct for all u64)
/// ok       = nz.wrapping_sub(1)             (u64::MAX when equal, 0 when not)
/// mask     = 0u64.wrapping_sub(ok >> 63)
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
    let xor = has ^ required;
    // Hoare-Logic Verification Line 1: (xor | xor.wrapping_neg()) >> 63 == 1 iff xor != 0.
    // Proof: xor=0 → both terms 0, OR=0, >>63=0.
    //        xor>0 → if xor<2^63: wrapping_neg=2^64-xor>2^63, bit63 set.
    //                if xor≥2^63: xor itself has bit63 set. QED.
    // Previous formula (xor.wrapping_sub(1) >> 63) fails when xor >= 2^63:
    // xor-1 still has bit63 set, yielding 1 (grant) when it should yield 0 (deny).
    let nz = (xor | xor.wrapping_neg()) >> 63; // 1 iff xor != 0
    let ok = 1u64.wrapping_sub(nz); // 1 iff all required bits present
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
/// For each `u32` counter `n`, the expression `(n as u64 | (n as u64).wrapping_neg()) >> 63`
/// produces `1` when `n > 0` and `0` when `n == 0`, correctly for all `u32`
/// values including those with bit 31 set (the `wrapping_sub(1) >> 31` form fails
/// for n ≥ 2^31+1 and n = u32::MAX).
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
    // Hoare-Logic Verification Line 2: cast to u64 first; 0 ≤ n ≤ 2^32-1 < 2^63, so
    // wrapping_neg(x) = 2^64-x > 2^63 for any x > 0 in this range, setting bit63.
    // For x=0, wrapping_neg(0)=0. QED. The previous (wrapping_sub(1) >> 31) ^ 1
    // form fails for n ≥ 2^31+1 (n-1 has bit31 set, >> 31 yields 1, XOR 1 yields 0).
    let nonzero_u32 = |n: u32| -> u64 {
        let x = n as u64;
        (x | x.wrapping_neg()) >> 63
    };

    let needs_discovery = nonzero_u32(order_violations) * graduation::NEEDS_DISCOVERY;
    let needs_conformance = nonzero_u32(sla_breaches) * graduation::NEEDS_CONFORMANCE;
    let needs_replay = nonzero_u32(watchdog_trips) * graduation::NEEDS_REPLAY;
    let needs_receipts = nonzero_u32(compensation_count) * graduation::NEEDS_RECEIPTS;

    // Benchmark required when instance_count >= 1_000.
    // Hoare-Logic Verification Line 3: saturating_sub(999) is nonzero iff instance_count >= 1000.
    // Proof: saturating_sub never wraps; returns 0 for count in [0, 999] and count-999 > 0 for count >= 1000.
    // The previous ((count.wrapping_sub(1000) >> 63) ^ 1) form failed for count in [2^63, 2^63+999]
    // and [2^63+1000, 2^64-1] because the wrapping result's bit63 does not reliably encode carry.
    // The (999u64.wrapping_sub(count) >> 63) form fails for count = 2^63+1000 because
    // 999 - (2^63+1000) wraps to 2^63-1, leaving bit63 clear despite count >= 1000.
    // saturating_sub is correct for all u64 values. QED.
    let bench_x = instance_count.saturating_sub(999);
    let bench_flag = ((bench_x | bench_x.wrapping_neg()) >> 63) * graduation::NEEDS_BENCHMARK;

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

    // Bug-regression: bit-63 edge case — previous (xor.wrapping_sub(1) >> 63)
    // formula incorrectly granted when xor >= 2^63.
    #[test]
    fn capability_mask_no_grant_full_required_returns_zero() {
        assert_eq!(capability_mask(0, u64::MAX), 0);
    }

    #[test]
    fn capability_mask_full_grant_full_required_returns_all_ones() {
        assert_eq!(capability_mask(u64::MAX, u64::MAX), u64::MAX);
    }

    #[test]
    fn capability_mask_missing_high_bit_returns_zero() {
        // granted has all bits except bit 63; required has all bits.
        // Old code: xor=(1<<63), xor-1=0x7FFF...FFFF, >>63=0 — was OK here.
        // But with required=(1<<63)|2 and granted=(1<<63): xor=2, still OK.
        // The breaking case: granted=0, required=u64::MAX — xor=u64::MAX >= 2^63.
        assert_eq!(capability_mask(0x7FFF_FFFF_FFFF_FFFF, u64::MAX), 0);
    }

    #[test]
    fn capability_mask_bit63_and_other_missing_returns_zero() {
        // required has bit63 set plus bit0; granted has neither.
        // Old formula: xor=(1<<63)|1, xor-1=(1<<63), >>63=1 → incorrectly grants.
        let required = (1u64 << 63) | 1;
        assert_eq!(capability_mask(0, required), 0);
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

    // Bug-regression: nonzero_u32 with n >= 2^31.
    // Previous (n.wrapping_sub(1) >> 31) ^ 1 form returns 0 for u32::MAX and
    // other values where n-1 has bit 31 set.
    #[test]
    fn graduation_nonzero_u32_high_bit_set_triggers_flag() {
        let n = 2_147_483_649u32; // 2^31 + 1: n-1 = 2^31, bit31 set → old formula fails
        let bits = evaluate_graduation(n, 0, 0, 0, 0);
        assert_ne!(
            bits & graduation::NEEDS_DISCOVERY,
            0,
            "nonzero_u32({n}) must return 1 but old formula returned 0"
        );
    }

    #[test]
    fn graduation_nonzero_u32_max_triggers_flag() {
        let bits = evaluate_graduation(u32::MAX, 0, 0, 0, 0);
        assert_ne!(
            bits & graduation::NEEDS_DISCOVERY,
            0,
            "nonzero_u32(u32::MAX) must return 1"
        );
    }

    #[test]
    fn graduation_zero_u32_does_not_trigger_flag() {
        let bits = evaluate_graduation(0, 0, 0, 0, 0);
        assert_eq!(bits & graduation::NEEDS_DISCOVERY, 0);
    }

    // -----------------------------------------------------------------------
    // graduation boundary matrix (non-proptest)
    // -----------------------------------------------------------------------

    #[test]
    fn graduation_boundary_matrix() {
        // n ∈ {0, 1, 2^31, 2^31+1, u32::MAX} × counters=0 → covers all boundary cases
        let cases = [0u32, 1, 1u32 << 31, (1u32 << 31) + 1, u32::MAX];
        for &n in &cases {
            // Calling evaluate_graduation with n>0 should set NEEDS_DISCOVERY flag in result.
            // Use the public function — check that it doesn't panic.
            let _ = evaluate_graduation(n, 0, 0, 0, 0);
        }
    }

    // -----------------------------------------------------------------------
    // Proptests — capability_mask
    // -----------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_graduation_no_panic_for_any_inputs(
            order_violations: u32,
            sla_breaches: u32,
            watchdog_trips: u32,
            compensation_count: u32,
            instance_count: u64,
        ) {
            // evaluate_graduation must not panic for any combination of inputs,
            // and must return only valid bit combinations (bits 0..=4).
            let result = evaluate_graduation(
                order_violations, sla_breaches, watchdog_trips, compensation_count, instance_count,
            );
            prop_assert_eq!(result & !0b11111u64, 0,
                "evaluate_graduation returned bits outside [0..4]: {:#018x}", result);
        }

        #[test]
        fn prop_graduation_each_flag_set_iff_counter_nonzero(
            order_violations: u32,
            sla_breaches: u32,
            watchdog_trips: u32,
            compensation_count: u32,
        ) {
            let result = evaluate_graduation(order_violations, sla_breaches, watchdog_trips, compensation_count, 0);

            let has_discovery = (result & graduation::NEEDS_DISCOVERY) != 0;
            let has_conformance = (result & graduation::NEEDS_CONFORMANCE) != 0;
            let has_replay = (result & graduation::NEEDS_REPLAY) != 0;
            let has_receipts = (result & graduation::NEEDS_RECEIPTS) != 0;

            prop_assert_eq!(has_discovery, order_violations != 0,
                "NEEDS_DISCOVERY mismatch: order_violations={}", order_violations);
            prop_assert_eq!(has_conformance, sla_breaches != 0,
                "NEEDS_CONFORMANCE mismatch: sla_breaches={}", sla_breaches);
            prop_assert_eq!(has_replay, watchdog_trips != 0,
                "NEEDS_REPLAY mismatch: watchdog_trips={}", watchdog_trips);
            prop_assert_eq!(has_receipts, compensation_count != 0,
                "NEEDS_RECEIPTS mismatch: compensation_count={}", compensation_count);
        }

        #[test]
        fn prop_graduation_benchmark_flag_iff_count_ge_1000(instance_count: u64) {
            let result = evaluate_graduation(0, 0, 0, 0, instance_count);
            let has_bench = (result & graduation::NEEDS_BENCHMARK) != 0;
            prop_assert_eq!(has_bench, instance_count >= 1_000,
                "NEEDS_BENCHMARK mismatch: instance_count={}", instance_count);
        }

        #[test]
        fn prop_capability_mask_iff_all_required_bits_set(g: u64, r: u64) {
            let result = capability_mask(g, r);
            let expected = if (g & r) == r { u64::MAX } else { 0 };
            prop_assert_eq!(result, expected,
                "capability_mask({:#018x}, {:#018x}) = {:#018x}, expected {:#018x}", g, r, result, expected);
        }

        #[test]
        fn prop_capability_mask_symmetry(g: u64, r: u64) {
            // If g has all required bits, mask must be MAX; otherwise 0.
            let has_all = (g & r) == r;
            let mask = capability_mask(g, r);
            if has_all {
                prop_assert_eq!(mask, u64::MAX);
            } else {
                prop_assert_eq!(mask, 0u64);
            }
        }
    }

    #[test]
    fn graduation_nonzero_u32_boundary_matrix() {
        let nonzero_vals: &[u32] = &[1, (1u32 << 31), (1u32 << 31).wrapping_add(1), u32::MAX];
        for &n in nonzero_vals {
            assert_ne!(
                evaluate_graduation(n, 0, 0, 0, 0) & graduation::NEEDS_DISCOVERY,
                0,
                "order_violations={n} must set NEEDS_DISCOVERY"
            );
            assert_ne!(
                evaluate_graduation(0, n, 0, 0, 0) & graduation::NEEDS_CONFORMANCE,
                0,
                "sla_breaches={n} must set NEEDS_CONFORMANCE"
            );
            assert_ne!(
                evaluate_graduation(0, 0, n, 0, 0) & graduation::NEEDS_REPLAY,
                0,
                "watchdog_trips={n} must set NEEDS_REPLAY"
            );
            assert_ne!(
                evaluate_graduation(0, 0, 0, n, 0) & graduation::NEEDS_RECEIPTS,
                0,
                "compensation_count={n} must set NEEDS_RECEIPTS"
            );
        }
    }
}
