# bcinr-cmca: DfLSS Control-Phase Summary (Round 3)

**Scope note:** this is a documentation artifact recording what was measured, what was
fixed, and what test/CI gates now exist. It is **not** a formal Six Sigma capability
study — no Cpk or sigma-level figures appear anywhere below. `bcinr-cmca` has no
production telemetry stream; a real capability calculation needs a sampled process
output over time, which does not exist for this crate. Any Cpk/sigma number attached
to this round would be fabricated. This document reports FMEA scores (a
structured-judgment risk ranking, not a statistical capability metric) and real,
named test/CI evidence instead.

## What was measured

`docs/jira/cmca/FMEA.md` scored the ten open round-2 tickets (CMCA-111, 113,
114, 116–122) via `RPN = Severity x Occurrence x Detection` (each 1-10), from the
perspective of a production consumer (autofde-lab-shaped: imports
`bcinr_cmca::allocator::*`, calls public entry points with real inputs, trusts typed
refusals/doc claims as ground truth). Ranked results:

| Rank | Ticket | RPN | Failure mode |
|------|--------|-----|--------------|
| 1 | CMCA-114 | 576 | "Certified" authority chain publicly constructible with zero verification, zero compiler signal |
| 2 | CMCA-111 | 504 | `allocate_single_lens`'s documented blend identity false for real (non-zero-payoff) calls |
| 3 | CMCA-113 | 432 | Stability-profile constants hardcoded with no derivation or regression check |
| 4 | CMCA-116 | 336 | `escort_distribution` gives identical `Ok` shape for ~0.5% and ~36% relative error |
| 5 | CMCA-122 | 210 | Combined `eta_err`+`price_err` masks the real refusal reason |
| 6 | CMCA-117 | 200 | Differential-test tolerance/coverage stale relative to CMCA-107 |
| 7 | CMCA-120 | 192 | `compute_kappa`'s admit (positive) branch untested |
| 8 | CMCA-119 | 135 | No runnable example touching the real allocation entry points |
| 9 | CMCA-121 | 84 | Dwell-time test proves only a single switch |
| 10 | CMCA-118 | 32 | `generator.py` `#`-in-literal truncation, confirmed not live |

## What was fixed this round

Six tickets closed, in RPN order, each with a real root cause and a regression test
or compile-time signal:

- **CMCA-114** (commit `cc4ab3ad`) — Root cause: `#[doc(hidden)]` (CMCA-102) changed
  only rustdoc visibility, not Rust reachability; all 7 authority-chain constructors
  stayed `pub` with no validation. Fix: `#[deprecated]` on the six unconditional
  `admit_*` constructors, giving a real, `-D warnings`-tripping compiler signal.
  Verified by a `trybuild` UI fixture,
  `crates/bcinr-cmca/tests/ui/fail_deprecated_admit_learning.rs`, which asserts the
  deprecated-use diagnostic actually fires under `#![deny(deprecated)]`.
- **CMCA-111** (commit `1900ea8d`) — Root cause: `allocate_single_lens`'s doc claimed
  the blend identity holds unconditionally, but it only holds for the
  post-MWU-update `weights` snapshot, not a caller-supplied raw array; the shipped
  test used all-zero payoffs, which make the MWU update a no-op and can't expose the
  divergence. Fix: corrected doc comment stating the precondition explicitly, plus
  `blend_identity_requires_the_post_mwu_update_weights_snapshot` in
  `crates/bcinr-cmca/tests/single_lens_allocation.rs`, a non-degenerate regression
  test with real, differentiated payoffs.
- **CMCA-113** (commit `b03c139a`) — Root cause: `stability_profile.rs`'s
  `certificate_digest`, `minimum_dwell_rounds`, and the gain/weight/margin trio had
  no derivation or provenance comment, unlike the ontology-generated constants
  elsewhere in the crate. Fix: added provenance documentation and
  `crates/bcinr-cmca/tests/stability_profile_invariants.rs`, with
  `gain_matrix_weight_vector_margin_satisfy_the_contraction_inequality`,
  `mode_dwell_rounds_min_is_a_positive_gate`, and
  `certificate_digest_round_trip_matches_and_a_single_flipped_byte_is_caught` as
  regression tests a future "plausible but wrong" tuning edit would now fail.
- **CMCA-116** (commit `7c849f92`) — Root cause: `escort_distribution`'s only domain
  check was a flat `|q|` magnitude cutoff; the ~36% relative error near `|q|=16` was
  disclosed in module docs but never surfaced as a runtime-checkable signal. Fix:
  added a `PathConfidence` signal to the return type so callers can distinguish
  high- from low-error fractional-`q` results, with regression tests in
  `crates/bcinr-cmca/src/escort.rs`'s own `#[cfg(test)]` module.
