//! Divan benchmarks for the CMCA cascade kernel — the production entry
//! points of `cascade.rs` NOT covered by `allocation_bench.rs`:
//! `consequence_mass` (the two-path descendant/flat cascade walk),
//! `escort_weight` (integer/negative-lens primitive), and `admit_fixed`
//! (the module's single arithmetic-admission choke point).
//!
//! Convention matches `benches/allocation_bench.rs` and
//! `bcinr-powl/benches/*`: Divan, `harness = false`, black-boxed inputs AND
//! outputs. Requires the `alloc` feature (`cascade` is `#[cfg(feature =
//! "alloc")]` in lib.rs); the `[[bench]]` target declares
//! `required-features = ["alloc"]` following this crate's own test-target
//! convention, so a default-feature build reports this suite as skipped
//! rather than passing it as an empty binary. Run:
//! `cargo bench -p bcinr-cmca --features std cascade_bench`.
//!
//! Fixture shape: an N=8 binary-heap tree (root 0, parent(i) = (i-1)/2)
//! with equal unit masses — the same flat, fully-degenerate shape
//! `allocation_bench.rs` uses for `allocate`, so per-call costs of the two
//! kernels are directly comparable at the generated case-studies N. Lenses
//! `vec![1]` (exact integer path) and `vec![0]` (uniform coverage path) are
//! the two lens classes the module's own fixture corpus
//! (`tests/cascade_residual_classification.rs`) benches against.
//!
//! Numbers are receipts, not claims.

use std::sync::OnceLock;

use bcinr_cmca::cascade::{
    admit_fixed, consequence_mass, escort_weight, CascadeTree, NumericContext,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use divan::black_box;

fn main() {
    divan::main();
}

/// Q16.16 ONE, by bits (65536 = 1.0), matching the crate's bit-level idiom.
const ONE_BITS: u32 = 1 << 16;

/// The N=8 generated case-studies constant, imported so the fixture shape
/// cannot silently drift from the production profile shape.
use bcinr_cmca::generated::consequence_mass::case_studies::N as CASE_STUDY_N;

const TREE_N: usize = CASE_STUDY_N; // 8

/// Root + 7 nodes in a binary-heap layout: parent(i) = (i-1)/2 for i >= 1.
/// Depth is data-independent by construction (fixed N).
fn heap_parents() -> Vec<Option<usize>> {
    let mut parent = vec![None; TREE_N];
    for (i, slot) in parent.iter_mut().enumerate().skip(1) {
        *slot = Some((i - 1) / 2);
    }
    parent
}

/// Equal unit masses: the flat fixture. Every sibling group normalizes to
/// the same share vector, so the measured cost is the walk's structural
/// cost, not any one skewed value's cost (the kernel's instruction sequence
/// is data-independent by construction; this fixture prices the shape).
fn unit_masses() -> Vec<NonNegativeFixed> {
    vec![NonNegativeFixed::from_bits(ONE_BITS); TREE_N]
}

/// The shared fixture, built once. `CascadeTree::new` validates ranges and
/// lengths (allocation included in the one-time init, not the benches).
fn fixture_tree() -> &'static CascadeTree {
    static TREE: OnceLock<CascadeTree> = OnceLock::new();
    TREE.get_or_init(|| {
        CascadeTree::new(heap_parents(), unit_masses()).expect("fixture tree must be well-formed")
    })
}

#[divan::bench]
fn consequence_mass_lens_1() {
    let lenses = [1i32];
    let out = consequence_mass(black_box(fixture_tree()), black_box(&lenses));
    let _ = black_box(out);
}

#[divan::bench]
fn consequence_mass_lens_0() {
    let lenses = [0i32];
    let out = consequence_mass(black_box(fixture_tree()), black_box(&lenses));
    let _ = black_box(out);
}

#[divan::bench]
fn escort_weight_positive_lens() {
    // lens = 1: exact repeated-multiply path (weight == mass at q=1).
    let out = escort_weight(
        black_box(NonNegativeFixed::from_bits(ONE_BITS)),
        black_box(1i32),
        black_box(3usize),
    );
    let _ = black_box(out);
}

#[divan::bench]
fn escort_weight_negative_lens() {
    // lens = -1: reciprocal path (1 / m^|q|), the second instruction class.
    let out = escort_weight(
        black_box(NonNegativeFixed::from_bits(ONE_BITS)),
        black_box(-1i32),
        black_box(3usize),
    );
    let _ = black_box(out);
}

#[divan::bench]
fn admit_fixed_clean() {
    // The Ok path: err == u32::MAX (clean value) — the choke point every
    // cascade arithmetic result flows through on the common case.
    let clean = NonNegativeFixed::from_bits(ONE_BITS);
    let out = admit_fixed(
        black_box(clean),
        black_box(NumericContext::ShareDivision),
        black_box(3usize),
    );
    let _ = black_box(out);
}

#[divan::bench]
fn consequence_mass_traced_full_tree() {
    // The traced twin of `consequence_mass_lens_1`: identical walk and
    // arithmetic, with the VecTrace sink actually listening — prices the
    // per-step bookkeeping the profiler lane needs attributed.
    let lenses = [1i32];
    let out =
        bcinr_cmca::cascade::consequence_mass_traced(black_box(fixture_tree()), black_box(&lenses));
    let _ = black_box(out);
}
