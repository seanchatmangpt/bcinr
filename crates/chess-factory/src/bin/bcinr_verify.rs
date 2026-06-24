//! `bcinr_verify` — replay verifier CLI for manufactured move-receipt chains.
//!
//! Reads a JSON stream of [`MoveReceipt`]s and prints, per move, whether it is
//! compliant (all four laws hold: chain hash, Petri conformance, re-derived
//! decision, legality) plus a final game verdict.
//!
//! Input forms (auto-detected):
//!   - a single JSON array of receipts, or
//!   - newline-delimited JSON (one receipt object per line).
//!
//! Usage:
//!     bcinr_factory ... > game.json
//!     bcinr_verify < game.json
//!     bcinr_verify game.json

use std::io::{self, Read};

use chess_factory::receipts::{verify_chain, MoveReceipt, MoveVerdict};

fn read_input() -> io::Result<String> {
    match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
    }
}

fn parse_receipts(raw: &str) -> Result<Vec<MoveReceipt>, String> {
    let trimmed = raw.trim_start();
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed).map_err(|e| e.to_string());
    }
    // Newline-delimited JSON: one receipt per non-empty line.
    let mut out = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let r: MoveReceipt =
            serde_json::from_str(line).map_err(|e| format!("line {}: {e}", i + 1))?;
        out.push(r);
    }
    Ok(out)
}

fn main() {
    let raw = match read_input() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read error: {e}");
            std::process::exit(2);
        }
    };
    let receipts = match parse_receipts(&raw) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("parse error: {e}");
            std::process::exit(2);
        }
    };

    let verdict = verify_chain(&receipts);
    println!("=== bcinr_verify: {} receipt(s) ===", receipts.len());
    for (i, (receipt, mv)) in receipts.iter().zip(verdict.moves.iter()).enumerate() {
        match mv {
            MoveVerdict::Admit { fitness } => println!(
                "  move {i:3} {:>6}  is_compliant=true  fitness={fitness:.4}",
                receipt.chosen_move
            ),
            MoveVerdict::Refuse(r) => println!(
                "  move {i:3} {:>6}  is_compliant=false refusal={r:?}",
                receipt.chosen_move
            ),
        }
    }
    println!(
        "game verdict: {}",
        if verdict.is_compliant {
            "COMPLIANT — every move re-derives to a lawful, Petri-conforming decision."
        } else {
            "NON-COMPLIANT — at least one move violated a decision law."
        }
    );
    std::process::exit(i32::from(!verdict.is_compliant));
}
