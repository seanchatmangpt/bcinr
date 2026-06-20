//! Byte-class authority: bounded `u8` state alphabets and the admission status vocabulary.
//!
//! The whole crate operates on small, bounded `u8` domains so that kernels remain
//! branchless and SIMD-friendly. Joining and clamping lower onto [`bcinr_logic`]
//! primitives ([`bcinr_logic::mask::max_u32`], [`bcinr_logic::fix::clamp_u32`]) rather
//! than `if`/`match`.
//!
//! # Status lattice
//!
//! [`status`] constants form a total order whose numeric value is the "severity" of the
//! observation. This lets a branchless `max` compute the worst-of join across an event
//! window. Terminal/abnormal codes (`REFUSED`, `RESIDUAL`) sort above the normal lifecycle
//! so they dominate a join.
//!
//! ```text
//! UNKNOWN(0) < BLOCKED(1) < PARTIAL(2) < PENDING(3) < ADMITTED(4)
//!           < PROJECTED(5) < RECEIPTED(6) < REFUSED(7) < RESIDUAL(8)
//! ```

use bcinr_logic::fix;
use bcinr_logic::mask;

/// The admission status vocabulary, as ordered `u8` codes.
///
/// Ordering is meaningful: it forms a coarse "admission lattice" so a worst-of join can
/// be computed with a branchless `max`. Terminal/abnormal codes (`REFUSED`, `RESIDUAL`)
/// sort above the normal lifecycle so they dominate a join.
///
/// All constants are in `[0, COUNT)`. `COUNT` itself is not a valid status code; it is
/// the cardinality of the vocabulary for bounds-checking.
pub mod status {
    /// Nothing is known yet. The zero/default state.
    pub const UNKNOWN: u8 = 0;
    /// A precondition prevented progress.
    pub const BLOCKED: u8 = 1;
    /// Partially observed; not yet admissible.
    pub const PARTIAL: u8 = 2;
    /// Awaiting a downstream decision.
    pub const PENDING: u8 = 3;
    /// Admitted into bounded state.
    pub const ADMITTED: u8 = 4;
    /// Projected to an engine/host surface.
    pub const PROJECTED: u8 = 5;
    /// Sealed into a receipt chain.
    pub const RECEIPTED: u8 = 6;
    /// Refused by an admission rule.
    pub const REFUSED: u8 = 7;
    /// Residual diagnostic preserved after repair.
    pub const RESIDUAL: u8 = 8;
    /// Cardinality of the vocabulary (number of distinct codes).
    ///
    /// All valid status codes satisfy `code < COUNT`.
    pub const COUNT: u8 = 9;

    /// Return the human-readable name of a status code, or `"?"` for out-of-range values.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::status;
    /// assert_eq!(status::name(status::ADMITTED), "ADMITTED");
    /// assert_eq!(status::name(99), "?");
    /// ```
    #[must_use]
    pub const fn name(code: u8) -> &'static str {
        match code {
            UNKNOWN => "UNKNOWN",
            BLOCKED => "BLOCKED",
            PARTIAL => "PARTIAL",
            PENDING => "PENDING",
            ADMITTED => "ADMITTED",
            PROJECTED => "PROJECTED",
            RECEIPTED => "RECEIPTED",
            REFUSED => "REFUSED",
            RESIDUAL => "RESIDUAL",
            _ => "?",
        }
    }

    /// Return `true` if `code` is a member of the vocabulary (`code < COUNT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::status;
    /// assert!(status::is_valid(status::REFUSED));
    /// assert!(!status::is_valid(status::COUNT));
    /// assert!(!status::is_valid(99));
    /// ```
    #[must_use]
    pub const fn is_valid(code: u8) -> bool {
        code < COUNT
    }
}

