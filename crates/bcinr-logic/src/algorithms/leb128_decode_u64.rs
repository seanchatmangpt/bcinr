// Academic-grade branchless algorithm library: leb128_decode_u64
// LEB128 (Little-Endian Base-128) variable-length integer decoding.
// Standard: DWARF 2.4, Protocol Buffers, WebAssembly
// Branchless: constant-time processing regardless of encoded length.

/// leb128_decode_u64 — Decode a LEB128-encoded u64 from packed bytes
///
/// Decodes a Little-Endian Base-128 encoded integer from a 64-bit chunk.
/// LEB128 encodes integers using 7 data bits + 1 continuation bit per byte.
/// This function processes a pre-extracted 8-byte window.
///
/// # Algorithm (DWARF 4.1)
/// LEB128 format: each byte has structure [continuation_bit | 7_data_bits]
/// For multi-byte integers:
///   Byte 0: [cont | bits 6:0]
///   Byte 1: [cont | bits 13:7]
///   Byte 2: [cont | bits 20:14]
///   ...
///
/// This decoder processes up to 8 bytes in a 64-bit value:
/// - Extract 7 bits from each byte and place at correct position
/// - Check continuation bit (bit 7) to determine length
/// - Zero-extend result
///
/// # CONTRACT
/// **Ensures:** result is correctly decoded LEB128 value from input bytes
/// **Invariant:** Zero conditional branches in hot path
///
/// # Examples
/// ```
/// use bcinr_logic::algorithms::leb128_decode_u64::leb128_decode_u64;
/// // Single-byte encoding of 42: [0x2A] (no continuation bit)
/// let result = leb128_decode_u64(0x2A, 0);
/// assert_eq!(result, 42);
/// ```
/// # Branchless Contract
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
pub fn leb128_decode_u64(bytes: u64, _aux: u64) -> u64 {
    // Extract bytes from the packed u64 in little-endian order
    let b0 = (bytes & 0xFF) as u8;
    let b1 = ((bytes >> 8) & 0xFF) as u8;
    let b2 = ((bytes >> 16) & 0xFF) as u8;
    let b3 = ((bytes >> 24) & 0xFF) as u8;
    let b4 = ((bytes >> 32) & 0xFF) as u8;
    let b5 = ((bytes >> 40) & 0xFF) as u8;
    let b6 = ((bytes >> 48) & 0xFF) as u8;
    let b7 = ((bytes >> 56) & 0xFF) as u8;

    // Branchless Contract: a byte's 7 payload bits are included iff EVERY preceding
    // byte set its continuation bit (bit 7). The gate is therefore the cumulative AND
    // of all prior continuation bits, expanded to a full 0/all-ones lane mask. No
    // control flow: the running mask is folded by repeated bitwise AND.

    // Per-byte continuation lane masks (all-ones when that byte continues).
    let c0 = 0u64.wrapping_sub(((b0 >> 7) & 1) as u64);
    let c1 = 0u64.wrapping_sub(((b1 >> 7) & 1) as u64);
    let c2 = 0u64.wrapping_sub(((b2 >> 7) & 1) as u64);
    let c3 = 0u64.wrapping_sub(((b3 >> 7) & 1) as u64);
    let c4 = 0u64.wrapping_sub(((b4 >> 7) & 1) as u64);
    let c5 = 0u64.wrapping_sub(((b5 >> 7) & 1) as u64);
    let c6 = 0u64.wrapping_sub(((b6 >> 7) & 1) as u64);

    // Cumulative include masks: gate_k is all-ones iff bytes 0..k all continued.
    let g1 = c0;
    let g2 = g1 & c1;
    let g3 = g2 & c2;
    let g4 = g3 & c3;
    let g5 = g4 & c4;
    let g6 = g5 & c5;
    let g7 = g6 & c6;

    (b0 & 0x7F) as u64
        | ((((b1 & 0x7F) as u64) << 7) & g1)
        | ((((b2 & 0x7F) as u64) << 14) & g2)
        | ((((b3 & 0x7F) as u64) << 21) & g3)
        | ((((b4 & 0x7F) as u64) << 28) & g4)
        | ((((b5 & 0x7F) as u64) << 35) & g5)
        | ((((b6 & 0x7F) as u64) << 42) & g6)
        | ((((b7 & 0x7F) as u64) << 49) & g7)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // REFERENCE: Standard LEB128 decoding via bit-by-bit construction
    // -------------------------------------------------------------------------
    fn leb128_decode_u64_reference(bytes: u64, _aux: u64) -> u64 {
        let b = [
            (bytes & 0xFF) as u8,
            ((bytes >> 8) & 0xFF) as u8,
            ((bytes >> 16) & 0xFF) as u8,
            ((bytes >> 24) & 0xFF) as u8,
            ((bytes >> 32) & 0xFF) as u8,
            ((bytes >> 40) & 0xFF) as u8,
            ((bytes >> 48) & 0xFF) as u8,
            ((bytes >> 56) & 0xFF) as u8,
        ];

        let mut result: u64 = 0;
        let mut shift = 0;

        for i in 0..8 {
            result |= ((b[i] & 0x7F) as u64) << shift;
            if (b[i] & 0x80) == 0 {
                break; // No continuation bit, stop decoding
            }
            shift += 7;
        }

        result
    }

    // -------------------------------------------------------------------------
    // PROPERTY TESTS: 1000+ random cases of equivalence
    // -------------------------------------------------------------------------
    proptest! {
        #[test]
        fn test_leb128_decode_u64_all(bytes in any::<u64>()) {
            let expected = leb128_decode_u64_reference(bytes, 0);
            let actual = leb128_decode_u64(bytes, 0);
            prop_assert_eq!(expected, actual, "leb128_decode_u64 mismatch for bytes=0x{:016X}", bytes);

        // Fixed-value assertions from single-byte test (value in 0..=127)
        for value_s in [0u8, 1u8, 42u8, 127u8].iter().copied() {
            let bytes_s = value_s as u64;
            prop_assert_eq!(leb128_decode_u64(bytes_s, 0), value_s as u64);
        }
        // Fixed-value assertions from two-byte test (val in 128..=16383)
        for val_t in [128u32, 256u32, 1000u32, 16383u32].iter().copied() {
            let b0_t = (0x80 | (val_t & 0x7F)) as u8;
            let b1_t = ((val_t >> 7) & 0x7F) as u8;
            let bytes_t = (b0_t as u64) | ((b1_t as u64) << 8);
            prop_assert_eq!(leb128_decode_u64(bytes_t, 0), val_t as u64);
        }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded critical cases (LEB128 standard)
    // -------------------------------------------------------------------------
    #[test]
    fn test_leb128_decode_u64_boundaries() {
        // Single byte: 0 (0x00)
        assert_eq!(leb128_decode_u64(0x00, 0), 0);

        // Single byte: 127 (0x7F)
        assert_eq!(leb128_decode_u64(0x7F, 0), 127);

        // Two bytes: 128 (0x80, 0x01) = [0x80, 0x01]
        let bytes_128 = 0x01_80_u64;
        assert_eq!(leb128_decode_u64(bytes_128, 0), 128);

        // Two bytes: 16383 (0xFF, 0x7F) = [0xFF, 0x7F]
        let bytes_16383 = 0x7F_FF_u64;
        assert_eq!(leb128_decode_u64(bytes_16383, 0), 16383);

        // Three bytes: 16384 (0x80, 0x80, 0x01) = [0x80, 0x80, 0x01]
        let bytes_16384 = 0x01_80_80_u64;
        assert_eq!(leb128_decode_u64(bytes_16384, 0), 16384);

        // All zeros: should decode as 0
        assert_eq!(leb128_decode_u64(0, 0), 0);

        // High bytes ignored (no continuation bits)
        assert_eq!(leb128_decode_u64(0x00FFFFFFFFFFFFFF00, 0), 0);
        assert_eq!(leb128_decode_u64(0x2A, 0), 42); // 0x2A = 42
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: LEB128 decoding correctness
    // -------------------------------------------------------------------------
    // Precondition:  { bytes ∈ U64 }
    // Postcondition: { result = LEB128 decoding of bytes[0..7] }
    //
    // Proof:
    // 1. Each byte b_i encodes 7 data bits: (b_i & 0x7F)
    // 2. Continuation bit: (b_i & 0x80) determines if more bytes follow
    // 3. Branchless: compute all 8 possible contributions in parallel
    // 4. Masking: for byte i, include if (b_{i-1} & 0x80) != 0
    //    Mask = -1 if bit set, 0 otherwise: (-(cont as i64)) as u64
    // 5. Result: result |= (contribution & mask)
    // 6. Proof: All bytes processed, masked by continuation chain
    // 7. No conditional branches: all contributions computed and masked
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_leb128_decode_u64(c: &mut Criterion) {
        c.bench_function("leb128_decode_u64_single_byte", |b| {
            // Single-byte encoding (0x2A = 42)
            b.iter(|| leb128_decode_u64(black_box(0x2A), black_box(0)))
        });

        c.bench_function("leb128_decode_u64_two_bytes", |b| {
            // Two-byte encoding (128 = 0x80, 0x01)
            b.iter(|| leb128_decode_u64(black_box(0x01_80), black_box(0)))
        });

        c.bench_function("leb128_decode_u64_max_bytes", |b| {
            // All 8 bytes with continuation bits
            b.iter(|| leb128_decode_u64(black_box(0xFF_FF_FF_FF_FF_FF_FF_FF), black_box(0)))
        });
    }
}
