//! denial — `DenialPolarity` is a branchless bitfield encoding why a manufacturing
//! step was refused admission.
//!
//! Each constant occupies a distinct byte lane in the u64 word so that
//! `compose` is a bitwise OR and `to_fired_mask` is a branchless lane-to-bit
//! scatter.  `ADMITTED` is the zero value: no denial lanes are set.

/// A packed denial-cause word.
///
/// Byte layout (little-endian lane numbering, lane 0 = bits 7..0):
///
/// | Lane | Bits        | Constant                    |
/// |------|-------------|-----------------------------|
/// | 0    | 7..0        | (ADMITTED = all zero)        |
/// | 1    | 15..8       | `PRECONDITION_FAILED`        |
/// | 2    | 23..16      | `SLA_BREACH`                 |
/// | 3    | 31..24      | `AUTHORIZATION_DENIED`       |
/// | 4    | 39..32      | `RESOURCE_EXHAUSTED`         |
/// | 5    | 47..40      | `OBJECT_LIFECYCLE_VIOLATION` |
/// | 6    | 55..48      | `CONFORMANCE_GATE_FAILED`    |
/// | 7    | 63..56      | `WATCHDOG_DRAINED`           |
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DenialPolarity(pub u64);

impl DenialPolarity {
    /// No denial — the step is admitted.
    pub const ADMITTED: Self = Self(0);

    /// Watchdog timer drained before the step completed (lane 7, bits 63..56).
    pub const WATCHDOG_DRAINED: Self = Self(0xFF00_0000_0000_0000);

    /// A declared precondition was not satisfied (lane 1, bits 15..8).
    pub const PRECONDITION_FAILED: Self = Self(0x0000_0000_0000_FF00);

    /// Service-level agreement deadline exceeded (lane 2, bits 23..16).
    pub const SLA_BREACH: Self = Self(0x0000_0000_00FF_0000);

    /// Authorization proof absent or expired (lane 3, bits 31..24).
    pub const AUTHORIZATION_DENIED: Self = Self(0x0000_0000_FF00_0000);

    /// Capacity or quota exhausted (lane 4, bits 39..32).
    pub const RESOURCE_EXHAUSTED: Self = Self(0x0000_00FF_0000_0000);

    /// Object lifecycle law violated (lane 5, bits 47..40).
    pub const OBJECT_LIFECYCLE_VIOLATION: Self = Self(0x0000_FF00_0000_0000);

    /// A process-conformance gate rejected the step (lane 6, bits 55..48).
    pub const CONFORMANCE_GATE_FAILED: Self = Self(0x00FF_0000_0000_0000);

    /// Returns `true` when no denial lane is set.
    #[inline]
    pub fn is_admitted(self) -> bool {
        self.0 == 0
    }

