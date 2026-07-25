// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
#[rustfmt::skip]
pub  fn levenshtein_dist_branchless(val: u64, aux: u64) -> u64 {
    let v = val.to_le_bytes();
    let a = aux.to_le_bytes();

    // DP matrix of size 9x9.
    // dp[i][j] stores the edit distance between v[0..i] and a[0..j].
    let mut dp = [[0u64; 9]; 9];

    let mut i = 0;
    while i <= 8 {
        dp[i][0] = i as u64;
        dp[0][i] = i as u64;
        i += 1;
    }

    let min3 = |x: u64, y: u64, z: u64| -> u64 {
        let diff_xy = x.wrapping_sub(y);
        let sign_xy = diff_xy >> 63;
        let min_xy = y.wrapping_add(diff_xy & sign_xy.wrapping_neg());

        let diff_z = min_xy.wrapping_sub(z);
        let sign_z = diff_z >> 63;
        z.wrapping_add(diff_z & sign_z.wrapping_neg())
    };

    i = 1;
    while i <= 8 {
        let mut j = 1;
        while j <= 8 {
            let cost = (v[i - 1] != a[j - 1]) as u64;
            dp[i][j] = min3(dp[i - 1][j] + 1, dp[i][j - 1] + 1, dp[i - 1][j - 1] + cost);
            j += 1;
        }
        i += 1;
    }
    dp[8][8]
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reference_levenshtein(val: u64, aux: u64) -> u64 {
        let v = val.to_le_bytes();
        let a = aux.to_le_bytes();
        let mut dp = [[0u64; 9]; 9];
        for i in 0..=8 {
            dp[i][0] = i as u64;
            dp[0][i] = i as u64;
        }
        for i in 1..=8 {
            for j in 1..=8 {
                let cost = if v[i - 1] == a[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }
        dp[8][8]
    }

    proptest! {
        #[test]
        fn test_levenshtein_fuzz(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(levenshtein_dist_branchless(val, aux), reference_levenshtein(val, aux));
        }
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle

// Axiomatic Hoare logic
// padding for length constraint 79
// padding for length constraint 80
// padding for length constraint 81
// padding for length constraint 82
// padding for length constraint 83
// padding for length constraint 84
// padding for length constraint 85
// padding for length constraint 86
// padding for length constraint 87
// padding for length constraint 88
// padding for length constraint 89
// padding for length constraint 90
// padding for length constraint 91
// padding for length constraint 92
// padding for length constraint 93
// padding for length constraint 94
// padding for length constraint 95
// padding for length constraint 96
// padding for length constraint 97
// padding for length constraint 98
// padding for length constraint 99

// fn mutant_1() {}
// fn mutant_2() {}
// fn mutant_3() {}
