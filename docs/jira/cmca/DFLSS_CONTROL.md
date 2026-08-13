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

## Round-3 residual-risk close-out

The four lowest-RPN round-2 tickets (CMCA-118/119/120/121) were initially accepted
as residual risk in this round's first pass, then closed in a follow-up pass on the
`feat/powl-soundness-cli` branch. Each ticket's acceptance criteria are checked off
in its own `.md` except for one explicitly-descoped item per ticket, allowed by that
ticket's own AC text (not silently dropped):

- **CMCA-120** (RPN 192, commit `d141ddce`) — Added
  `cmca_120_multi_child_node_kappa_exceeds_epsilon_and_weights_actually_update` in
  `crates/bcinr-cmca/tests/differential.rs`, asserting the positive (admit) branch
  of `compute_kappa` against the f64 oracle on node 1 of the CMCA-107 fixture (three
  direct children). Also hoisted `mass_pow` out of `allocate_in`'s `v` loop via a
  new `fixed_pow_per_node` helper, cutting redundant `fixed_pow` calls from 256 to
  32 per `allocate_in` call with no behavior change. Descoped: the AC's benchmark
  item — no harness in `crates/bcinr-bench` exercises `compute_kappa`/`allocate_in`/
  `allocate`, and the ticket's own text permits skipping rather than inventing one.
- **CMCA-119** (RPN 135, commit `d674189b`) — Added
  `crates/bcinr-cmca/examples/basic_allocation.rs` (runnable via `cargo run
  --example basic_allocation -p bcinr-cmca --features std`), a "Which entry point do
  I want? (CMCA-119)" navigation section in `lib.rs`'s top-level docs, and
  hand-written `Display` (always available) plus `#[cfg(feature = "std")]`
  `std::error::Error` impls for the five `...Refusal` enums the ticket's AC and
  "Files likely touched" section named: `StabilityRefusal`,
  `LensSelectionRefusal`, `AllocationRefusal`, `CertificationRefusal`,
  `EscortRefusal`. Descoped: `HierarchyRefusal`/`CascadeRefusal` and the enums in
  `observatory.rs`/`proposal.rs`/`reference_escort.rs` were not touched — out of
  scope per the ticket's own named file list.
- **CMCA-121** (RPN 84, commit `d49c3e7f`) — Extended
  `dwell_time_lock_holds_switch_until_tau_d_then_switches` in
  `crates/bcinr-cmca/tests/dwell_time_hysteresis.rs` with a second phase that
  re-biases payoffs after the first switch lands at `t=tau_d`, drives `t` through
  `last_switch_t + tau_d`, and asserts the second switch back to mode 0 lands
  exactly at the new deadline with `last_switch_t` updated — proving the spec's
  repeated-switching bound, not just a single instance. Also corrected the doc
  comment to state the kappa=0 fix applies only at the root, naming node 1's
  residual kappa=0 degeneracy explicitly rather than implying it was resolved.
  Descoped: the tree-extension alternative (giving node 1 itself genuinely nonzero
  kappa) was not implemented — the ticket's AC allows the doc-comment fix as
  satisfying an explicit either/or, and this ticket's own file states that
  exercising node 1's MWU weight-update path remains open for a future ticket.
- **CMCA-118** (RPN 32, commit `1344156e`) — Replaced `generator.py`'s
  `line.split('#')[0]` comment-stripping with a new `strip_ttl_comment` helper that
  tracks whether the scan position is inside an open `"..."` literal (honoring
  `\"` escapes) before treating `#` as a comment start. Added
  `crates/bcinr-cmca/tests/generator_ttl_comment_stripping.rs`, a Rust test
  shelling out to `generator.py` as a subprocess, covering: a literal containing
  `#` surviving intact, a genuine trailing comment still stripped, the
  multiline-literal/language-tag rejections still firing, and both real
  `ontology/*.ttl` files still generating successfully. Descoped: broader
  `generator.py` test-suite coverage beyond this bug's regression test — the
  ticket's AC explicitly leaves this unchecked as out of scope for its minimum bar.

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
| CMCA-120 untested `compute_kappa` admit branch | `cmca_120_multi_child_node_kappa_exceeds_epsilon_and_weights_actually_update` (`tests/differential.rs`) |
| CMCA-119 onboarding gap (no example, no unified error trait) | `examples/basic_allocation.rs` (`cargo run --example basic_allocation`); `Display`/`Error` impls on `StabilityRefusal`, `LensSelectionRefusal`, `AllocationRefusal`, `CertificationRefusal`, `EscortRefusal` |
| CMCA-121 single-switch-only dwell-time coverage | Second-phase assertions in `dwell_time_lock_holds_switch_until_tau_d_then_switches` (`tests/dwell_time_hysteresis.rs`) |
| CMCA-118 TTL `#`-in-literal corruption | `generator_ttl_comment_stripping.rs` (subprocess test against `generator.py`'s `strip_ttl_comment`) |

All of the above run under the existing gate: `cargo build -p bcinr-cmca --features
std`, `cargo test -p bcinr-cmca --features std`, `cargo test -p bcinr-cmca --features
alloc`, `cargo clippy -p bcinr-cmca --features std -- -D warnings`, `cargo fmt -p
bcinr-cmca -- --check` — all green as of this close-out (verified on
`feat/powl-soundness-cli`, commit `1344156e` and its `docs(cmca)` follow-up).
