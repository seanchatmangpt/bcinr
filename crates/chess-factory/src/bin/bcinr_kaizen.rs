//! `bcinr_kaizen` — minimal PDCA Kaizen step for the chess factory.
//!
//! PLAN/CHECK: read the measured `artifacts/elo_curve.json` (overall Elo /
//! score-rate signal) and `artifacts/benchmark.receipt.json` (run provenance),
//! then ATTRIBUTE that signal per station. Per-station attribution is computed
//! by replaying every generated feature station over a deterministic position
//! set (the curated `OPENINGS` lines, the same lines the benchmark uses) and
//! measuring each station's mean *signed weighted contribution* toward the
//! aggregate evaluation. A station whose mean |contribution| is zero (it never
//! moves the score) or whose sign systematically opposes the aggregate (it
//! fights the engine) is a Kaizen target.
//!
//! ACT (proposal only — never auto-applied): emit ONE
//! `KaizenRecommendation` of type `DemoteStationWeight` for the worst such
//! station: lower its `cf:weight_q8` in `ontology/chess.ttl`. The proposal is
//! written to `artifacts/kaizen_recommendation.json`. A human/operator applies
//! the TTL patch, re-runs `ggen sync`, and re-benches to confirm the curve
//! moved (P2: the ontology is the source of truth; code is never hand-edited).
//!
//! Determinism: fixed position set, fixed station order, integer arithmetic,
//! byte-stable JSON. No RNG, no wall-clock.

use std::path::PathBuf;

use chess::Board;
use std::str::FromStr;

use chess_factory::aggregator::aggregate;
use chess_factory::position::PositionView;
use chess_factory::stations::{self, STATION_REGISTRY as STATIONS};
use chess_factory::weights::{apply_weight, STATION_WEIGHTS_Q8};

