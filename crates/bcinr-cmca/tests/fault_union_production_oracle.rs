//! Production oracle tests for `NumericFaultSet::union` (`src/fixed.rs`), added for
//! the fault-union PRODUCTION-mutation evidence track (see
//! `chicago-tdd-tools/crates/chicago-claims/claims/cmca-fault-union-production.toml`
//! and `.../reports/cmca-fault-union-production-mutation.md`).
//!
//! This is a NEW test file only — it does not modify `src/fixed.rs` or
//! `src/allocator.rs`, and it exercises only the crate's existing public API
//! (`NumericFaultSet`'s public constants/methods, `NonNegativeFixed`'s public
//! constructors/arithmetic/accessors). No `pub(crate)` item is used.
//!
//! # Why these two oracles exist, beyond the pre-existing
//! `fixed::tests::union_accumulates_both_operands_distinct_faults`
//!
//! That existing unit test (in `src/fixed.rs`) calls `union` on two operands
//! whose fault bits are DISJOINT (`UNDERFLOW` = bit 1, `DIVIDE_BY_ZERO` = bit
//! 2). For any pair of DISTINCT single-bit `NumericFaultSet` constants,
//! bitwise-OR and bitwise-XOR are numerically IDENTICAL — disjoint bits never
//! cancel under XOR — so that test cannot distinguish the real union law
//! (`a | b`) from an `a ^ b` corruption. `oracle_union_is_idempotent_on_repeated_fault_bit`
//! below is the oracle that CAN: idempotence (`a | a == a`) holds for
//! bitwise-OR but fails for bitwise-XOR (`a ^ a == EMPTY` for any nonzero
//! `a`).
//!
//! Separately, `oracle_union_law_preserves_left_and_right_faults_when_local_is_clean`
//! exercises the full three-way accumulation law
//! `faults_out = faults_left UNION faults_right UNION faults_local` exactly as
//! it is actually composed at real production call sites (e.g.
//! `NonNegativeFixed::saturating_add`: `self.faults.union(other.faults).union(e)`),
//! rather than calling the two-argument `union` primitive in isolation. It is
//! the dedicated oracle for the `local-only` mutant, which — per the fixed.rs
//! call convention where the freshly-computed local fault `e` is always
//! threaded as the *second* argument of the outer `.union(e)` call — has the
//! IDENTICAL source patch as `right-only`/`overwrite` (see the claim TOML and
//! report for that documented equivalence, itself following the precedent
//! already set by `right-only`/`overwrite` in the sibling fixture claim
//! `cmca-fault-union.toml`).

use bcinr_cmca::fixed::{NonNegativeFixed, NumericFaultSet};

/// Falsifies a bitwise-XOR corruption of `NumericFaultSet::union`: unioning a
/// nonempty fault set with itself must be idempotent (`a | a == a`), which
/// holds for bitwise-OR but NOT for bitwise-XOR (`a ^ a == EMPTY`).
#[test]
fn oracle_union_is_idempotent_on_repeated_fault_bit() {
    let a = NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION);
    assert!(!a.is_empty());

    let doubled = a.union(a);

    assert_eq!(
        doubled, a,
        "union(a, a) must equal a (idempotent bitwise-OR); a corrupted xor-union \
         would instead cancel every shared bit back to EMPTY"
    );
    assert!(!doubled.is_empty(), "idempotent union of a nonempty fault set must stay nonempty");
}

/// Exercises the full three-way accumulation law
/// `faults_out = faults_left UNION faults_right UNION faults_local` as it is
/// actually composed in production call sites, using only publicly
/// constructible values: `left`/`right` each already carry a distinct
/// pre-existing fault from an earlier public-API operation, and this specific
/// `saturating_add` call is constructed to add no NEW ("local") fault of its
/// own, so the correct result must retain exactly `left`'s and `right`'s
/// pre-existing faults.
///
/// Falsifies a corruption that keeps only the freshly-computed local fault
/// and drops both incoming operands' faults (`local-only`): under such a
/// corruption, since no local fault occurs on this specific call, the result
/// would collapse to `NumericFaultSet::EMPTY`, silently erasing `left`'s and
/// `right`'s real, already-observed faults.
#[test]
fn oracle_union_law_preserves_left_and_right_faults_when_local_is_clean() {
    // left: UNDERFLOW only (0 - 1 saturates to 0, reports UNDERFLOW).
    let left = NonNegativeFixed::ZERO.saturating_sub(NonNegativeFixed::ONE);
    assert_eq!(left.faults(), NumericFaultSet::UNDERFLOW);
    assert_eq!(left.value_bits(), 0);

    // right: DIVIDE_BY_ZERO | INVALID_DOMAIN (x / 0), value saturates to MAX.
    let right = NonNegativeFixed::ZERO.saturating_div(NonNegativeFixed::ZERO);
    assert_eq!(
        right.faults(),
        NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN)
    );
    assert_eq!(right.value_bits(), u32::MAX);

    // left.value_bits() == 0, right.value_bits() == u32::MAX: this specific add
    // does NOT overflow (0 + u32::MAX wraps to u32::MAX, which is not less than
    // 0), so the local fault `e` computed by THIS saturating_add call is EMPTY
    // — the only faults in the correct result come from `left`/`right`
    // themselves, not from any new local computation.
    let combined = left.saturating_add(right);

    assert_eq!(
        combined.faults().bits(),
        left.faults().bits() | right.faults().bits(),
        "saturating_add must retain both pre-existing operand faults when it \
         adds no new local fault of its own"
    );
    assert!(!combined.faults().is_empty());
}
