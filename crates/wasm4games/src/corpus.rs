//! Golden-vector corpus — the portability oracle.
//!
//! The corpus binds every pattern's kernel output together with its IR / evidence shape
//! (id, event code, OTEL span, object codes, admission rule) into a single rolling receipt.
//! Any drift in a kernel, the registry, or the evidence wiring changes the digest.
//!
//! [`GOLDEN_CORPUS_DIGEST`] is the frozen oracle: every other projection target (C ABI,
//! WASM, engine adapters) must reproduce it to claim portability. That is the executable
//! form of the falsifier "same input -> same output -> same receipt across targets".

use crate::ir::PatternSpec;
use crate::patterns::{self, PATTERN_REGISTRY};
use bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt;

/// Fixed probe vectors applied to every pattern: zero, saturated, a representative
/// mid-range case, and an arbitrary case.
pub const PROBES: [(u64, u64); 4] = [
    (0, 0),
    (u64::MAX, u64::MAX),
    (100, 0x0001_0007),
    (0xABCD, 0x1234),
];

/// Dispatch a pattern id to its branchless kernel.
///
/// This is cold glue (not a hot kernel); a future ggen rule can emit this table from the
/// same ontology that produces [`PATTERN_REGISTRY`]. Unknown ids return 0.
#[must_use]
pub fn dispatch(pattern_id: u16, state: u64, input: u64) -> u64 {
    match pattern_id {
        1 => patterns::input_admitted(state, input),
        2 => patterns::fixed_tick_advanced(state, input),
        3 => patterns::entity_state_transitioned(state, input),
        4 => patterns::object_spawned(state, input),
        5 => patterns::aabb_collision_resolved(state, input),
        6 => patterns::ocel_event_linked(state, input),
        7 => patterns::otel_span_emitted(state, input),
        8 => patterns::replay_frame_recorded(state, input),
        9 => patterns::receipt_appended(state, input),
        10 => patterns::physics_value_rendered(state, input),
        11 => patterns::semantic_lod_selected(state, input),
        12 => patterns::projectile_advanced(state, input),
        13 => patterns::ai_action_selected(state, input),
        14 => patterns::damage_applied(state, input),
        15 => patterns::status_effect_ticked(state, input),
        16 => patterns::inventory_item_changed(state, input),
        17 => patterns::quest_step_advanced(state, input),
        18 => patterns::mastery_moment_detected(state, input),
        19 => patterns::share_artifact_generated(state, input),
        20 => patterns::nps_prompt_gated(state, input),
        _ => 0,
    }
}

/// Fold one pattern's IR + evidence shape + probe outputs into the receipt.
fn fold_pattern(r: &mut DeterministicSubstrateReceipt, spec: &PatternSpec) {
    r.record(
        spec.id.0 as u64,
        spec.event.code as u64,
        spec.otel_span as u64,
    );
    r.record(
        spec.admission.required_status as u64,
        spec.admission.refusal_status as u64,
        spec.state_card as u64,
    );
    for o in spec.objects {
        r.record(o.code as u64, 0, 0);
    }
    for (s, i) in PROBES {
        r.record(s, i, dispatch(spec.id.0, s, i));
    }
}

/// Per-pattern digest (binds one pattern's kernel + IR + evidence shape). Localizes drift.
#[must_use]
pub fn pattern_digest(spec: &PatternSpec) -> u64 {
    let mut r = DeterministicSubstrateReceipt::new();
    fold_pattern(&mut r, spec);
    r.finalize()
}

/// The single digest binding every pattern in [`PATTERN_REGISTRY`]. The portability oracle.
#[must_use]
pub fn corpus_digest() -> u64 {
    let mut r = DeterministicSubstrateReceipt::new();
    for spec in PATTERN_REGISTRY {
        fold_pattern(&mut r, spec);
    }
    r.finalize()
}

/// Pinned golden value of [`corpus_digest`]. Frozen so any kernel/registry/evidence drift
/// fails loudly, and so every other projection target has one fixed number to reproduce.
pub const GOLDEN_CORPUS_DIGEST: u64 = 0x436B_6BFF_B836_DBAF;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_total_and_deterministic_over_registry() {
        assert_eq!(PATTERN_REGISTRY.len(), 20, "expected 20 patterns");
        for (idx, spec) in PATTERN_REGISTRY.iter().enumerate() {
            assert_eq!(spec.id.0 as usize, idx + 1, "ids must be 1..=20 in order");
            assert_eq!(
                pattern_digest(spec),
                pattern_digest(spec),
                "per-pattern digest must be deterministic"
            );
        }
    }

    #[test]
    fn corpus_digest_matches_golden() {
        assert_eq!(
            corpus_digest(),
            GOLDEN_CORPUS_DIGEST,
            "corpus drifted: a kernel, the registry, or evidence wiring changed"
        );
    }
}
