# Tutorial 9: Property-Testing a Branchless Kernel

Hand-picked assertions miss bugs. A branchless kernel earns its place in bcinr by
matching a slow-but-obvious *reference* implementation for **every** input, and
by being sharp enough to *reject* deliberately broken variants. This is the
repository's reference-oracle + counterfactual-mutant pattern, and in this
tutorial you write one from scratch.

## What you'll build

A complete `proptest` suite for a branchless `min_u32`-style kernel:

1. an **equivalence** property — branchless output equals a branchful reference
   for all inputs;
2. three **counterfactual mutant** properties — intentionally wrong versions are
   provably *different* from the reference, proving the test can actually fail.

**Prerequisites:** [Tutorial 1](./tutorial-1.md) and
[Tutorial 2](./tutorial-2.md). You should know what `select_u32` and `min_u32`
do. `proptest` is already a dev-dependency of the workspace.

## Step 1: See the pattern in the source

Every algorithm module in `crates/bcinr-logic/src/algorithms/` follows the same
shape (see `aabb_intersect_branchless.rs` or `clamp_i64.rs`):

```text
fn kernel(...)            -> the branchless implementation under test
fn kernel_reference(...)  -> the slow, obviously-correct oracle
fn mutant_kernel_1(...)   -> reference with a deliberate bug (identity bluff)
fn mutant_kernel_2(...)   -> reference + 1 (bit-skip bluff)
fn mutant_kernel_3(...)   -> reference ^ mask (operator-swap bluff)

proptest! {
    equivalence:   kernel == reference          for all inputs
    mutant N:      reference != mutant_N         for non-trivial inputs
}
```

The mutant tests are not redundant — they guard the *guard*. If the equivalence
test were accidentally a tautology, the mutant tests would still demand that a
wrong answer be detectably wrong.

## Step 2: Write the kernel and its oracle

We will verify a branchless minimum built on the `mask` primitives.

```rust
use bcinr_logic::mask::{lt_mask_u32, select_u32};

/// Kernel under test: branchless minimum.
fn branchless_min(a: u32, b: u32) -> u32 {
    select_u32(lt_mask_u32(a, b), a, b)
}

/// Reference oracle: the obvious branchful version.
fn min_reference(a: u32, b: u32) -> u32 {
    if a < b {
        a
    } else {
        b
    }
}
```

## Step 3: Add three counterfactual mutants

Each mutant perturbs the *reference* so it returns a wrong answer. The suite must
prove these are distinguishable from the true reference.

```rust
fn mutant_1(a: u32, b: u32) -> u32 {
    !min_reference(a, b) // identity bluff: bitwise NOT
}
fn mutant_2(a: u32, b: u32) -> u32 {
    min_reference(a, b).wrapping_add(1) // off-by-one bluff
}
fn mutant_3(a: u32, b: u32) -> u32 {
    min_reference(a, b) ^ 0xFFFF_FFFF // operator-swap bluff
}
```

## Step 4: Write the property tests

```rust
use proptest::prelude::*;

proptest! {
    // (1) Equivalence: the branchless kernel matches the oracle for ALL inputs.
    #[test]
    fn min_equivalence(a in any::<u32>(), b in any::<u32>()) {
        prop_assert_eq!(branchless_min(a, b), min_reference(a, b));
    }

    // (2) Counterfactual mutants: each broken version must differ from the oracle.
    // We exclude the degenerate inputs where a bug happens to be invisible
    // (e.g. NOT of 0 collisions), mirroring the repository's guards.
    #[test]
    fn rejects_mutant_1(a in any::<u32>(), b in any::<u32>()) {
        prop_assume!(a != b && a != 0 && b != 0);
        prop_assert_ne!(min_reference(a, b), mutant_1(a, b));
    }

    #[test]
    fn rejects_mutant_2(a in any::<u32>(), b in any::<u32>()) {
        prop_assume!(a != b && a != 0 && b != 0);
        prop_assert_ne!(min_reference(a, b), mutant_2(a, b));
    }

    #[test]
    fn rejects_mutant_3(a in any::<u32>(), b in any::<u32>()) {
        prop_assume!(a != b && a != 0 && b != 0);
        prop_assert_ne!(min_reference(a, b), mutant_3(a, b));
    }
}
```

`any::<u32>()` makes proptest sample the whole 32-bit space (with shrinking on
failure). `prop_assume!` discards the trivial inputs where a mutation would be
invisible — the same approach the real modules use with their
`if val != aux && val != 0 && aux != 0` guards.

## Step 5: Run the suite

Place the code above in a `#[cfg(test)] mod tests` block and run:

```bash
cargo test min_equivalence rejects_mutant_1 rejects_mutant_2 rejects_mutant_3
```

Expected output:

```
test tests::min_equivalence ... ok
test tests::rejects_mutant_1 ... ok
test tests::rejects_mutant_2 ... ok
test tests::rejects_mutant_3 ... ok
```

Four green tests: the kernel is equivalent to the oracle across the input space,
and the suite demonstrably catches wrong answers.

## Step 6: Watch the test catch a real bug

Prove the equivalence test bites. Temporarily break the kernel by swapping the
`select` arguments (turning min into max):

```rust
fn branchless_min(a: u32, b: u32) -> u32 {
    select_u32(lt_mask_u32(a, b), b, a) // BUG: this is now max
}
```

```bash
cargo test min_equivalence
```

```
thread 'tests::min_equivalence' panicked at 'assertion failed:
  `(left == right)` ... minimal failing input: a = 0, b = 1'
test tests::min_equivalence ... FAILED
```

proptest shrinks the counterexample to the smallest case (`a = 0, b = 1`), which
points straight at the bug. Revert the swap and the suite goes green again.

## What you learned

- The repository's verification recipe is: branchless kernel + branchful oracle +
  three counterfactual mutants, all driven by `proptest`.
- Equivalence properties prove correctness across the whole input space; mutant
  properties prove the test itself can fail.
- `prop_assume!` excludes degenerate inputs where a mutation would be invisible,
  matching the guards in the real algorithm modules.

## Next steps

- [Tutorial 10: Benchmarking a kernel with Criterion](./tutorial-10.md) — once a
  kernel is *correct*, measure that it is also *fast and flat*.
- [Tutorial 8: A branchless AABB overlap test](./tutorial-8.md) — apply this
  exact suite to the AABB kernel you built there.
