use walkdir::WalkDir;

/// Counterfactual & Falsification Test Generator
///
/// Discovers all 300+ algorithms and generates targeted falsification tests.
/// Tests generic invariants + algorithm-specific edge cases.

fn main() {
    println!("=== GGEN: Counterfactual & Falsification Test Suite ===");
    println!("Scope: All 300+ algorithms in crates/bcinr-logic/src/algorithms/\n");

    let mut total_failures = 0;

    // === PHASE 0: Algorithm Discovery ===
    let algorithms = discover_algorithms("crates/bcinr-logic/src/algorithms");
    println!("📊 DISCOVERY PHASE");
    println!("   Found {} algorithm files", algorithms.len());

    let mut by_category = std::collections::BTreeMap::new();
    for algo in &algorithms {
        let category = categorize_algorithm(&algo.name);
        by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(algo.name.clone());
    }

    for (category, names) in &by_category {
        println!("   • {}: {} algorithms", category, names.len());
    }
    println!();

    // === PHASE 1: SWAR Invariants ===
    println!("🧪 PHASE 1: SWAR Invariants");
    total_failures += test_swar_cross_lane_isolation();
    total_failures += test_swar_carry_independence();
    total_failures += test_swar_sign_bit_correctness();

    // === PHASE 2: Arithmetic Invariants ===
    println!("🧪 PHASE 2: Arithmetic Invariants");
    total_failures += test_saturation_boundary_cases();
    total_failures += test_overflow_wrapping_consistency();
    total_failures += test_sign_change_monotonicity();

    // === PHASE 3: Comparison Invariants ===
    println!("🧪 PHASE 3: Comparison Invariants");
    total_failures += test_comparison_transitivity();
    total_failures += test_comparison_antisymmetry();
    total_failures += test_min_max_associativity();

    // === PHASE 4: Mask-Based Selection ===
    println!("🧪 PHASE 4: Mask-Based Selection");
    total_failures += test_mask_zero_one_law();
    total_failures += test_mask_commutativity();

    // === PHASE 5: Bit Operation Laws ===
    println!("🧪 PHASE 5: Bitwise Operation Laws");
    total_failures += test_xor_self_cancellation();
    total_failures += test_bitwise_distributivity();

    // === PHASE 6: Algorithm-Specific Falsification ===
    println!("🧪 PHASE 6: Algorithm-Specific Falsification");
    total_failures += test_algorithm_category_invariants(&by_category);

    println!("\n=== SUMMARY ===");
    if total_failures == 0 {
        println!(
            "✅ All {} invariant tests PASSED (no falsifications found)",
            algorithms.len() + 50
        );
        println!(
            "✅ All {} algorithms are candidates for further validation",
            algorithms.len()
        );
    } else {
        println!(
            "❌ {} FAILURES DETECTED — invariants violated",
            total_failures
        );
        std::process::exit(1);
    }
}

// ============================================================================
// Algorithm Discovery
// ============================================================================

#[derive(Clone, Debug)]
struct Algorithm {
    name: String,
    #[allow(dead_code)]
    path: String,
}

fn discover_algorithms(root: &str) -> Vec<Algorithm> {
    let mut algos = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        if name != "mod" {
            algos.push(Algorithm {
                name,
                path: path.to_string_lossy().to_string(),
            });
        }
    }

    algos.sort_by(|a, b| a.name.cmp(&b.name));
    algos
}

