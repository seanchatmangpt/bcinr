#![allow(warnings, clippy::all)]
#![allow(warnings)]
//! Branchless Legal Move Generation using Kogge-Stone Parallel Prefix
//! and compile-time bitboard masks.
//! CC = 1, 0 allocations, #![no_std].

/// Kogge-Stone parallel prefix algorithm for North rays
#[inline(always)]
pub fn south_attacks(mut gen: u64, mut pro: u64) -> u64 {
    gen |= pro & (gen >> 8);
    pro &= pro >> 8;
    gen |= pro & (gen >> 16);
    pro &= pro >> 16;
    gen |= pro & (gen >> 32);
    gen >> 8
}

/// Kogge-Stone parallel prefix algorithm for North rays.
#[inline(always)]
pub fn north_attacks(mut gen: u64, mut pro: u64) -> u64 {
    gen |= pro & (gen << 8);
    pro &= pro << 8;
    gen |= pro & (gen << 16);
    pro &= pro << 16;
    gen |= pro & (gen << 32);
    gen << 8
}

/// Kogge-Stone parallel prefix algorithm for East rays.
#[inline(always)]
pub fn east_attacks(mut gen: u64, mut pro: u64) -> u64 {
    pro &= 0xfefefefefefefefe;
    gen |= pro & (gen << 1);
    pro &= pro << 1;
    gen |= pro & (gen << 2);
    pro &= pro << 2;
    gen |= pro & (gen << 4);
    (gen << 1) & 0xfefefefefefefefe
}

/// Kogge-Stone parallel prefix algorithm for West rays.
#[inline(always)]
pub fn west_attacks(mut gen: u64, mut pro: u64) -> u64 {
    pro &= 0x7f7f7f7f7f7f7f7f;
    gen |= pro & (gen >> 1);
    pro &= pro >> 1;
    gen |= pro & (gen >> 2);
    pro &= pro >> 2;
    gen |= pro & (gen >> 4);
    (gen >> 1) & 0x7f7f7f7f7f7f7f7f
}

/// Kogge-Stone parallel prefix algorithm for North-East diagonal rays.
#[inline(always)]
pub fn no_ea_attacks(mut gen: u64, mut pro: u64) -> u64 {
    pro &= 0xfefefefefefefefe;
    gen |= pro & (gen << 9);
    pro &= pro << 9;
    gen |= pro & (gen << 18);
    pro &= pro << 18;
    gen |= pro & (gen << 36);
    (gen << 9) & 0xfefefefefefefefe
}

/// Kogge-Stone parallel prefix algorithm for South-East diagonal rays.
#[inline(always)]
pub fn so_ea_attacks(mut gen: u64, mut pro: u64) -> u64 {
    pro &= 0xfefefefefefefefe;
    gen |= pro & (gen >> 7);
    pro &= pro >> 7;
    gen |= pro & (gen >> 14);
    pro &= pro >> 14;
    gen |= pro & (gen >> 28);
    (gen >> 7) & 0xfefefefefefefefe
}

/// Kogge-Stone parallel prefix algorithm for North-West diagonal rays.
#[inline(always)]
pub fn no_we_attacks(mut gen: u64, mut pro: u64) -> u64 {
    pro &= 0x7f7f7f7f7f7f7f7f;
    gen |= pro & (gen << 7);
    pro &= pro << 7;
    gen |= pro & (gen << 14);
    pro &= pro << 14;
    gen |= pro & (gen << 28);
    (gen << 7) & 0x7f7f7f7f7f7f7f7f
}

/// Kogge-Stone parallel prefix algorithm for South-West diagonal rays.
#[inline(always)]
pub fn so_we_attacks(mut gen: u64, mut pro: u64) -> u64 {
    pro &= 0x7f7f7f7f7f7f7f7f;
    gen |= pro & (gen >> 9);
    pro &= pro >> 9;
    gen |= pro & (gen >> 18);
    pro &= pro >> 18;
    gen |= pro & (gen >> 36);
    (gen >> 9) & 0x7f7f7f7f7f7f7f7f
}

/// Rook attack set from `sq` on a board where `empty` marks empty squares:
/// the union of all four orthogonal ray directions.
#[inline(always)]
pub fn rook_attacks(sq: u64, empty: u64) -> u64 {
    north_attacks(sq, empty)
        | south_attacks(sq, empty)
        | east_attacks(sq, empty)
        | west_attacks(sq, empty)
}

/// Bishop attack set from `sq` on a board where `empty` marks empty
/// squares: the union of all four diagonal ray directions.
#[inline(always)]
pub fn bishop_attacks(sq: u64, empty: u64) -> u64 {
    no_ea_attacks(sq, empty)
        | so_ea_attacks(sq, empty)
        | no_we_attacks(sq, empty)
        | so_we_attacks(sq, empty)
}

/// Queen attack set from `sq`: the union of [`rook_attacks`] and
/// [`bishop_attacks`].
#[inline(always)]
pub fn queen_attacks(sq: u64, empty: u64) -> u64 {
    rook_attacks(sq, empty) | bishop_attacks(sq, empty)
}

/// Build the compile-time knight-attack lookup table: `table[sq]` is the
/// bitboard of every square a knight on `sq` attacks.
pub const fn compute_knight_attacks() -> [u64; 64] {
    let mut table = [0; 64];
    let mut i = 0;
    while i < 64 {
        let sq = 1u64 << i;
        let mut attacks = 0;
        let not_a = 0xfefefefefefefefe;
        let not_h = 0x7f7f7f7f7f7f7f7f;
        let not_ab = 0xfcfcfcfcfcfcfcfc;
        let not_gh = 0x3f3f3f3f3f3f3f3f;
        attacks |= (sq << 17) & not_a;
        attacks |= (sq << 10) & not_ab;
        attacks |= (sq >> 6) & not_ab;
        attacks |= (sq >> 15) & not_a;
        attacks |= (sq << 15) & not_h;
        attacks |= (sq << 6) & not_gh;
        attacks |= (sq >> 10) & not_gh;
        attacks |= (sq >> 17) & not_h;
        table[i] = attacks;
        i += 1;
    }
    table
}

/// Build the compile-time king-attack lookup table: `table[sq]` is the
/// bitboard of every square a king on `sq` attacks.
pub const fn compute_king_attacks() -> [u64; 64] {
    let mut table = [0; 64];
    let mut i = 0;
    while i < 64 {
        let sq = 1u64 << i;
        let mut attacks = 0;
        let not_a = 0xfefefefefefefefe;
        let not_h = 0x7f7f7f7f7f7f7f7f;
        attacks |= sq << 8;
        attacks |= sq >> 8;
        attacks |= (sq << 1) & not_a;
        attacks |= (sq >> 1) & not_h;
        attacks |= (sq << 9) & not_a;
        attacks |= (sq >> 9) & not_h;
        attacks |= (sq << 7) & not_h;
        attacks |= (sq >> 7) & not_a;
        table[i] = attacks;
        i += 1;
    }
    table
}

/// Precomputed knight-attack bitboard per square, indexed by square number.
pub static KNIGHT_MASKS: [u64; 64] = compute_knight_attacks();
/// Precomputed king-attack bitboard per square, indexed by square number.
pub static KING_MASKS: [u64; 64] = compute_king_attacks();
