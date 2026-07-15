//! Fuzz oracle: checks `bcinr`'s branchless `rook_attacks`/`bishop_attacks`
//! against the `chess` crate's reference move generators over randomized
//! squares and occupancy bitboards.
#![allow(unsafe_code)]
use chess::{get_bishop_moves, get_rook_moves, BitBoard, Square};
use playground::legal_moves::{bishop_attacks, rook_attacks};
use proptest::prelude::*;

// We fuzz the board with completely random 64-bit integers for obstacles (empty spaces).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    #[test]
    fn oracle_rook_attacks_match(sq in 0usize..64, occupied in any::<u64>()) {
        let empty_mask = !occupied;
        let bcinr_attacks = rook_attacks(1u64 << sq, empty_mask);

        let chess_sq = unsafe { Square::new(sq as u8) };
        let chess_attacks = get_rook_moves(chess_sq, BitBoard::new(occupied));

        prop_assert_eq!(bcinr_attacks, chess_attacks.0);
    }

    #[test]
    fn oracle_bishop_attacks_match(sq in 0usize..64, occupied in any::<u64>()) {
        let empty_mask = !occupied;
        let bcinr_attacks = bishop_attacks(1u64 << sq, empty_mask);

        let chess_sq = unsafe { Square::new(sq as u8) };
        let chess_attacks = get_bishop_moves(chess_sq, BitBoard::new(occupied));

        prop_assert_eq!(bcinr_attacks, chess_attacks.0);
    }
}
