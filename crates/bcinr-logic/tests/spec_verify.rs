//! Machine-Verified Algorithm Suite — Innovation 5 spec-verify integration test.
//!
//! Run with: cargo test -p bcinr-logic --features spec-verify --test spec_verify
//!
//! For each algorithm, a reference oracle is defined here (independent of the
//! algorithm file), and 206 deterministic test cases are compared. A BLAKE3
//! conformance receipt is computed over all results so that the outcome can
//! be independently reproduced and audited.
#![cfg(feature = "spec-verify")]

use bcinr_logic::algorithms::{
    count_consecutive_set_bits_u64::count_consecutive_set_bits_u64,
    fp_sqrt_u32_q16::fp_sqrt_u32_q16,
    gather_bits_u64::gather_bits_u64,
    gcd_u64_branchless::gcd_u64_branchless,
    halton_sequence_u32::halton_sequence_u32,
    lcm_u64_branchless::lcm_u64_branchless,
    linear_search_simd_u8::linear_search_simd_u8,
    locality_sensitive_hash_cosine::locality_sensitive_hash_cosine,
    norm_u32::norm_u32,
    normalize_slice_branchless::normalize_slice_branchless,
    parallel_bits_deposit_u64::parallel_bits_deposit_u64,
    parallel_bits_extract_u64::parallel_bits_extract_u64,
    rank_u128::rank_u128,
    scatter_bits_u64::scatter_bits_u64,
};

// ---------------------------------------------------------------------------
// Theorem result type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct TheoremResult {
    pub algorithm: &'static str,
    pub pass: bool,
    pub cases: u64,
    pub first_failure: Option<(u64, u64)>,
}

// ---------------------------------------------------------------------------
// Reference oracles (independent of algorithm implementations)
// ---------------------------------------------------------------------------

fn ref_gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn ref_lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a / ref_gcd(a, b)).wrapping_mul(b)
}

fn ref_norm_u32(val: u64) -> u64 {
    let x = (val & 0xFFFFFFFF) as u128;
    let y = (val >> 32) as u128;
    let sq = x * x + y * y;
    if sq == 0 {
        return 0;
    }
    let mut r = sq;
    loop {
        let next = (r + sq / r) / 2;
        if next >= r {
            break;
        }
        r = next;
    }
    r as u64
}

fn ref_fp_sqrt_u32_q16(val: u64) -> u64 {
    let scaled = (val as u128) << 16;
    if scaled == 0 {
        return 0;
    }
    let mut r = scaled;
    loop {
        let next = (r + scaled / r) / 2;
        if next >= r {
            break;
        }
        r = next;
    }
    r as u64
}

fn ref_count_consecutive_set_bits_u64(val: u64) -> u64 {
    let mut max_c = 0u64;
    let mut cur_c = 0u64;
    for i in 0..64 {
        if ((val >> i) & 1) == 1 {
            cur_c += 1;
            if cur_c > max_c {
                max_c = cur_c;
            }
        } else {
            cur_c = 0;
        }
    }
    max_c
}

fn ref_parallel_bits_extract_u64(val: u64, aux: u64) -> u64 {
    let mut res = 0u64;
    let mut r_idx = 0u32;
    for i in 0..64 {
        if ((aux >> i) & 1) == 1 {
            if ((val >> i) & 1) == 1 {
                res |= 1 << r_idx;
            }
            r_idx += 1;
        }
    }
    res
}

fn ref_parallel_bits_deposit_u64(val: u64, aux: u64) -> u64 {
    let mut res = 0u64;
    let mut v_idx = 0u32;
    for i in 0..64 {
        if ((aux >> i) & 1) == 1 {
            if ((val.wrapping_shr(v_idx)) & 1) == 1 {
                res |= 1 << i;
            }
            v_idx += 1;
        }
    }
    res
}

fn ref_gather_bits_u64(val: u64, aux: u64) -> u64 {
    let mut res = 0u64;
    let mut r_idx = 0u32;
    for i in 0u32..64 {
        if ((aux >> i) & 1) == 1 {
            if ((val >> i) & 1) == 1 {
                res |= 1 << r_idx;
            }
            r_idx += 1;
        }
    }
    res
}

fn ref_rank_u128(val: u64, aux: u64) -> u64 {
    let limit = (aux & 0x7F) as u32;
    let mut c = 0u64;
    for i in 0..limit.min(64) {
        c += (val >> i) & 1;
    }
    c
}

fn ref_halton_sequence_u32(val: u64) -> u64 {
    let mut f = 1.0f64;
    let mut r = 0.0f64;
    let mut i = val;
    for _ in 0..40 {
        let m = (i > 0) as u64;
        f /= 3.0;
        r += (f * (i % 3) as f64) * m as f64;
        i /= 3;
    }
    (r * u64::MAX as f64) as u64
}

fn ref_normalize_slice_branchless(val: u64, aux: u64) -> u64 {
    let scale = (aux & 0xFF) | ((aux & 0xFF) == 0) as u64;
    let mut res = 0u64;
    for i in 0..8 {
        let b = ((val >> (i * 8)) & 0xFF) / scale;
        res |= (b & 0xFF) << (i * 8);
    }
    res
}

fn ref_locality_sensitive_hash_cosine(val: u64, aux: u64) -> u64 {
    let mut dot = 0i64;
    for i in 0..8 {
        let a = (((val >> (i * 8)) & 0xFF) as i8) as i64;
        let b = (((aux >> (i * 8)) & 0xFF) as i8) as i64;
        dot += a * b;
    }
    (dot > 0) as u64
}

