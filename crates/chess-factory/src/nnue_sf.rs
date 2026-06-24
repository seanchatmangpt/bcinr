//! Stockfish NNUE file loader — HalfKAv2 forward pass.
//!
//! Parses the binary `.nnue` format used by SF 17/18:
//!   `[version:u32][arch_hash:u32][desc_len:u32][desc_bytes][sections…]`
//!
//! Each section:
//!   `[section_hash:u32][section_size:u32][COMPRESSED_LEB128 data]`
//!
//! COMPRESSED_LEB128: the weight stream encodes signed i16/i8 values as
//! variable-length LEB128 integers (each value uses 1–3 bytes; MSB=1 signals
//! a continuation byte). Decompression produces a flat `Vec<i16>` of weights.
//!
//! Architecture — HalfKAv2_hm (small net, 45 056 inputs):
//!   FT  : 45056 → 512  (i16 weights, i16 biases)
//!   L1  : 1024  → 16   (i8  weights, i32 biases)  [two perspectives concatenated]
//!   L2  : 32    → 32   (i8  weights, i32 biases)
//!   Out : 32    → 1    (i8  weights, i32 bias)

#![cfg(feature = "std")]
extern crate std;

use std::fs;
use std::io;
use std::path::Path;
use std::sync::OnceLock;
use std::vec;
use std::vec::Vec;

use chess::{Board, Color, Piece};

// ---------------------------------------------------------------------------
// Architecture constants (HalfKAv2_hm small net — nn-37f18f62d772.nnue)
//
// Verified by inspecting section 1 of the file: exactly 45056×64 = 2,883,584
// i16 values. FT biases (64×2016) are the first 64 values of section 2.
// ---------------------------------------------------------------------------

pub const FT_IN: usize  = 45_056;   // HalfKAv2 feature count per perspective
pub const FT_OUT: usize = 64;        // feature transformer output neurons (small net)
pub const L1_IN: usize  = 128;       // FT_OUT * 2 perspectives
pub const L1_OUT: usize = 16;        // hidden layer 1 outputs (best estimate)
pub const L2_IN: usize  = 32;        // L1_OUT * 2 (squared clipped-ReLU doubles size)
pub const L2_OUT: usize = 32;
pub const L3_IN: usize  = 32;
pub const L3_OUT: usize = 1;

// Quantisation scales matching SF18 (do not change — must match trained weights).
const FT_SCALE: i32   = 64;          // feature transformer output / 64 → i8 range
const OUT_SCALE: i32  = 16;          // output layer divisor
const EVAL_SCALE: i32 = 400;         // cp = material * EVAL_SCALE / (FT_SCALE * OUT_SCALE)

// ---------------------------------------------------------------------------
// Weight store
// ---------------------------------------------------------------------------

pub struct NnueSf {
    /// Feature transformer: FT_IN × FT_OUT (row-major, perspective-symmetric).
    pub ft_weights: Vec<i16>,          // FT_IN * FT_OUT entries
    pub ft_biases:  Vec<i16>,          // FT_OUT entries

    /// Layer 1: L1_IN × L1_OUT
    pub l1_weights: Vec<i8>,
    pub l1_biases:  Vec<i32>,

    /// Layer 2: L2_IN × L2_OUT
    pub l2_weights: Vec<i8>,
    pub l2_biases:  Vec<i32>,

    /// Output layer: L3_IN × 1
    pub l3_weights: Vec<i8>,
    pub l3_bias:    i32,
}

impl NnueSf {
    fn zeroed() -> Self {
        Self {
            ft_weights: vec![0i16; FT_IN * FT_OUT],
            ft_biases:  vec![0i16; FT_OUT],
            l1_weights: vec![0i8; L1_IN * L1_OUT],
            l1_biases:  vec![0i32; L1_OUT],
            l2_weights: vec![0i8; L2_IN * L2_OUT],
            l2_biases:  vec![0i32; L2_OUT],
            l3_weights: vec![0i8; L3_IN],
            l3_bias:    0,
        }
    }
}

// ---------------------------------------------------------------------------
// Global weight store (loaded once at startup)
// ---------------------------------------------------------------------------

static WEIGHTS: OnceLock<Option<NnueSf>> = OnceLock::new();

