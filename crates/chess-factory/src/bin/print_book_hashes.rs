// print_book_hashes.rs — prints opening_book.rs to stdout.
// Run: cargo run -p chess-factory --bin print_book_hashes 2>/dev/null > src/opening_book.rs
#![cfg(feature = "std")]
extern crate std;
extern crate chess;

use chess::{Board, ChessMove};
use std::str::FromStr;
use std::println;

fn main() {
    // Each entry: (slice of UCI moves to reach the position, UCI reply move string)
    // Covers: startpos replies, 1.e4 / 1.d4 trees ~4-5 plies deep
    // Openings: Ruy Lopez, Italian, Sicilian, French, Caro-Kann, QGD, QID, KID, Nimzo, English
    let entries: &[(&[&str], &str)] = &[
        // --- Startpos ---
        (&[], "e2e4"),

        // --- After 1.e4 ---
        (&["e2e4"], "e7e5"),          // 1...e5
        (&["e2e4"], "c7c5"),          // 1...c5 Sicilian
        (&["e2e4"], "e7e6"),          // 1...e6 French
        (&["e2e4"], "c7c6"),          // 1...c6 Caro-Kann
        (&["e2e4"], "d7d5"),          // 1...d5 Scandinavian
        (&["e2e4"], "d7d6"),          // 1...d6 Pirc

        // --- 1.e4 e5 lines ---
        (&["e2e4","e7e5"], "g1f3"),   // 2.Nf3
        (&["e2e4","e7e5","g1f3"], "b8c6"), // 2...Nc6
        (&["e2e4","e7e5","g1f3","b8c6"], "f1b5"), // 3.Bb5 Ruy Lopez
        (&["e2e4","e7e5","g1f3","b8c6","f1b5"], "a7a6"), // 3...a6
        (&["e2e4","e7e5","g1f3","b8c6","f1b5","a7a6"], "b5a4"), // 4.Ba4
        (&["e2e4","e7e5","g1f3","b8c6","f1b5","a7a6","b5a4"], "g8f6"), // 4...Nf6
        (&["e2e4","e7e5","g1f3","b8c6","f1b5","a7a6","b5a4","g8f6"], "e1g1"), // 5.O-O
        (&["e2e4","e7e5","g1f3","b8c6","f1b5","a7a6","b5a4","g8f6","e1g1"], "f8e7"), // 5...Be7

        // Italian
        (&["e2e4","e7e5","g1f3","b8c6","f1c4"], "f8c5"), // 3.Bc4 3...Bc5
        (&["e2e4","e7e5","g1f3","b8c6","f1c4","f8c5"], "c2c3"), // 4.c3
        (&["e2e4","e7e5","g1f3","b8c6","f1c4","f8c5","c2c3"], "g8f6"), // 4...Nf6
        (&["e2e4","e7e5","g1f3","b8c6","f1c4","f8c5","c2c3","g8f6"], "d2d4"), // 5.d4
        (&["e2e4","e7e5","g1f3","b8c6","f1c4","f8c5","c2c3","g8f6","d2d4"], "e5d4"), // 5...exd4

        // --- Sicilian ---
        (&["e2e4","c7c5"], "g1f3"),   // 2.Nf3
        (&["e2e4","c7c5","g1f3"], "d7d6"), // 2...d6 Najdorf setup
        (&["e2e4","c7c5","g1f3","d7d6"], "d2d4"), // 3.d4
        (&["e2e4","c7c5","g1f3","d7d6","d2d4"], "c5d4"), // 3...cxd4
        (&["e2e4","c7c5","g1f3","d7d6","d2d4","c5d4"], "f3d4"), // 4.Nxd4
        (&["e2e4","c7c5","g1f3","d7d6","d2d4","c5d4","f3d4"], "g8f6"), // 4...Nf6
        (&["e2e4","c7c5","g1f3","d7d6","d2d4","c5d4","f3d4","g8f6"], "b1c3"), // 5.Nc3
        (&["e2e4","c7c5","g1f3","d7d6","d2d4","c5d4","f3d4","g8f6","b1c3"], "a7a6"), // 5...a6 Najdorf

        // Sicilian 2...Nc6
        (&["e2e4","c7c5","g1f3"], "b8c6"),
        (&["e2e4","c7c5","g1f3","b8c6"], "d2d4"),
        (&["e2e4","c7c5","g1f3","b8c6","d2d4"], "c5d4"),
        (&["e2e4","c7c5","g1f3","b8c6","d2d4","c5d4"], "f3d4"),
        (&["e2e4","c7c5","g1f3","b8c6","d2d4","c5d4","f3d4"], "g7g6"), // Dragon
        (&["e2e4","c7c5","g1f3","b8c6","d2d4","c5d4","f3d4","g7g6"], "b1c3"),

        // --- French ---
        (&["e2e4","e7e6"], "d2d4"),
        (&["e2e4","e7e6","d2d4"], "d7d5"),
        (&["e2e4","e7e6","d2d4","d7d5"], "b1c3"), // 3.Nc3
        (&["e2e4","e7e6","d2d4","d7d5","b1c3"], "g8f6"), // 3...Nf6
        (&["e2e4","e7e6","d2d4","d7d5","b1c3","g8f6"], "e4e5"), // 4.e5
        (&["e2e4","e7e6","d2d4","d7d5","b1c3","g8f6","e4e5"], "f6d7"), // 4...Nfd7
        (&["e2e4","e7e6","d2d4","d7d5","b1c3","g8f6","e4e5","f6d7"], "f2f4"), // 5.f4

        // French exchange
        (&["e2e4","e7e6","d2d4","d7d5"], "e4d5"),
        (&["e2e4","e7e6","d2d4","d7d5","e4d5"], "e6d5"),
        (&["e2e4","e7e6","d2d4","d7d5","e4d5","e6d5"], "g1f3"),

        // --- Caro-Kann ---
        (&["e2e4","c7c6"], "d2d4"),
        (&["e2e4","c7c6","d2d4"], "d7d5"),
        (&["e2e4","c7c6","d2d4","d7d5"], "b1c3"), // 3.Nc3
        (&["e2e4","c7c6","d2d4","d7d5","b1c3"], "d5e4"), // 3...dxe4
        (&["e2e4","c7c6","d2d4","d7d5","b1c3","d5e4"], "c3e4"), // 4.Nxe4
        (&["e2e4","c7c6","d2d4","d7d5","b1c3","d5e4","c3e4"], "g8f6"), // 4...Nf6
        (&["e2e4","c7c6","d2d4","d7d5","b1c3","d5e4","c3e4","g8f6"], "e4f6"), // 5.Nxf6+

        // --- 1.d4 lines ---
        (&["d2d4"], "d7d5"),
        (&["d2d4"], "g8f6"),
        (&["d2d4"], "e7e6"),
        (&["d2d4"], "f7f5"),          // Dutch

        // QGD
        (&["d2d4","d7d5"], "c2c4"),
        (&["d2d4","d7d5","c2c4"], "e7e6"), // 2...e6 QGD
        (&["d2d4","d7d5","c2c4","e7e6"], "b1c3"),
        (&["d2d4","d7d5","c2c4","e7e6","b1c3"], "g8f6"),
        (&["d2d4","d7d5","c2c4","e7e6","b1c3","g8f6"], "c1g5"),
        (&["d2d4","d7d5","c2c4","e7e6","b1c3","g8f6","c1g5"], "f8e7"),
        (&["d2d4","d7d5","c2c4","e7e6","b1c3","g8f6","c1g5","f8e7"], "e2e3"),

        // QGA
        (&["d2d4","d7d5","c2c4"], "d5c4"), // 2...dxc4 QGA
        (&["d2d4","d7d5","c2c4","d5c4"], "g1f3"),
        (&["d2d4","d7d5","c2c4","d5c4","g1f3"], "g8f6"),

        // KID
        (&["d2d4","g8f6"], "c2c4"),
        (&["d2d4","g8f6","c2c4"], "g7g6"),
        (&["d2d4","g8f6","c2c4","g7g6"], "b1c3"),
        (&["d2d4","g8f6","c2c4","g7g6","b1c3"], "f8g7"),
        (&["d2d4","g8f6","c2c4","g7g6","b1c3","f8g7"], "e2e4"), // KID main line
        (&["d2d4","g8f6","c2c4","g7g6","b1c3","f8g7","e2e4"], "d7d6"),
        (&["d2d4","g8f6","c2c4","g7g6","b1c3","f8g7","e2e4","d7d6"], "g1f3"),
        (&["d2d4","g8f6","c2c4","g7g6","b1c3","f8g7","e2e4","d7d6","g1f3"], "e8g8"), // castles

        // Nimzo-Indian
        (&["d2d4","g8f6","c2c4","e7e6"], "b1c3"),
        (&["d2d4","g8f6","c2c4","e7e6","b1c3"], "f8b4"), // Nimzo
        (&["d2d4","g8f6","c2c4","e7e6","b1c3","f8b4"], "e2e3"),
        (&["d2d4","g8f6","c2c4","e7e6","b1c3","f8b4","e2e3"], "e8g8"), // castles

        // Queen's Indian
        (&["d2d4","g8f6","c2c4","e7e6","g1f3"], "b7b6"), // QID
        (&["d2d4","g8f6","c2c4","e7e6","g1f3","b7b6"], "g2g3"),
        (&["d2d4","g8f6","c2c4","e7e6","g1f3","b7b6","g2g3"], "c8b7"),
        (&["d2d4","g8f6","c2c4","e7e6","g1f3","b7b6","g2g3","c8b7"], "f1g2"),
        (&["d2d4","g8f6","c2c4","e7e6","g1f3","b7b6","g2g3","c8b7","f1g2"], "f8e7"),
    ];

    // Build (hash, move_str) pairs
    let mut pairs: std::vec::Vec<(u64, std::string::String)> = std::vec::Vec::new();
    for (moves, reply) in entries {
        let mut board = Board::default();
        let mut ok = true;
        for m in *moves {
            match ChessMove::from_str(m) {
                Ok(cm) => { board = board.make_move_new(cm); }
                Err(_) => { ok = false; break; }
            }
        }
        if !ok { continue; }
        let hash = board.get_hash();
        pairs.push((hash, reply.to_string()));
    }

    // Sort by hash for binary search
    pairs.sort_by_key(|p| p.0);
    pairs.dedup_by_key(|p| p.0);

    // Print opening_book.rs
    println!("// opening_book.rs — generated by print_book_hashes; DO NOT EDIT BY HAND.");
    println!("// Re-generate: cargo run -p chess-factory --bin print_book_hashes 2>/dev/null > src/opening_book.rs");
    println!("#![cfg(feature = \"std\")]");
    println!("extern crate std;");
    println!("extern crate chess;");
    println!();
    println!("use chess::ChessMove;");
    println!("use std::str::FromStr;");
    println!();
    println!("static BOOK: &[(u64, &str)] = &[");
    for (hash, mv) in &pairs {
        println!("    ({}, \"{}\"),", hash, mv);
    }
    println!("];");
    println!();
    println!("/// Probe the opening book for a position hash.");
    println!("/// Returns a legal `ChessMove` if the position is in the book.");
    println!("pub fn book_probe(hash: u64) -> Option<ChessMove> {{");
    println!("    let idx = BOOK.partition_point(|&(h, _)| h < hash);");
    println!("    if idx < BOOK.len() && BOOK[idx].0 == hash {{");
    println!("        ChessMove::from_str(BOOK[idx].1).ok()");
    println!("    }} else {{");
    println!("        None");
    println!("    }}");
    println!("}}");
}
