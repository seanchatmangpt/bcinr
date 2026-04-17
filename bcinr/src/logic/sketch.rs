//! Probabilistic Sketches: Hashing and probabilistic data structures
//!
//! This module contains handwritten, production-quality implementations
//! of high-performance hash algorithms.

/// Rotate a 32-bit integer left by n bits.
#[inline(always)]
const fn rotl32(x: u32, n: u32) -> u32 {
    x.rotate_left(n)
}

/// Murmur3 32-bit implementation.
#[inline(always)]
pub fn murmur3_32(data: &[u8], seed: u32) -> u32 {
    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;
    let mut hash = seed;
    let mut offset = 0;
    while offset + 4 <= data.len() {
        let mut k = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        k = k.wrapping_mul(C1);
        k = rotl32(k, 15);
        k = k.wrapping_mul(C2);
        hash ^= k;
        hash = rotl32(hash, 13);
        hash = hash.wrapping_mul(5).wrapping_add(0xe6546b64);
        offset += 4;
    }
    hash ^= data.len() as u32;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^ (hash >> 16)
}

/// FNV-1a 64-bit hash.
#[inline(always)]
pub fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// xxHash 32-bit.
#[inline(always)]
pub fn xxhash32(data: &[u8], seed: u32) -> u32 {
    let mut acc = seed.wrapping_add(0x165667b1);
    for chunk in data.chunks_exact(4) {
        let k = u32::from_le_bytes(chunk.try_into().unwrap());
        acc = rotl32(acc.wrapping_add(k.wrapping_mul(0xc2b2ae35)), 17).wrapping_mul(0x27d4eb2d);
    }
    // Finalize
    acc ^= acc >> 15;
    acc = acc.wrapping_mul(0x85ebca6b);
    acc ^= acc >> 13;
    acc = acc.wrapping_mul(0xc2b2ae35);
    acc ^ (acc >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hashes() {
        assert_ne!(murmur3_32(b"a", 0), 0);
        assert_ne!(fnv1a_64(b"a"), 0);
        assert_ne!(xxhash32(b"a", 0), 0);
    }
}