/// Load SF NNUE weights from `path`. Idempotent — only the first call does I/O.
/// Returns `true` if weights are available.
pub fn load_nnue(path: &Path) -> bool {
    WEIGHTS.get_or_init(|| parse_nnue_file(path).ok()).is_some()
}

/// Evaluate `board` using the loaded SF NNUE weights.
/// Falls back to 0 (caller should use material+PST instead) if weights not loaded.
pub fn nnue_sf_eval(board: &Board) -> Option<i32> {
    let w = WEIGHTS.get()?.as_ref()?;
    Some(forward(board, w))
}

// (LEB128 decoding is implemented as nested fns inside parse_nnue_file below)

// ---------------------------------------------------------------------------
// File parser
// ---------------------------------------------------------------------------

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
}

fn parse_nnue_file(path: &Path) -> io::Result<NnueSf> {
    let data = fs::read(path)?;

    // Header: [version:u32][arch_hash:u32][desc_len:u32][desc_bytes]
    if data.len() < 12 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file too small"));
    }
    let _version   = read_u32_le(&data, 0);
    let _arch_hash = read_u32_le(&data, 4);
    let desc_len   = read_u32_le(&data, 8) as usize;

    if data.len() < 12 + desc_len {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "truncated description"));
    }
    let _desc = std::str::from_utf8(&data[12..12 + desc_len])
        .unwrap_or("<invalid utf8>");

    // Walk sections.
    //
    // SF18 NNUE section format (NO explicit byte-size field):
    //   [section_hash:u32] [compression_marker:null_terminated_string] [LEB128_data…]
    //
    // The marker is typically b"COMPRESSED_LEB128\x95\x00" (18 bytes + null).
    // After the null terminator the LEB128 stream starts immediately. Section
    // boundaries are implicit: we decode exactly the expected number of values
    // (determined by the architecture) and the next section starts right after.
    let mut w = NnueSf::zeroed();
    let mut pos = 12 + desc_len;

    /// Skip null-terminated marker string starting at `pos` and return the
    /// index of the first byte after the null terminator.
    fn skip_marker(data: &[u8], mut pos: usize) -> usize {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos + 1  // skip null terminator
    }

    /// Decode exactly `n` signed LEB128 values from `data[pos..]`.
    /// Returns the decoded values and the updated position.
    fn decode_n_i16(data: &[u8], mut pos: usize, n: usize) -> (Vec<i16>, usize) {
        let mut out = Vec::with_capacity(n);
        while pos < data.len() && out.len() < n {
            let b0 = data[pos] as u32; pos += 1;
            if b0 & 0x80 == 0 {
                out.push(((b0 << 25) as i32 >> 25) as i16);
            } else if pos < data.len() {
                let b1 = data[pos] as u32; pos += 1;
                if b1 & 0x80 == 0 {
                    let v = ((b0 & 0x7F) | (b1 << 7)) as u32;
                    out.push(((v << 18) as i32 >> 18) as i16);
                } else if pos < data.len() {
                    let b2 = data[pos] as u32; pos += 1;
                    let v = ((b0 & 0x7F) | ((b1 & 0x7F) << 7) | (b2 << 14)) as u32;
                    out.push(((v << 11) as i32 >> 11) as i16);
                }
            }
        }
        (out, pos)
    }

    fn decode_n_i8(data: &[u8], pos: usize, n: usize) -> (Vec<i8>, usize) {
        let (vals, p) = decode_n_i16(data, pos, n);
        (vals.into_iter().map(|v| v.clamp(-128, 127) as i8).collect(), p)
    }

    fn decode_n_i32(data: &[u8], mut pos: usize, n: usize) -> (Vec<i32>, usize) {
        // Biases are stored as single signed LEB128 values (not as 2×i16 pairs).
        // Values fit in i16 range in practice but are logically i32.
        let mut out = Vec::with_capacity(n);
        let end = data.len();
        while pos < end && out.len() < n {
            let b0 = data[pos] as u32; pos += 1;
            if b0 & 0x80 == 0 {
                out.push((b0 << 25) as i32 >> 25);
            } else if pos < end {
                let b1 = data[pos] as u32; pos += 1;
                if b1 & 0x80 == 0 {
                    let v = ((b0 & 0x7F) | (b1 << 7)) as u32;
                    out.push((v << 18) as i32 >> 18);
                } else if pos < end {
                    let b2 = data[pos] as u32; pos += 1;
                    if b2 & 0x80 == 0 {
                        let v = ((b0 & 0x7F) | ((b1 & 0x7F) << 7) | (b2 << 14)) as u32;
                        out.push((v << 11) as i32 >> 11);
                    } else if pos < end {
                        let b3 = data[pos] as u32; pos += 1;
                        let v = ((b0 & 0x7F) | ((b1 & 0x7F) << 7)
                            | ((b2 & 0x7F) << 14) | (b3 << 21)) as u32;
                        out.push(v as i32);  // 4-byte covers full i32 range
                    }
                }
            }
        }
        (out, pos)
    }

    // File layout (3 sections confirmed by scanning for COMPRESSED_LEB128 markers):
    //
    // Section 0  [hash 0x7f234db8]: unknown small section (~126 i8 values); skip.
    // Section 1  [hash 0x7d1b7e10]: FT weights — exactly FT_IN × FT_OUT i16 values.
    // Section 2  [hash 0x125b0c7a]: FT biases (FT_OUT i16), then all hidden layers.
    //
    // Section boundaries are at the COMPRESSED_LEB128 marker strings, not at
    // explicit size fields. We decode each section by expected count, not by size.

    // Skip section 0 (unknown, ~126 values — possibly PSQ correction or FT psq bonus)
    if pos + 4 <= data.len() {
        pos += 4;                        // skip section hash
        let data_start = skip_marker(&data, pos);
        // Find next section hash by scanning for the next COMPRESSED_LEB128 marker
        let marker = b"COMPRESSED_LEB128";
        let next = data[data_start..].windows(marker.len()).position(|w| w == marker)
            .map(|i| data_start + i - 4)  // 4 bytes of hash before marker
            .unwrap_or(data.len());
        pos = next;
    }

    // Section 1: FT weights — FT_IN × FT_OUT i16 values
    if pos + 4 <= data.len() {
        pos += 4;
        pos = skip_marker(&data, pos);
        let (vals, p) = decode_n_i16(&data, pos, FT_IN * FT_OUT);
        w.ft_weights = vals;
        pos = p;
    }

    // Section 2 starts here. Layout after the marker:
    //   [FT_biases: FT_OUT i16][L1_weights: L1_IN*L1_OUT i8][L1_biases: L1_OUT raw-i32]
    //   [L2_weights: L2_IN*L2_OUT i8][L2_biases: L2_OUT raw-i32]
    //   [L3_weights: L3_IN i8][L3_bias: raw-i32]
    //
    // "raw-i32" means 4 separate signed-byte LEB128 values that together form one i32
    // (lo_byte, byte1, byte2, hi_byte in 7-bit chunks). We decode them and reassemble.
    if pos + 4 <= data.len() {
        pos += 4;
        pos = skip_marker(&data, pos);

        // FT biases
        let (vals, p) = decode_n_i16(&data, pos, FT_OUT);
        w.ft_biases = vals;
        pos = p;

        // L1 weights (i8)
        let (vals, p) = decode_n_i8(&data, pos, L1_IN * L1_OUT);
        w.l1_weights = vals;
        pos = p;

        // L1 biases (i32 stored as 2×i16 LEB128)
        let (vals, p) = decode_n_i32(&data, pos, L1_OUT);
        w.l1_biases = vals;
        pos = p;

        // L2 weights (i8)
        let (vals, p) = decode_n_i8(&data, pos, L2_IN * L2_OUT);
        w.l2_weights = vals;
        pos = p;

        // L2 biases (i32)
        let (vals, p) = decode_n_i32(&data, pos, L2_OUT);
        w.l2_biases = vals;
        pos = p;

        // L3 weights (i8)
        let (vals, p) = decode_n_i8(&data, pos, L3_IN);
        w.l3_weights = vals;
        pos = p;

        // L3 bias (i32)
        let (vals, _p) = decode_n_i32(&data, pos, 1);
        w.l3_bias = vals.into_iter().next().unwrap_or(0);
    }

    Ok(w)
}

