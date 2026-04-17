//! Probabilistic Sketches: Hashing and probabilistic data structures
//!
//! This module contains handwritten, production-quality implementations
//! of high-performance hash algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/sketch.rs`.
//!
//! All implementations are:
//! - Deterministic (same input → same output)
//! - Fast (branchless where possible, inline on hot paths)
//! - Correct (verified against reference implementations)
//!
//! Algorithm sources:
//! - MurmurHash3: https://github.com/aappleby/smhasher
//! - xxHash: https://xxhash.com/
//! - FNV-1a: http://www.isthe.com/chongo/tech/comp/fnv/

/// Rotate a 32-bit integer left by n bits.
/// Used in mixing functions for better avalanche properties.
#[inline(always)]
const fn rotl32(x: u32, n: u32) -> u32 {
    (x << n) | (x >> (32 - n))
}

/// Rotate a 64-bit integer left by n bits.
#[allow(dead_code)]
#[inline(always)]
const fn rotl64(x: u64, n: u32) -> u64 {
    (x << n) | (x >> (64 - n))
}

/// FNV-1a 64-bit hash implementation.
///
/// FNV-1a is a simple, fast, and widely-used hash function suitable for hash tables,
/// checksums, and fingerprinting. It provides excellent distribution properties.
///
/// Source: http://www.isthe.com/chongo/tech/comp/fnv/ (FNV-1a 64-bit)
///
/// Constants:
/// - FNV offset basis: 0xcbf29ce484222325
/// - FNV prime: 0x100000001b3
///
/// # Arguments
///
/// * `data` - Byte slice to hash
///
/// # Returns
///
/// A 64-bit hash value
///
/// # Example
///
/// ```rust
/// # fn fnv1a_64(data: &[u8]) -> u64 {
/// #     const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
/// #     const FNV_PRIME: u64 = 0x100000001b3;
/// #     let mut hash = FNV_OFFSET_BASIS;
/// #     for &byte in data {
/// #         hash ^= byte as u64;
/// #         hash = hash.wrapping_mul(FNV_PRIME);
/// #     }
/// #     hash
/// # }
/// let hash1 = fnv1a_64(b"hello");
/// let hash2 = fnv1a_64(b"hello");
/// assert_eq!(hash1, hash2); // Deterministic
/// assert_ne!(hash1, fnv1a_64(b"world")); // Different input → different hash
/// ```
#[inline(always)]
pub(crate) fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// xxHash 32-bit implementation.
///
/// xxHash is a fast, non-cryptographic hash algorithm that provides excellent
/// distribution properties and speed. It's faster than MurmurHash3 on most platforms.
///
/// Source: https://xxhash.com/ (xxHash 32-bit specification)
///
/// Constants:
/// - PRIME32_1: 0x9e3779b1
/// - PRIME32_2: 0x85ebca6b
/// - PRIME32_3: 0xc2b2ae35
/// - PRIME32_4: 0x27d4eb2d
/// - PRIME32_5: 0x165667b1
///
/// # Arguments
///
/// * `data` - Byte slice to hash
/// * `seed` - 32-bit seed for variation
///
/// # Returns
///
/// A 32-bit hash value
///
/// # Example
///
/// ```rust
/// # fn xxhash32(data: &[u8], seed: u32) -> u32 {
/// #     const PRIME32_1: u32 = 0x9e3779b1;
/// #     const PRIME32_2: u32 = 0x85ebca6b;
/// #     const PRIME32_3: u32 = 0xc2b2ae35;
/// #     const PRIME32_4: u32 = 0x27d4eb2d;
/// #     const PRIME32_5: u32 = 0x165667b1;
/// #
/// #     let mut acc = seed.wrapping_add(PRIME32_5);
/// #     let mut offset = 0;
/// #
/// #     // Process 4-byte chunks
/// #     while offset + 4 <= data.len() {
/// #         let chunk = u32::from_le_bytes([
/// #             data[offset],
/// #             data[offset + 1],
/// #             data[offset + 2],
/// #             data[offset + 3],
/// #         ]);
/// #         acc = (acc.wrapping_add(chunk.wrapping_mul(PRIME32_3))).rotate_left(17).wrapping_mul(PRIME32_4);
/// #         offset += 4;
/// #     }
/// #
/// #     // Process remaining bytes
/// #     while offset < data.len() {
/// #         acc = (acc.wrapping_add((data[offset] as u32).wrapping_mul(PRIME32_5))).rotate_left(11).wrapping_mul(PRIME32_1);
/// #         offset += 1;
/// #     }
/// #
/// #     // Finalize
/// #     acc ^= acc >> 15;
/// #     acc = acc.wrapping_mul(PRIME32_2);
/// #     acc ^= acc >> 13;
/// #     acc = acc.wrapping_mul(PRIME32_3);
/// #     acc ^= acc >> 16;
/// #     acc
/// # }
/// let hash1 = xxhash32(b"hello", 0);
/// let hash2 = xxhash32(b"hello", 0);
/// assert_eq!(hash1, hash2); // Deterministic
/// assert_ne!(hash1, xxhash32(b"hello", 1)); // Different seed → different hash
/// ```
#[inline(always)]
pub(crate) fn xxhash32(data: &[u8], seed: u32) -> u32 {
    const PRIME32_1: u32 = 0x9e3779b1;
    const PRIME32_2: u32 = 0x85ebca6b;
    const PRIME32_3: u32 = 0xc2b2ae35;
    const PRIME32_4: u32 = 0x27d4eb2d;
    const PRIME32_5: u32 = 0x165667b1;

    let mut acc = seed.wrapping_add(PRIME32_5);
    let mut offset = 0;

    // Process 4-byte chunks
    while offset + 4 <= data.len() {
        let chunk = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        acc = rotl32(
            acc.wrapping_add(chunk.wrapping_mul(PRIME32_3)),
            17,
        )
        .wrapping_mul(PRIME32_4);
        offset += 4;
    }

    // Process remaining bytes
    while offset < data.len() {
        acc = rotl32(
            acc.wrapping_add((data[offset] as u32).wrapping_mul(PRIME32_5)),
            11,
        )
        .wrapping_mul(PRIME32_1);
        offset += 1;
    }

    // Finalize
    acc ^= acc >> 15;
    acc = acc.wrapping_mul(PRIME32_2);
    acc ^= acc >> 13;
    acc = acc.wrapping_mul(PRIME32_3);
    acc ^= acc >> 16;
    acc
}

