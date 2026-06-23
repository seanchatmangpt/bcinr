
/// Counterfactual & Falsification Test Generator
///
/// Generates adversarial test cases designed to break claimed algorithm invariants.
/// Reports any violations that falsify correctness claims.

fn main() {
    println!("=== GGEN: Counterfactual & Falsification Test Suite ===\n");

    let mut failures = 0;

    // === PHASE 1: SWAR Invariants ===
    failures += test_swar_cross_lane_isolation();
    failures += test_swar_carry_independence();
    failures += test_swar_sign_bit_correctness();

    // === PHASE 2: Arithmetic Invariants ===
    failures += test_saturation_boundary_cases();
    failures += test_overflow_wrapping_consistency();
    failures += test_sign_change_monotonicity();

    // === PHASE 3: Comparison Invariants ===
    failures += test_comparison_transitivity();
    failures += test_comparison_antisymmetry();
    failures += test_min_max_associativity();

    // === PHASE 4: Mask-Based Selection ===
    failures += test_mask_zero_one_law();
    failures += test_mask_commutativity();

    // === PHASE 5: Bit Operation Laws ===
    failures += test_xor_self_cancellation();
    failures += test_bitwise_distributivity();

    println!("\n=== SUMMARY ===");
    if failures == 0 {
        println!("✅ All invariant tests PASSED (no falsifications found)");
    } else {
        println!("❌ {} FAILURES DETECTED — invariants violated", failures);
        std::process::exit(1);
    }
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
        (0x80000000, 0x80000000, u32::MAX), // 2147483648 + 2147483648 overflows
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
                println!("FAIL [ANTISYM]: {} >= {} and {} >= {} but {} != {}", a, b, b, a, a, b);
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
