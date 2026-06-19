//! Runtime span codes (OpenTelemetry-style), kept as 16-bit codes so hot paths never
//! carry strings. Names are resolved to text only at the boundary.

/// A 16-bit span code identifying a runtime operation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpanCode(pub u16);

impl SpanCode {
    /// The raw `u16` code.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
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
