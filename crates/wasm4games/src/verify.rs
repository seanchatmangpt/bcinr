//! In-crate self-checks: an *offline* admissibility proxy.
//!
//! These are NOT the authority. The external `wasm4pm` performs real admission/refusal
//! (see [`crate::compat`]). Offline, these registry-driven checks give fast, dependency-free
//! confidence: status codes stay in-bounds, replays are deterministic, and negative fixtures
//! are constructed so a real authority can be tested for refusal.

use crate::class::status;
use crate::evidence::ocel::OcelEvent;
use crate::evidence::replay::ReplayFrame;
use crate::ir::PatternSpec;
use bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt;

/// Check that an observed status code is within the known vocabulary.
#[inline]
#[must_use]
pub fn check_status_bounds(_spec: &PatternSpec, observed: u8) -> bool {
    observed < status::COUNT
}

/// Fold replay frames into a rolling digest.
#[must_use]
pub fn replay_digest(frames: &[ReplayFrame]) -> u64 {
    let mut r = DeterministicSubstrateReceipt::new();
    for f in frames {
        r.record(f.tick, f.input, f.state_digest);
    }
    r.finalize()
}

/// Check that replaying the same frames reproduces the same digest (determinism).
#[must_use]
pub fn check_replay_determinism(frames: &[ReplayFrame]) -> bool {
    replay_digest(frames) == replay_digest(frames)
}

/// Construct a negative fixture for a pattern: an event whose status is the pattern's
/// refusal code, which a correct authority MUST refuse.
#[must_use]
pub fn negative_fixture(spec: &PatternSpec) -> OcelEvent {
    OcelEvent::new(spec.event.code, spec.id.0, 0, spec.admission.refusal_status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::PATTERN_REGISTRY;

    #[test]
    fn registry_specs_are_self_consistent() {
        for spec in PATTERN_REGISTRY {
            // Refusal status must be a known code.
            assert!(check_status_bounds(spec, spec.admission.refusal_status));
            // Negative fixtures must carry the refusal status and link to the activity.
            let ev = negative_fixture(spec);
            assert_eq!(ev.status, spec.admission.refusal_status);
            assert_eq!(ev.activity, spec.id.0);
        }
    }
}