/// A bounded byte class: a `u8` drawn from a domain of known cardinality.
///
/// `ByteClass` is a newtype over `u8` that makes the bounded domain explicit.
/// The `Default` implementation returns `ByteClass(0)`, which is the first
/// (lowest) class in any alphabet.
///
/// # Examples
///
/// ```
/// use wasm4games::class::{ByteClass, status};
///
/// // Clamp an out-of-range value into the status vocabulary
/// let raw = ByteClass(200);
/// let clamped = raw.clamp(status::COUNT);
/// assert_eq!(clamped.raw(), status::COUNT - 1);
///
/// // Within-range values pass through unchanged
/// assert_eq!(ByteClass(3).clamp(status::COUNT).raw(), 3);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ByteClass(pub u8);

impl ByteClass {
    /// Construct a [`ByteClass`] from a raw `u8` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::ByteClass;
    /// assert_eq!(ByteClass::new(5).raw(), 5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(code: u8) -> Self {
        Self(code)
    }

    /// The raw `u8` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::ByteClass;
    /// assert_eq!(ByteClass(7).raw(), 7);
    /// ```
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Return `true` if this class is within the domain `[0, card)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{ByteClass, status};
    /// assert!(ByteClass(3).is_in_domain(status::COUNT));
    /// assert!(!ByteClass(200).is_in_domain(status::COUNT));
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_in_domain(self, card: u8) -> bool {
        self.0 < card
    }

    /// Branchlessly clamp this class into the domain `[0, card)`.
    ///
    /// A `card` of `0` is treated as `1` (the domain `{0}`), so the result is always
    /// `0` in that degenerate case. This matches the semantics of
    /// [`bcinr_logic::fix::clamp_u32`] which is used for the implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{ByteClass, status};
    ///
    /// assert_eq!(ByteClass(200).clamp(status::COUNT).raw(), status::COUNT - 1);
    /// assert_eq!(ByteClass(3).clamp(status::COUNT).raw(), 3);
    /// assert_eq!(ByteClass(5).clamp(0).raw(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn clamp(self, card: u8) -> ByteClass {
        let hi = card.saturating_sub(1) as u32;
        ByteClass(fix::clamp_u32(self.0 as u32, 0, hi) as u8)
    }
}

impl core::fmt::Display for ByteClass {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ByteClass({})", self.0)
    }
}

/// An admission [`status`] code as a typed value.
///
/// `Status` is a newtype over `u8` that wraps a code from the [`status`] vocabulary.
/// The `Default` implementation returns `Status(0)` which corresponds to
/// [`status::UNKNOWN`] — the "nothing known yet" state.
///
/// # Lattice semantics
///
/// Status codes form a total order. The [`join`](Self::join) method computes the
/// worst-of (max-code) combination of two statuses branchlessly using
/// [`bcinr_logic::mask::max_u32`], so an `ADMITTED` joined with `REFUSED` yields
/// `REFUSED` without any branch.
///
/// # Examples
///
/// ```
/// use wasm4games::class::{Status, status};
///
/// let a = Status::new(status::ADMITTED);
/// let b = Status::new(status::REFUSED);
/// assert_eq!(a.join(b).raw(), status::REFUSED);
/// assert_eq!(a.to_string(), "Status(ADMITTED)");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Status(pub u8);

impl Status {
    /// The `UNKNOWN` status: nothing is known yet. This is also the [`Default`] value.
    pub const UNKNOWN: Self = Self(status::UNKNOWN);
    /// The `BLOCKED` status: a precondition prevented progress.
    pub const BLOCKED: Self = Self(status::BLOCKED);
    /// The `PARTIAL` status: partially observed; not yet admissible.
    pub const PARTIAL: Self = Self(status::PARTIAL);
    /// The `PENDING` status: awaiting a downstream decision.
    pub const PENDING: Self = Self(status::PENDING);
    /// The `ADMITTED` status: admitted into bounded state.
    pub const ADMITTED: Self = Self(status::ADMITTED);
    /// The `PROJECTED` status: projected to an engine/host surface.
    pub const PROJECTED: Self = Self(status::PROJECTED);
    /// The `RECEIPTED` status: sealed into a receipt chain.
    pub const RECEIPTED: Self = Self(status::RECEIPTED);
    /// The `REFUSED` status: refused by an admission rule.
    pub const REFUSED: Self = Self(status::REFUSED);
    /// The `RESIDUAL` status: residual diagnostic preserved after repair.
    pub const RESIDUAL: Self = Self(status::RESIDUAL);

