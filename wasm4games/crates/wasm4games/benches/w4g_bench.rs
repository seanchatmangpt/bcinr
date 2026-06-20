//! Criterion benchmark harness for the `wasm4games` pattern kernels.
//!
//! Each pattern module `wasm4games::patterns::<name>` carries a
//! `#[cfg(feature = "bench")] pub mod bench { pub fn bench_<name>(c: &mut Criterion) }`
//! kernel benchmark. This harness wires every per-kernel `bench_<name>` into a single
//! Criterion group so the otherwise-dormant `bench` modules become runnable via:
//!
//! ```text
//! cargo bench -p wasm4games --features bench
//! ```
//!
//! When the `bench` feature is OFF (e.g. a default `cargo build`/`cargo build --benches`),
//! the criterion code is gated out and this file compiles to an empty `fn main() {}` so the
//! benchmark target never breaks the default build.

#![cfg_attr(not(feature = "bench"), allow(unused))]

#[cfg(feature = "bench")]
use criterion::{criterion_group, criterion_main};

#[cfg(feature = "bench")]
use wasm4games::patterns;

#[cfg(feature = "bench")]
criterion_group!(
    w4g,
    // --- Phase 1 — ids 1-20 ---
    patterns::input_admitted::bench::bench_input_admitted,
    patterns::fixed_tick_advanced::bench::bench_fixed_tick_advanced,
    patterns::entity_state_transitioned::bench::bench_entity_state_transitioned,
    patterns::object_spawned::bench::bench_object_spawned,
    patterns::aabb_collision_resolved::bench::bench_aabb_collision_resolved,
    patterns::ocel_event_linked::bench::bench_ocel_event_linked,
    patterns::otel_span_emitted::bench::bench_otel_span_emitted,
    patterns::replay_frame_recorded::bench::bench_replay_frame_recorded,
    patterns::receipt_appended::bench::bench_receipt_appended,
    patterns::physics_value_rendered::bench::bench_physics_value_rendered,
    patterns::semantic_lod_selected::bench::bench_semantic_lod_selected,
    patterns::projectile_advanced::bench::bench_projectile_advanced,
    patterns::ai_action_selected::bench::bench_ai_action_selected,
    patterns::damage_applied::bench::bench_damage_applied,
    patterns::status_effect_ticked::bench::bench_status_effect_ticked,
    patterns::inventory_item_changed::bench::bench_inventory_item_changed,
    patterns::quest_step_advanced::bench::bench_quest_step_advanced,
    patterns::mastery_moment_detected::bench::bench_mastery_moment_detected,
    patterns::share_artifact_generated::bench::bench_share_artifact_generated,
    patterns::nps_prompt_gated::bench::bench_nps_prompt_gated,
    // --- Phase 2 — ids 21-25: Pathfinding ---
    patterns::path_node_expanded::bench::bench_path_node_expanded,
    patterns::waypoint_reached::bench::bench_waypoint_reached,
    patterns::heuristic_distance_estimated::bench::bench_heuristic_distance_estimated,
    patterns::path_cost_bounded::bench::bench_path_cost_bounded,
    patterns::nav_state_advanced::bench::bench_nav_state_advanced,
    // --- Phase 2 — ids 26-30: Procedural Generation ---
    patterns::noise_value_sampled::bench::bench_noise_value_sampled,
    patterns::tile_variant_selected::bench::bench_tile_variant_selected,
    patterns::terrain_height_quantized::bench::bench_terrain_height_quantized,
    patterns::spawn_weight_evaluated::bench::bench_spawn_weight_evaluated,
    patterns::biome_class_selected::bench::bench_biome_class_selected,
    // --- Phase 2 — ids 31-35: Economy / Progression ---
    patterns::currency_delta_applied::bench::bench_currency_delta_applied,
    patterns::xp_threshold_crossed::bench::bench_xp_threshold_crossed,
    patterns::level_gate_evaluated::bench::bench_level_gate_evaluated,
    patterns::purchase_admitted::bench::bench_purchase_admitted,
    patterns::reward_tier_selected::bench::bench_reward_tier_selected,
    // --- Phase 2 — ids 36-40: Narrative / Dialogue ---
    patterns::dialogue_node_advanced::bench::bench_dialogue_node_advanced,
    patterns::condition_flag_evaluated::bench::bench_condition_flag_evaluated,
    patterns::narrative_branch_selected::bench::bench_narrative_branch_selected,
    patterns::dialogue_cooldown_bounded::bench::bench_dialogue_cooldown_bounded,
    patterns::choice_weight_selected::bench::bench_choice_weight_selected,
    // --- Phase 2 — ids 41-45: Camera ---
    patterns::camera_distance_clamped::bench::bench_camera_distance_clamped,
    patterns::look_target_weighted::bench::bench_look_target_weighted,
    patterns::fov_adjusted::bench::bench_fov_adjusted,
    patterns::camera_shake_applied::bench::bench_camera_shake_applied,
    patterns::camera_follow_lerped::bench::bench_camera_follow_lerped,
    // --- Phase 2 — ids 46-50: Audio ---
    patterns::audio_priority_selected::bench::bench_audio_priority_selected,
    patterns::volume_clamped::bench::bench_volume_clamped,
    patterns::audio_fade_applied::bench::bench_audio_fade_applied,
    patterns::audio_trigger_evaluated::bench::bench_audio_trigger_evaluated,
    patterns::audio_distance_attenuated::bench::bench_audio_distance_attenuated,
    // --- Phase 2 — ids 51-55: Multiplayer / Network ---
    patterns::tick_delta_bounded::bench::bench_tick_delta_bounded,
    patterns::lag_compensation_applied::bench::bench_lag_compensation_applied,
    patterns::packet_priority_evaluated::bench::bench_packet_priority_evaluated,
    patterns::prediction_error_bounded::bench::bench_prediction_error_bounded,
    patterns::sync_state_admitted::bench::bench_sync_state_admitted,
    // --- Phase 2 — ids 56-60: DfLSS / Quality ---
    patterns::sigma_level_computed::bench::bench_sigma_level_computed,
    patterns::defect_rate_quantized::bench::bench_defect_rate_quantized,
    patterns::ctq_threshold_evaluated::bench::bench_ctq_threshold_evaluated,
    patterns::nps_score_bounded::bench::bench_nps_score_bounded,
    patterns::quality_gate_evaluated::bench::bench_quality_gate_evaluated,
    // --- Phase 2 — ids 61-65: Engine Bridge ---
    patterns::command_opcode_encoded::bench::bench_command_opcode_encoded,
    patterns::capability_flag_evaluated::bench::bench_capability_flag_evaluated,
    patterns::bridge_state_transitioned::bench::bench_bridge_state_transitioned,
    patterns::payload_size_bounded::bench::bench_payload_size_bounded,
    patterns::adapter_priority_ranked::bench::bench_adapter_priority_ranked,
    // --- Phase 2 — ids 66-70: AI Agent / Benchmark ---
    patterns::reward_signal_clamped::bench::bench_reward_signal_clamped,
    patterns::policy_action_selected::bench::bench_policy_action_selected,
    patterns::observation_class_selected::bench::bench_observation_class_selected,
    patterns::action_mask_applied::bench::bench_action_mask_applied,
    patterns::episode_return_bounded::bench::bench_episode_return_bounded,
    // --- Phase 3 — ids 71-75: Anti-Cheat ---
    patterns::movement_legality_checked::bench::bench_movement_legality_checked,
    patterns::resource_bound_checked::bench::bench_resource_bound_checked,
    patterns::cooldown_legality_checked::bench::bench_cooldown_legality_checked,
    patterns::action_rate_bounded::bench::bench_action_rate_bounded,
    patterns::transition_legality_checked::bench::bench_transition_legality_checked,
);

#[cfg(feature = "bench")]
criterion_main!(w4g);

#[cfg(not(feature = "bench"))]
fn main() {}