    /// Compose two `DenialPolarity` words by OR-ing their lanes.
    ///
    /// Admitted (zero) is the identity element: `compose(x, ADMITTED) == x`.
    #[inline]
    pub fn compose(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Branchless lane-to-bit scatter: each active byte lane in `self.0`
    /// produces a distinct bit in the returned `u64`.
    ///
    /// Lane mapping (same order as the byte lanes above):
    ///
    /// | Byte lane | Output bit |
    /// |-----------|------------|
    /// | 0         | 0          |
    /// | 1         | 1          |
    /// | 2         | 2          |
    /// | 3         | 3          |
    /// | 4         | 4          |
    /// | 5         | 5          |
    /// | 6         | 6          |
    /// | 7         | 7          |
    ///
    /// A byte lane is "active" when its byte value is non-zero.
    /// The scatter is branchless: each lane contributes
    /// `((lane_byte != 0) as u64) << lane_index`.
    #[inline]
    pub fn to_fired_mask(self) -> u64 {
        let w = self.0;
        // Extract each byte lane and saturate to 0 or 1 branchlessly.
        // (byte >> 0) & 0xFF  — lane 0
        // ...
        // (byte >> 56) & 0xFF — lane 7
        //
        // `min(byte, 1)` is not branch-free in all targets; instead we use
        // the identity `!!x == (x != 0)` in integer arithmetic:
        // `((x | x.wrapping_neg()) >> 63) ^ 1` would give inverted;
        // simpler: clamp via `(byte != 0) as u64` which the compiler lowers
        // to a branchless `setne` / `cmov` on x86-64.
        let lane = |shift: u32| -> u64 {
            let byte = (w >> shift) & 0xFF;
            // branchless: non-zero → 1, zero → 0
            // Use unsigned trick: (byte | byte.wrapping_neg()) >> 63
            // gives 1 for any non-zero byte (the high bit of the two's-complement
            // negation is set for any non-zero value).
            (byte | byte.wrapping_neg()) >> 63
        };

        lane(0)       // bit 0
        | (lane(8)  << 1)  // bit 1
        | (lane(16) << 2)  // bit 2
        | (lane(24) << 3)  // bit 3
        | (lane(32) << 4)  // bit 4
        | (lane(40) << 5)  // bit 5
        | (lane(48) << 6)  // bit 6
        | (lane(56) << 7)  // bit 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_is_zero() {
        assert!(DenialPolarity::ADMITTED.is_admitted());
        assert_eq!(DenialPolarity::ADMITTED.0, 0);
    }

    #[test]
    fn non_admitted_constants_not_admitted() {
        assert!(!DenialPolarity::WATCHDOG_DRAINED.is_admitted());
        assert!(!DenialPolarity::PRECONDITION_FAILED.is_admitted());
        assert!(!DenialPolarity::SLA_BREACH.is_admitted());
        assert!(!DenialPolarity::AUTHORIZATION_DENIED.is_admitted());
        assert!(!DenialPolarity::RESOURCE_EXHAUSTED.is_admitted());
        assert!(!DenialPolarity::OBJECT_LIFECYCLE_VIOLATION.is_admitted());
        assert!(!DenialPolarity::CONFORMANCE_GATE_FAILED.is_admitted());
    }

    #[test]
    fn compose_is_or() {
        let combined = DenialPolarity::PRECONDITION_FAILED
            .compose(DenialPolarity::SLA_BREACH);
        assert_eq!(
            combined.0,
            DenialPolarity::PRECONDITION_FAILED.0 | DenialPolarity::SLA_BREACH.0
        );
        assert!(!combined.is_admitted());
    }

    #[test]
    fn compose_admitted_identity() {
        let x = DenialPolarity::RESOURCE_EXHAUSTED;
        assert_eq!(x.compose(DenialPolarity::ADMITTED), x);
        assert_eq!(DenialPolarity::ADMITTED.compose(x), x);
    }

    #[test]
    fn to_fired_mask_admitted_is_zero() {
        assert_eq!(DenialPolarity::ADMITTED.to_fired_mask(), 0);
    }

    #[test]
    fn to_fired_mask_each_lane_distinct_bit() {
        // Each constant maps to exactly one distinct bit.
        let cases: &[(DenialPolarity, u64)] = &[
            (DenialPolarity::PRECONDITION_FAILED,        1 << 1),
            (DenialPolarity::SLA_BREACH,                 1 << 2),
            (DenialPolarity::AUTHORIZATION_DENIED,       1 << 3),
            (DenialPolarity::RESOURCE_EXHAUSTED,         1 << 4),
            (DenialPolarity::OBJECT_LIFECYCLE_VIOLATION, 1 << 5),
            (DenialPolarity::CONFORMANCE_GATE_FAILED,    1 << 6),
            (DenialPolarity::WATCHDOG_DRAINED,           1 << 7),
        ];
        for (polarity, expected_bit) in cases {
            let mask = polarity.to_fired_mask();
            assert_eq!(
                mask, *expected_bit,
                "DenialPolarity({:#018x}).to_fired_mask() = {mask:#018x}, want {expected_bit:#018x}",
                polarity.0
            );
        }
    }

    #[test]
    fn to_fired_mask_combined_sets_multiple_bits() {
        let combined = DenialPolarity::PRECONDITION_FAILED
            .compose(DenialPolarity::WATCHDOG_DRAINED);
        let mask = combined.to_fired_mask();
        // Both bits 1 and 7 must be set, nothing else.
        assert_eq!(mask, (1 << 1) | (1 << 7));
    }

    #[test]
    fn to_fired_mask_all_lanes() {
        let all = DenialPolarity::PRECONDITION_FAILED
            .compose(DenialPolarity::SLA_BREACH)
            .compose(DenialPolarity::AUTHORIZATION_DENIED)
            .compose(DenialPolarity::RESOURCE_EXHAUSTED)
            .compose(DenialPolarity::OBJECT_LIFECYCLE_VIOLATION)
            .compose(DenialPolarity::CONFORMANCE_GATE_FAILED)
            .compose(DenialPolarity::WATCHDOG_DRAINED);
        // Bits 1..=7 must all be set; bit 0 (ADMITTED lane) stays clear.
        let expected: u64 = 0b1111_1110;
        assert_eq!(all.to_fired_mask(), expected);
    }
}
