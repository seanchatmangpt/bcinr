# Tutorial 10: Benchmarking a Kernel with Criterion

A branchless kernel makes two promises: it is *correct* (you verified that in
[Tutorial 9](./tutorial-9.md)) and its latency is *flat* — the same for every
input. The only way to back the second claim with evidence is to measure. bcinr
benchmarks with [Criterion](https://bheisler.github.io/criterion.rs/book/), and
this tutorial takes you from zero to a statistically rigorous measurement.

## What you'll build

A Criterion benchmark for the `mask::select_u32` kernel from
[Tutorial 1](./tutorial-1.md), measured across best/typical/worst inputs to show
that the timing does not depend on the data. You will mirror the exact harness
style used in `bcinr-bench/benches/`.

**Prerequisites:** [Tutorial 1](./tutorial-1.md). Having read
[Tutorial 9](./tutorial-9.md) helps — never benchmark a kernel you have not first
proven correct.

## Step 1: Understand the harness conventions

Open `bcinr-bench/benches/bcinr_bench.rs`. Two conventions matter:

```rust
use bcinr_core::logic::mask::select_u32;          // import the kernel
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_mask(c: &mut Criterion) {
    c.bench_function("mask::select_u32", |b| {
        b.iter(|| select_u32(0xFFFFFFFF, 10, 20))
    });
}

criterion_group!(benches, bench_mask);
criterion_main!(benches);
```

And in `bcinr-bench/Cargo.toml`, every bench file is registered with the Criterion
harness disabled (Criterion provides its own `main`):

```toml
[[bench]]
name = "select_latency"
harness = false
```

## Step 2: Use black_box to defeat the optimizer

`select_u32(0xFFFFFFFF, 10, 20)` has constant arguments — without help the
compiler would fold it to `10` at compile time and you would measure nothing.
`criterion::black_box` hides values from the optimizer so the kernel actually
runs. You can see this pattern throughout `patterns_bench.rs` and
`algorithms_1_100.rs`:

```rust
use criterion::black_box;
// ...
b.iter(|| select_u32(black_box(0xFFFFFFFF), black_box(10), black_box(20)))
```

## Step 3: Write the benchmark

Create `bcinr-bench/benches/select_latency.rs`:

```rust
use bcinr_core::logic::mask::select_u32;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_select(c: &mut Criterion) {
    // The "true" branch: mask is all-ones, kernel returns a.
    c.bench_function("select_u32/mask_ones", |b| {
        b.iter(|| select_u32(black_box(0xFFFF_FFFF), black_box(10), black_box(20)))
    });

    // The "false" branch: mask is all-zeros, kernel returns b.
    c.bench_function("select_u32/mask_zeros", |b| {
        b.iter(|| select_u32(black_box(0x0000_0000), black_box(10), black_box(20)))
    });

    // A "mixed" mask: still no branch, identical instruction path.
    c.bench_function("select_u32/mask_mixed", |b| {
        b.iter(|| select_u32(black_box(0xF0F0_F0F0), black_box(10), black_box(20)))
    });
}

criterion_group!(benches, bench_select);
criterion_main!(benches);
```

The three cases are the whole point: a *branchful* selector would time
differently for the taken vs not-taken branch. A branchless one should not.

## Step 4: Register the bench target

Add this to `bcinr-bench/Cargo.toml` under the existing `[[bench]]` entries:

```toml
[[bench]]
name = "select_latency"
harness = false
```

The `name` must match the file stem (`select_latency.rs`).

## Step 5: Run the benchmark

```bash
cd bcinr-bench
cargo bench --bench select_latency
```

Expected output (numbers vary by machine; the *shape* is what matters):

```
select_u32/mask_ones    time:   [612.4 ps 615.1 ps 618.3 ps]
select_u32/mask_zeros   time:   [611.8 ps 614.0 ps 616.9 ps]
select_u32/mask_mixed   time:   [613.2 ps 615.7 ps 619.1 ps]
```

All three confidence intervals overlap. That overlap *is* the evidence: the
latency of `select_u32` does not depend on the mask, exactly as a branchless
kernel should behave.

## Step 6: Read the HTML report

Criterion writes a full statistical report with plots:

```bash
xdg-open ../target/criterion/report/index.html   # Linux
# open ../target/criterion/report/index.html     # macOS
```

The report shows the probability density of each measurement. For a branchless
kernel the three curves sit on top of one another; a branchful one would show
two separated humps for the taken/not-taken paths.

## Step 7: Track regressions over time

Criterion compares against the previous run automatically. Save a named baseline,
make a change, then compare:

```bash
# record a baseline
cargo bench --bench select_latency -- --save-baseline before

# ...edit the kernel...

# measure against it
cargo bench --bench select_latency -- --baseline before
```

Criterion prints `change: [...]` with a verdict like `No change in performance`
or `Performance has regressed`, so a slowdown shows up in CI before it ships.

## What you learned

- bcinr benchmarks are Criterion files in `bcinr-bench/benches/`, registered with
  `harness = false` and a matching `[[bench]]` `name`.
- `criterion::black_box` is mandatory — without it the optimizer constant-folds
  your kernel away.
- Benchmarking the same kernel across best/typical/worst inputs is how you
  *prove* branchless latency: overlapping intervals mean data-independent timing.
- `--save-baseline` / `--baseline` turn Criterion into a regression gate.

## Next steps

- [Tutorial 9: Property-testing a branchless kernel](./tutorial-9.md) — pair every
  benchmark with a correctness proof.
- [Tutorial 1: Your first branchless select](./tutorial-1.md) — revisit the kernel
  you just measured, now that you can see its flat latency.
