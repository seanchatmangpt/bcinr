# How to Run Only the Library Tests Instead of the Full Suite

**Goal:** Get a fast inner feedback loop by running just the `bcinr-logic` unit tests, skipping the slow workspace-wide build of benchmarks, integration targets, and the other crates.

**Prerequisites:** A working checkout. The full suite (`cargo make test`) compiles every target in every crate with `--all-features --all-targets`, which is thorough but slow during development.

## Steps

1. For the tightest loop, run the library target of a single crate only. `--lib` skips `benches/`, `tests/`, and `examples/`:

   ```bash
   cargo test -p bcinr-logic --lib
   ```

2. To run one module or one test by substring, pass a filter. To see `println!`/`dbg!` output, add `-- --nocapture`:

   ```bash
   cargo test -p bcinr-logic --lib mask::         # everything under mask.rs
   cargo test -p bcinr-logic test_select_u32 -- --nocapture
   ```

3. To make failures deterministic and readable, run single-threaded:

   ```bash
   cargo test -p bcinr-logic --lib -- --test-threads=1
   ```

4. Doc examples (the ```` ```rust ```` blocks in `///` comments) are *not* run by `--lib`. Test them separately when you touch public-API docs:

   ```bash
   cargo test -p bcinr-logic --doc
   ```

5. When you are ready to validate everything before pushing, switch back to the full workspace run, which also builds benches and integration targets:

   ```bash
   cargo make test   # cargo test --workspace --all-features --all-targets
   ```

## Verify it worked

- The `--lib` run finishes in seconds, not minutes, and prints `test result: ok.` with a count that matches only the in-module `#[cfg(test)]` tests.
- A deliberately broken assertion fails fast under the filtered command, confirming you are exercising the code you think you are.

See also: [Write a counterfactual-mutant test](./guide-9.md), [Add a Criterion benchmark and read the report](./guide-6.md).
