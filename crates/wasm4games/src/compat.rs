//! wasm4pm reference surface — dependency-free mirror types.
//!
//! wasm4pm and wasm4pm-compat are *referenced, not vendored*. The types here mirror the
//! canonical `wasm4pm-compat` shapes so that emitted [`crate::evidence`] can be mapped 1:1
//! by the workspace-excluded `wasm4games-wasm4pm` bridge crate (which depends on the real
//! repos at <https://github.com/seanchatmangpt/wasm4pm> and
//! <https://github.com/seanchatmangpt/wasm4pm-compat>). Keeping these mirrors here means
//! the offline build pulls no git dependencies.

/// Lifecycle of a piece of evidence, mirroring `wasm4pm-compat`'s `Evidence<T, State, W>`
/// state set. Maps onto the [`crate::class::status`] lattice.
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

/// A conformance verdict mirroring `wasm4pm-compat`'s `ConformanceResult` in spirit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// Admitted under the active scope.
    Admitted,
    /// Refused with a [`crate::class::status`] refusal code.
    Refused(u8),
    /// Not enough information to decide.
    Unknown,
}
