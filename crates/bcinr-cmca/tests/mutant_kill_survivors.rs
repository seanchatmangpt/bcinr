//! Hostile kill tests for the mutants that survived every pre-existing fixture
//! (gate run 2026-10-01, branch release/26.9.15): `mutant_3` and `mutant_4`,
//! plus the replacement mutation occupying `mutant_5`'s slot after its
//! original clip-drop mutation was retired as equivalent-by-design (CMCA-122).
//!
//! # Mutant ledger (AGENTS.md §19 fields)
//!
//! | mutant id | source file | changed law | exact mutation | expected detection | actual detection (observed 2026-10-01) | test name | standing |
//! |---|---|---|---|---|---|---|---|
//! | mutant_3 | crates/bcinr-cmca/src/allocator/mod.rs:1219 (pre-edit numbering) | flow conservation inside `flow_step`: a node's flat emission `sum_x flat_part*leaf_w[x]/lw_sum[v]` must equal `flat_part` because `sum_x leaf_w == lw_sum` | forces the leaf-weight denominator to `ONE` (`lw_denom = 1`), dropping the `select(lw_sum==0, 1, lw_sum)` guard, so every internal node over-emits its flat part by a factor `lw_sum[v]` | exact-value assert on the leaf distribution of a depth-2 tree with uniform subtree-leaf softmax weights (leaf3 = 4/15, leaves 4..7 = 11/60 of flow): under the mutant leaf3 reads ~13500 bits vs expected 15291 | normal build: expected 15291 matched the hand-derived model (PASS); `--features=mutant_3,std`: leaf 3 expected 15291, got 13451, delta 1840 bits (FAIL = KILLED) | `kill_mutant_3_leaf_weight_denominator_is_lw_sum_not_one` | KILLED |
//! | mutant_4 | crates/bcinr-cmca/src/allocator/mod.rs:2162 (pre-edit numbering) | admitted-parameter law of the explore/priced leaf blend: `val = eta*(1/nl) + (1-eta)*p_mu` must use the caller's admitted `eta`, not `zeta` | `eta_actual = zeta` instead of `eta_actual = eta` | differential over two admitted etas (1/4 and 3/4, same zeta=0): outputs must differ and each must match its own eta's expected blend (reconstructed independently through `allocate_single_lens`, which mutant_4 does not touch) | normal build: A/B differ, both match expected within 8 bits (PASS); `--features=mutant_4,std`: both calls collapse to bare `p_mu` (zeta=0): eta=1/4 call's leaf 3 expected 16722, got 17929, delta 1207 bits (FAIL = KILLED) | `kill_mutant_4_leaf_blend_uses_admitted_eta_not_zeta` | KILLED |
//! | mutant_5 (original) | crates/bcinr-cmca/src/allocator/mod.rs:2122,2152 (pre-edit numbering) | defensive `clip(mu, 0, mu_max)` on `mu_actual` | removes the clip (`mu_actual = mu[x]`) | none possible | RETIRED: per CMCA-122 (code comment, both sites), `price_err` (mu > mu_max) refuses unconditionally before the result is observable on any `Ok` path, so the clip is provably non-identity only on already-refused paths -- no admitted-input test can kill it (survived by construction, gate run 2026-10-01; empirically re-confirmed: even this file's hostile pricing test PASSED under the clip-drop build before retirement) | n/a (equivalent mutant) | RETIRED-EQUIVALENT (CMCA-122) |
//! | mutant_5 (replacement, rejected proposal) | would-be: priced_sum accumulation | leaf-mask on `priced_sum += p` | drop `select(is_leaf[x], p, 0)` to `priced_sum += p` | n/a | REFUSED without implementation: statically equivalent. `pi_combined[x] == 0` for every internal node on every reachable state -- `compute_pi_kq_for_kq` returns `res[x] = flat_alloc[x] + alloc_flow[x]` where `flat_alloc` only ever accrues at `is_leaf[x]` slots and an internal node can hold nonzero `alloc_flow` at the end of step 8 only if its root path carries 7 descending (child-index < parent-index) edges, which needs >= 9 nodes. The masked terms are therefore all zero and mask-on/mask-off `priced_sum` are bit-identical: the replacement mutant would have SURVIVED exactly like the one it replaces (the same trap, re-dug). | n/a (never injected) | REFUSED (statically equivalent; failed edge recorded per AGENTS.md 柵) |
//! | mutant_5 (replacement, adopted) | crates/bcinr-cmca/src/allocator/mod.rs (cfg sites at the former 2122/2152) | pricing law `p = pi_combined * exp(-mu*costs)` must use the admitted price vector `mu`, not the region ceiling | `mu_actual = mu_max` (both cfg sites) -- the price fed to the softmax is fabricated from the ceiling instead of the admitted `mu` | exact-value assert with `mu = 0`, `costs = 1`: normal path computes `exp(0) == ONE` exactly, so leaves follow the documented blend `eta*(1/nl) + (1-eta)*p_mu`; under the mutant every price underflows (`exp(-100) == 0` exactly), `priced_sum == 0`, and every leaf collapses to the bare explore floor `eta*(1/nl)` | normal build: PASS (deltas within the 8-bit house tolerance); `--features=mutant_5,std`: leaf 3 expected 15517, got 6553 (exactly eta*nl_recip), delta 8964 bits (FAIL = KILLED) | `kill_mutant_5_priced_softmax_uses_admitted_mu_not_ceiling` | KILLED |
//!
//! The `lib.rs` doc-comment list "mutant_1..mutant_11" stays valid: the
//! feature name `mutant_5` is unchanged, only the mutation it injects.
//!
//! All three tests drive the public `allocate_in` entry point with
//! `FeasibleRegion::CURRENT` (the region `allocate()` itself passes) and use
//! exact-value assertions (exact bits, exact orderings, or bounded deltas
//! against an independently hand-derived rational model) -- never bare
//! `is_err()`/`assert_ne!` (AGENTS.md §18/§19).