    /// Construct a [`Status`] from a raw `u8` code.
    ///
    /// The value is not validated against [`status::COUNT`]; use
    /// [`is_valid`](Self::is_valid) if you need a bounds check.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{Status, status};
    /// assert_eq!(Status::new(status::ADMITTED).raw(), status::ADMITTED);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(code: u8) -> Self {
        Self(code)
    }

    /// Return the raw `u8` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{Status, status};
    /// assert_eq!(Status::new(status::REFUSED).raw(), 7);
    /// ```
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Return `true` if this status code is in the vocabulary (`self.0 < status::COUNT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{Status, status};
    /// assert!(Status::new(status::RESIDUAL).is_valid());
    /// assert!(!Status::new(status::COUNT).is_valid());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        status::is_valid(self.0)
    }

    /// Return the human-readable name of this status code.
    ///
    /// Delegates to [`status::name`]. Returns `"?"` for out-of-vocabulary codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{Status, status};
    /// assert_eq!(Status::ADMITTED.status_name(), "ADMITTED");
    /// assert_eq!(Status::new(99).status_name(), "?");
    /// ```
    #[inline]
    #[must_use]
    pub const fn status_name(self) -> &'static str {
        status::name(self.0)
    }

    /// Worst-of (max-code) lattice join, computed branchlessly.
    ///
    /// Uses [`bcinr_logic::mask::max_u32`] so there is no branch in the machine code.
    /// Because status codes are ordered by severity, this always returns the "worse"
    /// of the two statuses.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::class::{Status, status};
    ///
    /// assert_eq!(
    ///     Status::new(status::ADMITTED).join(Status::new(status::REFUSED)).raw(),
    ///     status::REFUSED,
    /// );
    /// assert_eq!(
    ///     Status::new(status::UNKNOWN).join(Status::new(status::ADMITTED)).raw(),
    ///     status::ADMITTED,
    /// );
    /// // Join is commutative
    /// assert_eq!(
    ///     Status::PARTIAL.join(Status::REFUSED),
    ///     Status::REFUSED.join(Status::PARTIAL),
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub fn join(self, other: Status) -> Status {
        Status(mask::max_u32(self.0 as u32, other.0 as u32) as u8)
    }
}

impl core::fmt::Display for Status {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Status({})", status::name(self.0))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;

    #[test]
    fn status_vocabulary_and_byte_class() {
        // Lattice order
        assert!(status::UNKNOWN < status::BLOCKED);
        assert!(status::BLOCKED < status::PARTIAL);
        assert!(status::PARTIAL < status::PENDING);
        assert!(status::PENDING < status::ADMITTED);
        assert!(status::ADMITTED < status::PROJECTED);
        assert!(status::PROJECTED < status::RECEIPTED);
        assert!(status::RECEIPTED < status::REFUSED);
        assert!(status::REFUSED < status::RESIDUAL);
        assert!(status::RESIDUAL < status::COUNT);

        // is_valid
        for code in 0..status::COUNT {
            assert!(status::is_valid(code), "code {code} should be valid");
        }
        assert!(!status::is_valid(status::COUNT));
        assert!(!status::is_valid(99));

        // name table
        let expected: &[(u8, &str)] = &[
            (status::UNKNOWN, "UNKNOWN"),
            (status::BLOCKED, "BLOCKED"),
            (status::PARTIAL, "PARTIAL"),
            (status::PENDING, "PENDING"),
            (status::ADMITTED, "ADMITTED"),
            (status::PROJECTED, "PROJECTED"),
            (status::RECEIPTED, "RECEIPTED"),
            (status::REFUSED, "REFUSED"),
            (status::RESIDUAL, "RESIDUAL"),
        ];
        for &(code, name) in expected {
            assert_eq!(status::name(code), name, "wrong name for code {code}");
        }
        assert_eq!(status::name(99), "?");

        // ByteClass: clamp, is_in_domain, Display, default, Ord, raw
        assert_eq!(ByteClass(200).clamp(status::COUNT).raw(), status::COUNT - 1);
        assert_eq!(ByteClass(3).clamp(status::COUNT).raw(), 3);
        assert_eq!(ByteClass(5).clamp(0).raw(), 0);
        assert_eq!(ByteClass(8).clamp(9).raw(), 8);
        assert_eq!(ByteClass(9).clamp(9).raw(), 8);
        assert!(ByteClass(0).is_in_domain(status::COUNT));
        assert!(ByteClass(8).is_in_domain(status::COUNT));
        assert!(!ByteClass(9).is_in_domain(status::COUNT));
        assert!(!ByteClass(200).is_in_domain(status::COUNT));
        assert_eq!(ByteClass(3).to_string(), "ByteClass(3)");
        assert_eq!(ByteClass::default().raw(), 0);
        assert_eq!(ByteClass::default(), ByteClass::new(0));
        assert!(ByteClass(0) < ByteClass(1));
        let mut classes = [ByteClass(5), ByteClass(1), ByteClass(3)];
        classes.sort();
        assert_eq!(classes, [ByteClass(1), ByteClass(3), ByteClass(5)]);
        for v in [0u8, 1, 8, 127, 255] {
            assert_eq!(ByteClass::new(v).raw(), v);
        }
    }

