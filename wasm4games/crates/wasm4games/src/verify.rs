//! In-crate self-checks: an *offline* admissibility proxy.
//!
//! # What This Module Can and Cannot Verify
//!
//! **Can verify (offline):**
//! - Status codes are in-bounds for the known [`crate::class::status`] vocabulary.
//! - Replay frames produce a deterministic digest when folded twice with the same input.
//! - Negative fixtures carry the correct refusal status for their pattern.
//! - All items in [`crate::patterns::PATTERN_REGISTRY`] are internally self-consistent.
//!
//! **Cannot verify (requires `wasm4pm`):**
//! - Whether a real authority would admit or refuse a given event.
//! - Receipt chain integrity across process boundaries.
//! - Scope-level conformance rules from the `wasm4pm-compat` ontology.
//!
//! **This module is NOT the admission authority.** The external `wasm4pm` service
//! performs real admission/refusal (see [`crate::compat`]). Offline checks here give
//! fast, dependency-free confidence that the registry is self-consistent and that
//! individual pattern kernels behave deterministically, but they do not substitute for
//! a live `wasm4pm` conformance check.

use crate::class::status;
use crate::evidence::ocel::OcelEvent;
use crate::evidence::replay::ReplayFrame;
use crate::ir::PatternSpec;
use bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt;

/// A structured report from a single offline validation pass.
///
/// Collects the results of all checks performed by [`validate_pattern`] so that callers
/// can inspect each finding individually rather than receiving a bare `bool`. Fields are
/// `true` when the check passed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ValidationReport {
    /// The pattern id that was checked.
    pub pattern_id: u16,
    /// `true` if the required status code is within the known vocabulary.
    pub required_status_in_bounds: bool,
    /// `true` if the refusal status code is within the known vocabulary.
    pub refusal_status_in_bounds: bool,
    /// `true` if the negative fixture carries the correct refusal status.
    pub negative_fixture_status_correct: bool,
    /// `true` if the negative fixture links to the correct activity id.
    pub negative_fixture_activity_correct: bool,
}

impl ValidationReport {
    /// `true` when all checks in this report passed.
    #[inline]
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.required_status_in_bounds
            && self.refusal_status_in_bounds
            && self.negative_fixture_status_correct
            && self.negative_fixture_activity_correct
    }
}

/// Run every offline check for a single pattern spec and return a [`ValidationReport`].
///
/// # Must Use
///
/// Ignoring the report silently discards potential conformance failures.
#[must_use = "offline validation result — ignoring may miss conformance failures"]
pub fn validate_pattern(spec: &PatternSpec) -> ValidationReport {
    let nf = negative_fixture(spec);
    ValidationReport {
        pattern_id: spec.id.0,
        required_status_in_bounds: check_status_bounds(spec, spec.admission.required_status),
        refusal_status_in_bounds: check_status_bounds(spec, spec.admission.refusal_status),
        negative_fixture_status_correct: nf.status == spec.admission.refusal_status,
        negative_fixture_activity_correct: nf.activity == spec.id.0,
    }
}

/// Check that an observed status code is within the known vocabulary.
///
/// Returns `true` if `observed < `[`crate::class::status::COUNT`].
#[inline]
#[must_use = "offline validation result — ignoring may miss conformance failures"]
pub fn check_status_bounds(_spec: &PatternSpec, observed: u8) -> bool {
    observed < status::COUNT
}

/// Fold replay frames into a rolling digest.
///
/// The digest is deterministic: identical frames always produce the same value.
/// Use [`check_replay_determinism`] to verify this property at runtime.
#[must_use = "offline validation result — ignoring may miss conformance failures"]
pub fn replay_digest(frames: &[ReplayFrame]) -> u64 {
    let mut r = DeterministicSubstrateReceipt::new();
    for f in frames {
        r.record(f.tick, f.input, f.state_digest);
    }
    r.finalize()
}

/// Check that replaying the same frames reproduces the same digest (determinism).
///
/// A `false` return would indicate a non-deterministic implementation, which violates
/// the core replay-evidence contract.
#[must_use = "offline validation result — ignoring may miss conformance failures"]
pub fn check_replay_determinism(frames: &[ReplayFrame]) -> bool {
    replay_digest(frames) == replay_digest(frames)
}

/// Construct a negative fixture for a pattern: an event whose status is the pattern's
/// refusal code, which a correct authority MUST refuse.
///
/// Use this in integration tests against a live `wasm4pm` instance to verify that it
/// correctly refuses events carrying the refusal status.
#[must_use]
pub fn negative_fixture(spec: &PatternSpec) -> OcelEvent {
    OcelEvent::new(spec.event.code, spec.id.0, 0, spec.admission.refusal_status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::replay::ReplayFrame;
    use crate::ir::{AdmissionRule, EventKind, LoweringKind, ObjectKind, PatternId, PatternSpec};
    use crate::patterns::PATTERN_REGISTRY;

    #[test]
    fn registry_validation_and_replay_determinism() {
        // All registry specs must pass offline validation (status bounds + negative fixtures).
        for spec in PATTERN_REGISTRY {
            assert!(check_status_bounds(spec, spec.admission.refusal_status));
            let ev = negative_fixture(spec);
            assert_eq!(ev.status, spec.admission.refusal_status);
            assert_eq!(ev.activity, spec.id.0);
            let report = validate_pattern(spec);
            assert!(
                report.is_ok(),
                "pattern id={} failed: {:?}",
                spec.id.0,
                report
            );
        }

        // Replay determinism: non-empty and empty frames.
        let frames = [
            ReplayFrame::new(0, 0x0000_0001, 0xDEAD_BEEF),
            ReplayFrame::new(1, 0x0000_0002, 0xCAFE_BABE),
            ReplayFrame::new(2, 0x0000_0003, 0x1234_5678),
        ];
        assert!(check_replay_determinism(&frames));
        assert!(check_replay_determinism(&[]));
    }

    #[test]
    fn out_of_bounds_refusal_status_fails_validation() {
        let bad_spec = PatternSpec {
            id: PatternId(0xFFFF),
            name: "bad_test_pattern",
            lowering: LoweringKind::Mask,
            state_card: 2,
            event: EventKind {
                code: 0x9999,
                name: "BadEvent",
            },
            objects: &[ObjectKind {
                code: 0x0001,
                name: "player",
            }],
            admission: AdmissionRule {
                required_status: 4,
                refusal_status: 255,
            },
            otel_span: 0x9999,
        };
        let report = validate_pattern(&bad_spec);
        assert!(!report.refusal_status_in_bounds);
        assert!(!report.is_ok());
    }
}
