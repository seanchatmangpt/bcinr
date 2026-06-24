//! Game phase classification and topology selection.
//!
//! Hand-authored (algorithmic identity — the classifier IS its control flow).
//!
//! Phase classification is computed from the position's material signature.
//! The result selects a compiled POWL topology (an index into the topology table).
//! Selection is O(1): no search, no discovery, no dynamic architecture work.
//!
//! The topology table is promoted by the Manufacturing Graph (offline benchmarking).
//! At runtime, the engine reads only the promoted winner per (phase, hardware, budget).

#![cfg(feature = "std")]

use chess::{Board, Color};

use crate::position::{PositionView, BISHOP, KNIGHT, PAWN, QUEEN, ROOK};

// ---------------------------------------------------------------------------
// Phase tokens — compile-time proof of classifier output, runtime zero-cost
// ---------------------------------------------------------------------------

/// The five mutually-exclusive game phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Opening: both queens present, most minor pieces undeveloped.
    Opening,
    /// Tactical crisis: position has immediate checks, captures, or threats.
    Tactical,
    /// Quiet middlegame: complex position, no forcing lines.
    Quiet,
    /// Endgame: reduced material, king becomes active.
    Endgame,
    /// Low-material / near-tablebase: 6 or fewer pieces total.
    Tablebase,
}

// ---------------------------------------------------------------------------
// Hardware class — admitted from environment, not detected at move time
// ---------------------------------------------------------------------------

/// Coarse hardware class. Determines which topology variant to select.
/// Admitted once per session, not detected per move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardwareClass {
    Single,  // 1 worker
    Small,   // 2–4 workers
    Large,   // 8+ workers
}

impl HardwareClass {
    pub fn detect() -> Self {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        if cores >= 8 { Self::Large } else if cores >= 2 { Self::Small } else { Self::Single }
    }
}

// ---------------------------------------------------------------------------
// Budget class — admitted from the caller
// ---------------------------------------------------------------------------

/// Coarse time budget bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetClass {
    Micro,   // ≤ 200µs
    Short,   // 200µs – 2ms
    Long,    // > 2ms
}

impl BudgetClass {
    pub fn from_us(budget_us: u128) -> Self {
        if budget_us <= 200 { Self::Micro } else if budget_us <= 2_000 { Self::Short } else { Self::Long }
    }
}

// ---------------------------------------------------------------------------
// Admitted O* — the full position state before topology selection
// ---------------------------------------------------------------------------

/// Fully admitted position state. Selection key for the topology table.
///
/// This is the Chatman Equation's O*:
///   O* = { board, phase, budget, hardware, tt_occupancy }
///   μ  = topology_table.select(&self)
///   A  = receipted move
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdmittedState {
    pub phase: Phase,
    pub hardware: HardwareClass,
    pub budget: BudgetClass,
}

// ---------------------------------------------------------------------------
// Phase classification — O(1) branchless arithmetic on material counts
// ---------------------------------------------------------------------------

/// Material phase scalar in [0, 256].
/// 256 = full opening material; 0 = bare kings (never reached).
/// Matches the "game phase" concept from PeSTO and similar engines.
#[inline(always)]
pub fn material_phase(v: &PositionView) -> u32 {
    const PHASE_WEIGHTS: [u32; 5] = [0, 1, 1, 2, 4]; // P, N, B, R, Q
    let mut phase = 0u32;
    let mut p = 1usize; // skip pawns (no phase weight)
    while p < 5 {
        phase += (v.by_piece[0][p].count_ones() + v.by_piece[1][p].count_ones())
            * PHASE_WEIGHTS[p];
        p += 1;
    }
    // Max: 4N + 4B + 4R + 2Q = 4+4+8+8 = 24; scale to 256
    ((phase * 256 / 24).min(256)) as u32
}

/// Total piece count (excluding kings).
#[inline(always)]
pub fn total_pieces(v: &PositionView) -> u32 {
    v.occ.count_ones() - 2 // subtract both kings
}

/// Returns true if the position has immediate tactical threats (checks or hanging pieces).
/// Conservative approximation: checks in hand or queen + pawn count imbalance.
pub fn has_tactical_crisis(board: &Board) -> bool {
    // In check = immediate tactical crisis
    if board.checkers().0 != 0 { return true; }

    // Hanging queens (queens with no defenders nearby) — simplified proxy
    let v = PositionView::from_board(board);
    let stm = if board.side_to_move() == Color::White { 0 } else { 1 };
    let opp = stm ^ 1;

    // Material imbalance > 3cp (rook for minor, etc.) is a tactical signal
    let my_queens = v.by_piece[stm][QUEEN].count_ones() as i32;
    let opp_queens = v.by_piece[opp][QUEEN].count_ones() as i32;
    let my_rooks = v.by_piece[stm][ROOK].count_ones() as i32;
    let opp_rooks = v.by_piece[opp][ROOK].count_ones() as i32;
    let my_minor = (v.by_piece[stm][KNIGHT] | v.by_piece[stm][BISHOP]).count_ones() as i32;
    let opp_minor = (v.by_piece[opp][KNIGHT] | v.by_piece[opp][BISHOP]).count_ones() as i32;

    let balance = (my_queens - opp_queens) * 9
        + (my_rooks - opp_rooks) * 5
        + (my_minor - opp_minor) * 3;

    // Significant material advantage for opponent = tactical crisis for us
    balance < -3
}

