//! Divan benchmarks for the `bcinr-logic` branchless kernels — the crate's
//! first benchmark suite. Covers one representative hot kernel per kernel
//! family, each a real public production entry point called with
//! representative fixed-width inputs:
//!
//! - `mask`: masked select / comparison masks / min-max-abs primitives
//! - `swar`: SWAR PHD gate and ones-mask
//! - `scan`: gate word and byte-scan-to-mask
//! - `bitset`: rank/select, slice intersect/union/parity/any-bit-set
//! - `reduce`: horizontal SWAR reductions and slice min/sum
//! - `fix`: Q16.16 fixed-point mul/div/recip/sqrt/log2, clamp, saturating add
//!
//! Convention matches `bcinr-powl/benches/*` and
//! `bcinr-cmca/benches/allocation_bench.rs`: Divan, `harness = false`,
//! `black_box` on inputs AND outputs. Slice-mutating APIs
//! (`intersect`/`union`) require a caller-side copy into the working buffer
//! per iteration — that copy is part of the real call contract (in-place
//! mutation of caller memory) and is measured with the kernel.
//!
//! Fixed inputs: a 64-word (4096-bit) slice shape matching the workspace's
//! 8-word-`PowlTapeLarge` / 8-word-`EventSet` bitset ethos at 8x scale, with
//! bit patterns generated from the splitmix64 golden-gamma constant
//! (`GOLDEN`: 0x9E3779B97F4A7C15) — a named, sourced constant, not a magic
//! literal. The same fixed input is used for every iteration of a given
//! benchmark; the kernels' instruction sequences are data-independent by
//! construction, so one representative value per operand class suffices
//! (same pricing argument as `bcinr-cmca`'s decomposition benches).
//!
//! Numbers are receipts, not claims.

use bcinr_logic::{bitset, fix, mask, reduce, scan, swar};
use divan::black_box;

fn main() {
    divan::main();
}

/// Splitmix64 golden-gamma constant — the standard additive constant for
/// fixed-pattern word generation (S. Ridicieux, splitmix64; also used as
/// PCG's multiplier increment family). Named here so no bare magic literal
/// controls input shape.
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// Fixed word i: golden-ratio-distributed bits, dense and unstructured —
/// the worst case for branchless word kernels (no early-zero shortcuts to
/// exploit, none of which exist in these implementations anyway).
const fn golden_word(i: usize) -> u64 {
    (i as u64).wrapping_mul(GOLDEN)
}

/// 64-word working set (4096 bits): the 8-word EventSet/PowlTape shape at
/// 8x, so per-call fixed overhead is visible against word-loop throughput.
const WORDS: usize = 64;

/// Dense pseudo-random 64-word pattern A (bit i*GOLDEN mixing).
const WORDS_A: [u64; WORDS] = {
    let mut words = [0u64; WORDS];
    let mut i = 0;
    while i < WORDS {
        words[i] = golden_word(i);
        i += 1;
    }
    words
};

/// Dense pseudo-random 64-word pattern B (offset by 1 so A and B differ).
const WORDS_B: [u64; WORDS] = {
    let mut words = [0u64; WORDS];
    let mut i = 0;
    while i < WORDS {
        words[i] = golden_word(i + 1);
        i += 1;
    }
    words
};

/// 64 pseudo-random u32 lanes (the reduce-slice operand shape).
const U32S: [u32; WORDS] = {
    let mut lanes = [0u32; WORDS];
    let mut i = 0;
    while i < WORDS {
        lanes[i] = golden_word(i) as u32;
        i += 1;
    }
    lanes
};

/// 64 pseudo-random bytes, with `FIND_TARGET` guaranteed present at a known
/// mid-scan position (word shape for `find_byte_mask`).
const BYTES: [u8; WORDS] = {
    let mut bytes = [0u8; WORDS];
    let mut i = 0;
    while i < WORDS {
        bytes[i] = (golden_word(i) >> 8) as u8;
        i += 1;
    }
    bytes
};

/// Byte scanned for by `find_byte_mask`: present at position 33 of `BYTES`.
const FIND_TARGET: u8 = (golden_word(33) >> 8) as u8;

// ---------------------------------------------------------------------------
// mask — masked select / comparison-mask construction / min-max-abs
// ---------------------------------------------------------------------------

#[divan::bench]
fn mask_select_u64() {
    let out = mask::select_u64(
        black_box(0xFFFF_0000_F0F0_AAAA),
        black_box(golden_word(1)),
        black_box(golden_word(2)),
    );
    let _ = black_box(out);
}

#[divan::bench]
fn mask_eq_mask_u32() {
    let out = mask::eq_mask_u32(black_box(0x1234_5678), black_box(0x1234_5678));
    let _ = black_box(out);
}

#[divan::bench]
fn mask_lt_mask_u32() {
    let out = mask::lt_mask_u32(black_box(0x0000_00FF), black_box(0xFFFF_0000));
    let _ = black_box(out);
}

#[divan::bench]
fn mask_min_u32() {
    let out = mask::min_u32(black_box(0x0000_00FF), black_box(0xFFFF_0000));
    let _ = black_box(out);
}

#[divan::bench]
fn mask_max_u32() {
    let out = mask::max_u32(black_box(0x0000_00FF), black_box(0xFFFF_0000));
    let _ = black_box(out);
}

