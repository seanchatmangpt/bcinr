//! Runtime span codes (OpenTelemetry-style), kept as 16-bit codes so hot paths never
//! carry strings. Names are resolved to text only at the boundary.
//!
//! # Design
//!
//! OTel spans are normally identified by strings (`"db.query"`, `"http.request"`, etc.).
//! String allocation on hot paths (every game tick) is unacceptable in a `no_std`
//! deterministic kernel. [`SpanCode`] wraps a `u16` instead; strings are resolved only
//! at export time (e.g. when emitting to a collector).
//!
//! # Namespacing
//!
//! The `u16` code space is split into two non-overlapping regions:
//!
//! * `0x0001..=0x0FFF` — reserved lifecycle markers in the [`span`] sub-module.
//! * `0x1000..=0xFFFF` — per-pattern codes assigned via [`SpanCode::for_pattern`].
//!
//! # `no_std` compatibility
//!
//! [`SpanCode`] is `Copy`, `Hash`, and `Ord`; it carries no heap allocations.

use core::fmt;

/// A 16-bit span code identifying a runtime operation.
///
/// `SpanCode` is an opaque wrapper around a `u16` to prevent accidental mixing of raw
/// integers and typed codes. The two namespaces ([`span`] lifecycle markers and per-pattern
/// codes above [`SpanCode::PATTERN_BASE`]) are kept apart by construction.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::otel::SpanCode;
/// let code = SpanCode::for_pattern(3);
/// assert_eq!(code.raw(), SpanCode::PATTERN_BASE + 3);
/// let code2: SpanCode = 0x1003u16.into();
/// assert_eq!(code, code2);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SpanCode(pub u16);

impl SpanCode {
    /// Base offset for per-pattern span codes.
    ///
    /// The low range below this is reserved for the fixed lifecycle markers in [`span`]
    /// (`TICK`, `INPUT_ADMIT`, `RECEIPT_APPEND`); per-pattern codes live above it so the two
    /// namespaces never collide.
    pub const PATTERN_BASE: u16 = 0x1000;

    /// The raw `u16` code.
    ///
    /// Use this when crossing the FFI boundary or writing to a wire format that expects a
    /// plain integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::otel::SpanCode;
    /// let code = SpanCode(0x0001);
    /// assert_eq!(code.raw(), 0x0001);
    /// ```
    #[inline]
    #[must_use = "returns the raw u16 code; ignoring it is likely a bug"]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Whether this code falls in the lifecycle-marker range (`< PATTERN_BASE`).
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::otel::{SpanCode, span};
    /// assert!(SpanCode(span::TICK).is_lifecycle());
    /// assert!(!SpanCode::for_pattern(0).is_lifecycle());
    /// ```
    #[inline]
    #[must_use = "returns true if this is a reserved lifecycle span code"]
    pub const fn is_lifecycle(self) -> bool {
        self.0 < Self::PATTERN_BASE
    }

    /// Whether this code falls in the per-pattern range (`>= PATTERN_BASE`).
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::otel::SpanCode;
    /// assert!(SpanCode::for_pattern(7).is_pattern());
    /// ```
    #[inline]
    #[must_use = "returns true if this is a per-pattern span code"]
    pub const fn is_pattern(self) -> bool {
        self.0 >= Self::PATTERN_BASE
    }

    /// The canonical span code for a pattern, derived from its `pattern_id`.
    ///
    /// Computed as [`Self::PATTERN_BASE`] `+ pattern_id`, keeping per-pattern codes in their
    /// own contiguous block above the reserved lifecycle markers. The addition saturates so a
    /// near-`u16::MAX` id can never wrap back into the lifecycle range.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::otel::SpanCode;
    /// assert_eq!(SpanCode::for_pattern(0).raw(), SpanCode::PATTERN_BASE);
    /// assert_eq!(SpanCode::for_pattern(5).raw(), SpanCode::PATTERN_BASE + 5);
    /// // Saturates — never wraps into lifecycle range.
    /// assert!(SpanCode::for_pattern(u16::MAX).is_pattern());
    /// ```
    #[inline]
    #[must_use = "returns the SpanCode for the given pattern id; bind it to use it"]
    pub const fn for_pattern(pattern_id: u16) -> SpanCode {
        SpanCode(Self::PATTERN_BASE.saturating_add(pattern_id))
    }
}