// Exercises the CMCA-102/CMCA-114 authority chain on purpose: the proof
// parameter's type carries deprecation marks that mean "unverified for
// production use", not "unavailable to tests" (same as every allocator test).
#![allow(deprecated)]

use bcinr_cmca::allocator::{
    allocate_in, allocate_single_lens, AdaptiveUpdate, AdmittedControlState, CertificateReceipt,
    CertifiedLearning, EnvelopeReceipt, FeasibleRegion, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::consequence_mass::case_studies::{
    LensSpec, PackedSemanticState, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::{CERTIFICATE_DIGEST, MODE_DWELL_ROUNDS_MIN};

const K: usize = 4;

/// Depth-2 tree (same shape as `single_lens_allocation.rs`'s
/// `depth_two_tree_parent`): node 0 is the root; nodes 1 and 2 are internal
/// children of 0 (each with two leaf children of their own); node 3 is a
/// direct leaf child of 0.
///
///   0 -- 1 -- 4
///     \    \- 5
///      \-2 -- 6
///      |   \- 7
///      \-3 (leaf)
fn depth_two_tree_parent() -> [i32; N] {
    let mut parent = [-1i32; N];
    parent[1] = 0;
    parent[2] = 0;
    parent[3] = 0;
    parent[4] = 1;
    parent[5] = 1;
    parent[6] = 2;
    parent[7] = 2;
    parent
}

/// Leaves of `depth_two_tree_parent` in `pi_res` order, and their count.
const LEAVES: [usize; 5] = [3, 4, 5, 6, 7];
const NL_RECIP_BITS: u32 = 13107; // LEAF_RECIP[5] == 1/5, allocator/mod.rs's own table

fn flat_weights() -> [[NonNegativeFixed; 2 * Q]; N] {
    [[NonNegativeFixed::ONE; 2 * Q]; N]
}

fn get_proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
    AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        CertifiedLearning::admit_learning(),
    )
}