#[divan::bench]
fn mask_abs_i32() {
    let out = mask::abs_i32(black_box(-123_456_789i32));
    let _ = black_box(out);
}

// ---------------------------------------------------------------------------
// swar — SWAR gate kernels
// ---------------------------------------------------------------------------

#[divan::bench]
fn swar_phd_gate() {
    let out = swar::swar_phd_gate(black_box(golden_word(3)));
    let _ = black_box(out);
}

#[divan::bench]
fn swar_mask_ones() {
    let out = swar::swar_mask_ones(black_box(golden_word(4)));
    let _ = black_box(out);
}

// ---------------------------------------------------------------------------
// scan — gate word and byte scan
// ---------------------------------------------------------------------------

#[divan::bench]
fn scan_gate() {
    let out = scan::scan_gate(black_box(golden_word(5)));
    let _ = black_box(out);
}

#[divan::bench]
fn scan_find_byte_mask() {
    let out = scan::find_byte_mask(black_box(&BYTES), black_box(FIND_TARGET));
    let _ = black_box(out);
}

// ---------------------------------------------------------------------------
// bitset — rank/select on words, slice set operations
// ---------------------------------------------------------------------------

#[divan::bench]
fn bitset_rank_u64() {
    let out = bitset::rank_u64(black_box(golden_word(6)), black_box(40usize));
    let _ = black_box(out);
}

#[divan::bench]
fn bitset_select_bit_u64() {
    let out = bitset::select_bit_u64(black_box(golden_word(7)), black_box(20usize));
    let _ = black_box(out);
}

#[divan::bench]
fn bitset_intersect_64_words() {
    // Caller-side copy into the destination buffer is part of the real
    // in-place contract; measured with the kernel.
    let mut dest = WORDS_A;
    bitset::intersect_u64_slices(black_box(&mut dest), black_box(&WORDS_B));
    let _ = black_box(dest);
}

#[divan::bench]
fn bitset_union_64_words() {
    let mut dest = WORDS_A;
    bitset::union_u64_slices(black_box(&mut dest), black_box(&WORDS_B));
    let _ = black_box(dest);
}

#[divan::bench]
fn bitset_parity_64_words() {
    let out = bitset::parity_u64_slice(black_box(&WORDS_A));
    let _ = black_box(out);
}

#[divan::bench]
fn bitset_any_bit_set_64_words() {
    let out = bitset::any_bit_set_u64_slice(black_box(&WORDS_A));
    let _ = black_box(out);
}

// ---------------------------------------------------------------------------
// reduce — horizontal SWAR reductions and slice min/sum
// ---------------------------------------------------------------------------

#[divan::bench]
fn reduce_horizontal_sum_u8x8() {
    let out = reduce::horizontal_sum_u8x8(black_box(golden_word(8)));
    let _ = black_box(out);
}

#[divan::bench]
fn reduce_swar_count_eq_u8() {
    let out = reduce::swar_count_eq_u8(black_box(golden_word(9)), black_box(0x7Cu8));
    let _ = black_box(out);
}

#[divan::bench]
fn reduce_sum_u64_64_lanes() {
    let out = reduce::reduce_sum_u64(black_box(&U32S));
    let _ = black_box(out);
}

#[divan::bench]
fn reduce_min_u32_64_lanes() {
    let out = reduce::reduce_min_u32(black_box(&U32S));
    let _ = black_box(out);
}

// ---------------------------------------------------------------------------
// fix — Q16.16 fixed-point arithmetic (16.16: 65536 == 1.0)
// ---------------------------------------------------------------------------

const Q16_TWO: i32 = 2 << 16;
const Q16_THREE: i32 = 3 << 16;

#[divan::bench]
fn fix_q16_mul() {
    let out = fix::q16_mul(black_box(Q16_THREE), black_box(Q16_TWO));
    let _ = black_box(out);
}

#[divan::bench]
fn fix_q16_div() {
    // Nonzero denominator: the admitted domain.
    let out = fix::q16_div(black_box(Q16_THREE), black_box(Q16_TWO));
    let _ = black_box(out);
}

#[divan::bench]
fn fix_q16_recip() {
    // recip(2.0) = 0.5 — nonzero, in-domain.
    let out = fix::q16_recip(black_box(Q16_TWO));
    let _ = black_box(out);
}

#[divan::bench]
fn fix_isqrt_u32() {
    let out = fix::isqrt_u32(black_box(0x5A5A_5A5Au32));
    let _ = black_box(out);
}

#[divan::bench]
fn fix_ilog2_u32() {
    let out = fix::ilog2_u32(black_box(0x0008_0000u32));
    let _ = black_box(out);
}

#[divan::bench]
fn fix_q16_log2() {
    let out = fix::q16_log2(black_box(Q16_TWO));
    let _ = black_box(out);
}

#[divan::bench]
fn fix_clamp_u32() {
    let out = fix::clamp_u32(
        black_box(0x0001_0000u32),
        black_box(0x0000_00FFu32),
        black_box(0x0000_FFFFu32),
    );
    let _ = black_box(out);
}

#[divan::bench]
fn fix_add_sat() {
    let out = fix::add_sat(black_box(0xF000_0000u32), black_box(0x2000_0000u32));
    let _ = black_box(out);
}
