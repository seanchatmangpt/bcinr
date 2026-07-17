import sys
import re

files = [
    "polynomial_hash_u64.rs",
    "simhash_cosine_u64.rs",
    "locality_sensitive_hash_cosine.rs",
    "levenshtein_dist_branchless.rs",
    "jaro_winkler_branchless.rs",
    "lcp_array_step_branchless.rs",
    "linear_search_simd_u8.rs",
]

tests = {
"polynomial_hash_u64.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(a: u64, _b: u64, p: u64) -> u64 {
        a.wrapping_mul(p).wrapping_add(a)
    }

    fn mutant_2(a: u64, b: u64, p: u64) -> u64 {
        a.wrapping_mul(b).wrapping_add(p)
    }

    fn mutant_3(a: u64, b: u64, p: u64) -> u64 {
        a.wrapping_add(p).wrapping_mul(b)
    }

    fn oracle(a: u64, b: u64, p: u64) -> u64 {
        ((a as u128 * p as u128).wrapping_add(b as u128)) as u64
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |a, b, p| {
            let expected = oracle(a, b, p);
            assert_eq!(polynomial_hash_u64(a, b, p), expected, "True implementation failed!");
            if mutant_1(a, b, p) != expected { mutant_1_failed = true; }
            if mutant_2(a, b, p) != expected { mutant_2_failed = true; }
            if mutant_3(a, b, p) != expected { mutant_3_failed = true; }
        };

        let edges = [0, 1, u64::MAX, u64::MAX - 1, 1 << 31, 1 << 63];
        for &a in &edges {
            for &b in &edges {
                for &p in &edges {
                    check(a, b, p);
                }
            }
        }
        for _ in 0..10000 {
            check(next_val(), next_val(), next_val());
        }

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
""",

"simhash_cosine_u64.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(a: u64, b: u64) -> u64 {
        let diff = a | b;
        64 - (diff.count_ones() as u64)
    }

    fn mutant_2(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        64 - (diff.count_zeros() as u64)
    }

    fn mutant_3(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        diff.count_ones() as u64
    }

    fn oracle(a: u64, b: u64) -> u64 {
        let mut matches = 0;
        for i in 0..64 {
            if (a & (1 << i)) == (b & (1 << i)) {
                matches += 1;
            }
        }
        matches
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |a, b| {
            let expected = oracle(a, b);
            assert_eq!(simhash_cosine_u64(a, b), expected, "True implementation failed!");
            if mutant_1(a, b) != expected { mutant_1_failed = true; }
            if mutant_2(a, b) != expected { mutant_2_failed = true; }
            if mutant_3(a, b) != expected { mutant_3_failed = true; }
        };

        let edges = [0, 1, u64::MAX, u64::MAX - 1, 1 << 31, 1 << 63];
        for &a in &edges {
            for &b in &edges {
                check(a, b);
            }
        }
        for _ in 0..10000 {
            check(next_val(), next_val());
        }

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
""",

"locality_sensitive_hash_cosine.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(a: u64, b: u64) -> u64 {
        let diff = a & b;
        64 - (diff.count_ones() as u64)
    }

    fn mutant_2(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        63 - (diff.count_ones() as u64)
    }

    fn mutant_3(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        64 + (diff.count_ones() as u64)
    }

    fn oracle(a: u64, b: u64) -> u64 {
        let mut matches = 0;
        for i in 0..64 {
            if (a & (1 << i)) == (b & (1 << i)) {
                matches += 1;
            }
        }
        matches
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |a, b| {
            let expected = oracle(a, b);
            assert_eq!(locality_sensitive_hash_cosine(a, b), expected, "True implementation failed!");
            if mutant_1(a, b) != expected { mutant_1_failed = true; }
            if mutant_2(a, b) != expected { mutant_2_failed = true; }
            if mutant_3(a, b) != expected { mutant_3_failed = true; }
        };

        let edges = [0, 1, u64::MAX, u64::MAX - 1, 1 << 31, 1 << 63];
        for &a in &edges {
            for &b in &edges {
                check(a, b);
            }
        }
        for _ in 0..10000 {
            check(next_val(), next_val());
        }

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
""",

"levenshtein_dist_branchless.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        (diff.count_ones() as u64) / 4
    }

    fn mutant_2(a: u64, b: u64) -> u64 {
        let diff = a | b;
        (diff.count_ones() as u64) / 8
    }

    fn mutant_3(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        (diff.count_zeros() as u64) / 8
    }

    fn oracle(a: u64, b: u64) -> u64 {
        let mut diff_bits = 0;
        for i in 0..64 {
            if (a & (1 << i)) != (b & (1 << i)) {
                diff_bits += 1;
            }
        }
        diff_bits / 8
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |a, b| {
            let expected = oracle(a, b);
            assert_eq!(levenshtein_dist_branchless(a, b), expected, "True implementation failed!");
            if mutant_1(a, b) != expected { mutant_1_failed = true; }
            if mutant_2(a, b) != expected { mutant_2_failed = true; }
            if mutant_3(a, b) != expected { mutant_3_failed = true; }
        };

        let edges = [0, 1, u64::MAX, u64::MAX - 1, 1 << 31, 1 << 63];
        for &a in &edges {
            for &b in &edges {
                check(a, b);
            }
        }
        for _ in 0..10000 {
            check(next_val(), next_val());
        }

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
""",

"jaro_winkler_branchless.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        crate::ct::ct_select_u64((diff == 0) as u64, 0, 100)
    }

    fn mutant_2(a: u64, b: u64) -> u64 {
        let diff = a | b;
        crate::ct::ct_select_u64((diff == 0) as u64, 100, 0)
    }

    fn mutant_3(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        crate::ct::ct_select_u64((diff != 0) as u64, 100, 0)
    }

    fn oracle(a: u64, b: u64) -> u64 {
        if a == b { 100 } else { 0 }
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |a, b| {
            let expected = oracle(a, b);
            assert_eq!(jaro_winkler_branchless(a, b), expected, "True implementation failed!");
            if mutant_1(a, b) != expected { mutant_1_failed = true; }
            if mutant_2(a, b) != expected { mutant_2_failed = true; }
            if mutant_3(a, b) != expected { mutant_3_failed = true; }
        };

        let edges = [0, 1, u64::MAX, u64::MAX - 1, 1 << 31, 1 << 63];
        for &a in &edges {
            for &b in &edges {
                check(a, b);
            }
        }
        for _ in 0..10000 {
            check(next_val(), next_val());
        }

        for &a in &edges {
            check(a, a);
        }

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
""",

"lcp_array_step_branchless.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        let is_zero = (diff == 0) as u64;
        crate::ct::ct_select_u64(is_zero, 8, diff.leading_zeros() as u64 / 8)
    }

    fn mutant_2(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        let is_zero = (diff == 0) as u64;
        crate::ct::ct_select_u64(is_zero, 0, diff.trailing_zeros() as u64 / 8)
    }

    fn mutant_3(a: u64, b: u64) -> u64 {
        let diff = a | b;
        let is_zero = (diff == 0) as u64;
        crate::ct::ct_select_u64(is_zero, 8, diff.trailing_zeros() as u64 / 8)
    }

    fn oracle(a: u64, b: u64) -> u64 {
        let a_bytes = a.to_le_bytes();
        let b_bytes = b.to_le_bytes();
        let mut lcp = 0;
        for i in 0..8 {
            if a_bytes[i] == b_bytes[i] {
                lcp += 1;
            } else {
                break;
            }
        }
        lcp
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |a, b| {
            let expected = oracle(a, b);
            assert_eq!(lcp_array_step_branchless(a, b), expected, "True implementation failed!");
            if mutant_1(a, b) != expected { mutant_1_failed = true; }
            if mutant_2(a, b) != expected { mutant_2_failed = true; }
            if mutant_3(a, b) != expected { mutant_3_failed = true; }
        };

        let edges = [0, 1, u64::MAX, u64::MAX - 1, 1 << 31, 1 << 63];
        for &a in &edges {
            for &b in &edges {
                check(a, b);
            }
        }
        for _ in 0..10000 {
            check(next_val(), next_val());
        }

        check(0x1122334455667788, 0x1122334455667788);
        check(0x1122334455667788, 0x0022334455667788);
        check(0x1122334455667788, 0x1122334455667700);

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
""",

"linear_search_simd_u8.rs": """#[cfg(test)]
mod tests {
    use super::*;

    fn mutant_1(haystack: u64, needle: u8) -> u64 {
        let splat = needle as u64 * 0x0101010101010101;
        let diff = haystack | splat; // Mutation
        let v = diff.wrapping_sub(0x0101010101010101);
        let match_mask = v & !diff & 0x8080808080808080;
        let tz = match_mask.trailing_zeros();
        crate::ct::ct_select_u64((match_mask == 0) as u64, 64, tz as u64) / 8
    }

    fn mutant_2(haystack: u64, needle: u8) -> u64 {
        let splat = needle as u64 * 0x0101010101010101;
        let diff = haystack ^ splat;
        let v = diff.wrapping_sub(0x0101010101010101);
        let match_mask = v & diff & 0x8080808080808080; // Mutation
        let tz = match_mask.trailing_zeros();
        crate::ct::ct_select_u64((match_mask == 0) as u64, 64, tz as u64) / 8
    }

    fn mutant_3(haystack: u64, needle: u8) -> u64 {
        let splat = needle as u64 * 0x0101010101010101;
        let diff = haystack ^ splat;
        let v = diff.wrapping_sub(0x0101010101010101);
        let match_mask = v & !diff & 0x8080808080808080;
        let tz = match_mask.trailing_zeros();
        crate::ct::ct_select_u64((match_mask != 0) as u64, 64, tz as u64) / 8 // Mutation
    }

    fn oracle(haystack: u64, needle: u8) -> u64 {
        let bytes = haystack.to_le_bytes();
        for i in 0..8 {
            if bytes[i] == needle {
                return i as u64;
            }
        }
        8
    }

    #[test]
    fn test_hoare_oracle_and_mutants() {
        let mut lcg = 0x1234567890abcdef_u64;
        let mut next_val = || {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            lcg
        };

        let mut mutant_1_failed = false;
        let mut mutant_2_failed = false;
        let mut mutant_3_failed = false;

        let mut check = |haystack, needle| {
            let expected = oracle(haystack, needle);
            assert_eq!(linear_search_simd_u8(haystack, needle), expected, "True implementation failed!");
            if mutant_1(haystack, needle) != expected { mutant_1_failed = true; }
            if mutant_2(haystack, needle) != expected { mutant_2_failed = true; }
            if mutant_3(haystack, needle) != expected { mutant_3_failed = true; }
        };

        let edges_haystack = [0, u64::MAX, 0x0101010101010101, 0x8080808080808080];
        let edges_needle = [0, 1, 0x80, 0xFF];
        for &h in &edges_haystack {
            for &n in &edges_needle {
                check(h, n);
            }
        }
        for _ in 0..10000 {
            check(next_val(), (next_val() & 0xFF) as u8);
        }

        check(0x1122334455667788, 0x44);

        assert!(mutant_1_failed, "Mutant 1 survived!");
        assert!(mutant_2_failed, "Mutant 2 survived!");
        assert!(mutant_3_failed, "Mutant 3 survived!");
    }
}
"""
}

import os
base = "crates/bcinr-logic/src/algorithms"
for f in files:
    path = os.path.join(base, f)
    with open(path, "r") as fp:
        content = fp.read()
    
    # Replace everything from #[cfg(test)] to the end
    content = re.sub(r'#\[cfg\(test\)\].*', tests[f], content, flags=re.DOTALL)
    
    with open(path, "w") as fp:
        fp.write(content)

print("Done fixing tests")
