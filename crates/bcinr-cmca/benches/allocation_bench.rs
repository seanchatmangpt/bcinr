//! Divan benchmarks for the CMCA allocation kernel -- the crate's first
//! benchmark suite (the repo's bench-auditor had been truthfully reporting
//! zero coverage). Convention matches `bcinr-powl/benches/*`: Divan,
//! `harness = false`, black-boxed inputs.
//!
//! # What is measured, and honestly not
//!
//! - `allocate_kernel`: one full `allocate()` call over the case-studies
//!   fixture (N=8, K=4, Q=4, fully unrolled kernel), including the cheap
//!   `Copy` of the weights array in (state initialization is part of the
//!   real call contract -- `allocate` mutates caller state).
//! - `allocate_single_lens_query`: one stateless single-lens query
//!   (rebuilds its topology tables per call -- that is the API's
//!   documented cost shape).
//! - `escort_exact_integer_q` / `escort_fractional_q` (require `--features
//!   std|alloc`): the escort distribution on the exact repeated-multiply
//!   path vs the approximate log2/exp2 path.
//! - `power_fractional`: the underlying `allocator::power` primitive.
//!
//! Numbers are receipts, not claims: see `docs/BENCH_BASELINE_v26.9.15.md`
//! for the recorded environment and reproduction command.

use bcinr_cmca::allocator::{allocate, allocate_single_lens, power};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
use divan::black_box;

fn main() {
    divan::main();
}

fn flat_parent() -> [i32; N] {
    [-1; N]
}

#[divan::bench]
fn allocate_kernel() {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let parent = flat_parent();
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let result = allocate(
        black_box(&OBJECT_REGISTRY),
        black_box(&LENS_REGISTRY),
        black_box(&LAMBDA),
        black_box(ETA),
        black_box(&parent),
        &mut weights,
        black_box(&payoffs),
        black_box(NonNegativeFixed::ZERO),
        black_box(NonNegativeFixed::ZERO),
        black_box(&mu),
        black_box(&costs),
        black_box(0),
        &mut last_switch_t,
        &mut prev_mode,
        black_box(500),
        black_box(CERTIFICATE_DIGEST),
        black_box(None),
    );
    let _ = black_box(result);
}

#[divan::bench]
fn allocate_single_lens_query() {
    let weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let parent = flat_parent();
    let result = allocate_single_lens(
        black_box(&OBJECT_REGISTRY),
        black_box(&LENS_REGISTRY),
        black_box(0),
        black_box(0),
        black_box(&parent),
        black_box(&weights),
    );
    let _ = black_box(result);
}

#[divan::bench]
fn power_fractional() {
    let out = power(
        black_box(NonNegativeFixed::from_bits(2 << 16)),
        black_box(SignedFixed::from_bits(1 << 15)), // q = 0.5
    );
    let _ = black_box(out);
}

// The escort paths live behind the `alloc` feature (Vec-returning API);
// without it these benchmarks compile out rather than silently vanishing
// from a default-feature run -- run `cargo bench --features std` for the
// full suite.
/// PROFILER LANE decomposition benches (perf wave 26.9.15): price the
/// primitives `allocate`'s unrolled kernel executes thousands of times per
/// call, so the sampling profile's per-symbol split (flow_step /
/// compute_pi_kq_for_kq / compute_kappa / allocate_in) can be attributed to
/// primitive costs x static call counts. Operand shapes below mirror the
/// live values under the `allocate_kernel` fixture (flat parent, all-ONE
/// weights: leaf divisions are 1/1, child divisions are 0/1, the MWU
/// normalization is w/2); `saturating_div`'s Newton-Raphson path has a
/// fixed instruction sequence independent of operand values (no
/// data-dependent branches by construction), so one shape per call-site
/// class is sufficient to price it. A second, different-magnitude shape is
/// included per primitive as a value-invariance cross-check.
mod decomposition {
    use super::*;

    const ONE: NonNegativeFixed = NonNegativeFixed {
        val: 65536,
        err: u32::MAX,
    };
    const HALF: NonNegativeFixed = NonNegativeFixed {
        val: 32768,
        err: u32::MAX,
    };
    const EIGHTH: NonNegativeFixed = NonNegativeFixed {
        val: 8192,
        err: u32::MAX,
    };
    const THREE: NonNegativeFixed = NonNegativeFixed {
        val: 196608,
        err: u32::MAX,
    };