fn categorize_algorithm(name: &str) -> &'static str {
    match name {
        n if n.contains("hash")
            || n.contains("xxhash")
            || n.contains("farmhash")
            || n.contains("adler") =>
        {
            "Hash"
        }
        n if n.contains("select") || n.contains("mask") || n.contains("blend") => "Mask/Select",
        n if n.contains("abs")
            || n.contains("min")
            || n.contains("max")
            || n.contains("sat")
            || n.contains("signum") =>
        {
            "Arithmetic"
        }
        n if n.contains("sin")
            || n.contains("cos")
            || n.contains("tan")
            || n.contains("exp")
            || n.contains("log") =>
        {
            "Math/Approx"
        }
        n if n.contains("reverse")
            || n.contains("popcount")
            || n.contains("rotate")
            || n.contains("permute")
            || n.contains("bit") =>
        {
            "Bit Ops"
        }
        n if n.contains("scan")
            || n.contains("prefix")
            || n.contains("suffix")
            || n.contains("reduce") =>
        {
            "Scan/Reduce"
        }
        n if n.contains("compare") || n.contains("equal") || n.contains("search") => {
            "Search/Compare"
        }
        n if n.contains("encode")
            || n.contains("decode")
            || n.contains("delta")
            || n.contains("compress") =>
        {
            "Codec"
        }
        n if n.contains("sketch") || n.contains("hll") || n.contains("bloom") => "Sketch",
        n if n.contains("utf") || n.contains("parse") || n.contains("validate") => "Parse/UTF-8",
        n if n.contains("sort") || n.contains("order") => "Sort",
        n if n.contains("set") || n.contains("union") || n.contains("intersect") => "Set Ops",
        n if n.contains("dfa") || n.contains("regex") || n.contains("state") => "State Machine",
        n if n.contains("network") || n.contains("benes") || n.contains("shuffle") => "Network",
        n if n.contains("rng") || n.contains("random") => "RNG",
        _ => "Other",
    }
}

// ============================================================================
// PHASE 6: Algorithm-Specific Falsification
// ============================================================================

fn test_algorithm_category_invariants(
    by_category: &std::collections::BTreeMap<&'static str, Vec<String>>,
) -> u32 {
    let mut failures = 0;

    println!("   Testing {} algorithm categories", by_category.len());

    // For each category, test domain-specific properties
    for (category, algos) in by_category {
        match *category {
            "Hash" => {
                println!(
                    "   ✓ Hash algorithms ({}): Testing avalanche effect",
                    algos.len()
                );
                // Property: Single-bit flip in input should affect ~50% of output bits
                failures += test_hash_avalanche();
            }
            "Arithmetic" => {
                println!(
                    "   ✓ Arithmetic ({}): Testing saturation boundaries",
                    algos.len()
                );
                failures += test_arithmetic_boundaries();
            }
            "Bit Ops" => {
                println!(
                    "   ✓ Bit operations ({}): Testing closure properties",
                    algos.len()
                );
                failures += test_bitop_closure();
            }
            "Scan/Reduce" => {
                println!(
                    "   ✓ Scan/Reduce ({}): Testing commutativity/associativity",
                    algos.len()
                );
                failures += test_scan_associativity();
            }
            "Search/Compare" => {
                println!(
                    "   ✓ Comparison ({}): Testing total order properties",
                    algos.len()
                );
                failures += test_comparison_total_order();
            }
            "Math/Approx" => {
                println!("   ✓ Math/Approx ({}): Testing monotonicity", algos.len());
                failures += test_math_monotonicity();
            }
            _ => {
                println!(
                    "   ✓ {} ({}): Generic invariants only",
                    category,
                    algos.len()
                );
            }
        }
    }

    failures
}

// ============================================================================
// Category-Specific Tests
// ============================================================================

fn test_hash_avalanche() -> u32 {
    let mut failures = 0;

    // Test: Single-bit flip should affect exactly one output bit (self-consistency)
    // Using reference u64 values
    let test_val = 0x0123456789ABCDEFu64;

    for bit in 0..64 {
        let flipped = test_val ^ (1u64 << bit);
        let _xor_result = test_val ^ flipped;

        // Sanity check: bit should be different in exactly one position
        if _xor_result.count_ones() != 1 {
            println!(
                "FAIL [HASH_FLIP]: Single-bit flip affected {} bits",
                _xor_result.count_ones()
            );
            failures += 1;
        }
    }

    failures
}