/// Runs one admitted `allocate_in` call over `depth_two_tree_parent` with the
/// given eta/zeta and zero payoffs/mu/costs, returning the allocation vector.
#[allow(clippy::too_many_arguments)]
fn run_allocate(
    states: &[PackedSemanticState; N],
    mu: &[NonNegativeFixed; N],
    costs: &[NonNegativeFixed; N],
    eta: NonNegativeFixed,
    zeta: NonNegativeFixed,
) -> Result<[NonNegativeFixed; N], bcinr_cmca::allocator::StabilityRefusal> {
    let lenses: &[LensSpec; Q] = &LENS_REGISTRY;
    let lambda = run_lambda();
    let mut weights = flat_weights();
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    allocate_in(
        &FeasibleRegion::CURRENT,
        states,
        lenses,
        &lambda,
        eta,
        &depth_two_tree_parent(),
        &mut weights,
        &payoffs,
        zeta,
        NonNegativeFixed::ZERO, // epsilon_kappa
        mu,
        costs,
        0, // t
        &mut last_switch_t,
        &mut prev_mode,
        MODE_DWELL_ROUNDS_MIN,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    )
}

/// The blend weights `run_allocate` hands to `allocate_in`: all measure/lens
/// mass on `(k, q) = (0, 0)`, none elsewhere. Zero weights elsewhere keep the
/// expected side free of LAMBDA rounding, and pin the reconstruction to the
/// exact same single kernel evaluation the actual run blends.
fn run_lambda() -> [[NonNegativeFixed; Q]; K] {
    let mut lambda = [[NonNegativeFixed::ZERO; Q]; K];
    lambda[0][0] = NonNegativeFixed::ONE;
    lambda
}

/// Reconstructs `pi_combined` -- the measure/lens-blended allocation
/// `allocate_in` builds internally before mu/costs/eta pricing -- through the
/// independent `allocate_single_lens` entry point (the documented house
/// idiom from `single_lens_allocation.rs`), weighted by `run_lambda` so the
/// reconstruction blends exactly what the actual run blends. Valid here
/// because every test in this file passes all-zero payoffs: with
/// `exp(beta*0) == ONE` the MWU multiplicative step is a no-op and pre-/post-
/// call weights are related by a scale-invariant renormalization, and
/// `compute_pi_kq_for_kq` depends on `local_weights` only through the
/// scale-invariant ratio `rho = w_desc/(w_flat + w_desc)`.
fn reconstruct_pi_combined(
    states: &[PackedSemanticState; N],
    parent: &[i32; N],
) -> [NonNegativeFixed; N] {
    let weights = flat_weights();
    let lambda = run_lambda();
    let mut pi_combined = [NonNegativeFixed::ZERO; N];
    for (k, lambda_k) in lambda.iter().enumerate() {
        for (q_idx, lambda_kq) in lambda_k.iter().enumerate() {
            let single = allocate_single_lens(states, &LENS_REGISTRY, k, q_idx, parent, &weights)
                .unwrap_or_else(|e| panic!("allocate_single_lens({k}, {q_idx}) refused: {e:?}"));
            for (pi, single_x) in pi_combined.iter_mut().zip(single.iter()) {
                *pi += *lambda_kq * *single_x;
            }
        }
    }
    pi_combined
}