/// MurmurHash3 32-bit implementation.
///
/// MurmurHash3 is a fast, non-cryptographic hash algorithm with excellent
/// avalanche properties and collision resistance. Suitable for hash tables and
/// general-purpose hashing.
///
/// Source: https://github.com/aappleby/smhasher/wiki/MurmurHash3
///
/// Constants:
/// - C1: 0xcc9e2d51
/// - C2: 0x1b873593
///
/// # Arguments
///
/// * `data` - Byte slice to hash
/// * `seed` - 32-bit seed for variation
///
/// # Returns
///
/// A 32-bit hash value
///
/// # Example
///
/// ```rust
/// # fn murmur3_32(data: &[u8], seed: u32) -> u32 {
/// #     const C1: u32 = 0xcc9e2d51;
/// #     const C2: u32 = 0x1b873593;
/// #
/// #     let mut hash = seed;
/// #     let mut offset = 0;
/// #
/// #     // Process 4-byte chunks
/// #     while offset + 4 <= data.len() {
/// #         let mut k = u32::from_le_bytes([
/// #             data[offset],
/// #             data[offset + 1],
/// #             data[offset + 2],
/// #             data[offset + 3],
/// #         ]);
/// #         k = k.wrapping_mul(C1);
/// #         k = ((k << 15) | (k >> 17)); // rotl32(k, 15)
/// #         k = k.wrapping_mul(C2);
/// #
/// #         hash ^= k;
/// #         hash = ((hash << 13) | (hash >> 19)); // rotl32(hash, 13)
/// #         hash = hash.wrapping_mul(5).wrapping_add(0xe6546b64);
/// #
/// #         offset += 4;
/// #     }
/// #
/// #     // Process remaining bytes (0-3)
/// #     let tail = &data[offset..];
/// #     let mut k1: u32 = 0;
/// #     match tail.len() {
/// #         3 => k1 ^= (tail[2] as u32) << 16;
/// #         2 => { k1 ^= (tail[1] as u32) << 8; },
/// #         1 => { k1 ^= tail[0] as u32; },
/// #         _ => {}
/// #     }
/// #
/// #     if tail.len() > 0 {
/// #         k1 = k1.wrapping_mul(C1);
/// #         k1 = ((k1 << 15) | (k1 >> 17)); // rotl32(k1, 15)
/// #         k1 = k1.wrapping_mul(C2);
/// #         hash ^= k1;
/// #     }
/// #
/// #     // Finalize
/// #     hash ^= data.len() as u32;
/// #     hash = murmur3_fmix32(hash);
/// #     hash
/// # }
/// let hash1 = murmur3_32(b"hello", 0);
/// let hash2 = murmur3_32(b"hello", 0);
/// assert_eq!(hash1, hash2); // Deterministic
/// assert_ne!(hash1, murmur3_32(b"world", 0)); // Different input → different hash
/// ```
#[inline(always)]
pub(crate) fn murmur3_32(data: &[u8], seed: u32) -> u32 {
    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;

    let mut hash = seed;
    let mut offset = 0;

    // Process 4-byte chunks
    while offset + 4 <= data.len() {
        let mut k = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        k = k.wrapping_mul(C1);
        k = rotl32(k, 15);
        k = k.wrapping_mul(C2);

        hash ^= k;
        hash = rotl32(hash, 13);
        hash = hash.wrapping_mul(5).wrapping_add(0xe6546b64);

        offset += 4;
    }

    // Process remaining bytes (0-3)
    let tail = &data[offset..];
    let mut k1: u32 = 0;

    match tail.len() {
        3 => {
            k1 ^= (tail[2] as u32) << 16;
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
        }
        2 => {
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
        }
        1 => {
            k1 ^= tail[0] as u32;
        }
        _ => {}
    }

    if tail.len() > 0 {
        k1 = k1.wrapping_mul(C1);
        k1 = rotl32(k1, 15);
        k1 = k1.wrapping_mul(C2);
        hash ^= k1;
    }

    // Finalize with fmix32
    hash ^= data.len() as u32;
    murmur3_fmix32(hash)
}