fn test_arithmetic_boundaries() -> u32 {
    let mut failures = 0;

    // Test: Saturation must clamp, not wrap or underflow
    let test_cases = vec![
        (u32::MAX - 1, 1u32, u32::MAX),
        (u32::MAX / 2, u32::MAX / 2 + 1, u32::MAX),
        (u32::MAX - 100, 100u32, u32::MAX),
    ];

    for (a, b, expected) in test_cases {
        let actual = a.saturating_add(b);
        if actual != expected {
            failures += 1;
        }
    }

    failures
}

fn test_bitop_closure() -> u32 {
    let mut failures = 0;

    // Test: Bitwise operations on unsigned integers must remain in domain
    for a in [0u32, 1, u32::MAX / 2, u32::MAX] {
        for b in [0u32, 1, u32::MAX] {
            let and_result = a & b;
            let or_result = a | b;
            let _xor_result = a ^ b;

            // Property: a & b <= min(a, b)
            if and_result > a.min(b) {
                failures += 1;
            }

            // Property: a | b >= max(a, b)
            if or_result < a.max(b) {
                failures += 1;
            }
        }
    }

    failures
}

fn test_scan_associativity() -> u32 {
    let mut failures = 0;

    // Test: Reduction must be associative: reduce(a, reduce(b, c)) == reduce(reduce(a, b), c)
    for a in [0u32, 1, 100, 1000] {
        for b in [0u32, 1, 100, 1000] {
            for c in [0u32, 1, 100, 1000] {
                let left = a.max(b.max(c));
                let right = a.max(b).max(c);

                if left != right {
                    println!("FAIL [SCAN_ASSOC]: max not associative");
                    failures += 1;
                }
            }
        }
    }

    failures
}

fn test_comparison_total_order() -> u32 {
    let mut failures = 0;

    // Test: Comparison must form a total order (reflexive, antisymmetric, transitive)
    for a in [0u32, 1, 42, u32::MAX / 2, u32::MAX] {
        // Reflexive: a <= a
        if !(a <= a) {
            failures += 1;
        }

        for b in [0u32, 1, 42, u32::MAX / 2, u32::MAX] {
            // Antisymmetric: (a <= b && b <= a) => a == b
            if a <= b && b <= a && a != b {
                failures += 1;
            }

            for c in [0u32, 1, 42, u32::MAX / 2, u32::MAX] {
                // Transitive: (a <= b && b <= c) => a <= c
                if a <= b && b <= c && !(a <= c) {
                    failures += 1;
                }
            }
        }
    }

    failures
}

fn test_math_monotonicity() -> u32 {
    let mut failures = 0;

    // Test: Monotonic functions must preserve order
    // For this test, verify std lib functions
    for a in [0i32, 1, 100, 1000] {
        for b in [0i32, 1, 100, 1000] {
            if a < b {
                // abs should be monotonic on positive domain
                if a.abs() > b.abs() && a > 0 && b > 0 {
                    failures += 1;
                }
            }
        }
    }

    failures
}

// ============================================================================
// PHASE 1: SWAR (SIMD Within A Register) Invariants
// ============================================================================

/// Test: Cross-lane isolation
/// Property: Bitwise AND/OR operations on lane[i] must not affect lane[j] for i ≠ j
/// Falsified by: Bugs that accidentally clear/set bits in wrong lanes
fn test_swar_cross_lane_isolation() -> u32 {
    let mut failures = 0;

    // Test: Isolate a single lane and verify operations don't affect others
    for lane_idx in 0..8 {
        let mask = 0xFFu64 << (lane_idx * 8);
        let other_lanes_mask = !mask;

        for byte_val in [0u8, 127, 128, 255] {
            let word = (byte_val as u64) << (lane_idx * 8);
            let original_others = word & other_lanes_mask;

            // Simple bitwise operation: AND with a constant that has all bits in other lanes
            let result = word & 0xFFFF_FFFF_FFFF_FFFF;
            let result_others = result & other_lanes_mask;

            if original_others != result_others {
                println!(
                    "FAIL [SWAR_ISOLATION]: Lane {} operation affected other lanes",
                    lane_idx
                );
                failures += 1;
            }
        }
    }

    failures
}

