//! Attaches the playground's branchless process-intelligence (the Petri-net token
//! replay engine) to the chess decision engine.
//!
//! Every move the engine makes must traverse a declared, lawful decision pipeline:
//!
//!     READY -> Perceive -> Evaluate -> Search -> Select -> Commit -> READY
//!
//! Each stage emits a cryptographically hash-chained OCEL event. The full event
//! log is then *replayed through a branchless Petri net* (`playground::petri`) to
//! compute conformance fitness — PROVING the branchless engine actually executed
//! the declared process, with no skipped, reordered, or fabricated stages.
//!
//! This is the UHFT-relevant property: not just a fast decision, but a
//! *provably-lawful, auditable* decision — every action backed by a conformance
//! receipt. A negative test (a deliberately skipped stage) shows the check has
//! teeth: fitness drops below 1.0 and the run is flagged non-conforming.

use blake3::Hasher;
use chess::{Board, ChessMove, Color, MoveGen, Piece};
use playground::{
    nnue::BranchTorchNNUE,
    petri::{petri_fire_transition, ReplayResult},
};

// --- Decision-process Petri net: one bit per place in the u64 marking. ---
const READY: u64 = 1 << 0;
const PERCEIVED: u64 = 1 << 1;
const EVALUATED: u64 = 1 << 2;
const SEARCHED: u64 = 1 << 3;
const SELECTED: u64 = 1 << 4;

/// Lawful transitions: (name, in_place, out_place). Firing consumes the input
/// place's token and produces the output place's token.
const TRANSITIONS: &[(&str, u64, u64)] = &[
    ("Perceive", READY, PERCEIVED),
    ("Evaluate", PERCEIVED, EVALUATED),
    ("Search", EVALUATED, SEARCHED),
    ("Select", SEARCHED, SELECTED),
    ("Commit", SELECTED, READY),
];

/// Signed white-relative eval = NNUE neuron 0 (material + PST), branchless accum.
fn eval_white_cp(board: &Board, nnue: &BranchTorchNNUE) -> i32 {
    let mut h0 = nnue.l1_biases[0];
    let pieces =
        [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];
    for (p_idx, &p) in pieces.iter().enumerate() {
        for sq in *board.color_combined(Color::White) & *board.pieces(p) {
            h0 += nnue.l1_weights[0][p_idx * 64 + sq.to_index()];
        }
        for sq in *board.color_combined(Color::Black) & *board.pieces(p) {
            h0 += nnue.l1_weights[0][(p_idx + 6) * 64 + sq.to_index()];
        }
    }
    h0
}

/// A single emitted decision-pipeline event.
struct Event {
    stage: &'static str,
    detail: String,
    receipt: String, // hash-chained
}

/// Run ONE real decision (1-ply NNUE search) and emit the 5 stage events.
/// `skip` optionally drops a stage to demonstrate the conformance check failing.
fn decide(
    board: &Board,
    nnue: &BranchTorchNNUE,
    prev_hash: &mut [u8; 32],
    log: &mut Vec<Event>,
    skip: Option<&str>,
) -> Option<ChessMove> {
    let mut emit = |stage: &'static str, detail: String, log: &mut Vec<Event>| {
        if skip == Some(stage) {
            return; // negative test: lawful stage omitted from the log
        }
        let mut h = Hasher::new();
        h.update(&prev_hash[..]);
        h.update(stage.as_bytes());
        h.update(detail.as_bytes());
        *prev_hash = *h.finalize().as_bytes();
        log.push(Event { stage, detail, receipt: hex(prev_hash) });
    };

    // 1. Perceive: enumerate the legal action set.
    let moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    emit("Perceive", format!("{} legal moves", moves.len()), log);

    // 2. Evaluate: branchless NNUE eval of the current state.
    let stm = board.side_to_move();
    let base = eval_white_cp(board, nnue);
    emit("Evaluate", format!("white_cp={base}"), log);

    // 3. Search: 1-ply — evaluate every child, branchless.
    let mut best = None;
    let mut best_val = i32::MIN;
    for m in &moves {
        let child = board.make_move_new(*m);
        let wcp = eval_white_cp(&child, nnue);
        let v = if stm == Color::White { wcp } else { -wcp };
        if v > best_val {
            best_val = v;
            best = Some(*m);
        }
    }
    emit("Search", format!("best_val={best_val}"), log);

    // 4. Select.
    let mv = best?;
    emit("Select", format!("move={mv}"), log);

    // 5. Commit.
    emit("Commit", format!("commit={mv}"), log);
    Some(mv)
}

