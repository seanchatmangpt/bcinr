# Fixture Scanner Note — `tests/fixtures/pre_migration/case_studies.rs`

**Verdict: scanner false positive.** This fixture file is not, and cannot be,
compiled or executed as a Rust `#[test]` target.

## Evidence

1. **Cargo test-target discovery.** `crates/bcinr-cmca/Cargo.toml` declares no
   `[[test]]` sections at all, so Cargo falls back to its default rule: every
   `*.rs` file placed directly under `tests/` becomes its own integration-test
   binary; files nested inside subdirectories of `tests/` (e.g.
   `tests/fixtures/pre_migration/`) are never auto-discovered as test targets.
   `pre_migration/case_studies.rs` sits two directories below `tests/`, so it
   is excluded by construction.

2. **`cargo test -- --list` binary set.** Running the test binary list for
   `bcinr-cmca` shows compiled/attempted targets `case_studies`,
   `differential`, `consumer_correspondence`, etc. — one binary per top-level
   `tests/*.rs` file. There is no `pre_migration::case_studies` or similar
   target; the frozen fixture never appears as a compilation unit.

3. **Only consumer of the fixture path.** The sole place
   `tests/fixtures/pre_migration/case_studies.rs` is referenced anywhere in
   `crates/bcinr-cmca` is `tests/consumer_correspondence.rs`, which opens it
   with `std::fs::read_to_string(path)` and greps `from_value_bits(N)` tokens
   out of the raw text — i.e. it is read as a **data file**, not `include!`d
   or `mod`-declared as Rust source. `consumer_correspondence.rs`'s own
   header further notes that binary currently fails to build for an unrelated
   reason (`src/lrc.rs` API mismatch), which is orthogonal to this question.

## Conclusion

The LADDER finding that flagged this frozen fixture under the same
`test_rejection_invariance` check used for the **live**
`tests/case_studies.rs` conflated a fixture path with a live test path. The
live `tests/case_studies.rs` genuinely fails to compile against the new
sealed API (`AllocationOutcome`/`.is_refused()`/etc. — real, tracked
separately) and does need reconciling. The frozen
`tests/fixtures/pre_migration/case_studies.rs` does not compile, does not
run, and is not itself a test — it is inert reference text per
`PRE_MIGRATION_BASELINE.md`, and per that document must not be hand-edited to
match the new API regardless.

No fix is required here. Future ladder runs should treat any hit under
`tests/fixtures/**` as non-actionable unless it can name an actual compiled
target that includes it.
