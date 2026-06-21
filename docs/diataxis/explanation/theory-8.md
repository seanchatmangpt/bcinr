# Verification by Reference Oracles and Counterfactual Mutants

A branchless primitive is harder to read than its branching equivalent —
that is the price of turning control flow into arithmetic. So how do we know
a clever bit-twiddle actually computes what it claims? bcinr's answer is a
two-part testing discipline: an **independent reference oracle** establishes
*correctness*, and **counterfactual mutants** establish that the test is
actually *capable of failing*. This document explains why both halves are
necessary and how they appear in the code.

## The trap: a test that cannot fail

The obvious way to test `f` is to compare it against a reference `f_ref` for
many inputs. This is sound *only if* `f_ref` is genuinely independent of `f`.
If `f_ref` is a copy of `f`'s logic, the comparison is a tautology: both are
wrong in the same way, the assertion passes, and you have learned nothing.
The project calls this the *circular reference oracle* and lists it as an
explicit anti-pattern (`anti-patterns.md`, item 6). The whole verification
strategy is built to avoid it.

## Half one: an independent reference oracle

The reference implementation is written to be *obviously correct*, even if
slow or branchy — the opposite of the optimised primitive. A branchless
`min_u32` is checked against `if a < b { a } else { b }`; a SWAR popcount is
checked against a naive bit-counting loop; a saturating add is checked
against a widening add followed by a clamp. The reference is allowed to
branch, allocate scratch, and be ten times slower, because its only job is
to be *evidently* right. Correctness of `f` is then "for all inputs in the
domain, `f(x) == f_ref(x)`."

Coverage of "for all inputs" comes from `proptest`, which samples the input
space (including boundary values) rather than relying on a handful of
hand-picked cases. Over a finite or well-sampled domain, agreement with an
independent oracle is the executable form of the Hoare postcondition
discussed in `theory-5.md`.

## Half two: counterfactual mutants

A passing oracle test still leaves a question: *was the test able to fail at
all?* A test that always passes — because of a tautological oracle, a
no-op assertion, or a domain that never exercises the logic — provides false
assurance. Counterfactual mutants answer this by deliberately *breaking* the
function and demanding the test notice.

Each module ships a small family of intentional mutants and asserts they are
*distinguishable* from the reference. The pattern is uniform across the
source — `mask.rs`, `int.rs`, `bitset.rs`, and the rest all carry it:

```rust
fn mutant_1(val: u64, aux: u64) -> u64 { !reference(val, aux) }          // bit-flip
fn mutant_2(val: u64, aux: u64) -> u64 { reference(val, aux).wrapping_add(1) } // off-by-one
fn mutant_3(val: u64, aux: u64) -> u64 { reference(val, aux) ^ 0xFF }    // low-byte corruption

#[test] fn rejects_mutant_1() { assert!(reference(1,1) != mutant_1(1,1)); }
```

If the test suite could *not* tell the mutant from the truth, the assertion
fails — exposing a test that lacks discriminating power. This is mutation
testing in miniature: the mutants are the falsification challenge, and a
verification that survives a mutant it should have caught is not a
verification at all.

## Why both halves, together

The two halves guard against opposite failures:

```
   reference oracle      -> catches a WRONG implementation
   counterfactual mutant -> catches a WEAK test (one that can't detect wrong)
```

A correct implementation with a weak test is a latent bug waiting for a
refactor; a strong test against a circular oracle is theatre. Only the
conjunction — an independent oracle *and* demonstrated mutant-killing power —
licenses the claim a PhD Gate records. As `phd_gates.md` puts it, the
equivalence test *is* the executable proof, and the mutant tests are what
make that proof credible rather than decorative.

## The limits, stated plainly

This methodology establishes *behavioural* equivalence on the tested domain;
it does not by itself prove *constant time* (a timing property — see
`theory-2`, `theory-7`) or *memory safety* (handled by
`#![forbid(unsafe_code)]` and, for the audited exceptions, the proofs in
`SAFETY.md`). Property tests sample rather than exhaust an infinite domain,
and a fixed mutant set catches the mistakes it is shaped to catch, not all
possible ones. The discipline is strong evidence, honestly bounded — which
is exactly why the project treats unbacked "verified" comments as an
anti-pattern rather than a proof.
