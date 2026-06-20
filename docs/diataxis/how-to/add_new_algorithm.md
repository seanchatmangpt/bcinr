# How to Add a New Branchless Algorithm

This guide walks through the complete workflow for adding a new algorithm to
`crates/bcinr-logic`. Follow every step to keep the codebase consistent and
ensure your algorithm passes CI.

## Prerequisites

- Rust 1.70+
- `cargo-make` installed (`cargo install cargo-make`)
- Familiarity with branchless arithmetic — read
  [SWAR Techniques](../explanation/swar_techniques.md) if you haven't already

## Step 1: Create the module file

Create `crates/bcinr-logic/src/algorithms/your_algorithm_name.rs`.

The filename must be lowercase snake_case and descriptive. Look at existing
files in that directory for naming conventions (e.g., `min_u32`, `xxhash64`,
`bit_parallel_sort8_u32`).

```rust
#![forbid(unsafe_code)]
//! Brief one-line description of what this algorithm computes.
//!
//! Longer explanation of the mathematical operation, the branchless technique
//! used, and any notable properties (e.g., constant-time, side-channel safe).

/// Full description of what this function computes.
///
/// Explain the branchless technique in the body if it is non-obvious.
///
/// # Arguments
///
/// - `input1`: description and valid range
/// - `input2`: description and valid range
///
/// # Returns
///
/// Description of the return value and its relationship to the inputs.
///
/// # Examples
///
/// ```rust
/// use bcinr_logic::algorithms::your_algorithm_name::your_function;
/// assert_eq!(your_function(5, 3), 8);
/// ```
#[inline(always)]
pub fn your_function(input1: u64, input2: u64) -> u64 {
    // NO if/else or match on data values.
    // Use mask arithmetic for conditional logic:
    //   let mask = 0u64.wrapping_sub(condition as u64); // all-ones or all-zeros
    //   let result = (value_if_true & mask) | (value_if_false & !mask);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference implementation: correct but may use branches.
    /// Used as a ground-truth oracle for property-based tests.
    fn reference_impl(a: u64, b: u64) -> u64 {
        // Simple, obviously correct implementation using standard Rust.
        // Example: a + b, a.min(b), etc.
        todo!()
    }

    #[test]
    fn test_spot_check() {
        assert_eq!(your_function(5, 3), reference_impl(5, 3));
        assert_eq!(your_function(0, 42), reference_impl(0, 42));
        assert_eq!(your_function(100, 1), reference_impl(100, 1));
    }

    #[test]
    fn test_boundaries() {
        assert_eq!(your_function(0, 0), reference_impl(0, 0));
        assert_eq!(your_function(u64::MAX, u64::MAX), reference_impl(u64::MAX, u64::MAX));
        assert_eq!(your_function(u64::MAX, 0), reference_impl(u64::MAX, 0));
        assert_eq!(your_function(0, u64::MAX), reference_impl(0, u64::MAX));
    }

    // Property-based test (uncomment if proptest is available in this crate):
    // use proptest::prelude::*;
    // proptest! {
    //     #[test]
    //     fn test_matches_reference(a: u64, b: u64) {
    //         prop_assert_eq!(your_function(a, b), reference_impl(a, b));
    //     }
    // }
}
```

## Step 2: Register in `algorithms/mod.rs`

Open `crates/bcinr-logic/src/algorithms/mod.rs` and add your module in
**alphabetical order** among the existing `pub mod` declarations:

```rust
pub mod your_algorithm_name;
```

## Step 3: Run the checks

```bash
cd crates/bcinr-logic

# Compile and run just your new tests with output
cargo test your_function -- --nocapture

# Lint with zero-warnings policy
cargo clippy -- -D warnings

# Verify formatting
cargo fmt --check
```

All three must pass before proceeding.

## Step 4: Add a benchmark

Open `bcinr-bench/benches/bcinr_bench.rs` (or create a dedicated bench file
for your algorithm family if it has many variants):

```rust
use bcinr_logic::algorithms::your_algorithm_name::your_function;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_your_function(c: &mut Criterion) {
    c.bench_function("your_function", |b| {
        b.iter(|| your_function(black_box(42u64), black_box(1337u64)))
    });
}

criterion_group!(benches, bench_your_function);
criterion_main!(benches);
```

Run it to establish a baseline:

```bash
cd bcinr-bench
cargo bench --bench bcinr_bench -- your_function
```

## Step 5: Run the full workspace check

```bash
cd /path/to/bcinr
make check && make test && make clippy && make fmt
```

All four must succeed before you open a pull request.

## Step 6: Commit

Use conventional commit format:

```
feat(algorithms): add branchless <algorithm name> for <type>

One sentence explaining why this algorithm is useful or what property
it preserves (e.g., "Constant-time absolute value avoids branch
misprediction on sign changes.").
```

## Branchless Patterns Cheat Sheet

When writing your implementation, reach for these patterns instead of `if`/`else`:

| Goal | Branching version | Branchless version |
|------|------------------|--------------------|
| Select based on condition | `if cond { x } else { y }` | `let m = 0u64.wrapping_sub(cond as u64);`<br>`(x & m) \| (y & !m)` |
| Maximum of two u64 | `if a > b { a } else { b }` | `b + ((a.wrapping_sub(b)) & 0u64.wrapping_sub((a > b) as u64))` |
| Minimum of two u64 | `if a < b { a } else { b }` | `a + ((b.wrapping_sub(a)) & 0u64.wrapping_sub((b < a) as u64))` |
| Absolute value of i64 | `if x < 0 { -x } else { x }` | `let m = x >> 63; (x ^ m).wrapping_sub(m)` |
| Clamp to `[lo, hi]` | `x.max(lo).min(hi)` | Two branchless min/max calls (compiler optimizes) |
| Is zero? (mask form) | `if x == 0 { all_ones } else { 0 }` | `let nz = (x \| x.wrapping_neg()) >> 63;`<br>`nz.wrapping_sub(1)` |
| Is non-zero? (mask form) | `if x != 0 { all_ones } else { 0 }` | `let nz = (x \| x.wrapping_neg()) >> 63;`<br>`0u64.wrapping_sub(nz)` |

For more patterns, see `crates/bcinr-logic/src/mask.rs` (the canonical source
of branchless selection primitives in this library).

## Common Mistakes to Avoid

**Using `if` on data values:**
```rust
// Wrong — introduces a branch
pub fn my_algo(a: u64, b: u64) -> u64 {
    if a > b { a - b } else { b - a }
}

// Right — branchless absolute difference
pub fn my_algo(a: u64, b: u64) -> u64 {
    let mask = 0u64.wrapping_sub((a > b) as u64);
    (a.wrapping_sub(b) & mask) | (b.wrapping_sub(a) & !mask)
}
```

**Forgetting `#![forbid(unsafe_code)]`:**
Every file under `algorithms/` must have this as its first line. The CI
build enforces it.

**Skipping the reference oracle in tests:**
Without a reference implementation, tests only check for panics, not
correctness. Always write `reference_impl` and compare.

**Not checking boundary values:**
`u64::MAX`, `0`, `u64::MAX - 1`, and powers of two are the most common
sources of off-by-one bugs in branchless arithmetic. Always test them.
