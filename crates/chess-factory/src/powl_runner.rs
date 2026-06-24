//! POWL v2 Type-State Runtime Scheduler.
//!
//! A Powl64Op fires when ((!completed) & pred_mask) == 0.
//! This is branchless SWAR evaluation of the dependency graph.
//!
//! Architecture:
//!   - Each op has pred_mask (prerequisites) and succ_mask (completion signal)
//!   - run_topology() loops until all ops in the DAG are complete
//!   - TypeState structs enforce correct op ordering at compile time
//!   - Five topology op arrays (one per Phase) specify different search graphs
//!
//! The value of POWL is topology-derived concurrency:
//!   Ops whose pred_mask is satisfied can run in parallel.
//!   The scheduler finds runnable ops from mask algebra; no explicit spawn needed.
#![cfg(feature = "std")]
extern crate std;

use chess::{Board, ChessMove};
use std::time::Instant;
use crate::phase::{Phase, TopologyId};

// ---------------------------------------------------------------------------
// Op kinds — the vocabulary of operations in the search DAG
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpKind {
    /// Probe the opening book. Sets best_move if hit.
    BookProbe,
    /// Run phase admission (phase::admit). Sets topology.
    PhaseAdmit,
    /// Probe the transposition table. Sets tt_hint.
    TtProbe,
    /// Generate all legal moves. Sets move_list.
    MoveGen,
    /// Order moves by TT hint + history + killers + SEE.
    MoveOrder,
    /// Iterative deepening alpha-beta. Sets best_move + score.
    IterativeDeepening,
    /// Store result in transposition table.
    TtStore,
    /// Emit OCEL move receipt.
    ReceiptEmit,
}

// ---------------------------------------------------------------------------
// A single schedulable operation
// ---------------------------------------------------------------------------

pub struct Powl64Op {
    /// All bits must be set in completed before this op fires.
    pub pred_mask: u64,
    /// Bits to OR into completed when this op finishes.
    pub succ_mask: u64,
    pub kind: OpKind,
}

// ---------------------------------------------------------------------------
// Bit assignments for the op dependency graph
// ---------------------------------------------------------------------------
// Bit 0: BookProbe done
// Bit 1: PhaseAdmit done
// Bit 2: TtProbe done
// Bit 3: MoveGen done
// Bit 4: MoveOrder done
// Bit 5: IterativeDeepening done
// Bit 6: TtStore done
// Bit 7: ReceiptEmit done

const BIT_BOOK:  u64 = 1 << 0;
const BIT_ADMIT: u64 = 1 << 1;
const BIT_TT:    u64 = 1 << 2;
const BIT_MOVES: u64 = 1 << 3;
const BIT_ORDER: u64 = 1 << 4;
const BIT_SEARCH:u64 = 1 << 5;
const BIT_STORE: u64 = 1 << 6;
const BIT_RCPT:  u64 = 1 << 7;

// ---------------------------------------------------------------------------
// Topology op arrays — one DAG per Phase
// Different phases route to different search graphs.
// ---------------------------------------------------------------------------

/// Standard opening: book → admit → tt → moves → order → search → store → receipt
pub const OPENING_OPS: &[Powl64Op] = &[
    Powl64Op { pred_mask: 0,          succ_mask: BIT_BOOK,   kind: OpKind::BookProbe },
    Powl64Op { pred_mask: 0,          succ_mask: BIT_ADMIT,  kind: OpKind::PhaseAdmit },
    Powl64Op { pred_mask: BIT_ADMIT,  succ_mask: BIT_TT,     kind: OpKind::TtProbe },
    Powl64Op { pred_mask: BIT_TT,     succ_mask: BIT_MOVES,  kind: OpKind::MoveGen },
    Powl64Op { pred_mask: BIT_MOVES | BIT_TT, succ_mask: BIT_ORDER, kind: OpKind::MoveOrder },
    Powl64Op { pred_mask: BIT_ORDER,  succ_mask: BIT_SEARCH, kind: OpKind::IterativeDeepening },
    Powl64Op { pred_mask: BIT_SEARCH, succ_mask: BIT_STORE,  kind: OpKind::TtStore },
    Powl64Op { pred_mask: BIT_STORE,  succ_mask: BIT_RCPT,   kind: OpKind::ReceiptEmit },
];

/// Tactical: skip book probe (position is in crisis) — admit → tt → moves → order → search → store → receipt
pub const TACTICAL_OPS: &[Powl64Op] = &[
    Powl64Op { pred_mask: 0,          succ_mask: BIT_BOOK,   kind: OpKind::BookProbe },
    Powl64Op { pred_mask: 0,          succ_mask: BIT_ADMIT,  kind: OpKind::PhaseAdmit },
    Powl64Op { pred_mask: BIT_ADMIT,  succ_mask: BIT_TT,     kind: OpKind::TtProbe },
    Powl64Op { pred_mask: BIT_TT,     succ_mask: BIT_MOVES,  kind: OpKind::MoveGen },
    Powl64Op { pred_mask: BIT_MOVES | BIT_TT, succ_mask: BIT_ORDER, kind: OpKind::MoveOrder },
    Powl64Op { pred_mask: BIT_ORDER,  succ_mask: BIT_SEARCH, kind: OpKind::IterativeDeepening },
    Powl64Op { pred_mask: BIT_SEARCH, succ_mask: BIT_STORE,  kind: OpKind::TtStore },
    Powl64Op { pred_mask: BIT_STORE,  succ_mask: BIT_RCPT,   kind: OpKind::ReceiptEmit },
];

