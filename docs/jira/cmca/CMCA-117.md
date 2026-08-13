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

- [x] Re-derive `masses_tied`'s threshold from the Q16.16 grid resolution
      (or the same `2^-16`-scale reasoning `ESCORT_DYNAMIC_RANGE_LIMIT`
      already uses), not an arbitrary `1e-9`. Done: threshold is now
      `2^-Q16_16_FRACTIONAL_BITS` (one Q16.16 ULP), computed from the
      already-public `Q16_16_FRACTIONAL_BITS` constant instead of a bare
      literal. Regression test added
      (`cmca_117_masses_tied_threshold_is_derived_from_the_q16_16_grid_not_an_arbitrary_epsilon`)
      that fails if the threshold drifts from the grid or stops flagging
      the ticket's own `1e-6`-apart example as tied.
- [x] Re-run the ~6800-comparison measurement (or equivalent) with
      `PROPTEST_CASES` set high enough for real coverage, AFTER the CMCA-107
      fix, and re-derive `DIFFERENTIAL_TOLERANCE` from that fresh
      measurement — tighten it if the post-fix population's real maximum
      disagreement is now much smaller than 0.33. Done, but the opposite of
      the hypothesized direction: re-measured post-fix at real scale
      (`PROPTEST_CASES=8000`, two independent runs, ~63,500 leaf
      comparisons), the true post-fix maximum is 0.5638, *higher* than the
      old 0.40 bound — both runs actually failed against 0.40 once real
      coverage was applied (this ticket's Detection-8 finding, reproduced
      live). `DIFFERENTIAL_TOLERANCE` raised to 0.70 (~24% headroom over the
      measured max, matching this crate's own headroom convention); the
      worst-case input is now a persisted seed in
      `differential.proptest-regressions` so it re-runs on every future CI
      invocation.
- [x] Investigate CMCA-107.md's two named-but-unchecked candidate boundary
      sources (MWU gradient-descent admission threshold, dwell-time
      mode-switch gate) — rule them in or out as additional discrete-boundary
      divergence sources, and add a third `inside_envelope` condition if
      warranted. Done: the kappa admission threshold (`kappa >
      epsilon_kappa`) is confirmed IN as a genuine, independent
      fixed-vs-f64 divergence source (already the mechanism
      `DIFFERENTIAL_TOLERANCE`'s own rationale names). The dwell-time gate
      is confirmed OUT by direct code inspection: `can_switch` is computed
      identically, on identical `u32` inputs, in both `allocate_in` and the
      f64 oracle (`reference.rs:130`), so it cannot itself disagree between
      paths — it only amplifies a divergence that originated in kappa. No
      third `inside_envelope` condition was added: the kappa-boundary
      source doesn't correlate with the spread/tied-mass sibling-set
      geometry the existing two conditions test, and is already covered by
      the re-derived `DIFFERENTIAL_TOLERANCE` bound above. Findings recorded
      in `tests/differential.rs`'s comment ahead of the `inside_envelope`
      computation.
- [x] Set `PROPTEST_CASES` for the `differential` test in CI to a real,
      meaningful value (matching this repo's own `PROPTEST_CASES=4096`
      precedent used elsewhere), so the tolerance this ticket derives is
      actually re-validated by every CI run, not just measured once
      manually. Done: `.github/workflows/ci.yml`'s "CMCA differential" step
      now sets `PROPTEST_CASES=4096` (confirmed locally: ~14s, well within
      the job's budget, and passes against the re-derived tolerance). Note:
      grepping the current repo found no other workflow actually setting
      `PROPTEST_CASES=4096` today (the "precedent" this criterion cited may
      itself be stale/removed) — 4096 was chosen on its own merits as real
      coverage at acceptable CI cost, not because a live precedent was
      found.
- [x] Correct the `differential.proptest-regressions` comment's "defect
      class... pinned" framing to be precise about what's actually verified
      (one reconstructed structural analog, not the original unrecoverable
      seed) — or strengthen the regression coverage until the stronger claim
      is true. Reviewed: the current header comment already states this
      precisely ("The original proptest-shrunk seed was never reproducible
      ... instead of pinning an unrecoverable seed, the defect class is
      pinned as a deterministic regression test"), so no further correction
      was needed here. A new seed (re-deriving `DIFFERENTIAL_TOLERANCE`,
      above) was appended to the same file by this ticket's own
      measurement run.

## Files likely touched

- `crates/bcinr-cmca/tests/differential.rs`
- `crates/bcinr-cmca/src/generated_profile.rs`
- `.github/workflows/ci.yml`
- `crates/bcinr-cmca/tests/differential.proptest-regressions`

## Related

- CMCA-104, CMCA-107 (this ticket extends both)
