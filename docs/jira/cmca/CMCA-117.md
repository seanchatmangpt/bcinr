# CMCA-117: differential test's envelope classification is miscalibrated, possibly stale post-CMCA-107, and CI never actually fuzzes it

**Type:** Test Rigor
**Priority:** Medium-High

## Summary

Four compounding gaps in `crates/bcinr-cmca/tests/differential.rs`'s
CMCA-104 envelope classification and `DIFFERENTIAL_TOLERANCE`:

1. `masses_tied`'s `1e-9` absolute-difference threshold is calibrated to
   "float noise," not to the Q16.16 grid (resolution ≈1.5e-5) the test is
   actually comparing against — masses differing by e.g. `1e-6` (1000x
   `masses_tied`'s threshold, but 15x finer than Q16.16 resolution) round to
   bit-identical fixed-point values while staying distinct in f64, and are
   NOT flagged as "tied," so they get the tight tolerance check applied to a
   case that's legitimately on a rounding knife-edge.
2. `DIFFERENTIAL_TOLERANCE = 0.40`'s measured basis (0.3309/0.3211 across
   ~6800 comparisons) was set in the same commit as the CMCA-107 kappa-guard
   fix, with no git evidence the measurement was re-run *after* the fix. The
   crate's own new regression test asserts a post-fix diff of `< 0.05` on the
   reconstructed case — an order of magnitude tighter — suggesting 0.40 may
   now be a stale, overly loose bound left over from before the fix.
3. `inside_envelope` still has only the original two conditions (spread,
   tied-mass); CMCA-107's own ticket explicitly anticipated a possible third
   "near a decision boundary" condition if the root cause were something
   other than the kappa gate, and named two other candidate discrete-boundary
   sources (the MWU admission threshold, the dwell-time gate) that were never
   ruled out or root-caused.
4. `PROPTEST_CASES` defaults to `1` and is never overridden by any CI
   workflow for this specific test — the "~6800 comparisons" measurement
   that derived the tolerance was a one-off manual invocation, not something
   CI reproduces or re-validates on any subsequent run. Real per-run coverage
   is 1 fresh case plus the ~5 pinned regression seeds.

## Context

Found by adversarial review of the CMCA-104 differential-tolerance fix.

- `masses_tied`: `crates/bcinr-cmca/tests/differential.rs:510-516` (`1e-9`
  threshold, justifying comment reasons about "float noise," never mentions
  the Q16.16 grid the surrounding envelope logic elsewhere does reason about
  correctly, e.g. `ESCORT_DYNAMIC_RANGE_LIMIT`'s `2^-16` underflow
  threshold).
- Tolerance staleness: confirmed via `git log`/`git show` that
  `DIFFERENTIAL_TOLERANCE = 0.40` and the CMCA-107 kappa-guard fix landed in
  one squashed commit (`835c7945`); `docs/jira/cmca/CMCA-107.md` cites the
  same 0.3309/0.3211 figures as pre-fix context.
- Classification incompleteness: `tests/differential.rs`'s `inside_envelope`
  (search for that name) is unchanged at two conditions;
  `docs/jira/cmca/CMCA-107.md`'s acceptance criteria explicitly named the
  third-condition possibility for the "(b) inherent boundary artifact"
  branch, which wasn't taken (branch (a), a real fix, was) — but the two
  other named candidate boundary sources (MWU admission threshold, dwell-time
  gate) were never independently checked.
- CI coverage: confirmed by grep across `.github/workflows/ci.yml`,
  `post-pr13-audit.yml`, `exhaustive.yml`,
  `repair-post-integration-verification-v2.yml` — none set `PROPTEST_CASES`
  for the `differential` test specifically (other suites in this repo do use
  `PROPTEST_CASES=4096`, confirming the mechanism is known and just not
  applied here).

## Acceptance Criteria

- [ ] Re-derive `masses_tied`'s threshold from the Q16.16 grid resolution
      (or the same `2^-16`-scale reasoning `ESCORT_DYNAMIC_RANGE_LIMIT`
      already uses), not an arbitrary `1e-9`.
- [ ] Re-run the ~6800-comparison measurement (or equivalent) with
      `PROPTEST_CASES` set high enough for real coverage, AFTER the CMCA-107
      fix, and re-derive `DIFFERENTIAL_TOLERANCE` from that fresh
      measurement — tighten it if the post-fix population's real maximum
      disagreement is now much smaller than 0.33.
- [ ] Investigate CMCA-107.md's two named-but-unchecked candidate boundary
      sources (MWU gradient-descent admission threshold, dwell-time
      mode-switch gate) — rule them in or out as additional discrete-boundary
      divergence sources, and add a third `inside_envelope` condition if
      warranted.
- [ ] Set `PROPTEST_CASES` for the `differential` test in CI to a real,
      meaningful value (matching this repo's own `PROPTEST_CASES=4096`
      precedent used elsewhere), so the tolerance this ticket derives is
      actually re-validated by every CI run, not just measured once
      manually.
- [ ] Correct the `differential.proptest-regressions` comment's "defect
      class... pinned" framing to be precise about what's actually verified
      (one reconstructed structural analog, not the original unrecoverable
      seed) — or strengthen the regression coverage until the stronger claim
      is true.

## Files likely touched

- `crates/bcinr-cmca/tests/differential.rs`
- `crates/bcinr-cmca/src/generated_profile.rs`
- `.github/workflows/ci.yml`
- `crates/bcinr-cmca/tests/differential.proptest-regressions`

## Related

- CMCA-104, CMCA-107 (this ticket extends both)