/// Same DAG for Quiet, Endgame (topology differences are in SearchCtx flags, not op order)
pub const QUIET_OPS: &[Powl64Op]   = OPENING_OPS;
pub const ENDGAME_OPS: &[Powl64Op] = OPENING_OPS;

/// Tablebase: admit → moves (pick best SEE move) → receipt
pub const TABLEBASE_OPS: &[Powl64Op] = &[
    Powl64Op { pred_mask: 0,          succ_mask: BIT_ADMIT,  kind: OpKind::PhaseAdmit },
    Powl64Op { pred_mask: BIT_ADMIT,  succ_mask: BIT_MOVES,  kind: OpKind::MoveGen },
    Powl64Op { pred_mask: BIT_MOVES,  succ_mask: BIT_SEARCH, kind: OpKind::IterativeDeepening },
    Powl64Op { pred_mask: BIT_SEARCH, succ_mask: BIT_RCPT,   kind: OpKind::ReceiptEmit },
];

/// Select the op array for a given topology.
pub fn ops_for_topology(topology: TopologyId) -> &'static [Powl64Op] {
    match topology {
        TopologyId::OPENING_MICRO_SINGLE
        | TopologyId::OPENING_MICRO_SMALL
        | TopologyId::OPENING_MICRO_LARGE  => OPENING_OPS,
        TopologyId::TACTICAL_MICRO          => TACTICAL_OPS,
        TopologyId::QUIET_MICRO_SINGLE
        | TopologyId::QUIET_MICRO_SMALL
        | TopologyId::QUIET_MICRO_LARGE    => QUIET_OPS,
        TopologyId::ENDGAME_MICRO           => ENDGAME_OPS,
        TopologyId::TABLEBASE_MICRO         => TABLEBASE_OPS,
        _                                   => OPENING_OPS,
    }
}

// ---------------------------------------------------------------------------
// TypeState markers — compile-time phase enforcement
// These zero-sized types prove that ops ran in the correct order.
// ---------------------------------------------------------------------------

pub struct Initial;
pub struct BookChecked;
pub struct PhaseAdmitted;
pub struct TtProbed;
pub struct MovesGenerated;
pub struct MovesOrdered;
pub struct Searched;
pub struct Resolved;

/// A typed search handle. The phase parameter S prevents calling ops out of order.
/// In single-threaded execution this is a compile-time documentation artifact;
/// in multi-threaded execution the type system prevents data-race-prone patterns.
pub struct SearchHandle<'b, S> {
    pub board: &'b Board,
    pub budget_us: u128,
    pub start: Instant,
    pub best_move: Option<ChessMove>,
    pub score: i32,
    pub depth_reached: u8,
    pub nodes: u32,
    pub topology: TopologyId,
    pub phase: Phase,
    pub endgame_mode: bool,
    pub allow_null: bool,
    pub tt_hint: Option<ChessMove>,
    _state: std::marker::PhantomData<S>,
}

