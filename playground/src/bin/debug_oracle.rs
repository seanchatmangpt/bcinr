#![allow(unsafe_code)]
use chess::{get_bishop_moves, get_rook_moves, BitBoard, Square};
use playground::legal_moves::{bishop_attacks, rook_attacks};

fn print_bb(name: &str, bb: u64) {
    println!("{}:", name);
    for r in (0..8).rev() {
        for f in 0..8 {
            let sq = r * 8 + f;
            if (bb & (1 << sq)) != 0 {
                print!("X ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
    println!();
}

fn main() {
    let sq = 47;
    let occupied = 0;
    let empty_mask = !occupied;

    let bcinr = bishop_attacks(1u64 << sq, empty_mask);
    let chess_sq = unsafe { Square::new(sq as u8) };
    let chess_attacks = get_bishop_moves(chess_sq, BitBoard::new(occupied)).0;

    println!("SQ: {}", sq);
    print_bb("BCINR", bcinr);
    print_bb("CHESS", chess_attacks);
}