/// non-vacuity: guards the `#[cfg(feature = "mutant_3")] let lw_denom =
/// NonNegativeFixed::ONE.val;` site at allocator/mod.rs:1219 (pre-edit
/// numbering) -- the exact mutation "leaf-weight denominator forced to ONE,
/// `select(lw_sum==0, 1, lw_sum)` guard dropped".
///
/// Why every earlier fixture missed it: with *normalized* caller weights the
/// division by 1 is invisible, and the one prior tree-shaped test reconstructs
/// its expected values through `allocate_single_lens`, which runs the SAME
/// `flow_step` kernel -- under the mutant both sides mutate identically (the
/// self-reference trap). This test closes both holes at once:
///
/// 1. The states are crafted so every node's four measures are equal, which
///    makes the softmax leaf/child weights inside `compute_pi_kq_for_kq`
///    exactly `exp2(0) == ONE` per subtree leaf / child. `lw_sum[v]` is then
///    exactly the subtree-leaf COUNT (root: 5, nodes 1 and 2: 2) -- the
///    maximally unnormalized case -- while remaining exactly hand-computable.
/// 2. The expected leaf distribution is derived from the mathematical
///    conservation contract of `flow_step` alone (each node re-emits exactly
///    what it receives; shares are count ratios), not from any crate kernel:
///    leaf 3 receives 1/5 of the root's flat half plus 1/3 of its desc half
///    (4/15 of flow); leaves 4..7 receive 1/5 flat + node-1/2's halves split
///    over 2 (11/60 each). Asserted as exact bounded deltas (<= 64 bits) in
///    Q16.16 -- under the mutant the root over-emits its flat half by 5x and
///    nodes 1/2 by 2x, moving leaf 3 by ~1791 bits and leaves 4..7 by ~445
///    bits, far outside the tolerance.
///
/// The blend's renormalization is why conservation of the *output* cannot
/// kill this mutant (`sum val == eta + (1-eta) == 1` for ANY pi_combined
/// scale); only the exact relative distribution can, hence the exact-value
/// assertions.
#[test]
fn kill_mutant_3_leaf_weight_denominator_is_lw_sum_not_one() {
    // All ten factors equal => m_cache == m_search == m_retrieval ==
    // m_sched uniformly across nodes (96.0 / 32.0 / 256.0 / 256.0), inside
    // FeasibleRegion::CURRENT's clip bounds, so every softmax group in
    // compute_pi_kq_for_kq has equal exponents and its max-shifted weights
    // are exp2(0) == ONE exactly.
    let uniform = NonNegativeFixed::from_bits(4096); // 1/16
    let states = [PackedSemanticState {
        id: 0,
        factors: [uniform; 10],
    }; N];

    let eta = NonNegativeFixed::from_bits(32768); // 1/2, admitted [ETA_G_MIN, 1]
    let zeta = NonNegativeFixed::ZERO; // admitted: beta = min(0, beta_max) = 0

    let result = run_allocate(&states, &zero_mu(), &zero_costs(), eta, zeta)
        .unwrap_or_else(|e| panic!("allocate_in refused a fully admitted fixture: {e:?}"));

    // Internal nodes carry exactly zero allocation (the blend only ever
    // writes leaf slots) -- holds under mutant and normal alike.
    for (i, share) in result.iter().take(3).enumerate() {
        assert_eq!(
            share.to_bits(),
            0,
            "internal node {i} must carry zero allocation, got {:?}",
            share
        );
    }

    // Hand-derived rational model (independent of every crate kernel):
    //   pi_3 = 1/5*1/2 + 1/3*1/2 = 4/15   (root flat share + root desc share)
    //   pi_4..7 = 1/5*1/2 + (1/6)*1/2 + (1/6)*1/2 = 11/60
    // (nodes 1 and 2 re-emit their 1/6 intake as two 1/12 halves over their
    // two leaf children). The Q16.16 encodings of the rationals:
    let pi3 = NonNegativeFixed::from_bits(4 * 65536 / 15); // 17476
    let pi_leaf = NonNegativeFixed::from_bits(11 * 65536 / 60); // 12014
    let nl_recip = NonNegativeFixed::from_bits(NL_RECIP_BITS);

    // The documented blend with mu = costs = 0: val = eta*(1/nl) + (1-eta)*p_mu,
    // and since the rationals sum to exactly 1, p_mu == pi.
    let expected3 = eta * nl_recip + (NonNegativeFixed::ONE - eta) * pi3;
    let expected_leaf = eta * nl_recip + (NonNegativeFixed::ONE - eta) * pi_leaf;

    // Measured envelope: the fixed-point pipeline (three truncating divides
    // and the priced_sum != exactly-ONE renormalization) moves the actual
    // values by single bits against this model on the normal build. 64 bits
    // is 20x that measured envelope and still ~7x below the smallest
    // mutant-induced delta (~445 bits on leaves 4..7).
    const TOL_BITS: i64 = 64;

    let diff3 = (result[3].to_bits() as i64 - expected3.to_bits() as i64).abs();
    assert!(
        diff3 <= TOL_BITS,
        "leaf 3 must carry 4/15 of the flow blended with the explore floor: \
         expected {} bits, got {} bits (delta {diff3})",
        expected3.to_bits(),
        result[3].to_bits()
    );
    for &i in &LEAVES[1..] {
        let diff = (result[i].to_bits() as i64 - expected_leaf.to_bits() as i64).abs();
        assert!(
            diff <= TOL_BITS,
            "leaf {i} must carry 11/60 of the flow blended with the explore \
             floor: expected {} bits, got {} bits (delta {diff})",
            expected_leaf.to_bits(),
            result[i].to_bits()
        );
    }

    // Symmetry law of the fixture: the four grandchildren are exchangeable.
    let b = result[4].to_bits();
    assert!(
        result[5].to_bits() == b && result[6].to_bits() == b && result[7].to_bits() == b,
        "leaves 4..7 are exchangeable in this fixture and must carry equal \
         allocations, got {:?}",
        &result[4..8]
    );
}