    // --- NonNegativeFixed::saturating_div (3x i128 Newton-Raphson reciprocal) ---
    // flow_step leaf shape: leaf_w / lw_sum == 1/1 under the flat fixture.
    #[divan::bench]
    fn nn_div_one_over_one() {
        let out = black_box(ONE).saturating_div(black_box(ONE));
        let _ = black_box(out);
    }
    // flow_step child shape: child_w / cw_denom == 0/1 under the flat fixture.
    #[divan::bench]
    fn nn_div_zero_over_one() {
        let out = black_box(NonNegativeFixed::ZERO).saturating_div(black_box(ONE));
        let _ = black_box(out);
    }
    // MWU normalization shape: w / (w + w') == 1/2.
    #[divan::bench]
    fn nn_div_one_over_two() {
        let out = black_box(ONE).saturating_div(black_box(NonNegativeFixed {
            val: 131072,
            err: u32::MAX,
        }));
        let _ = black_box(out);
    }
    // kappa s_meas shape: numerator > denominator (ratio up to MAX).
    #[divan::bench]
    fn nn_div_one_over_eighth() {
        let out = black_box(ONE).saturating_div(black_box(EIGHTH));
        let _ = black_box(out);
    }

    // --- NonNegativeFixed::saturating_mul (single u64 mul + overflow select) ---
    #[divan::bench]
    fn nn_mul_half_half() {
        let out = black_box(HALF).saturating_mul(black_box(HALF));
        let _ = black_box(out);
    }
    #[divan::bench]
    fn nn_mul_three_half() {
        let out = black_box(THREE).saturating_mul(black_box(HALF));
        let _ = black_box(out);
    }

    // --- NonNegativeFixed::saturating_add ---
    #[divan::bench]
    fn nn_add_half_half() {
        let out = black_box(HALF).saturating_add(black_box(HALF));
        let _ = black_box(out);
    }

    // --- NonNegativeFixed::log2 (lz + mantissa polynomial) ---
    #[divan::bench]
    fn nn_log2_one() {
        let out = black_box(ONE).log2();
        let _ = black_box(out);
    }
    #[divan::bench]
    fn nn_log2_three() {
        let out = black_box(THREE).log2();
        let _ = black_box(out);
    }
    #[divan::bench]
    fn nn_log2_eighth() {
        let out = black_box(EIGHTH).log2();
        let _ = black_box(out);
    }

    // --- SignedFixed::exp2 (degree-4 Horner polynomial + saturating shift) ---
    #[divan::bench]
    fn sf_exp2_zero() {
        let out = black_box(SignedFixed::from_bits(0)).exp2();
        let _ = black_box(out);
    }
    #[divan::bench]
    fn sf_exp2_minus_1_5() {
        let out = black_box(SignedFixed::from_bits(-98304)).exp2();
        let _ = black_box(out);
    }
    #[divan::bench]
    fn sf_exp2_plus_0_5() {
        let out = black_box(SignedFixed::from_bits(32768)).exp2();
        let _ = black_box(out);
    }

    // --- SignedFixed::exp (ln2 prescale + exp2) ---
    #[divan::bench]
    fn sf_exp_zero() {
        let out = black_box(SignedFixed::from_bits(0)).exp();
        let _ = black_box(out);
    }
    #[divan::bench]
    fn sf_exp_minus_0_5() {
        let out = black_box(SignedFixed::from_bits(-32768)).exp();
        let _ = black_box(out);
    }

    // --- Topology stage: ancestor-doubling table + cycle witness ---
    #[divan::bench]
    fn topology_acyclic_flat() {
        let parent = flat_parent();
        let out = bcinr_cmca::allocator::check_hierarchy_acyclic(black_box(&parent));
        let _ = black_box(out);
    }

    // --- SignedFixed::saturating_mul (power()'s exponent product) ---
    #[divan::bench]
    fn sf_mul_half_minus_1() {
        let out = black_box(SignedFixed::from_bits(32768))
            .saturating_mul(black_box(SignedFixed::from_bits(-65536)));
        let _ = black_box(out);
    }

    /// Throughput shapes: the kernel never executes one primitive in
    /// isolation -- `flow_step`'s inner x-loop issues 16 independent
    /// saturating_divs per v (8 leaf + 8 child), so the divider's *latency*
    /// (the single-call benches above) overstates its amortized in-kernel
    /// cost. These benches issue 8 independent ops per iteration and report
    /// per-iteration time; divide by 8 for the throughput price the kernel
    /// actually pays. Operands pass through `black_box` so nothing folds
    /// or CSEs; the eight results are combined only at the end, keeping the
    /// ops independent.
    mod throughput {
        use super::super::*;
        use bcinr_cmca::fixed::NonNegativeFixed;

        const NUMS: [u32; 8] = [65536, 131072, 196608, 262144, 32768, 8192, 16384, 4096];
        const DENS: [u32; 8] = [65536, 32768, 98304, 16384, 229376, 262144, 131072, 524288];