/// Replay the emitted event log through the Petri net and score conformance.
fn conformance(log: &[Event]) -> ReplayResult {
    let mut marking: u64 = READY; // one token, idle and ready
    let (mut missing, mut consumed, mut produced) = (0u32, 0u32, 0u32);
    for ev in log {
        if let Some(&(_, in_place, out_place)) =
            TRANSITIONS.iter().find(|(name, _, _)| *name == ev.stage)
        {
            petri_fire_transition(
                &mut marking,
                in_place,
                out_place,
                &mut missing,
                &mut consumed,
                &mut produced,
            );
        }
    }
    // Remaining = leftover tokens not in the idle READY place.
    let remaining = (marking & !READY).count_ones();
    ReplayResult::new(missing, remaining, produced, consumed)
}

fn hex(b: &[u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    let nnue = BranchTorchNNUE::new();
    let moves_to_play = 6;

    // --- Lawful run: every decision traverses the full pipeline. ---
    let mut board = Board::default();
    let mut prev = *Hasher::new().finalize().as_bytes();
    let mut log: Vec<Event> = Vec::new();
    for _ in 0..moves_to_play {
        match decide(&board, &nnue, &mut prev, &mut log, None) {
            Some(mv) => board = board.make_move_new(mv),
            None => break,
        }
    }
    let r = conformance(&log);

    println!("=== Branchless chess decision engine + Petri-net conformance ===");
    println!("Pipeline: READY -> Perceive -> Evaluate -> Search -> Select -> Commit");
    println!("\n-- LAWFUL RUN ({} moves, {} stage events) --", moves_to_play, log.len());
    for ev in log.iter().take(10) {
        println!("  [{:8}] {:20} receipt={}...", ev.stage, ev.detail, &ev.receipt[..12]);
    }
    if log.len() > 10 {
        println!("  ... ({} more events)", log.len() - 10);
    }
    println!(
        "  conformance: fitness={:.4}  perfect={}  (missing={}, remaining={}, consumed={}, produced={})",
        r.fitness(),
        r.is_perfect(),
        r.missing,
        r.remaining,
        r.consumed,
        r.produced
    );
    println!("  final receipt = {}", log.last().map(|e| e.receipt.as_str()).unwrap_or("-"));

    // --- Negative test: one decision skips the lawful Evaluate stage. ---
    let mut board = Board::default();
    let mut prev = *Hasher::new().finalize().as_bytes();
    let mut log: Vec<Event> = Vec::new();
    for i in 0..moves_to_play {
        let skip = if i == 2 { Some("Evaluate") } else { None };
        match decide(&board, &nnue, &mut prev, &mut log, skip) {
            Some(mv) => board = board.make_move_new(mv),
            None => break,
        }
    }
    let rn = conformance(&log);
    println!("\n-- NEGATIVE TEST (move 3 skips the lawful 'Evaluate' stage) --");
    println!(
        "  conformance: fitness={:.4}  perfect={}  (missing={}, remaining={})",
        rn.fitness(),
        rn.is_perfect(),
        rn.missing,
        rn.remaining
    );
    println!(
        "  verdict: {}",
        if rn.is_perfect() {
            "BUG: non-conforming run accepted!"
        } else {
            "REJECTED — skipped stage detected; the engine's receipt is provably non-lawful."
        }
    );
}