fn zero_mu() -> [NonNegativeFixed; N] {
    [NonNegativeFixed::ZERO; N]
}

fn zero_costs() -> [NonNegativeFixed; N] {
    [NonNegativeFixed::ZERO; N]
}

/// non-vacuity: guards the `#[cfg(feature = "mutant_4")] let eta_actual =
/// zeta;` site at allocator/mod.rs:2162 (pre-edit numbering) -- the exact
/// mutation "leaf blend reads zeta where the admitted eta is contractual".
///
/// Differential kill: `allocate_in` admits `eta` as a free parameter in
/// [ETA_G_MIN, 1] independent of `zeta` (which only shapes the MWU learning
/// rate beta). Two calls identical except eta = 1/4 vs eta = 3/4 must produce
/// different leaf blends, and each must match the documented blend formula
/// evaluated at ITS OWN eta, with `pi_combined` reconstructed through
/// `allocate_single_lens` -- an entry point mutant_4 does not corrupt, so the
/// expected side stays honest (no self-reference trap). Under the mutant both
/// calls read `eta_actual = zeta = 0`, so (a) the two results collapse to
/// bit-identical vectors, killing the differ/ordering assertions, and (b)
/// each result equals bare `p_mu`, missing its expected blend at every leaf
/// where `p_mu != 1/nl` by `eta*|1/nl - p_mu|` -- thousands of bits against
/// the 8-bit house tolerance.
///
/// The ordering assertions are the exact direction law d(val)/d(eta) =
/// 1/nl - p_mu: the max-p_mu leaf must strictly decrease and the min-p_mu
/// leaf strictly increase as eta rises from 1/4 to 3/4.
#[test]
fn kill_mutant_4_leaf_blend_uses_admitted_eta_not_zeta() {
    let parent = depth_two_tree_parent();
    let eta_a = NonNegativeFixed::from_bits(16384); // 1/4, admitted
    let eta_b = NonNegativeFixed::from_bits(49152); // 3/4, admitted
    let zeta = NonNegativeFixed::ZERO; // admitted; keeps beta = 0 => static weights

    let res_a = run_allocate(&OBJECT_REGISTRY, &zero_mu(), &zero_costs(), eta_a, zeta)
        .unwrap_or_else(|e| panic!("allocate_in(eta=1/4) refused an admitted fixture: {e:?}"));
    let res_b = run_allocate(&OBJECT_REGISTRY, &zero_mu(), &zero_costs(), eta_b, zeta)
        .unwrap_or_else(|e| panic!("allocate_in(eta=3/4) refused an admitted fixture: {e:?}"));

    // Independent expected values per eta (house reconstruction idiom).
    let pi_combined = reconstruct_pi_combined(&OBJECT_REGISTRY, &parent);
    let mut priced_sum = NonNegativeFixed::ZERO;
    for &i in &LEAVES {
        priced_sum += pi_combined[i];
    }
    assert!(
        priced_sum.to_bits() > 0,
        "degenerate fixture: priced_sum over leaves must be positive"
    );
    let nl_recip = NonNegativeFixed::from_bits(NL_RECIP_BITS);
    let expected = |eta: NonNegativeFixed| -> [NonNegativeFixed; N] {
        let mut out = [NonNegativeFixed::ZERO; N];
        for &i in &LEAVES {
            let p_mu = pi_combined[i] / priced_sum;
            out[i] = eta * nl_recip + (NonNegativeFixed::ONE - eta) * p_mu;
        }
        out
    };
    let expected_a = expected(eta_a);
    let expected_b = expected(eta_b);

    // House tolerance from single_lens_allocation.rs's blend-identity tests:
    // the reconstruction reproduces the internal blend bit-for-bit up to
    // measured rounding on this registry (observed max 0 bits there; 8 here).
    const TOL_BITS: i64 = 8;
    for &i in &LEAVES {
        let diff_a = (res_a[i].to_bits() as i64 - expected_a[i].to_bits() as i64).abs();
        assert!(
            diff_a <= TOL_BITS,
            "leaf {i}: eta=1/4 call must match the blend at eta=1/4: expected \
             {} bits, got {} bits (delta {diff_a})",
            expected_a[i].to_bits(),
            res_a[i].to_bits()
        );
        let diff_b = (res_b[i].to_bits() as i64 - expected_b[i].to_bits() as i64).abs();
        assert!(
            diff_b <= TOL_BITS,
            "leaf {i}: eta=3/4 call must match the blend at eta=3/4: expected \
             {} bits, got {} bits (delta {diff_b})",
            expected_b[i].to_bits(),
            res_b[i].to_bits()
        );
    }

    // Fixture sanity + exact ordering law: some leaf must have p_mu above the
    // uniform floor and some below (else this registry/tree pair is degenerate
    // and eta cannot move the blend at all).
    let mut max_leaf = LEAVES[0];
    let mut min_leaf = LEAVES[0];
    for &i in &LEAVES {
        if pi_combined[i].to_bits() > pi_combined[max_leaf].to_bits() {
            max_leaf = i;
        }
        if pi_combined[i].to_bits() < pi_combined[min_leaf].to_bits() {
            min_leaf = i;
        }
    }
    assert!(
        pi_combined[max_leaf].to_bits() > pi_combined[min_leaf].to_bits(),
        "degenerate fixture: pi_combined is uniform over leaves, so eta \
         cannot move the blend and this differential cannot discriminate"
    );
    // d(val)/d(eta) = 1/nl - p_mu < 0 on the max-p_mu leaf: raising eta from
    // 1/4 to 3/4 must strictly lower it. Strict bit inequality -- exact.
    assert!(
        res_b[max_leaf].to_bits() < res_a[max_leaf].to_bits(),
        "max-p_mu leaf {max_leaf} must strictly decrease as eta rises (1/4 -> \
         3/4): got {} -> {}",
        res_a[max_leaf].to_bits(),
        res_b[max_leaf].to_bits()
    );
    assert!(
        res_b[min_leaf].to_bits() > res_a[min_leaf].to_bits(),
        "min-p_mu leaf {min_leaf} must strictly increase as eta rises (1/4 -> \
         3/4): got {} -> {}",
        res_a[min_leaf].to_bits(),
        res_b[min_leaf].to_bits()
    );
}