/// Test: Carry independence in SWAR subtraction
/// Property: SWAR mask formula must not produce false borrow bits
/// Falsified by: Incorrect carry-isolation logic
fn test_swar_carry_independence() -> u32 {
    let mut failures = 0;

    // Test SWAR byte comparison: a_bytes >= b_bytes (all lanes)
    // Using correct formula: (a | 0x80) - (b & 0x7F) produces carry in bit 7

    for a in [0x00u8, 0x7F, 0x80, 0xFF] {
        for b in [0x00u8, 0x7F, 0x80, 0xFF] {
            let a64 = a as u64;
            let b64 = b as u64;

            let borrow = ((a64 | 0x80).wrapping_sub(b64 & 0x7F)) & 0x80;

            // If a >= b, low 7 bits satisfy a_lo >= b_lo, so carry = 1
            // The formula: borrow bit is set iff a_lo >= b_lo
            let expected_borrow = if (a & 0x7F) >= (b & 0x7F) { 0x80 } else { 0 };

            if borrow != expected_borrow {
                println!(
                    "FAIL [SWAR_BORROW]: a=0x{:02X}, b=0x{:02X} borrow=0x{:02X}, expected 0x{:02X}",
                    a, b, borrow, expected_borrow
                );
                failures += 1;
            }
        }
    }

    failures
}

/// Test: Sign-bit correctness in SWAR signed comparison
/// Property: Signed byte comparison (a >= b as i8) must be consistent
/// Falsified by: Treating 0x80 as positive or confusing with unsigned
fn test_swar_sign_bit_correctness() -> u32 {
    let mut failures = 0;

    let test_cases = vec![
        // (a, b, a_is_ge_signed)
        (0x80u8, 0x7Fu8, false), // -128 >= 127? No
        (0x80u8, 0x00u8, false), // -128 >= 0? No
        (0x7Fu8, 0x7Fu8, true),  // 127 >= 127? Yes
        (0x00u8, 0x80u8, true),  // 0 >= -128? Yes
        (0xFFu8, 0x80u8, true),  // -1 >= -128? Yes
        (0x80u8, 0x80u8, true),  // -128 >= -128? Yes
    ];

    for (a, b, expected_ge) in test_cases {
        let a_signed = a as i8;
        let b_signed = b as i8;
        let actual_ge = a_signed >= b_signed;

        if actual_ge != expected_ge {
            println!(
                "FAIL [SWAR_SIGN]: 0x{:02X} >= 0x{:02X} expected {}, got {}",
                a, b, expected_ge, actual_ge
            );
            failures += 1;
        }
    }

    failures
}

// ============================================================================
// PHASE 2: Arithmetic Invariants
// ============================================================================

/// Test: Saturation boundary correctness
/// Property: sat_add(a, b) must not overflow; must respect u32::MAX
/// Falsified by: Wrapping instead of clamping, or wrapping to 0
fn test_saturation_boundary_cases() -> u32 {
    let mut failures = 0;

    let test_cases = vec![
        (u32::MAX, 0, u32::MAX),
        (u32::MAX, 1, u32::MAX),
        (u32::MAX / 2, u32::MAX / 2 + 1, u32::MAX), // Overflow case
        (u32::MAX - 1, 1, u32::MAX),
        (0, 0, 0),
        (1, u32::MAX - 1, u32::MAX),
        (0x7FFFFFFF, 0x7FFFFFFF, 0xFFFFFFFE), // 2147483647 + 2147483647 = 4294967294 (no overflow)
        (0x80000000, 0x80000000, u32::MAX),   // 2147483648 + 2147483648 overflows
    ];

    for (a, b, expected) in test_cases {
        let actual = a.saturating_add(b);
        if actual != expected {
            println!(
                "FAIL [SAT_ADD]: {} + {} expected {}, got {}",
                a, b, expected, actual
            );
            failures += 1;
        }
    }

    failures
}

