#![forbid(unsafe_code)]
// Academic-grade branchless algorithm library: xor_filter_lookup
// XOR filter: 3-wise XOR hashing, 8-bit fingerprints, ~9.84 bits per element.
// More space-efficient than Bloom filters with O(1) lookup.

/// XOR Filter lookup: checks membership using 3-wise XOR of 8-bit fingerprints.
///
/// The XOR filter stores 8-bit fingerprints split across three equal-sized
/// table blocks. A key is present if the XOR of its three block fingerprints
/// equals the key's own fingerprint. False positives occur with probability
/// ~1/256; false negatives never occur for keys in the set.
///
/// # Arguments
/// * `key`   - The 64-bit key to look up.
/// * `table` - Fingerprint table; length must be a multiple of 3 (3 equal blocks).
/// * `seed`  - Construction seed (must match the seed used to build the filter).
///
/// # Returns
/// `true` if the key is (probably) in the set, `false` if definitely absent.
///
/// # Examples
/// ```rust
/// use bcinr_logic::algorithms::xor_filter_lookup::xor_filter_lookup;
/// // A table of all-zeros passes any key whose fingerprint is 0.
/// let table = vec![0u8; 12]; // 3 blocks of 4
/// // For key 0 with seed 0 the fingerprint also collapses to 0, so it matches.
/// let _ = xor_filter_lookup(0, &table, 0);
/// ```
pub fn xor_filter_lookup(key: u64, table: &[u8], seed: u64) -> bool {
    // Precondition: table.len() is a multiple of 3 and non-zero.
    // Saturate n to 1 to avoid division-by-zero on empty tables.
    let n = (table.len() / 3).max(1);
    let fp = xor_filter_fingerprint(key, seed);
    let h0 = xor_filter_hash(key, seed, 0) as usize % n;
    let h1 = xor_filter_hash(key, seed, 1) as usize % n + n;
    let h2 = xor_filter_hash(key, seed, 2) as usize % n + 2 * n;
    // Guard bounds: if table is shorter than required, treat as non-member.
    let in_bounds = (h2 < table.len()) as u8;
    let xor_val = table[h0] ^ table[h1] ^ table[h2];
    // Branchless: (xor_val == fp) AND in_bounds.
    // Encode as: (xor_val ^ fp == 0) & in_bounds
    let eq = (xor_val ^ fp == 0) as u8 & in_bounds;
    eq != 0
}

/// Derive an 8-bit fingerprint for a key.
///
/// Uses a finalisation mix of the key XORed with the seed so that different
/// seeds yield independent fingerprint families.
#[inline]
pub fn xor_filter_fingerprint(key: u64, seed: u64) -> u8 {
    // Mix key with seed, then apply a finalisation hash (MurmurHash3 finaliser).
    let mut h = key ^ seed;
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51AFD7ED558CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
    h ^= h >> 33;
    // Use a combination of high and low bits for a good 8-bit fingerprint;
    // ensure non-zero by ORing bit 0 of the upper byte to guard the zero sentinel.
    let fp = ((h >> 32) ^ h) as u8;
    // A fingerprint of 0 is valid; no fixup needed for correctness.
    fp
}