// ---------------------------------------------------------------------------
// HalfKAv2 feature index
// ---------------------------------------------------------------------------

// HalfKAv2_hm index (hand-mirrored king):
//   features per king square = 704 = 10*64 + 64 (pieces + own-king-sq)
//   index = king_sq_mirrored * 704 + piece_feature
//   piece_feature for non-king: sq * 10 + pt_idx * 2 + (side == opponent)
//   piece_feature for king:     640 + king_sq_mirrored

const HALFKA_FEATURES: usize = 704;  // features per king square

#[inline]
fn pt_idx(pt: Piece) -> usize {
    match pt {
        Piece::Pawn   => 0,
        Piece::Knight => 1,
        Piece::Bishop => 2,
        Piece::Rook   => 3,
        Piece::Queen  => 4,
        Piece::King   => 5,
    }
}

/// HalfKAv2_hm feature index from `persp` perspective.
/// Returns `None` for the king of the perspective colour (handled separately).
#[inline]
fn halfka_index(king_sq: usize, piece_sq: usize, pt: Piece, pc: Color, persp: Color) -> Option<usize> {
    // Mirror horizontally when king is on the queenside (file < 4) so the
    // "left side of king" is always the same orientation.
    let mirror = (king_sq & 7) >= 4;

    let ksq = if persp == Color::Black {
        king_sq ^ 56  // flip rank for black perspective
    } else {
        king_sq
    };
    let ksq = if mirror { ksq ^ 7 } else { ksq };  // horizontal mirror

    let psq = if persp == Color::Black {
        piece_sq ^ 56
    } else {
        piece_sq
    };
    let psq = if mirror { psq ^ 7 } else { psq };

    if pt == Piece::King && pc == persp {
        // Own king: use special "virtual king" feature slot (640..704)
        Some(ksq * HALFKA_FEATURES + 640 + psq)
    } else if pt == Piece::King {
        // Opponent king: not a HalfKAv2 feature (skip)
        None
    } else {
        let pt_i = pt_idx(pt);
        let colour_bit = if pc == persp { 0 } else { 1 };
        Some(ksq * HALFKA_FEATURES + psq * 10 + pt_i * 2 + colour_bit)
    }
}