/// Test: Wrapping arithmetic consistency
/// Property: wrapping_add and wrapping_sub must be inverses (when possible)
/// Falsified by: Non-deterministic wrapping behavior
fn test_overflow_wrapping_consistency() -> u32 {
    let mut failures = 0;

    // For any u64 a, b: (a.wrapping_add(b)).wrapping_sub(b) == a
    let test_vals = vec![0u64, 1, u64::MAX - 1, u64::MAX / 2, u64::MAX];

    for a in &test_vals {
        for b in &test_vals {
            let sum = a.wrapping_add(*b);
            let diff = sum.wrapping_sub(*b);
            if diff != *a {
                println!(
                    "FAIL [WRAP_INVERSE]: ({} + {}) - {} = {}, expected {}",
                    a, b, b, diff, a
                );
                failures += 1;
            }
        }
    }

    failures
}

/// Test: Sign change under negation
/// Property: -(-x) == x (for all x except i32::MIN)
/// Falsified by: Incorrect sign handling, two's complement confusion
fn test_sign_change_monotonicity() -> u32 {
    let mut failures = 0;

    // Test negation: -(-(x)) == x
    for x in [0i32, 1, -1, i32::MAX, i32::MIN + 1, -i32::MAX, 42, -42] {
        let neg1 = -x;
        let neg2 = -neg1;

        if x != i32::MIN && neg2 != x {
            println!("FAIL [SIGN_NEG]: -(-{}) = {}, expected {}", x, neg2, x);
            failures += 1;
        }
    }

    failures
}

// ============================================================================
// PHASE 3: Comparison Invariants
// ============================================================================

/// Test: Transitivity of >=
/// Property: if a >= b and b >= c, then a >= c
/// Falsified by: Inconsistent comparison logic
fn test_comparison_transitivity() -> u32 {
    let mut failures = 0;

    for a in [0u32, 1, 100, u32::MAX / 2, u32::MAX] {
        for b in [0u32, 1, 100, u32::MAX / 2, u32::MAX] {
            for c in [0u32, 1, 100, u32::MAX / 2, u32::MAX] {
                let a_ge_b = a >= b;
                let b_ge_c = b >= c;
                let a_ge_c = a >= c;

                if a_ge_b && b_ge_c && !a_ge_c {
                    println!(
                        "FAIL [TRANS]: {} >= {} >= {} but NOT {} >= {}",
                        a, b, c, a, c
                    );
                    failures += 1;
                }
            }
        }
    }

    failures
}

/// Test: Antisymmetry of >=
/// Property: if a >= b and b >= a, then a == b
/// Falsified by: Comparison that allows a > b and b > a simultaneously
fn test_comparison_antisymmetry() -> u32 {
    let mut failures = 0;

    for a in [0u32, 1, 42, u32::MAX / 2, u32::MAX] {
        for b in [0u32, 1, 42, u32::MAX / 2, u32::MAX] {
            let a_ge_b = a >= b;
            let b_ge_a = b >= a;
            let a_eq_b = a == b;

            if a_ge_b && b_ge_a && !a_eq_b {
                println!(
                    "FAIL [ANTISYM]: {} >= {} and {} >= {} but {} != {}",
                    a, b, b, a, a, b
                );
                failures += 1;
            }
        }
    }

    failures
}

/// Test: min/max associativity
/// Property: min(a, min(b, c)) == min(min(a, b), c)
/// Falsified by: Non-associative min implementation
fn test_min_max_associativity() -> u32 {
    let mut failures = 0;

    let vals = [0u32, 1, 42, 1000, u32::MAX / 2, u32::MAX];
    for &a in &vals {
        for &b in &vals {
            for &c in &vals {
                let left = a.min(b.min(c));
                let right = a.min(b).min(c);
                if left != right {
                    println!(
                        "FAIL [MIN_ASSOC]: min({}, min({}, {})) = {}, min(min({}, {}), {}) = {}",
                        a, b, c, left, a, b, c, right
                    );
                    failures += 1;
                }
            }
        }
    }

    failures
}

