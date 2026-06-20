//! Golden-vector corpus — the portability oracle.
//!
//! The corpus binds every pattern's kernel output together with its IR / evidence shape
//! (id, event code, OTEL span, object codes, admission rule) into a single rolling receipt.
//! Any drift in a kernel, the registry, or the evidence wiring changes the digest.
//!
//! [`GOLDEN_CORPUS_DIGEST`] is the frozen oracle: every other projection target (C ABI,
//! WASM, engine adapters) must reproduce it to claim portability. That is the executable
//! form of the falsifier "same input -> same output -> same receipt across targets".
//!
//! # Testing Your Port
//!
//! Call [`assert_corpus_stable`] in your integration test suite to confirm that the
//! corpus digest is stable on your target. A mismatch means a kernel, the registry, or
//! the evidence wiring diverged from the reference build.

use crate::ir::PatternSpec;
use crate::patterns::{self, PATTERN_REGISTRY};
use bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt;

/// Fixed probe vectors applied to every pattern.
///
/// Covers the boundary cases `(0, 0)` and `(u64::MAX, u64::MAX)`, plus two
/// representative mid-range inputs. Every kernel is exercised with all four probes
/// before its contribution is folded into the corpus digest.
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
        // Phase 2 — Pathfinding
        21 => patterns::path_node_expanded(state, input),
        22 => patterns::waypoint_reached(state, input),
        23 => patterns::heuristic_distance_estimated(state, input),
        24 => patterns::path_cost_bounded(state, input),
        25 => patterns::nav_state_advanced(state, input),
        // Phase 2 — Procedural Generation
        26 => patterns::noise_value_sampled(state, input),
        27 => patterns::tile_variant_selected(state, input),
        28 => patterns::terrain_height_quantized(state, input),
        29 => patterns::spawn_weight_evaluated(state, input),
        30 => patterns::biome_class_selected(state, input),
        // Phase 2 — Economy / Progression
        31 => patterns::currency_delta_applied(state, input),
        32 => patterns::xp_threshold_crossed(state, input),
        33 => patterns::level_gate_evaluated(state, input),
        34 => patterns::purchase_admitted(state, input),
        35 => patterns::reward_tier_selected(state, input),
        // Phase 2 — Narrative / Dialogue
        36 => patterns::dialogue_node_advanced(state, input),
        37 => patterns::condition_flag_evaluated(state, input),
        38 => patterns::narrative_branch_selected(state, input),
        39 => patterns::dialogue_cooldown_bounded(state, input),
        40 => patterns::choice_weight_selected(state, input),
        // Phase 2 — Camera
        41 => patterns::camera_distance_clamped(state, input),
        42 => patterns::look_target_weighted(state, input),
        43 => patterns::fov_adjusted(state, input),
        44 => patterns::camera_shake_applied(state, input),
        45 => patterns::camera_follow_lerped(state, input),
        // Phase 2 — Audio
        46 => patterns::audio_priority_selected(state, input),
        47 => patterns::volume_clamped(state, input),
        48 => patterns::audio_fade_applied(state, input),
        49 => patterns::audio_trigger_evaluated(state, input),
        50 => patterns::audio_distance_attenuated(state, input),
        // Phase 2 — Multiplayer / Network
        51 => patterns::tick_delta_bounded(state, input),
        52 => patterns::lag_compensation_applied(state, input),
        53 => patterns::packet_priority_evaluated(state, input),
        54 => patterns::prediction_error_bounded(state, input),
        55 => patterns::sync_state_admitted(state, input),
        // Phase 2 — DfLSS / Quality
        56 => patterns::sigma_level_computed(state, input),
        57 => patterns::defect_rate_quantized(state, input),
        58 => patterns::ctq_threshold_evaluated(state, input),
        59 => patterns::nps_score_bounded(state, input),
        60 => patterns::quality_gate_evaluated(state, input),
        // Phase 2 — Engine Bridge
        61 => patterns::command_opcode_encoded(state, input),
        62 => patterns::capability_flag_evaluated(state, input),
        63 => patterns::bridge_state_transitioned(state, input),
        64 => patterns::payload_size_bounded(state, input),
        65 => patterns::adapter_priority_ranked(state, input),
        // Phase 2 — AI Agent / Benchmark
        66 => patterns::reward_signal_clamped(state, input),
        67 => patterns::policy_action_selected(state, input),
        68 => patterns::observation_class_selected(state, input),
        69 => patterns::action_mask_applied(state, input),
        70 => patterns::episode_return_bounded(state, input),
        // Phase 3 — Anti-Cheat
        71 => patterns::movement_legality_checked(state, input),
        72 => patterns::resource_bound_checked(state, input),
        73 => patterns::cooldown_legality_checked(state, input),
        74 => patterns::action_rate_bounded(state, input),
        75 => patterns::transition_legality_checked(state, input),
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

