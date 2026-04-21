//! # UBitCapability — Bounded authority tokens for the U_{1,262144} operating substrate.
//!
//! Each `UBitCapability` carries a 64-bit transition mask: bit `i` set means transition `i`
//! is permitted. `cap_admit` uses the same admission polarity as `cell_admit`:
//! **0 = admitted, nonzero = denied**.
//!
//! ```
//! use bcinr_logic::patterns::universe64::ubit_capability::{UBitCapabilityTable, NullCapability, cap_admit};
//! let all = NullCapability::all_allowed();
//! assert_eq!(cap_admit(0, all), 0);
//! assert_eq!(cap_admit(63, all), 0);
//! ```

#![allow(dead_code)]
use super::scratch::UScope;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const UBIT_CAPABILITY_TABLE_CAPACITY: usize = 64;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single authority token. `allowed_transitions` is a 64-bit bitmask:
/// bit `i` set means transition `i` is permitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct UBitCapability {
    pub allowed_transitions: u64,
    pub scope: UScope,
    pub expiry: u32,
    pub receipt_tag: u8,
    pub valid: bool,
}

/// Opaque index into a `UBitCapabilityTable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UBitCapabilityId(pub u8);

/// Fixed-capacity table of capability tokens.
#[derive(Clone, Copy)]
pub struct UBitCapabilityTable {
    pub caps: [UBitCapability; UBIT_CAPABILITY_TABLE_CAPACITY],
    pub count: u8,
}

/// Zero-size marker providing convenience constructors for capability masks.
pub struct NullCapability;

// ---------------------------------------------------------------------------
// impl UBitCapability
// ---------------------------------------------------------------------------

impl UBitCapability {
    /// Create a valid capability with the given `allowed_transitions` mask.
    #[inline(always)]
    pub const fn new(allowed_transitions: u64, scope: UScope, expiry: u32) -> Self {
        Self {
            allowed_transitions,
            scope,
            expiry,
            receipt_tag: 0,
            valid: true,
        }
    }

    /// Create an expired capability (`expiry = 0`).
    #[inline(always)]
    pub const fn expired() -> Self {
        Self {
            allowed_transitions: 0,
            scope: UScope::Cell,
            expiry: 0,
            receipt_tag: 0,
            valid: false,
        }
    }

    /// Deny-all capability: no transitions permitted, valid but zero mask.
    #[inline(always)]
    pub const fn deny_all() -> Self {
        Self {
            allowed_transitions: 0,
            scope: UScope::Cell,
            expiry: 1,
            receipt_tag: 0,
            valid: true,
        }
    }

    /// Returns `true` when `expiry == 0` (capability has expired).
    #[inline(always)]
    pub const fn is_expired(self) -> bool {
        self.expiry == 0
    }
}

// ---------------------------------------------------------------------------
// impl NullCapability
// ---------------------------------------------------------------------------

impl NullCapability {
    /// Returns `!0u64` — all 64 transition bits allowed.
    #[inline(always)]
    pub const fn all_allowed() -> u64 {
        !0u64
    }
}

// ---------------------------------------------------------------------------
// impl UBitCapabilityTable
// ---------------------------------------------------------------------------

impl UBitCapabilityTable {
    /// Construct an empty table.
    pub const fn new() -> Self {
        Self {
            caps: [UBitCapability {
                allowed_transitions: 0,
                scope: UScope::Cell,
                expiry: 0,
                receipt_tag: 0,
                valid: false,
            }; UBIT_CAPABILITY_TABLE_CAPACITY],
            count: 0,
        }
    }

    /// Insert a capability, returning its `UBitCapabilityId`.
    /// Returns `None` when the table is full.
    pub fn insert(&mut self, cap: UBitCapability) -> Option<UBitCapabilityId> {
        if self.count as usize >= UBIT_CAPABILITY_TABLE_CAPACITY {
            return None;
        }
        let id = self.count;
        self.caps[id as usize] = cap;
        self.count += 1;
        Some(UBitCapabilityId(id))
    }

    /// Lookup a capability by `id` (bounds-checked).
    pub fn get(&self, id: UBitCapabilityId) -> Option<&UBitCapability> {
        if (id.0 as usize) < self.count as usize {
            Some(&self.caps[id.0 as usize])
        } else {
            None
        }
    }
}

impl Default for UBitCapabilityTable {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// cap_admit — branchless capability gate
// ---------------------------------------------------------------------------

/// Branchless capability check for a single transition.
///
/// Returns `0` when `transition_id < 64` AND bit `transition_id` is set in `cap_mask`.
/// Returns nonzero (denied) otherwise — same polarity as `cell_admit`.
///
/// ```
/// use bcinr_logic::patterns::universe64::ubit_capability::{cap_admit, NullCapability};
/// assert_eq!(cap_admit(0, NullCapability::all_allowed()), 0);
/// assert_eq!(cap_admit(0, 0u64), !0u64); // denied
/// ```
#[inline(always)]
pub fn cap_admit(transition_id: u32, cap_mask: u64) -> u64 {
    // In-range check: transition_id must be < 64 (i.e., bits [31:6] == 0).
    let in_range = ((transition_id >> 6) == 0) as u64; // 1 if in range, 0 otherwise
    let bit = (cap_mask >> (transition_id & 63)) & 1;
    let allowed = in_range & bit;
    // allowed == 1 → admitted (return 0); allowed == 0 → denied (return !0)
    let denied = allowed ^ 1;
    denied.wrapping_neg()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubit_capability_allows_configured_transition() {
        // Bit 5 set in mask → transition 5 should be admitted (cap_admit returns 0).
        let mask = 1u64 << 5;
        assert_eq!(cap_admit(5, mask), 0, "transition 5 should be admitted");
    }

    #[test]
    fn ubit_capability_denies_unset_transition() {
        // Bit 5 set but transition 7 not set → denied.
        let mask = 1u64 << 5;
        assert_ne!(cap_admit(7, mask), 0, "transition 7 should be denied");
    }

    #[test]
    fn ubit_capability_invalid_denies() {
        // A capability with valid=false should result in denial.
        // We simulate by using its allowed_transitions (0 for deny_all / expired).
        let cap = UBitCapability {
            allowed_transitions: 0,
            scope: UScope::Cell,
            expiry: 1,
            receipt_tag: 0,
            valid: false,
        };
        // cap_admit with mask 0 denies every transition.
        assert_ne!(
            cap_admit(0, cap.allowed_transitions),
            0,
            "invalid (zero-mask) cap must deny all transitions"
        );
    }

    #[test]
    fn ubit_capability_expired_denies() {
        let cap = UBitCapability::expired();
        // expired() sets expiry = 0.
        assert!(cap.is_expired(), "expired() must set is_expired() == true");
        // The allowed_transitions of an expired cap is 0 (deny-all mask).
        assert_ne!(
            cap_admit(0, cap.allowed_transitions),
            0,
            "expired cap must deny all transitions"
        );
    }

    #[test]
    fn ubit_capability_null_allows_all() {
        let all = NullCapability::all_allowed();
        // All 64 bit positions must be admitted.
        for tid in 0u32..64 {
            assert_eq!(
                cap_admit(tid, all),
                0,
                "NullCapability must admit transition {tid}"
            );
        }
        // transition_id >= 64 must be denied even with all-allowed mask.
        assert_ne!(cap_admit(64, all), 0, "out-of-range transition must be denied");
    }
}