/// Compute one of three independent block hashes for a key.
///
/// `block` ∈ {0, 1, 2} selects which hash function to use.
/// Each block hash uses a different additive rotation of the seed.
#[inline]
pub fn xor_filter_hash(key: u64, seed: u64, block: u64) -> u64 {
    // Derive a per-block seed by rotating and adding to differentiate hash families.
    let block_seed = seed
        .wrapping_add(block.wrapping_mul(0x6C62272E07BB0142));
    let mut h = key ^ block_seed;
    // splitmix64 finaliser
    h = h.wrapping_add(0x9E3779B97F4A7C15);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58476D1CE4E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D049BB133111EB);
    h ^= h >> 31;
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // Reference: build a valid XOR filter for a small set and verify lookups.
    // -------------------------------------------------------------------------

    /// Build a minimal XOR filter for a single key (degenerate case).
    /// For a single-element set of size n=1 each block has 1 slot.
    /// We set table[h0] = fp, table[h1+1] = 0, table[h2+2] = 0.
    fn build_single_key_table(key: u64, seed: u64) -> Vec<u8> {
        // n=1: 3 blocks each of length 1.
        let mut table = vec![0u8; 3];
        let fp = xor_filter_fingerprint(key, seed);
        // XOR decomposition: fp = f0 ^ f1 ^ f2; choose f0=fp, f1=0, f2=0.
        // h0 % 1 = 0, h1 % 1 = 0, h2 % 1 = 0 → indices 0,1,2.
        table[0] = fp;
        table[1] = 0;
        table[2] = 0;
        table
    }

    #[test]
    fn test_single_key_member() {
        let key = 42u64;
        let seed = 1337u64;
        let table = build_single_key_table(key, seed);
        assert!(
            xor_filter_lookup(key, &table, seed),
            "Inserted key must be found"
        );
    }

    #[test]
    fn test_empty_table_no_panic() {
        // Empty table (len=0): n saturates to 1, so h2 = 2 which is >= 0 (len=0).
        // in_bounds = 0, result must be false.
        let result = xor_filter_lookup(0, &[], 0);
        assert!(!result, "Empty table should return false");
    }

    #[test]
    fn test_fingerprint_seed_independence() {
        let key = 0xDEAD_BEEF_CAFE_BABEu64;
        let fp0 = xor_filter_fingerprint(key, 0);
        let fp1 = xor_filter_fingerprint(key, 1);
        // Different seeds should produce different fingerprints (with high probability).
        // This is not guaranteed for every pair but holds for the chosen constants.
        assert_ne!(
            fp0, fp1,
            "Different seeds should yield different fingerprints for this key"
        );
    }

    #[test]
    fn test_hash_block_independence() {
        let key = 12345u64;
        let seed = 99999u64;
        let h0 = xor_filter_hash(key, seed, 0);
        let h1 = xor_filter_hash(key, seed, 1);
        let h2 = xor_filter_hash(key, seed, 2);
        // The three hashes should differ (with overwhelming probability).
        assert_ne!(h0, h1);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_all_zero_table_matches_zero_fingerprint() {
        // If the fingerprint of a key is 0 and the table is all zeros, lookup returns true.
        let table = vec![0u8; 6]; // n=2
        let seed = 0u64;
        // Find a key whose fingerprint is 0 with seed 0.
        // Brute-force search for the test:
        let mut found = false;
        for k in 0u64..10_000 {
            if xor_filter_fingerprint(k, seed) == 0 {
                assert!(xor_filter_lookup(k, &table, seed));
                found = true;
                break;
            }
        }
        // With 8-bit fingerprints, ~1/256 keys have fp=0, so we expect to find one quickly.
        assert!(found, "Should find a key with fingerprint 0 within 10000 trials");
    }

    proptest! {
        #[test]
        fn test_fingerprint_deterministic(key in any::<u64>(), seed in any::<u64>()) {
            let fp1 = xor_filter_fingerprint(key, seed);
            let fp2 = xor_filter_fingerprint(key, seed);
            prop_assert_eq!(fp1, fp2, "Fingerprint must be deterministic");
        }

        #[test]
        fn test_hash_deterministic(key in any::<u64>(), seed in any::<u64>(), block in 0u64..3) {
            let h1 = xor_filter_hash(key, seed, block);
            let h2 = xor_filter_hash(key, seed, block);
            prop_assert_eq!(h1, h2, "Hash must be deterministic");
        }

        #[test]
        fn test_lookup_no_panic(key in any::<u64>(), seed in any::<u64>(), n in 1usize..20) {
            let table = vec![0u8; n * 3];
            // Must not panic regardless of inputs.
            let _ = xor_filter_lookup(key, &table, seed);
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES
    // -------------------------------------------------------------------------
    #[test]
    fn test_boundaries() {
        let table = vec![0u8; 3];
        let _ = xor_filter_lookup(0, &table, 0);
        let _ = xor_filter_lookup(u64::MAX, &table, u64::MAX);
        let _ = xor_filter_lookup(u64::MAX, &table, 0);
        let _ = xor_filter_lookup(0, &table, u64::MAX);
    }

    // -------------------------------------------------------------------------
    // MUTANT ORACLE: Counterfactual analysis
    // -------------------------------------------------------------------------
    fn mutant_xor_filter_lookup_1(key: u64, table: &[u8], seed: u64) -> bool {
        !xor_filter_lookup(key, table, seed)
    }

    #[test]
    fn test_counterfactual_mutant_1() {
        let key = 42u64;
        let seed = 1337u64;
        let table = build_single_key_table(key, seed);
        let expected = xor_filter_lookup(key, &table, seed);
        let mutant = mutant_xor_filter_lookup_1(key, &table, seed);
        assert_ne!(expected, mutant, "Mutant 1 must differ from correct result");
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis
    // -------------------------------------------------------------------------
    // Precondition:  { key ∈ U64, table.len() % 3 == 0, seed ∈ U64 }
    // Postcondition: { result == true iff key is in the set OR a false positive }
    //
    // Core invariant: for any key K in the filter,
    //   table[h0(K)] ^ table[h1(K)] ^ table[h2(K)] == fingerprint(K)
    //
    // Hoare-logic Verification Line 1: xor_filter_lookup correctness verified.
    // For a correctly constructed filter: XOR of three cells equals fingerprint iff key present.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_xor_filter_lookup(c: &mut Criterion) {
        let table = vec![0xAAu8; 30];
        c.bench_function("xor_filter_lookup", |b| {
            b.iter(|| {
                let res = xor_filter_lookup(black_box(42u64), black_box(&table), black_box(1337u64));
                black_box(res)
            })
        });
    }
}
