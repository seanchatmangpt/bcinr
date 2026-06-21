//! # Integer Bit Operations Example
//!
//! Demonstrates `bcinr_logic::int`: population count, bit reversal, leading/trailing
//! zeros, next-power-of-two, parity, and signed saturating arithmetic.
//!
//! **Doc reference:** `crates/bcinr-logic/src/int.rs`
//! **Also see:** `examples/mask_primitives.rs` — masks built on these bit ops.
//!
//! These are constant-time intrinsic wrappers. The assertions below would fail
//! if any function wrapped instead of saturating, or counted the wrong direction.

use bcinr::int::{
    is_pow2_u32, leading_zeros_u32, leading_zeros_u64, next_power_of_two_u32, parity_u32,
    popcount_u32, popcount_u64, reverse_bits_u32, reverse_bits_u64, saturating_add_i64,
    saturating_mul_i64, saturating_sub_i64, trailing_zeros_u32, trailing_zeros_u64,
};

fn main() {
    // --- popcount: count set bits ---
    assert_eq!(popcount_u64(0), 0);
    assert_eq!(popcount_u64(u64::MAX), 64);
    assert_eq!(popcount_u64(0xFFFF_FFFF), 32);
    assert_eq!(popcount_u64(0b1011), 3);
    assert_eq!(popcount_u32(0b1011), 3);
    assert_eq!(popcount_u32(0), 0);
    println!(
        "popcount_u64(0b1011)={}, popcount_u64(MAX)={}",
        popcount_u64(0b1011),
        popcount_u64(u64::MAX)
    );

    // --- leading zeros: from MSB ---
    assert_eq!(leading_zeros_u64(0), 64, "all zeros → 64 leading zeros");
    assert_eq!(leading_zeros_u64(1), 63);
    assert_eq!(leading_zeros_u64(u64::MAX), 0, "MAX has no leading zeros");
    assert_eq!(leading_zeros_u32(1), 31);
    assert_eq!(leading_zeros_u32(0x8000_0000), 0);
    println!("leading_zeros_u64(1)={}", leading_zeros_u64(1));

    // --- trailing zeros: from LSB ---
    assert_eq!(trailing_zeros_u64(0), 64, "all zeros → 64 trailing zeros");
    assert_eq!(trailing_zeros_u64(1), 0);
    assert_eq!(trailing_zeros_u64(0x10), 4, "bit 4 set → 4 trailing zeros");
    assert_eq!(trailing_zeros_u32(8), 3);
    println!("trailing_zeros_u64(0x10)={}", trailing_zeros_u64(0x10));

    // --- reverse_bits ---
    assert_eq!(reverse_bits_u64(1), 0x8000_0000_0000_0000, "LSB → MSB");
    assert_eq!(reverse_bits_u64(0x8000_0000_0000_0000), 1, "MSB → LSB");
    assert_eq!(
        reverse_bits_u64(reverse_bits_u64(0xABCD_1234_5678_EF00)),
        0xABCD_1234_5678_EF00,
        "double reverse = identity"
    );
    assert_eq!(reverse_bits_u32(1), 0x8000_0000);
    println!("reverse_bits_u64(1)={:#018x}", reverse_bits_u64(1));

    // --- next_power_of_two_u32 ---
    assert_eq!(next_power_of_two_u32(0), 1, "0 → 1 (defined behavior)");
    assert_eq!(next_power_of_two_u32(1), 1);
    assert_eq!(next_power_of_two_u32(2), 2);
    assert_eq!(next_power_of_two_u32(3), 4);
    assert_eq!(next_power_of_two_u32(100), 128);
    assert_eq!(next_power_of_two_u32(128), 128, "exact power stays");
    println!("next_power_of_two_u32(100)={}", next_power_of_two_u32(100));

    // --- is_pow2_u32 ---
    assert!(!is_pow2_u32(0), "0 is not a power of two");
    assert!(is_pow2_u32(1));
    assert!(is_pow2_u32(64));
    assert!(!is_pow2_u32(3));
    assert!(!is_pow2_u32(100));
    println!(
        "is_pow2_u32(64)={}, is_pow2_u32(100)={}",
        is_pow2_u32(64),
        is_pow2_u32(100)
    );

    // --- parity_u32: 1 if odd popcount, 0 if even ---
    assert_eq!(parity_u32(0), 0, "0 bits set → even parity");
    assert_eq!(parity_u32(1), 1, "1 bit set → odd parity");
    assert_eq!(parity_u32(0b11), 0, "2 bits set → even parity");
    assert_eq!(parity_u32(0b111), 1, "3 bits set → odd parity");
    // Cross-check: parity matches popcount parity
    let v: u32 = 0b1101_0110;
    assert_eq!(
        parity_u32(v),
        popcount_u32(v) & 1,
        "parity must match popcount mod 2"
    );
    println!("parity_u32(0b111)={}", parity_u32(0b111));

    // --- signed saturating arithmetic ---
    assert_eq!(
        saturating_add_i64(i64::MAX, 1),
        i64::MAX,
        "saturates at MAX"
    );
    assert_eq!(
        saturating_sub_i64(i64::MIN, 1),
        i64::MIN,
        "saturates at MIN"
    );
    assert_eq!(saturating_mul_i64(i64::MAX, 2), i64::MAX, "mul saturates");
    assert_eq!(saturating_add_i64(10, -3), 7, "normal case");
    println!(
        "saturating_add_i64(MAX,1)={}",
        saturating_add_i64(i64::MAX, 1)
    );

    println!("\nAll integer operation assertions passed.");
}
