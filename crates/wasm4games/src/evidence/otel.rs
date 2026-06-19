//! Runtime span codes (OpenTelemetry-style), kept as 16-bit codes so hot paths never
//! carry strings. Names are resolved to text only at the boundary.

/// A 16-bit span code identifying a runtime operation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpanCode(pub u16);

impl SpanCode {
    /// Base offset for per-pattern span codes.
    ///
    /// The low range below this is reserved for the fixed lifecycle markers in [`span`]
    /// (`TICK`, `INPUT_ADMIT`, `RECEIPT_APPEND`); per-pattern codes live above it so the two
    /// namespaces never collide.
    pub const PATTERN_BASE: u16 = 0x1000;

    /// The raw `u16` code.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// The canonical span code for a pattern, derived from its `pattern_id`.
    ///
    /// Computed as [`Self::PATTERN_BASE`] `+ pattern_id`, keeping per-pattern codes in their
    /// own contiguous block above the reserved lifecycle markers. The addition saturates so a
    /// near-`u16::MAX` id can never wrap back into the lifecycle range.
    ///
    /// # Examples
    /// ```
    /// use wasm4games::evidence::otel::SpanCode;
    /// assert_eq!(SpanCode::for_pattern(0).raw(), SpanCode::PATTERN_BASE);
    /// assert_eq!(SpanCode::for_pattern(5).raw(), SpanCode::PATTERN_BASE + 5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn for_pattern(pattern_id: u16) -> SpanCode {
        SpanCode(Self::PATTERN_BASE.saturating_add(pattern_id))
    }
}

/// Canonical span codes. One lifecycle marker plus per-pattern codes that match each
/// pattern's `otel_span` in [`crate::patterns::PATTERN_REGISTRY`].
pub mod span {
    /// A fixed-step authority tick advanced.
    pub const TICK: u16 = 0x0001;
    /// An input was admitted or refused.
    pub const INPUT_ADMIT: u16 = 0x0002;
    /// A receipt was appended to a chain.
    pub const RECEIPT_APPEND: u16 = 0x0003;
}