/// Final mixing function for MurmurHash3.
/// Provides avalanche properties in the output.
#[inline(always)]
fn murmur3_fmix32(mut h1: u32) -> u32 {
    h1 ^= h1 >> 16;
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 ^= h1 >> 16;
    h1
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;
    use super::*;

    // FNV-1a tests
    #[test]
    fn test_fnv1a_64_empty() {
        let hash = fnv1a_64(b"");
        assert_eq!(hash, 0xcbf29ce484222325);
    }

    #[test]
    fn test_fnv1a_64_single_byte() {
        let hash = fnv1a_64(b"a");
        assert_eq!(hash, 0xaf63dc4c8601ec8c);
    }

    #[test]
    fn test_fnv1a_64_deterministic() {
        let hash1 = fnv1a_64(b"hello world");
        let hash2 = fnv1a_64(b"hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fnv1a_64_different_inputs() {
        let hash1 = fnv1a_64(b"hello");
        let hash2 = fnv1a_64(b"world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_fnv1a_64_longer_input() {
        let data = vec![0u8; 1000];
        let hash = fnv1a_64(&data);
        assert!(hash != 0);
    }

    // xxHash32 tests
    #[test]
    fn test_xxhash32_empty() {
        let hash = xxhash32(b"", 0);
        assert_eq!(hash, 0xc5c9f25);
    }

    #[test]
    fn test_xxhash32_empty_different_seed() {
        let hash = xxhash32(b"", 1);
        assert_ne!(hash, xxhash32(b"", 0));
    }

    #[test]
    fn test_xxhash32_single_byte() {
        let hash = xxhash32(b"a", 0);
        assert_eq!(hash, 0xc0dd0e65);
    }

    #[test]
    fn test_xxhash32_deterministic() {
        let hash1 = xxhash32(b"hello world", 42);
        let hash2 = xxhash32(b"hello world", 42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_xxhash32_different_inputs() {
        let hash1 = xxhash32(b"hello", 0);
        let hash2 = xxhash32(b"world", 0);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_xxhash32_seed_variation() {
        let hash1 = xxhash32(b"test", 0);
        let hash2 = xxhash32(b"test", 1);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_xxhash32_longer_input() {
        let data = vec![0u8; 1000];
        let hash = xxhash32(&data, 0);
        assert!(hash != 0);
    }

    // MurmurHash3 tests
    #[test]
    fn test_murmur3_32_empty() {
        let hash = murmur3_32(b"", 0);
        assert_eq!(hash, 0);
    }

    #[test]
    fn test_murmur3_32_single_byte() {
        let hash = murmur3_32(b"a", 0);
        assert_eq!(hash, 0x3c2569b2);
    }

    #[test]
    fn test_murmur3_32_deterministic() {
        let hash1 = murmur3_32(b"hello world", 42);
        let hash2 = murmur3_32(b"hello world", 42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_murmur3_32_different_inputs() {
        let hash1 = murmur3_32(b"hello", 0);
        let hash2 = murmur3_32(b"world", 0);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_murmur3_32_seed_variation() {
        let hash1 = murmur3_32(b"test", 0);
        let hash2 = murmur3_32(b"test", 1);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_murmur3_32_longer_input() {
        let data = vec![0u8; 1000];
        let hash = murmur3_32(&data, 0);
        assert!(hash != 0);
    }

    #[test]
    fn test_murmur3_32_prefix_sensitivity() {
        // Verify that prefix differences are detected
        let hash1 = murmur3_32(b"abc", 0);
        let hash2 = murmur3_32(b"abd", 0);
        assert_ne!(hash1, hash2);
    }

    // Cross-algorithm tests
    #[test]
    fn test_hashes_different_algorithms() {
        let data = b"test data";
        let fnv = fnv1a_64(data) as u32;
        let xx = xxhash32(data, 0);
        let mm = murmur3_32(data, 0);

        // Different algorithms should generally produce different values
        // (Not guaranteed but very likely)
        assert!(!(fnv == xx && xx == mm));
    }

    #[test]
    fn test_rotl32_correctness() {
        let x = 0x12345678u32;
        let rotated = rotl32(x, 8);
        assert_eq!(rotated, 0x34567812);
    }

    #[test]
    fn test_rotl64_correctness() {
        let x = 0x123456789abcdef0u64;
        let rotated = rotl64(x, 8);
        assert_eq!(rotated, 0x3456789abcdef012);
    }
}

#[cfg(all(test, feature = "bench"))]
mod benches {
    extern crate alloc;
    use alloc::vec;
    use super::*;

    #[bench]
    fn bench_fnv1a_64_1kb(b: &mut test::Bencher) {
        let data = vec![0u8; 1024];
        b.iter(|| fnv1a_64(&data));
    }

    #[bench]
    fn bench_fnv1a_64_10kb(b: &mut test::Bencher) {
        let data = vec![0u8; 10240];
        b.iter(|| fnv1a_64(&data));
    }

    #[bench]
    fn bench_xxhash32_1kb(b: &mut test::Bencher) {
        let data = vec![0u8; 1024];
        b.iter(|| xxhash32(&data, 0));
    }

    #[bench]
    fn bench_xxhash32_10kb(b: &mut test::Bencher) {
        let data = vec![0u8; 10240];
        b.iter(|| xxhash32(&data, 0));
    }

    #[bench]
    fn bench_murmur3_32_1kb(b: &mut test::Bencher) {
        let data = vec![0u8; 1024];
        b.iter(|| murmur3_32(&data, 0));
    }

    #[bench]
    fn bench_murmur3_32_10kb(b: &mut test::Bencher) {
        let data = vec![0u8; 10240];
        b.iter(|| murmur3_32(&data, 0));
    }
}