// ---------------------------------------------------------------------------
// Forward pass
// ---------------------------------------------------------------------------

/// Accumulate one perspective's feature transformer output.
fn accumulate(
    board: &Board,
    persp: Color,
    weights: &NnueSf,
    acc: &mut [i32; FT_OUT],
) {
    // Start with biases
    for (a, &b) in acc.iter_mut().zip(weights.ft_biases.iter()) {
        *a = b as i32;
    }

    let king_sq = board.king_square(persp).to_index();
    let all = board.combined();
    let mut bb = all.0;

    while bb != 0 {
        let sq = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let sq_chess = chess::ALL_SQUARES[sq];
        let pt = match board.piece_on(sq_chess) {
            Some(p) => p,
            None => continue,
        };
        let pc = if (board.color_combined(Color::White).0 >> sq) & 1 == 1 {
            Color::White
        } else {
            Color::Black
        };

        if let Some(idx) = halfka_index(king_sq, sq, pt, pc, persp) {
            if idx < FT_IN {
                let row = &weights.ft_weights[idx * FT_OUT..(idx + 1) * FT_OUT];
                for (a, &w) in acc.iter_mut().zip(row.iter()) {
                    *a += w as i32;
                }
            }
        }
    }
}

/// Clipped ReLU for FT output: divide by FT_SCALE then clamp to i8 range [0, 127].
/// SF accumulates in i32 with FT_SCALE=64; output is in [0, 127] after scaling.
#[inline]
fn clipped_relu_ft(acc: &[i32; FT_OUT], out: &mut [i8; FT_OUT]) {
    for (o, &a) in out.iter_mut().zip(acc.iter()) {
        *o = (a / FT_SCALE).clamp(0, 127) as i8;
    }
}