        fn fx(v: u32) -> NonNegativeFixed {
            NonNegativeFixed::from_bits(v)
        }

        #[divan::bench]
        fn nn_div_indep8() {
            let nums = black_box(NUMS);
            let dens = black_box(DENS);
            let r0 = fx(nums[0]).saturating_div(fx(dens[0]));
            let r1 = fx(nums[1]).saturating_div(fx(dens[1]));
            let r2 = fx(nums[2]).saturating_div(fx(dens[2]));
            let r3 = fx(nums[3]).saturating_div(fx(dens[3]));
            let r4 = fx(nums[4]).saturating_div(fx(dens[4]));
            let r5 = fx(nums[5]).saturating_div(fx(dens[5]));
            let r6 = fx(nums[6]).saturating_div(fx(dens[6]));
            let r7 = fx(nums[7]).saturating_div(fx(dens[7]));
            let acc = r0.val ^ r1.val ^ r2.val ^ r3.val ^ r4.val ^ r5.val ^ r6.val ^ r7.val;
            let _ = black_box(acc);
        }

        #[divan::bench]
        fn nn_mul_indep8() {
            let nums = black_box(NUMS);
            let dens = black_box(DENS);
            let r0 = fx(nums[0]).saturating_mul(fx(dens[0]));
            let r1 = fx(nums[1]).saturating_mul(fx(dens[1]));
            let r2 = fx(nums[2]).saturating_mul(fx(dens[2]));
            let r3 = fx(nums[3]).saturating_mul(fx(dens[3]));
            let r4 = fx(nums[4]).saturating_mul(fx(dens[4]));
            let r5 = fx(nums[5]).saturating_mul(fx(dens[5]));
            let r6 = fx(nums[6]).saturating_mul(fx(dens[6]));
            let r7 = fx(nums[7]).saturating_mul(fx(dens[7]));
            let acc = r0.val ^ r1.val ^ r2.val ^ r3.val ^ r4.val ^ r5.val ^ r6.val ^ r7.val;
            let _ = black_box(acc);
        }

        #[divan::bench]
        fn nn_log2_indep8() {
            let nums = black_box(NUMS);
            let r0 = fx(nums[0]).log2();
            let r1 = fx(nums[1]).log2();
            let r2 = fx(nums[2]).log2();
            let r3 = fx(nums[3]).log2();
            let r4 = fx(nums[4]).log2();
            let r5 = fx(nums[5]).log2();
            let r6 = fx(nums[6]).log2();
            let r7 = fx(nums[7]).log2();
            let acc = r0.val ^ r1.val ^ r2.val ^ r3.val ^ r4.val ^ r5.val ^ r6.val ^ r7.val;
            let _ = black_box(acc);
        }

        #[divan::bench]
        fn sf_exp2_indep8() {
            let nums = black_box(NUMS);
            let g =
                |i: usize| bcinr_cmca::fixed::SignedFixed::from_bits(nums[i] as i32 - 65536).exp2();
            let r0 = g(0);
            let r1 = g(1);
            let r2 = g(2);
            let r3 = g(3);
            let r4 = g(4);
            let r5 = g(5);
            let r6 = g(6);
            let r7 = g(7);
            let acc = r0.val ^ r1.val ^ r2.val ^ r3.val ^ r4.val ^ r5.val ^ r6.val ^ r7.val;
            let _ = black_box(acc);
        }
    }
}

#[cfg(feature = "alloc")]
mod escort {
    use super::*;

    const MASSES: [NonNegativeFixed; 8] = [
        NonNegativeFixed::from_bits(1 << 16),
        NonNegativeFixed::from_bits(2 << 16),
        NonNegativeFixed::from_bits(3 << 16),
        NonNegativeFixed::from_bits(4 << 16),
        NonNegativeFixed::from_bits(5 << 16),
        NonNegativeFixed::from_bits(6 << 16),
        NonNegativeFixed::from_bits(7 << 16),
        NonNegativeFixed::from_bits(8 << 16),
    ];

    #[divan::bench]
    fn escort_exact_integer_q() {
        let out = bcinr_cmca::escort::escort_distribution(
            black_box(&MASSES),
            black_box(SignedFixed::from_num(3i32)),
        );
        let _ = black_box(out);
    }

    #[divan::bench]
    fn escort_fractional_q() {
        let out = bcinr_cmca::escort::escort_distribution(
            black_box(&MASSES),
            black_box(SignedFixed::from_bits(1 << 15)), // q = 0.5
        );
        let _ = black_box(out);
    }
}