fn ref_linear_search_simd_u8(val: u64, aux: u64) -> u64 {
    // The branchless implementation scans from the high byte (pos=7) to the
    // low byte (pos=0), overwriting result on every match. The last match wins,
    // so the result is the lowest byte position that equals target (or 8 if none).
    let target = aux & 0xFF;
    let mut result = 8u64;
    for pos in (0..8u64).rev() {
        let b = (val >> (pos * 8)) & 0xFF;
        if b == target {
            result = pos; // keeps updating; final value = smallest matching pos
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Deterministic test-case generator (LCG seeded with bcinr magic constant)
// ---------------------------------------------------------------------------

const SPEC_SEED: u64 = 0xBC1_1_2_BEAD_CAFE;

fn gen_cases(n: usize) -> std::vec::Vec<(u64, u64)> {
    let mut s = SPEC_SEED;
    let mut cases = std::vec::Vec::with_capacity(n + 8);
    for _ in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = s;
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let aux = s;
        cases.push((val, aux));
    }
    // Always include boundary cases
    cases.extend_from_slice(&[
        (0, 0),
        (u64::MAX, u64::MAX),
        (u64::MAX, 0),
        (0, u64::MAX),
        (1, 1),
        (42, 1337),
    ]);
    cases
}

// ---------------------------------------------------------------------------
// Theorem check runner
// ---------------------------------------------------------------------------

pub fn run_all_theorem_checks() -> std::vec::Vec<TheoremResult> {
    let cases = gen_cases(200);
    let mut results = std::vec::Vec::new();

    macro_rules! check {
        ($name:expr, $impl_fn:expr, $ref_fn:expr) => {{
            let mut pass = true;
            let mut first_failure: Option<(u64, u64)> = None;
            let mut tested = 0u64;
            for &(val, aux) in &cases {
                let got = $impl_fn(val, aux);
                let expected = $ref_fn(val, aux);
                tested += 1;
                if got != expected && first_failure.is_none() {
                    pass = false;
                    first_failure = Some((val, aux));
                }
            }
            results.push(TheoremResult {
                algorithm: $name,
                pass,
                cases: tested,
                first_failure,
            });
        }};
    }

    check!("gcd_u64_branchless", gcd_u64_branchless, ref_gcd);
    check!("lcm_u64_branchless", lcm_u64_branchless, ref_lcm);
    check!(
        "norm_u32",
        |v: u64, _a: u64| norm_u32(v, 0),
        |v: u64, _a: u64| ref_norm_u32(v)
    );
    check!(
        "fp_sqrt_u32_q16",
        |v: u64, _a: u64| fp_sqrt_u32_q16(v, 0),
        |v: u64, _a: u64| ref_fp_sqrt_u32_q16(v)
    );
    check!(
        "count_consecutive_set_bits_u64",
        |v: u64, _a: u64| count_consecutive_set_bits_u64(v, 0),
        |v: u64, _a: u64| ref_count_consecutive_set_bits_u64(v)
    );
    check!(
        "parallel_bits_extract_u64",
        parallel_bits_extract_u64,
        ref_parallel_bits_extract_u64
    );
    check!(
        "parallel_bits_deposit_u64",
        parallel_bits_deposit_u64,
        ref_parallel_bits_deposit_u64
    );
    check!("gather_bits_u64", gather_bits_u64, ref_gather_bits_u64);
    check!(
        "scatter_bits_u64",
        scatter_bits_u64,
        |v: u64, a: u64| ref_parallel_bits_deposit_u64(v, a)
    );
    check!("rank_u128", rank_u128, ref_rank_u128);
    check!(
        "halton_sequence_u32",
        |v: u64, _a: u64| halton_sequence_u32(v, 0),
        |v: u64, _a: u64| ref_halton_sequence_u32(v)
    );
    check!(
        "normalize_slice_branchless",
        normalize_slice_branchless,
        ref_normalize_slice_branchless
    );
    check!(
        "locality_sensitive_hash_cosine",
        locality_sensitive_hash_cosine,
        ref_locality_sensitive_hash_cosine
    );
    check!(
        "linear_search_simd_u8",
        linear_search_simd_u8,
        ref_linear_search_simd_u8
    );

    results
}

// ---------------------------------------------------------------------------
// BLAKE3 conformance receipt
// ---------------------------------------------------------------------------

pub fn conformance_receipt(results: &[TheoremResult]) -> std::string::String {
    let mut hasher = blake3::Hasher::new();
    for r in results {
        hasher.update(r.algorithm.as_bytes());
        hasher.update(&[r.pass as u8]);
        hasher.update(&r.cases.to_le_bytes());
        let (fv, fa) = r.first_failure.unwrap_or((0, 0));
        hasher.update(&fv.to_le_bytes());
        hasher.update(&fa.to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

// ---------------------------------------------------------------------------
// Integration test
// ---------------------------------------------------------------------------

#[test]
fn all_algorithms_pass_spec_verify() {
    let results = run_all_theorem_checks();
    let receipt = conformance_receipt(&results);

    let failures: std::vec::Vec<_> = results.iter().filter(|r| !r.pass).collect();

    std::println!("\n=== Innovation 5: Machine-Verified Algorithm Suite ===");
    std::println!("Algorithms tested : {}", results.len());
    std::println!(
        "Algorithms passing: {}",
        results.iter().filter(|r| r.pass).count()
    );
    std::println!("Algorithms failing: {}", failures.len());
    std::println!("Conformance receipt (BLAKE3): {receipt}");
    for r in &results {
        let status = if r.pass { "PASS" } else { "FAIL" };
        std::println!("  [{status}] {} ({} cases)", r.algorithm, r.cases);
        if let Some((v, a)) = r.first_failure {
            std::println!(
                "         first failure: val=0x{v:016x} aux=0x{a:016x}"
            );
        }
    }

    assert!(
        failures.is_empty(),
        "Algorithm theorem failures:\n{:?}\nConformance receipt: {receipt}",
        failures
    );
}
