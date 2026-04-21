//! # Universe1_n Geometry Constants
//!
//! Universe1_n profile constants. U_{1,n} = 𝔹ⁿ where n is the count of one-bit truth atoms:
//!
//! | Power | Bits | Bytes | Role                                  |
//! |-------|------|-------|---------------------------------------|
//! | 8¹    | 8    | 1     | Place atom (one byte / `u8`)          |
//! | 8²    | 64   | 8     | Cell — one `u64` register             |
//! | 8³    | 512  | 64    | Block — one L1 cache line (`[u64; 8]`) |
//! | 8⁴    | 4096 | 512   | Domain — half a 4 KiB page (`[u64; 64]`) |

/// Number of place atoms (bits) in one U8 dimension (8¹).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_PLACE_COUNT;
/// assert_eq!(U1_PLACE_COUNT, 8);
/// ```
pub const U1_PLACE_COUNT: usize = 8;

/// Number of bits in one U1_64 (8²).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_64_BITS;
/// assert_eq!(U1_64_BITS, 64);
/// ```
pub const U1_64_BITS: usize = 64;

/// Number of `u64` words in one U1_512 (8³ / 64 = 8).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_512_WORDS;
/// assert_eq!(U1_512_WORDS, 8);
/// ```
pub const U1_512_WORDS: usize = 8;

/// Number of `u64` words in one U1_4096 (8⁴ / 64 = 64).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_4096_WORDS;
/// assert_eq!(U1_4096_WORDS, 64);
/// ```
pub const U1_4096_WORDS: usize = 64;

/// Size in bytes of one U1_64.
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_64_BYTES;
/// assert_eq!(U1_64_BYTES, 8);
/// ```
pub const U1_64_BYTES: usize = 8;

/// Size in bytes of one U1_512 (one L1 cache line).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_512_BYTES;
/// assert_eq!(U1_512_BYTES, 64);
/// ```
pub const U1_512_BYTES: usize = 64;

/// Size in bytes of one U1_4096 (half a 4 KiB page).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_4096_BYTES;
/// assert_eq!(U1_4096_BYTES, 512);
/// ```
pub const U1_4096_BYTES: usize = 512;

/// Number of bits needed to encode one U8 axis coordinate (log2(8) = 3).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_COORD_BITS;
/// assert_eq!(U1_COORD_BITS, 3);
/// ```
pub const U1_COORD_BITS: u32 = 3;

/// Bitmask for clamping a value to one U8 axis range [0, 7].
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_COORD_MASK;
/// assert_eq!(U1_COORD_MASK, 0x7);
/// ```
pub const U1_COORD_MASK: u32 = 0x7;

/// Bit shift for domain field in packed U1Coord (occupies bits [8:6]).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_DOMAIN_SHIFT;
/// assert_eq!(U1_DOMAIN_SHIFT, 6);
/// ```
pub const U1_DOMAIN_SHIFT: u32 = 6;

/// Bit shift for cell field in packed U1Coord (occupies bits [5:3]).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_CELL_SHIFT;
/// assert_eq!(U1_CELL_SHIFT, 3);
/// ```
pub const U1_CELL_SHIFT: u32 = 3;

/// Bit shift for place field in packed U1Coord (occupies bits [2:0]).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_PLACE_SHIFT;
/// assert_eq!(U1_PLACE_SHIFT, 0);
/// ```
pub const U1_PLACE_SHIFT: u32 = 0;

/// Maximum valid flat word index within a U1_4096 (inclusive).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_MAX_WORD_INDEX;
/// assert_eq!(U1_MAX_WORD_INDEX, 63);
/// ```
pub const U1_MAX_WORD_INDEX: usize = U1_4096_WORDS - 1;

/// PHD integrity gate for U1_512 (0x08 repeating byte pattern).
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_512_GATE;
/// assert_eq!(U1_512_GATE, 0x0808080808080808);
/// ```
pub const U1_512_GATE: u64 = 0x0808080808080808;

/// PHD integrity gate for U1_4096.
///
/// ```
/// use bcinr_logic::patterns::universe1::constants::U1_4096_GATE;
/// assert_eq!(U1_4096_GATE, 0xA0A0A0A0A0A0A0A0);
/// ```
pub const U1_4096_GATE: u64 = 0xA0A0A0A0A0A0A0A0;

// --- Compile-time consistency checks ---
const _: () = assert!(U1_64_BITS == 64);
const _: () = assert!(U1_512_BYTES == 64);
const _: () = assert!(U1_4096_BYTES == 512);
const _: () = assert!(U1_512_WORDS * 8 == U1_512_BYTES);
const _: () = assert!(U1_4096_WORDS * 8 == U1_4096_BYTES);
const _: () = assert!(U1_MAX_WORD_INDEX == U1_4096_WORDS - 1);