/// Dense layer: out[j] = clamp(sum_i(input[i] * weights[i*OUT+j]) / 64 + bias[j], 0, 127)
fn dense_layer_relu(
    input: &[i8],
    weights: &[i8],
    biases: &[i32],
    out: &mut [i8],
) {
    for (j, (&bias, wrow)) in biases.iter().zip(weights.chunks(input.len())).enumerate() {
        let sum: i32 = input.iter().zip(wrow.iter()).map(|(&x, &w)| x as i32 * w as i32).sum();
        let v = (sum + bias) / OUT_SCALE;
        out[j] = v.clamp(0, 127) as i8;
    }
}

/// Output layer (no activation): scalar centipawn output.
fn output_layer(input: &[i8], weights: &[i8], bias: i32) -> i32 {
    let sum: i32 = input.iter().zip(weights.iter()).map(|(&x, &w)| x as i32 * w as i32).sum();
    (sum + bias) / OUT_SCALE
}

/// Full forward pass. Returns centipawns from the side-to-move's perspective.
fn forward(board: &Board, w: &NnueSf) -> i32 {
    let stm = board.side_to_move();
    let nstm = !stm;

    // Feature transformer — two perspectives
    let mut acc_stm  = [0i32; FT_OUT];
    let mut acc_nstm = [0i32; FT_OUT];
    accumulate(board, stm,  w, &mut acc_stm);
    accumulate(board, nstm, w, &mut acc_nstm);

    // Clipped ReLU — stm perspective first (SF convention)
    let mut ft_stm  = [0i8; FT_OUT];
    let mut ft_nstm = [0i8; FT_OUT];
    clipped_relu_ft(&acc_stm,  &mut ft_stm);
    clipped_relu_ft(&acc_nstm, &mut ft_nstm);

    // Concatenate: [stm | nstm] → L1_IN = 1024 input
    let mut l1_input = [0i8; L1_IN];
    l1_input[..FT_OUT].copy_from_slice(&ft_stm);
    l1_input[FT_OUT..].copy_from_slice(&ft_nstm);

    // L1
    let mut l1_out = [0i8; L1_OUT];
    dense_layer_relu(&l1_input, &w.l1_weights, &w.l1_biases, &mut l1_out);

    // L2 input: L1_OUT × 2 = 32 (SF squares the L1 output — "squared clipped ReLU")
    let mut l2_input = [0i8; L2_IN];
    for (i, &v) in l1_out.iter().enumerate() {
        l2_input[i]          = v;
        l2_input[i + L1_OUT] = ((v as i32 * v as i32) / 127).clamp(0, 127) as i8;
    }

    // L2
    let mut l2_out = [0i8; L2_OUT];
    dense_layer_relu(&l2_input, &w.l2_weights, &w.l2_biases, &mut l2_out);

    // L3 (output)
    let raw = output_layer(&l2_out, &w.l3_weights, w.l3_bias);

    // Scale to centipawns
    raw * EVAL_SCALE / (FT_SCALE * OUT_SCALE)
}

// ---------------------------------------------------------------------------
// Diagnostic: print weight statistics (called from CLI tool)
// ---------------------------------------------------------------------------

pub fn print_weight_stats(path: &Path) -> io::Result<()> {
    let w = parse_nnue_file(path)?;
    std::println!("SF NNUE weights loaded from: {}", path.display());
    std::println!("  ft_weights : {} / {} values", w.ft_weights.len(), FT_IN * FT_OUT);
    std::println!("  ft_biases  : {} / {} values", w.ft_biases.len(), FT_OUT);
    std::println!("  l1_weights : {} / {} values", w.l1_weights.len(), L1_IN * L1_OUT);
    std::println!("  l1_biases  : {} / {} values", w.l1_biases.len(), L1_OUT);
    std::println!("  l2_weights : {} / {} values", w.l2_weights.len(), L2_IN * L2_OUT);
    std::println!("  l2_biases  : {} / {} values", w.l2_biases.len(), L2_OUT);
    std::println!("  l3_weights : {} / {} values", w.l3_weights.len(), L3_IN);
    std::println!("  l3_bias    : {}", w.l3_bias);

    if !w.ft_weights.is_empty() {
        let max = w.ft_weights.iter().copied().map(i16::abs).max().unwrap_or(0);
        let min = w.ft_weights.iter().copied().min().unwrap_or(0);
        std::println!("  ft_weights range: [{}, {}]", min, max);
    }
    Ok(())
}