/// Classify the phase from board + PositionView.
/// This is the only place phase is computed. O* admits this result.
pub fn classify(board: &Board, v: &PositionView) -> Phase {
    let pieces = total_pieces(v);
    let phase = material_phase(v);

    // Tablebase zone: 6 or fewer pieces total (including kings)
    if pieces <= 4 { return Phase::Tablebase; }

    // Endgame: material phase below 1/3 of opening
    if phase <= 85 { return Phase::Endgame; }

    // Tactical crisis: check or material imbalance
    if has_tactical_crisis(board) { return Phase::Tactical; }

    // Opening: both queens on board + high material phase
    let both_queens = v.by_piece[0][QUEEN] != 0 && v.by_piece[1][QUEEN] != 0;
    let pawn_count = (v.by_piece[0][PAWN] | v.by_piece[1][PAWN]).count_ones();
    if both_queens && pawn_count >= 12 && phase >= 200 { return Phase::Opening; }

    Phase::Quiet
}

// ---------------------------------------------------------------------------
// Topology identifier — index into the compiled topology table
// ---------------------------------------------------------------------------

/// A compiled topology variant. At runtime this is just an integer;
/// the Manufacturing Graph promotes winners per (phase, hardware, budget).
///
/// Variants are manufactured offline by GGEN from `cf:SearchTopology` TTL
/// individuals. The table below is the current promoted set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopologyId(pub u8);

impl TopologyId {
    pub const OPENING_MICRO_SINGLE: Self = Self(0);
    pub const OPENING_MICRO_SMALL: Self  = Self(1);
    pub const OPENING_MICRO_LARGE: Self  = Self(2);
    pub const TACTICAL_MICRO: Self       = Self(3);
    pub const QUIET_MICRO_SINGLE: Self   = Self(4);
    pub const QUIET_MICRO_SMALL: Self    = Self(5);
    pub const QUIET_MICRO_LARGE: Self    = Self(6);
    pub const ENDGAME_MICRO: Self        = Self(7);
    pub const TABLEBASE_MICRO: Self      = Self(8);
    pub const FALLBACK: Self             = Self(9);
}

// ---------------------------------------------------------------------------
// Topology table — promoted winners per (phase, hardware, budget)
// This table is the output of the Manufacturing Graph's benchmark loop.
// Edit by promoting benchmark winners, not by hand-tuning.
// ---------------------------------------------------------------------------

/// O(1) topology selection: admitted O* → compiled topology.
///
/// This is μ in the Chatman Equation: A = μ(O*).
#[must_use]
pub fn select_topology(state: &AdmittedState) -> TopologyId {
    use BudgetClass::*;
    use HardwareClass::*;
    use Phase::*;

    // Topology table: promoted by Manufacturing Graph.
    // Current state: all slots use baseline topology (pre-benchmark-promotion).
    // Replace slot values with benchmark winners as they are receipted.
    match (state.phase, state.hardware, state.budget) {
        (Opening,   Single, Micro)  => TopologyId::OPENING_MICRO_SINGLE,
        (Opening,   Small,  Micro)  => TopologyId::OPENING_MICRO_SMALL,
        (Opening,   Large,  Micro)  => TopologyId::OPENING_MICRO_LARGE,
        (Opening,   _,      _)      => TopologyId::OPENING_MICRO_SINGLE,

        (Tactical,  _,      _)      => TopologyId::TACTICAL_MICRO,

        (Quiet,     Single, Micro)  => TopologyId::QUIET_MICRO_SINGLE,
        (Quiet,     Small,  Micro)  => TopologyId::QUIET_MICRO_SMALL,
        (Quiet,     Large,  Micro)  => TopologyId::QUIET_MICRO_LARGE,
        (Quiet,     _,      _)      => TopologyId::QUIET_MICRO_SINGLE,

        (Endgame,   _,      _)      => TopologyId::ENDGAME_MICRO,
        (Tablebase, _,      _)      => TopologyId::TABLEBASE_MICRO,
    }
}

// ---------------------------------------------------------------------------
// Public entry point: admit position → O* → topology
// ---------------------------------------------------------------------------

/// Full admission pipeline: board → O* → TopologyId.
///
/// This is the runtime-side of the Chatman Equation.
/// The result is used by search.rs to select the search graph.
#[must_use]
pub fn admit(board: &Board, budget_us: u128) -> (AdmittedState, TopologyId) {
    let v = PositionView::from_board(board);
    let phase = classify(board, &v);
    let hardware = HardwareClass::detect();
    let budget = BudgetClass::from_us(budget_us);
    let state = AdmittedState { phase, hardware, budget };
    let topology = select_topology(&state);
    (state, topology)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use chess::Board;

    #[test]
    fn startpos_is_opening() {
        let board = Board::default();
        let v = PositionView::from_board(&board);
        assert_eq!(classify(&board, &v), Phase::Opening);
    }

    #[test]
    fn admit_returns_topology() {
        let board = Board::default();
        let (state, topo) = admit(&board, 100);
        assert_eq!(state.phase, Phase::Opening);
        assert_eq!(state.budget, BudgetClass::Micro);
        // Topology is non-fallback for a legal position
        assert_ne!(topo, TopologyId::FALLBACK);
    }

    #[test]
    fn material_phase_full_board() {
        let board = Board::default();
        let v = PositionView::from_board(&board);
        let p = material_phase(&v);
        // Full board: 4N+4B+4R+2Q = 24 units → 256
        assert_eq!(p, 256);
    }
}
