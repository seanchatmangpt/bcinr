# Analysis of `mutant_1` in `hostile_mutants.rs`

Based on the inspection of `crates/bcinr-cmca/tests/hostile_mutants.rs`, here is the documentation for `mutant_1`:

### Mathematical Law Broken
**The per-measure canonical-mixing law.**
The law states that *each of the K measures must independently weight the allocation*.

### Mutant Implementation
`mutant_1` corrupts this process by forcing (pinning) the `k_actual` index to measure `0` for every `k`. By doing so, it collapses the required independent per-measure mixing computation into a single-measure result.

### Expected Outcome
The repository enforces a strict mutant-kill protocol (`AGENTS.md` SS19) which rejects bare `assert_ne!` checks. To prove that the specific law violation is handled, the test must capture the exact deterministic consequence of the corruption.

For `mutant_1`, the expected outcome is that the `allocate()` function will output a very specific corrupted allocation array:
`[8528, 7445, 7506, 7506, 7506, 7506, 12033, 7506]` (defined as `WRONG_M1_MEASURE_COLLAPSE`).

The test `kill_mutant_1_single_measure_collapse` evaluates this by explicitly asserting equality (`assert_eq!`) against this exact array, proving that the exact named corruption was caught instead of a generalized or unrelated divergence.