/// Pinned golden value of [`corpus_digest`]. The portability oracle.
///
/// Frozen at the commit that introduced or last updated the 75-pattern registry.
/// Any drift in a kernel implementation, the [`crate::patterns::PATTERN_REGISTRY`]
/// metadata, or the evidence wiring (event codes, object codes, span codes) will
/// produce a different [`corpus_digest`] and cause the corpus test to fail loudly.
///
/// # Stability
///
/// This value **only changes intentionally**, as part of a tracked ggen regeneration
/// that produces a new set of verified kernels. Incidental changes (e.g., refactoring
/// a kernel without changing its observable behavior) must NOT change this value — if
/// they do, that is a portability regression, not a feature.
///
/// Other projection targets (C ABI, WASM, engine adapters) must reproduce this exact
/// `u64` to claim byte-for-byte kernel parity with the reference Rust build.
pub const GOLDEN_CORPUS_DIGEST: u64 = 0x0501_B4DE_76D8_78C0;

/// Assert that [`corpus_digest`] matches [`GOLDEN_CORPUS_DIGEST`].
///
/// Call this in your own test suite to verify that the corpus is stable on your
/// target platform or after a code change. On mismatch, panics with a diagnostic
/// message identifying which digest was computed vs. expected.
///
/// # Examples
///
/// ```
/// # #[cfg(test)]
/// wasm4games::corpus::assert_corpus_stable();
/// ```
pub fn assert_corpus_stable() {
    let computed = corpus_digest();
    assert_eq!(
        computed, GOLDEN_CORPUS_DIGEST,
        "corpus digest mismatch: a pattern kernel output changed — \
         run `cargo test` to see which pattern failed \
         (computed=0x{:016X}, expected=0x{:016X})",
        computed, GOLDEN_CORPUS_DIGEST
    );
}

/// Check whether [`corpus_digest`] matches [`GOLDEN_CORPUS_DIGEST`] without panicking.
///
/// Returns `true` if the corpus is stable. Prefer [`assert_corpus_stable`] in test code;
/// use this variant where panicking is not acceptable (e.g., embedded runtime diagnostics).
#[must_use]
pub fn verify_corpus() -> bool {
    corpus_digest() == GOLDEN_CORPUS_DIGEST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_stable_and_registry_ordered() {
        // Registry must be exactly 75 entries in id order.
        assert_eq!(PATTERN_REGISTRY.len(), 75, "expected 75 patterns");
        for (idx, spec) in PATTERN_REGISTRY.iter().enumerate() {
            assert_eq!(spec.id.0 as usize, idx + 1, "ids must be 1..=75 in order");
            assert_eq!(
                pattern_digest(spec),
                pattern_digest(spec),
                "per-pattern digest must be deterministic"
            );
        }

        // Golden oracle: corpus_digest must match the pinned value.
        let computed = corpus_digest();
        assert_eq!(
            computed, GOLDEN_CORPUS_DIGEST,
            "corpus digest mismatch (computed=0x{:016X}, expected=0x{:016X})",
            computed, GOLDEN_CORPUS_DIGEST
        );
        assert!(
            verify_corpus(),
            "corpus must be stable on the reference build"
        );
    }
}
