//! `wasm4pm` reference surface — dependency-free mirror types.
//!
//! `wasm4pm` and `wasm4pm-compat` are *referenced, not vendored*. The types here mirror
//! the canonical `wasm4pm-compat` shapes so that emitted [`crate::evidence`] can be
//! mapped 1:1 by the workspace-excluded `wasm4games-wasm4pm` bridge crate (which depends
//! on the real repos at <https://github.com/seanchatmangpt/wasm4pm> and
//! <https://github.com/seanchatmangpt/wasm4pm-compat>). Keeping these mirrors here means
//! the offline build pulls no git dependencies.

use core::fmt;

/// Mirror of `wasm4pm_compat::EvidenceState`, maintained here for offline operation
/// without the `wasm4pm` git dependency.
///
/// Represents the lifecycle of a piece of evidence, mapping onto the
/// [`crate::class::status`] lattice. The ordering matches the lattice: later variants
/// carry higher status codes and dominate a worst-of join.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvidenceState {
    /// Observed but unprocessed.
    Raw,
    /// Parsed into typed form.
    Parsed,
    /// Admitted into bounded state.
    Admitted,
    /// Projected to a host/engine surface.
    Projected,
    /// Ready for export.
    Exportable,
    /// Sealed into a receipt.
    Receipted,
}

impl EvidenceState {
    /// Map a lifecycle state to its [`crate::class::status`] code.
    #[inline]
    #[must_use]
    pub fn to_status(self) -> u8 {
        use crate::class::status;
        match self {
            EvidenceState::Raw => status::UNKNOWN,
            EvidenceState::Parsed => status::PARTIAL,
            EvidenceState::Admitted => status::ADMITTED,
            EvidenceState::Projected => status::PROJECTED,
            EvidenceState::Exportable => status::PROJECTED,
            EvidenceState::Receipted => status::RECEIPTED,
        }
    }
}

impl fmt::Display for EvidenceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvidenceState::Raw => f.write_str("Raw"),
            EvidenceState::Parsed => f.write_str("Parsed"),
            EvidenceState::Admitted => f.write_str("Admitted"),
            EvidenceState::Projected => f.write_str("Projected"),
            EvidenceState::Exportable => f.write_str("Exportable"),
            EvidenceState::Receipted => f.write_str("Receipted"),
        }
    }
}

/// Mirror of `wasm4pm_compat::ConformanceResult`, maintained here for offline operation
/// without the `wasm4pm` git dependency.
///
/// Represents the outcome of an offline admissibility check. Note that this is **not**
/// the admission authority — the external `wasm4pm` service performs real admission.
/// This type is used for local pre-screening and test fixture construction only.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// Admitted under the active scope.
    Admitted,
    /// Refused with a [`crate::class::status`] refusal code.
    Refused(u8),
    /// Not enough information to decide.
    Unknown,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Admitted => f.write_str("Admitted"),
            Verdict::Refused(code) => write!(f, "Refused(status={})", code),
            Verdict::Unknown => f.write_str("Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class::status;
    use core::fmt::Write as _;

    /// Write a `Display` impl into a stack-allocated byte buffer; returns the filled slice.
    fn display_into<'a>(val: &dyn fmt::Display, buf: &'a mut [u8]) -> &'a str {
        struct BufWriter<'a> {
            buf: &'a mut [u8],
            pos: usize,
        }
        impl fmt::Write for BufWriter<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let bytes = s.as_bytes();
                let remaining = self.buf.len() - self.pos;
                let n = bytes.len().min(remaining);
                self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
                self.pos += n;
                Ok(())
            }
        }
        let mut w = BufWriter { buf, pos: 0 };
        let _ = write!(w, "{}", val);
        let len = w.pos;
        core::str::from_utf8(&w.buf[..len]).unwrap_or("")
    }

    #[test]
    fn evidence_state_to_status_covers_all_variants() {
        assert_eq!(EvidenceState::Raw.to_status(), status::UNKNOWN);
        assert_eq!(EvidenceState::Parsed.to_status(), status::PARTIAL);
        assert_eq!(EvidenceState::Admitted.to_status(), status::ADMITTED);
        assert_eq!(EvidenceState::Projected.to_status(), status::PROJECTED);
        assert_eq!(EvidenceState::Exportable.to_status(), status::PROJECTED);
        assert_eq!(EvidenceState::Receipted.to_status(), status::RECEIPTED);
    }

    #[test]
    fn evidence_state_display() {
        let mut buf = [0u8; 32];
        assert_eq!(display_into(&EvidenceState::Raw, &mut buf), "Raw");
        assert_eq!(display_into(&EvidenceState::Admitted, &mut buf), "Admitted");
        assert_eq!(
            display_into(&EvidenceState::Receipted, &mut buf),
            "Receipted"
        );
    }

    #[test]
    fn verdict_display() {
        let mut buf = [0u8; 32];
        assert_eq!(display_into(&Verdict::Admitted, &mut buf), "Admitted");
        assert_eq!(
            display_into(&Verdict::Refused(7), &mut buf),
            "Refused(status=7)"
        );
        assert_eq!(display_into(&Verdict::Unknown, &mut buf), "Unknown");
    }
}
