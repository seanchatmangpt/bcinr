# How to Add a Criterion Benchmark and Read Its Report

**Goal:** Measure the latency of a primitive with statistical rigor and inspect the result, so you can spot regressions and confirm low timing variance.

**Prerequisites:** The standalone `bcinr-bench` crate was removed from the workspace; benches now live in each crate's `benches/` directory (for example [`bcinr-powl`](../../../crates/bcinr-powl/benches/) and [`bcinr-pddl`](../../../crates/bcinr-pddl/benches/)). `bcinr-logic` has no bench files yet, so put new ones in `crates/bcinr-logic/benches/` and enable its `bench` feature. Benchmarks use Criterion with `harness = false`. `jq` is optional, for the text report. Latency is reported by [Criterion](https://bheisler.github.io/criterion.rs/book/), which warms up and samples each function many times.

## Steps

1. Add a benchmark function to a bench file in `crates/bcinr-logic/benches/` (create it if it does not exist yet). Import the kernel from `bcinr_logic`:

   ```rust
   use bcinr_logic::mask::min_u32;
   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn bench_min(c: &mut Criterion) {
       c.bench_function("mask::min_u32", |b| {
           b.iter(|| min_u32(black_box(10), black_box(20)))
       });
   }
   ```

   Wrap inputs in `black_box` so the optimizer cannot constant-fold the call away.

2. Register the function in this file's `criterion_group!`/`criterion_main!` (each `benches/*.rs` is its own harness):

   ```rust
   criterion_group!(benches, bench_mask, bench_int, bench_fix, bench_min);
   criterion_main!(benches);
   ```

   If you instead create a *new* `benches/<name>.rs` file, add a matching `[[bench]]` entry with `harness = false` and `required-features = ["bench"]` to [`crates/bcinr-logic/Cargo.toml`](../../../crates/bcinr-logic/Cargo.toml).

3. Run the whole bench suite, or just one harness, or filter to one function inside it:

   ```bash
   cargo make bench                                   # all benches (cargo bench --all-features)
   cargo bench -p bcinr-logic --features bench --bench <name>     # one harness
   cargo bench -p bcinr-logic --features bench --bench <name> -- min_u32   # one function
   ```

4. Read the result. The terminal prints the estimated time and any change vs. the last run. For full detail open the generated HTML, or produce the text table:

   ```bash
   xdg-open target/criterion/report/index.html   # per-function plots, PDF of the distribution
   cargo make bench-report                        # markdown table (needs jq)
   ```

## Verify it worked

- `target/criterion/mask::min_u32/` now exists and `report/index.html` shows the timing distribution.
- The estimate is in the expected ns range and the distribution is tight (a narrow PDF / small confidence interval) — wide spread signals a data-dependent path worth auditing per [guide-1](./guide-1.md).
- Re-running reports a "Change" line; a large positive change flags a regression.

See also: [Verify a function compiles to branchless code](./guide-1.md), [Guarantee WCET](./guarantee-wcet.md), [Run only the library tests](./guide-5.md).