    #[test]
    fn status_typed_behavior() {
        // join: worst-of, commutative, idempotent
        assert_eq!(
            Status(status::ADMITTED).join(Status(status::REFUSED)).raw(),
            status::REFUSED
        );
        assert_eq!(
            Status(status::UNKNOWN).join(Status(status::ADMITTED)).raw(),
            status::ADMITTED
        );
        for a in 0..status::COUNT {
            for b in 0..status::COUNT {
                assert_eq!(
                    Status(a).join(Status(b)),
                    Status(b).join(Status(a)),
                    "join not commutative for ({a},{b})"
                );
            }
        }
        for code in 0..status::COUNT {
            let s = Status(code);
            assert_eq!(s.join(s), s, "join not idempotent for {code}");
        }

        // Display
        assert_eq!(
            Status::new(status::ADMITTED).to_string(),
            "Status(ADMITTED)"
        );
        assert_eq!(Status::new(status::UNKNOWN).to_string(), "Status(UNKNOWN)");
        assert_eq!(Status::new(status::REFUSED).to_string(), "Status(REFUSED)");
        assert_eq!(
            Status::new(status::RESIDUAL).to_string(),
            "Status(RESIDUAL)"
        );
        assert_eq!(Status::new(99).to_string(), "Status(?)");

        // default, const associated values, is_valid, status_name
        assert_eq!(Status::default().raw(), status::UNKNOWN);
        assert_eq!(Status::default(), Status::UNKNOWN);
        assert_eq!(Status::UNKNOWN.raw(), status::UNKNOWN);
        assert_eq!(Status::REFUSED.raw(), status::REFUSED);
        assert_eq!(Status::RESIDUAL.raw(), status::RESIDUAL);
        for code in 0..status::COUNT {
            assert!(Status::new(code).is_valid(), "code {code} should be valid");
        }
        assert!(!Status::new(status::COUNT).is_valid());
        assert_eq!(Status::ADMITTED.status_name(), "ADMITTED");
        assert_eq!(Status::new(99).status_name(), "?");

        // Ord + BTreeMap key
        assert!(Status::UNKNOWN < Status::BLOCKED);
        assert!(Status::REFUSED < Status::RESIDUAL);
        let mut statuses = [Status::REFUSED, Status::UNKNOWN, Status::ADMITTED];
        statuses.sort();
        assert_eq!(
            statuses,
            [Status::UNKNOWN, Status::ADMITTED, Status::REFUSED]
        );
        let mut map = BTreeMap::new();
        map.insert(Status::ADMITTED, "ok");
        map.insert(Status::REFUSED, "no");
        assert_eq!(map[&Status::ADMITTED], "ok");
        assert_eq!(map[&Status::REFUSED], "no");
    }
}
