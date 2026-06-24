//! Diagnostic tool: load SF NNUE file, print weight stats, eval test positions.
#![cfg(feature = "std")]

extern crate std;
use std::path::Path;

fn main() {
    let path_str = std::env::args().nth(1).unwrap_or_else(|| {
        "/tmp/nn-small.nnue".to_string()
    });
    let path = Path::new(&path_str);

    match chess_factory::nnue_sf::print_weight_stats(path) {
        Ok(()) => {}
        Err(e) => {
            std::eprintln!("Error loading {}: {}", path.display(), e);
            std::process::exit(1);
        }
    }

    // Diagnostics: check accumulation for startpos
    {
        use chess::{Board, Color};
        let board: Board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".parse().unwrap();
        let combined = board.combined();
        std::println!("\nDiagnostic (startpos WHITE perspective):");
        std::println!("  combined bitboard: {:#018x}", combined.0);
        std::println!("  piece count: {}", combined.popcnt());
        let king_sq = board.king_square(Color::White).to_index();
        std::println!("  white king at sq {}", king_sq);

        // Count features that would be added
        let mut feat_count = 0;
        let mut bb = combined.0;
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            let sq_chess = chess::ALL_SQUARES[sq];
            if board.piece_on(sq_chess).is_some() { feat_count += 1; }
        }
        std::println!("  feature candidates: {}", feat_count);
    }

    // Load weights into global store and eval a few positions
    if chess_factory::nnue_sf::load_nnue(path) {
        std::println!("\nWeights loaded OK — evaluating test positions:");
        let positions = [
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "startpos"),
            ("rnbqkb1r/pp2pppp/3p1n2/2p5/3PP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 0 5", "Sicilian after 5.Nc3"),
            ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", "endgame pawns"),
            ("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4", "Italian Game"),
        ];
        for (fen, name) in &positions {
            let board: chess::Board = fen.parse().expect("valid fen");
            match chess_factory::nnue_sf::nnue_sf_eval(&board) {
                Some(cp) => std::println!("  {:<35} {:>+6} cp (SF NNUE)", name, cp),
                None     => std::println!("  {:<35} weights not loaded", name),
            }
        }
    } else {
        std::eprintln!("Failed to load weights from {}", path.display());
    }
}