/// Deterministic position set: startpos plus each curated opening line applied.
/// Lifted (as UCI move lists) from the benchmark opening book so the Kaizen
/// attribution is measured on the SAME distribution the Elo curve is measured on.
const OPENINGS: &[&[&str]] = &[
    &[],
    &["e2e4", "e7e5"],
    &["e2e4", "c7c5"],
    &["e2e4", "e7e6"],
    &["e2e4", "c7c6"],
    &["e2e4", "d7d5"],
    &["d2d4", "d7d5"],
    &["d2d4", "g8f6"],
    &["d2d4", "f7f5"],
    &["c2c4", "e7e5"],
    &["g1f3", "d7d5"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6"],
    &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4"],
    &["d2d4", "g8f6", "c2c4", "e7e6"],
    // Deeper lines with captures / imbalance so material, pst, mobility and
    // center_control produce non-trivial, *asymmetric* signal (symmetric
    // openings alone leave every station near zero).
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4", "c5b4", "c2c3", "b4a5"],
    &["d2d4", "d7d5", "c2c4", "d5c4", "e2e4", "b7b5", "a2a4", "c7c6", "a4b5", "c6b5"],
    &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6"],
    &["e2e4", "e7e6", "d2d4", "d7d5", "e4d5", "e6d5", "g1f3", "g8f6", "f1d3", "f8d6"],
];

/// Per-station raw (pre-weight) centipawn contribution for one position, in id order.
fn raw_contributions(v: &PositionView) -> [i32; 6] {
    [
        stations::evaluate_material(v).evidence.raw_cp,
        stations::evaluate_pst(v).evidence.raw_cp,
        stations::evaluate_mobility(v).evidence.raw_cp,
        stations::evaluate_king_safety(v).evidence.raw_cp,
        stations::evaluate_pawn_structure(v).evidence.raw_cp,
        stations::evaluate_center_control(v).evidence.raw_cp,
    ]
}

/// Apply a UCI move list from startpos; returns the resulting board.
fn board_from_uci(moves: &[&str]) -> Board {
    let mut b = Board::default();
    for m in moves {
        let mv = chess::ChessMove::from_str(m).expect("valid uci move");
        b = b.make_move_new(mv);
    }
    b
}

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let artifacts = crate_dir.join("artifacts");

    // CHECK: ingest the measured Elo curve + run receipt. We require them to
    // exist (the benchmark must have run first); we read the headline score.
    let curve_path = artifacts.join("elo_curve.json");
    let receipt_path = artifacts.join("benchmark.receipt.json");
    let curve: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&curve_path).expect("elo_curve.json (run factory-bench first)"),
    )
    .expect("elo_curve.json is valid JSON");
    let receipt: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&receipt_path).expect("benchmark.receipt.json"),
    )
    .expect("benchmark.receipt.json is valid JSON");

    // Headline measured score-rate at 100us (the metric Kaizen aims to move).
    let measured_score_rate = curve
        .get("elo_at_100us")
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("score_rate_at_100us"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    // PLAN/attribution: accumulate per-station signed weighted contribution and
    // its agreement with the aggregate sign across the deterministic position set.
    let mut sum_abs_weighted = [0i64; 6];
    let mut sum_agree = [0i64; 6]; // + when station pushes same direction as aggregate
    let mut positions = 0i64;

    for line in OPENINGS {
        let board = board_from_uci(line);
        let view = PositionView::from_board(&board);
        let agg = aggregate(&view);
        let agg_sign = (agg > 0) as i64 - (agg < 0) as i64;
        let raw = raw_contributions(&view);
        for i in 0..6 {
            let weighted = apply_weight(raw[i], STATION_WEIGHTS_Q8[i]) as i64;
            sum_abs_weighted[i] += weighted.abs();
            let st_sign = (weighted > 0) as i64 - (weighted < 0) as i64;
            sum_agree[i] += st_sign * agg_sign;
        }
        positions += 1;
    }

    // Contribution score per station: mean |weighted contribution| times its
    // directional agreement with the aggregate. <= 0 means the station either
    // never moves the score or systematically fights it -> demote candidate.
    // Worst = least useful: rank by (signed_contribution asc, mean_abs asc, id asc).
    // signed_contribution = -mean_abs when the station fights the aggregate
    // (agreement<=0), else +mean_abs. The smallest value is the worst station.
    let mut worst: Option<(usize, i64, i64)> = None; // (idx, signed_contribution, mean_abs)
    let mut contributions = Vec::with_capacity(6);
    for i in 0..6 {
        let mean_abs = sum_abs_weighted[i] / positions.max(1);
        let agreement = sum_agree[i]; // range [-positions, positions]
        let signed_contribution = if agreement <= 0 { -mean_abs } else { mean_abs };
        contributions.push(serde_json::json!({
            "station_id": STATIONS[i].id,
            "station_name": STATIONS[i].name,
            "weight_q8": STATION_WEIGHTS_Q8[i],
            "mean_abs_weighted_cp": mean_abs,
            "directional_agreement": agreement,
            "signed_contribution": signed_contribution,
        }));
        let better_candidate = match worst {
            None => true,
            Some((_, w_sc, w_abs)) => {
                (signed_contribution, mean_abs) < (w_sc, w_abs)
            }
        };
        if better_candidate {
            worst = Some((i, signed_contribution, mean_abs));
        }
    }

    let (worst_idx, worst_score, _) = worst.expect("six stations");
    let cur_weight = STATION_WEIGHTS_Q8[worst_idx];
    // Propose halving the weight (floored), a conservative demotion step.
    let proposed_weight = (cur_weight / 2).max(0);

    let recommendation = serde_json::json!({
        "schema": "bcinr.chess-factory.kaizen.recommendation.v1",
        "type": "DemoteStationWeight",
        "rationale": "Station has zero/negative signed contribution to the aggregate \
                      over the benchmark opening distribution; lowering its weight should \
                      not reduce (and may improve) Elo-at-100us. Proposal only — apply the \
                      TTL patch, re-run `ggen sync`, re-bench, and keep only if the curve improves.",
        "evidence": {
            "measured_score_rate_at_100us": measured_score_rate,
            "benchmark_receipt": {
                "input_matrix_blake3": receipt.get("input_matrix_blake3"),
                "output_curve_blake3": receipt.get("output_curve_blake3"),
                "replay_pointer": receipt.get("replay_pointer"),
            },
            "position_set_size": positions,
            "per_station_contributions": contributions,
        },
        "target": {
            "ontology_file": "ontology/chess.ttl",
            "subject": format!("cf:{}", STATIONS[worst_idx].name),
            "property": "cf:weight_q8",
            "station_id": STATIONS[worst_idx].id,
            "station_name": STATIONS[worst_idx].name,
            "current_weight_q8": cur_weight,
            "proposed_weight_q8": proposed_weight,
            "worst_signed_contribution": worst_score,
        },
        "apply_command": format!(
            "edit cf:{} cf:weight_q8 {} -> {} in ontology/chess.ttl, then `ggen sync`",
            STATIONS[worst_idx].name, cur_weight, proposed_weight
        ),
        "auto_applied": false,
    });

    let out_path = artifacts.join("kaizen_recommendation.json");
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(&recommendation).expect("serialize recommendation"),
    )
    .expect("write kaizen_recommendation.json");

    println!(
        "kaizen: DemoteStationWeight for station {} (id={}) {} -> {} (signed_contribution={})",
        STATIONS[worst_idx].name,
        STATIONS[worst_idx].id,
        cur_weight,
        proposed_weight,
        worst_score
    );
    println!("kaizen: recommendation written to {}", out_path.display());
}