impl<'b> SearchHandle<'b, Initial> {
    pub fn new(board: &'b Board, budget_us: u128, topology: TopologyId, phase: Phase) -> Self {
        SearchHandle {
            board, budget_us,
            start: Instant::now(),
            best_move: None, score: 0,
            depth_reached: 0, nodes: 0,
            topology, phase,
            endgame_mode: phase == Phase::Endgame,
            allow_null: phase != Phase::Endgame,
            tt_hint: None,
            _state: std::marker::PhantomData,
        }
    }
    pub fn admit_phase(self) -> SearchHandle<'b, PhaseAdmitted> {
        SearchHandle {
            board: self.board, budget_us: self.budget_us, start: self.start,
            best_move: self.best_move, score: self.score,
            depth_reached: self.depth_reached, nodes: self.nodes,
            topology: self.topology, phase: self.phase,
            endgame_mode: self.endgame_mode, allow_null: self.allow_null,
            tt_hint: self.tt_hint,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'b> SearchHandle<'b, PhaseAdmitted> {
    pub fn set_tt_hint(mut self, hint: Option<ChessMove>) -> SearchHandle<'b, TtProbed> {
        self.tt_hint = hint;
        SearchHandle {
            board: self.board, budget_us: self.budget_us, start: self.start,
            best_move: self.best_move, score: self.score,
            depth_reached: self.depth_reached, nodes: self.nodes,
            topology: self.topology, phase: self.phase,
            endgame_mode: self.endgame_mode, allow_null: self.allow_null,
            tt_hint: self.tt_hint,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'b> SearchHandle<'b, TtProbed> {
    pub fn moves_generated(self) -> SearchHandle<'b, MovesGenerated> {
        SearchHandle {
            board: self.board, budget_us: self.budget_us, start: self.start,
            best_move: self.best_move, score: self.score,
            depth_reached: self.depth_reached, nodes: self.nodes,
            topology: self.topology, phase: self.phase,
            endgame_mode: self.endgame_mode, allow_null: self.allow_null,
            tt_hint: self.tt_hint,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'b> SearchHandle<'b, MovesGenerated> {
    pub fn moves_ordered(self) -> SearchHandle<'b, MovesOrdered> {
        SearchHandle {
            board: self.board, budget_us: self.budget_us, start: self.start,
            best_move: self.best_move, score: self.score,
            depth_reached: self.depth_reached, nodes: self.nodes,
            topology: self.topology, phase: self.phase,
            endgame_mode: self.endgame_mode, allow_null: self.allow_null,
            tt_hint: self.tt_hint,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'b> SearchHandle<'b, MovesOrdered> {
    pub fn set_result(self, mv: Option<ChessMove>, score: i32, depth: u8, nodes: u32) -> SearchHandle<'b, Searched> {
        SearchHandle {
            board: self.board, budget_us: self.budget_us, start: self.start,
            best_move: mv, score,
            depth_reached: depth, nodes,
            topology: self.topology, phase: self.phase,
            endgame_mode: self.endgame_mode, allow_null: self.allow_null,
            tt_hint: self.tt_hint,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'b> SearchHandle<'b, Searched> {
    pub fn resolve(self) -> SearchHandle<'b, Resolved> {
        SearchHandle {
            board: self.board, budget_us: self.budget_us, start: self.start,
            best_move: self.best_move, score: self.score,
            depth_reached: self.depth_reached, nodes: self.nodes,
            topology: self.topology, phase: self.phase,
            endgame_mode: self.endgame_mode, allow_null: self.allow_null,
            tt_hint: self.tt_hint,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'b> SearchHandle<'b, Resolved> {
    pub fn best_move(&self) -> Option<ChessMove> { self.best_move }
    pub fn score(&self) -> i32 { self.score }
}

// ---------------------------------------------------------------------------
// SWAR scheduler — the runtime POWL engine
// ---------------------------------------------------------------------------

/// A mutable context passed through the POWL op executor.
/// Equivalent to the union of all SearchHandle<S> fields.
pub struct SearchCtx<'b> {
    pub board: &'b Board,
    pub budget_us: u128,
    pub start: Instant,
    pub best_move: Option<ChessMove>,
    pub score: i32,
    pub depth_reached: u8,
    pub nodes: u32,
    pub topology: TopologyId,
    pub phase: Phase,
    pub endgame_mode: bool,
    pub allow_null: bool,
    pub tt_hint: Option<ChessMove>,
    /// Set to true by BookProbe if a book move was found; IterativeDeepening skips.
    pub book_hit: bool,
}

impl<'b> SearchCtx<'b> {
    pub fn new(board: &'b Board, budget_us: u128, topology: TopologyId, phase: Phase) -> Self {
        SearchCtx {
            board, budget_us,
            start: Instant::now(),
            best_move: None, score: 0,
            depth_reached: 0, nodes: 0,
            topology, phase,
            endgame_mode: phase == Phase::Endgame,
            allow_null: phase != Phase::Endgame,
            tt_hint: None,
            book_hit: false,
        }
    }
}

/// Execute all ops in the topology's DAG in dependency order.
/// Fires any op whose pred_mask bits are all set in completed.
/// Runs until all ops have fired (total_done == all succ bits OR).
pub fn run_topology<F>(ops: &[Powl64Op], ctx: &mut SearchCtx, mut execute: F)
where
    F: FnMut(OpKind, &mut SearchCtx),
{
    let total_succ: u64 = ops.iter().fold(0u64, |acc, op| acc | op.succ_mask);
    let mut completed = 0u64;

    while completed != total_succ {
        let mut progress = false;
        for op in ops {
            // Fire if: not yet done AND all prerequisites met
            let not_done = (completed & op.succ_mask) == 0;
            let prereqs_met = (!completed & op.pred_mask) == 0;
            if not_done && prereqs_met {
                execute(op.kind, ctx);
                completed |= op.succ_mask;
                progress = true;
                // Early exit: if book hit, skip remaining non-receipt ops
                if ctx.book_hit && op.kind == OpKind::BookProbe {
                    // Mark all non-receipt bits as done
                    completed |= BIT_ADMIT | BIT_TT | BIT_MOVES | BIT_ORDER | BIT_SEARCH | BIT_STORE;
                }
            }
        }
        if !progress { break; } // cycle guard (unreachable for valid DAGs)
    }
}