- **CMCA-122** (commit `26692f1a`) — Root cause: the `err_val` priority chain in
  `allocate_in` was never updated when CMCA-103 made `eta_err`/`price_err`
  independently, unconditionally refusing conditions — `eta_err` still rode along in
  the `lr_err`/`beta_err` OR-group, masking `price_err`'s dedicated
  `PriceGainUnsafe` reason. Fix: dedicated refusal reasons for each, verified by
  `eta_err_and_price_err_co_occurring_reports_price_gain_unsafe_not_learning_rate`
  and `eta_err_alone_reports_its_own_dedicated_reason` in
  `crates/bcinr-cmca/tests/hostile_mutants.rs`.
- **CMCA-117** (commit `ef3896dc`) — Root cause (5-whys): the differential test's
  safety net was decorative because nothing forced CI to run it at the scale its own
  `DIFFERENTIAL_TOLERANCE` was calibrated against — `PROPTEST_CASES` defaulted to 1
  and no workflow overrode it for this suite, while other suites in the same repo
  already used this pattern. Fix: `PROPTEST_CASES: "4096"` added to
  `.github/workflows/ci.yml` for the differential suite, and the `masses_tied`
  threshold was re-derived from the Q16.16 grid instead of an arbitrary epsilon,
  verified by
  `cmca_117_masses_tied_threshold_is_derived_from_the_q16_16_grid_not_an_arbitrary_epsilon`
  in `crates/bcinr-cmca/tests/differential.rs`.

## What remains as accepted residual risk

The four lowest-RPN round-2 tickets were reviewed but not fixed this round. Each is
accepted as residual risk with the rationale below (see each ticket's `.md` for full
detail):

- **CMCA-120** (RPN 192) — `compute_kappa`'s positive (admit) branch, where
  `kappa > epsilon_kappa` and weights actually update, is exercised by the broader
  `differential.rs` proptest sweep but not asserted against a deterministic f64
  oracle in a targeted regression test. Accepted because CMCA-117's
  `PROPTEST_CASES: "4096"` fix (this round) substantially raises the sweep's real
  per-run coverage of this exact branch, narrowing the gap the ticket originally
  scored against.
- **CMCA-119** (RPN 135) — No runnable example touches `allocate`/
  `allocate_single_lens`/`escort_distribution`, and 9+ `...Refusal` enums have no
  shared `Error`/`Display` impl. Accepted as onboarding friction, not a correctness
  or security defect — a careful integrator reading module docs can still get it
  right; this is real toil, not a live bug.
- **CMCA-121** (RPN 84) — The dwell-time hysteresis test proves only a single mode
  switch, not the spec's bound over a sequence of switches; a stale-baseline bug
  that only manifests on switch #2 would go uncaught. Accepted because the ticket
  itself states no such bug is confirmed to exist — this is a coverage gap on an
  otherwise-correct `t=0` baseline, not a known defect.
- **CMCA-118** (RPN 32) — `generator.py`'s `#`-stripping would silently corrupt a
  TTL literal containing `#`, but is confirmed not live: no current ontology file
  (`cmca-rdf.ttl`, `generalization.ttl`) uses `#` inside a literal. Accepted as
  latent, not live, and lowest-priority in the FMEA ranking.

## Control mechanisms going forward

Regression of the six fixed-this-round failure modes is now caught by:

| Failure mode (ticket) | Test/CI gate |
|---|---|
| CMCA-114 rubber-stamp authority chain | `crates/bcinr-cmca/tests/ui/fail_deprecated_admit_learning.rs` (trybuild UI test asserting `#[deny(deprecated)]` fires) |
| CMCA-111 false blend-identity claim | `blend_identity_requires_the_post_mwu_update_weights_snapshot` (`tests/single_lens_allocation.rs`) |
| CMCA-113 opaque stability constants | `gain_matrix_weight_vector_margin_satisfy_the_contraction_inequality`, `mode_dwell_rounds_min_is_a_positive_gate`, `certificate_digest_round_trip_matches_and_a_single_flipped_byte_is_caught` (`tests/stability_profile_invariants.rs`) |
| CMCA-116 undifferentiated high/low-error results | `PathConfidence`-asserting `#[test]`s in `src/escort.rs` |
| CMCA-122 misdirected refusal reason | `eta_err_and_price_err_co_occurring_reports_price_gain_unsafe_not_learning_rate`, `eta_err_alone_reports_its_own_dedicated_reason` (`tests/hostile_mutants.rs`) |
| CMCA-117 decorative differential safety net | `PROPTEST_CASES: "4096"` in `.github/workflows/ci.yml`; `cmca_117_masses_tied_threshold_is_derived_from_the_q16_16_grid_not_an_arbitrary_epsilon` (`tests/differential.rs`) |

All of the above run under the existing gate: `cargo build -p bcinr-cmca --features
std`, `cargo test -p bcinr-cmca --features std`, `cargo test -p bcinr-cmca --features
alloc`, `cargo clippy -p bcinr-cmca --features std -- -D warnings`, `cargo fmt -p
bcinr-cmca -- --check` — all green as of this round's close-out.