impl fmt::Display for SpanCode {
    /// Formats the span code as `lifecycle:0xNNNN` or `pattern:0xNNNN` depending on namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::otel::{SpanCode, span};
    /// // Display format — exact text depends on namespace.
    /// let lifecycle = SpanCode(span::TICK);
    /// let pattern = SpanCode::for_pattern(2);
    /// let ls = format!("{}", lifecycle);
    /// let ps = format!("{}", pattern);
    /// assert!(ls.starts_with("lifecycle:"));
    /// assert!(ps.starts_with("pattern:"));
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_lifecycle() {
            write!(f, "lifecycle:{:#06x}", self.0)
        } else {
            write!(f, "pattern:{:#06x}", self.0)
        }
    }
}

/// Convert a raw `u16` into a [`SpanCode`].
///
/// This is a lossless conversion; the raw value is preserved exactly.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::otel::SpanCode;
/// let code: SpanCode = 0x0002u16.into();
/// assert_eq!(code.raw(), 0x0002);
/// ```
impl From<u16> for SpanCode {
    #[inline]
    fn from(raw: u16) -> Self {
        SpanCode(raw)
    }
}

/// Extract the raw `u16` from a [`SpanCode`].
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::otel::SpanCode;
/// let raw: u16 = SpanCode::for_pattern(10).into();
/// assert_eq!(raw, SpanCode::PATTERN_BASE + 10);
/// ```
impl From<SpanCode> for u16 {
    #[inline]
    fn from(code: SpanCode) -> u16 {
        code.0
    }
}

/// Canonical span codes. One lifecycle marker plus per-pattern codes that match each
/// pattern's `otel_span` in [`crate::patterns::PATTERN_REGISTRY`].
///
/// All constants are `u16` raw values; wrap them in [`SpanCode`] when you need the typed
/// wrapper: `SpanCode(span::TICK)`.
pub mod span {
    /// A fixed-step authority tick advanced.
    pub const TICK: u16 = 0x0001;
    /// An input was admitted or refused.
    pub const INPUT_ADMIT: u16 = 0x0002;
    /// A receipt was appended to a chain.
    pub const RECEIPT_APPEND: u16 = 0x0003;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_code_conversions_and_predicates() {
        // raw / From / Into roundtrips
        assert_eq!(SpanCode(0xABCD).raw(), 0xABCD);
        let raw: u16 = 0x1234;
        let code: SpanCode = raw.into();
        let back: u16 = code.into();
        assert_eq!(back, raw);

        // for_pattern: base offset and saturation
        assert_eq!(SpanCode::for_pattern(0).raw(), SpanCode::PATTERN_BASE);
        assert_eq!(SpanCode::for_pattern(5).raw(), SpanCode::PATTERN_BASE + 5);
        assert_eq!(SpanCode::for_pattern(u16::MAX).raw(), u16::MAX);
        assert!(SpanCode::for_pattern(u16::MAX).is_pattern());

        // lifecycle / pattern predicates
        for &lc in &[span::TICK, span::INPUT_ADMIT, span::RECEIPT_APPEND] {
            assert!(
                SpanCode(lc).is_lifecycle(),
                "expected lifecycle for {:#x}",
                lc
            );
            assert!(
                !SpanCode(lc).is_pattern(),
                "expected not-pattern for {:#x}",
                lc
            );
            assert!(lc < SpanCode::PATTERN_BASE);
        }
        assert!(SpanCode::for_pattern(0).is_pattern());
        assert!(!SpanCode::for_pattern(0).is_lifecycle());

        // Copy + Eq
        let a = SpanCode::for_pattern(7);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn span_code_ordering() {
        assert!(SpanCode(1) < SpanCode(2));
        assert!(SpanCode(2) > SpanCode(1));
        assert_eq!(SpanCode(1), SpanCode(1));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn span_code_display_prefixes() {
        // (raw_value, expected_prefix)
        let cases: &[(u16, &str)] = &[
            (span::TICK, "lifecycle:"),
            (span::INPUT_ADMIT, "lifecycle:"),
            (span::RECEIPT_APPEND, "lifecycle:"),
            (SpanCode::PATTERN_BASE, "pattern:"),
            (SpanCode::for_pattern(3).raw(), "pattern:"),
        ];
        for &(raw, prefix) in cases {
            let s = alloc::format!("{}", SpanCode(raw));
            assert!(s.starts_with(prefix), "got: {}", s);
        }
    }
}