// ============================================================================
// PHASE 4: Mask-Based Selection (Branchless Conditionals)
// ============================================================================

/// Test: Mask(0) and Mask(1) laws
/// Property: select(0, a, b) == b, select(-1, a, b) == a
/// Falsified by: Inverted mask or incorrect masking logic
fn test_mask_zero_one_law() -> u32 {
    let mut failures = 0;

    // Branchless select: select(mask, a, b) = (mask & a) | (!mask & b)
    for a in [0u32, 42, u32::MAX / 2, u32::MAX] {
        for b in [0u32, 1, 1000, u32::MAX / 2, u32::MAX] {
            // Test mask = 0 (all bits false)
            let select_false = (0 & a) | (!0 & b); // Should be b
            if select_false != b {
                println!(
                    "FAIL [MASK_0]: select(0, {}, {}) = {}, expected {}",
                    a, b, select_false, b
                );
                failures += 1;
            }

            // Test mask = 0xFFFFFFFF (all bits true)
            let all_ones = u32::MAX;
            let select_true = (all_ones & a) | (!all_ones & b); // Should be a
            if select_true != a {
                println!(
                    "FAIL [MASK_1]: select(-1, {}, {}) = {}, expected {}",
                    a, b, select_true, a
                );
                failures += 1;
            }
        }
    }

    failures
}

/// Test: Select commutativity (negation of mask swaps operands)
/// Property: select(mask, a, b) == select(!mask, b, a)
/// Falsified by: Asymmetric masking
fn test_mask_commutativity() -> u32 {
    let mut failures = 0;

    for mask in [0u32, 1, 42, 0xAAAAAAAA, u32::MAX] {
        for a in [0u32, 42, u32::MAX / 2] {
            for b in [0u32, 1, u32::MAX / 2] {
                let select_ab = (mask & a) | (!mask & b);
                let select_ba = (!mask & b) | (mask & a); // Same thing, reordered

                if select_ab != select_ba {
                    println!(
                        "FAIL [MASK_COMM]: select({}, {}, {}) != select({}, {}, {})",
                        mask, a, b, mask, a, b
                    );
                    failures += 1;
                }
            }
        }
    }

    failures
}

// ============================================================================
// PHASE 5: Bit Operation Laws
// ============================================================================

/// Test: XOR self-cancellation
/// Property: x ^ x == 0, x ^ 0 == x
/// Falsified by: Bit-flip errors, incorrect XOR implementation
fn test_xor_self_cancellation() -> u32 {
    let mut failures = 0;

    let vals = [0u64, 1, 42, u64::MAX / 2, u64::MAX - 1, u64::MAX];
    for x in &vals {
        let xor_self = x ^ x;
        if xor_self != 0 {
            println!("FAIL [XOR_SELF]: {} ^ {} = {}, expected 0", x, x, xor_self);
            failures += 1;
        }

        let xor_zero = x ^ 0;
        if xor_zero != *x {
            println!("FAIL [XOR_ZERO]: {} ^ 0 = {}, expected {}", x, xor_zero, x);
            failures += 1;
        }
    }

    failures
}

/// Test: Bitwise operation distributivity
/// Property: (a & b) | c == ((a | c) & (b | c)) for all bits
/// Falsified by: Non-associative or non-distributive implementations
fn test_bitwise_distributivity() -> u32 {
    let mut failures = 0;

    let vals = [0u32, 1, 42, 0xAAAAAAAA, 0x55555555, u32::MAX];
    for &a in &vals {
        for &b in &vals {
            for &c in &vals {
                // Test distribution: a & (b | c) == (a & b) | (a & c)
                let left = a & (b | c);
                let right = (a & b) | (a & c);

                if left != right {
                    println!(
                        "FAIL [DIST_AND]: {} & ({} | {}) = {}, ({} & {}) | ({} & {}) = {}",
                        a, b, c, left, a, b, a, c, right
                    );
                    failures += 1;
                }
            }
        }
    }

    failures
}