/// non-vacuity: guards the `mutant_5` cfg sites at allocator/mod.rs (the two
/// former `let mu_actual = ...` blocks, pre-edit lines 2122 and 2152), whose
/// mutation since the 2026-10-01 retirement is `mu_actual = mu_max` -- the
/// exact mutation "the priced softmax reads the region price CEILING where
/// the admitted price vector mu is contractual".
///
/// The slot's original mutation (dropping the defensive `clip(mu, 0,
/// mu_max)`) was retired as equivalent-by-design per CMCA-122: on every
/// admitted path `price_err` refuses before the value is observable, so no
/// test can kill it. The coordinator-proposed replacement (dropping the
/// leaf-mask on `priced_sum += p`) was refused without injection: internal
/// nodes carry `pi_combined == 0` on every reachable state (see the ledger
/// header), so the masked terms are all zero and the mask is provably dead.
/// The adopted mutation corrupts a value that IS observable on Ok paths.
///
/// Kill mechanism (all exact): with `mu = 0` and `costs = 1` the normal path
/// computes `exp(-0*1) == exp(0) == ONE` bit-exactly, so leaves follow the
/// documented blend `eta*(1/nl) + (1-eta)*p_mu` with `p_mu = pi_combined /
/// sum_leaves(pi_combined)`. Under the mutant the softmax price is `mu_max =
/// 100`, so `exp(-100*1)` underflows to exactly 0, `priced_sum` collapses to
/// 0, the `psd` guard substitutes 1, and every leaf collapses to the bare
/// explore floor `eta*(1/nl) == 6553` bits -- thousands of bits below the
/// expected priced blend wherever `p_mu > 0`.
#[test]
fn kill_mutant_5_priced_softmax_uses_admitted_mu_not_ceiling() {
    let parent = depth_two_tree_parent();
    let eta = NonNegativeFixed::from_bits(32768); // 1/2, admitted
    let zeta = NonNegativeFixed::ZERO;

    // Admitted: every mu[i] = 0 <= mu_max (= 100.0, price_gain_max); costs
    // are un-gated inputs. exp(-mu*costs) == exp(0) == ONE exactly.
    let mu = zero_mu();
    let costs = [NonNegativeFixed::ONE; N];

    let result = run_allocate(&OBJECT_REGISTRY, &mu, &costs, eta, zeta)
        .unwrap_or_else(|e| panic!("allocate_in refused an admitted fixture: {e:?}"));

    let pi_combined = reconstruct_pi_combined(&OBJECT_REGISTRY, &parent);
    let mut priced_sum = NonNegativeFixed::ZERO;
    for &i in &LEAVES {
        priced_sum += pi_combined[i];
    }
    let nl_recip = NonNegativeFixed::from_bits(NL_RECIP_BITS);
    let explore_floor = eta * nl_recip;

    // Exact non-vacuity of the fixture: the priced term must actually move
    // some leaf away from the explore floor, or this test could never
    // distinguish "priced blend" from "floor collapsed by a dead price".
    let mut max_priced_excess: i64 = 0;
    let mut expected = [NonNegativeFixed::ZERO; N];
    for &i in &LEAVES {
        let p_mu = pi_combined[i] / priced_sum;
        expected[i] = explore_floor + (NonNegativeFixed::ONE - eta) * p_mu;
        let excess = expected[i].to_bits() as i64 - explore_floor.to_bits() as i64;
        if excess > max_priced_excess {
            max_priced_excess = excess;
        }
    }
    assert!(
        max_priced_excess > 100,
        "degenerate fixture: no leaf's expected blend exceeds the explore \
         floor by more than {max_priced_excess} bits, so the priced term is \
         unobservable and this test cannot discriminate"
    );

    // House 8-bit tolerance (same reconstruction pipeline as the eta
    // differential; on the normal build the two computations are the same
    // formula over the same values, so deltas are pure rounding).
    const TOL_BITS: i64 = 8;
    for &i in &LEAVES {
        let diff = (result[i].to_bits() as i64 - expected[i].to_bits() as i64).abs();
        assert!(
            diff <= TOL_BITS,
            "leaf {i}: priced blend with admitted mu must match \
             eta*(1/nl) + (1-eta)*p_mu: expected {} bits, got {} bits \
             (delta {diff})",
            expected[i].to_bits(),
            result[i].to_bits()
        );
    }

    // Internal nodes still carry exactly zero.
    for (i, share) in result.iter().take(3).enumerate() {
        assert_eq!(
            share.to_bits(),
            0,
            "internal node {i} must carry zero allocation, got {:?}",
            share
        );
    }
}
