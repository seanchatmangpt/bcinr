# FMEA: bcinr-cmca round-2 remaining tickets (CMCA-111, 113, 114, 116–122)

Failure Mode and Effects Analysis of the ten open round-2 tickets, scored from the
perspective of a production consumer shaped like autofde-lab: a caller that imports
`bcinr_cmca::allocator::*` externally, calls the public entry points with real
(non-degenerate) inputs, and trusts typed refusals / doc claims as ground truth
rather than re-reading the crate's internals.

Scoring: `RPN = Severity x Occurrence x Detection`, each 1-10. Severity = how bad the
effect is if hit. Occurrence = how likely realistic (non-adversarial) usage hits it.
Detection = how likely current tests/CI would catch it before it reaches a caller
(10 = would never be caught).

## Ranked table

| Rank | Ticket | Sev | Occ | Det | RPN | One-line failure mode |
|------|--------|-----|-----|-----|-----|------------------------|
| 1 | CMCA-114 | 9 | 8 | 8 | 576 | "Certified" authority chain is publicly constructible with zero verification and zero compiler signal; autofde-lab can build a fake certificate and the type system won't stop it. |
| 2 | CMCA-111 | 8 | 7 | 9 | 504 | `allocate_single_lens`'s documented identity is false for any real (non-zero-payoff) call; nothing detects the divergence because the only test uses the one input class where it can't appear. |
| 3 | CMCA-113 | 8 | 6 | 9 | 432 | `CERTIFICATE_DIGEST`/`MODE_DWELL_ROUNDS_MIN`/gain constants are hardcoded with no derivation; a "reasonable" production tuning edit changes allocator admission with no test able to flag it as wrong. |
| 4 | CMCA-116 | 7 | 6 | 8 | 336 | `escort_distribution` returns `Ok` with identical shape for ~0.5% and ~36% relative error; a caller has no signal to distinguish trustworthy from untrustworthy output. |
| 5 | CMCA-122 | 6 | 5 | 7 | 210 | Combined `eta_err`+`price_err` reports `LearningRateOutsideEnvelope`, masking the real `PriceGainUnsafe` cause; a caller debugging off the refusal variant is sent to the wrong subsystem. |
| 6 | CMCA-117 | 5 | 5 | 8 | 200 | Differential tolerance (0.40) may be stale post-CMCA-107, CI runs exactly 1 proptest case for this suite, and `masses_tied`'s 1e-9 threshold isn't grid-calibrated — the safety net for numeric drift is mostly decorative. |
| 7 | CMCA-120 | 4 | 8 | 6 | 192 | `compute_kappa`'s "admit" branch (kappa > epsilon, weights actually update) is untested — only the always-zero negative path is asserted; a broken positive-path update could ship silently. |
| 8 | CMCA-119 | 3 | 9 | 5 | 135 | No runnable example touches `allocate`/`allocate_single_lens`/`escort_distribution`; a new integrator (autofde-lab) burns real time picking the wrong entry point or missing the N=8/K=4/Q=4 lock-in until it breaks at compile/runtime. |
| 9 | CMCA-121 | 3 | 4 | 7 | 84 | Dwell-time test proves only a single switch; a stale-baseline bug that only manifests on switch #2 would not be caught. |
| 10 | CMCA-118 | 2 | 2 | 8 | 32 | `generator.py` truncates TTL literals containing `#`, but no current ontology file uses `#` in a literal — latent, not live. |

## Per-ticket rationale

### CMCA-114 — RPN 576 (Sev 9 x Occ 8 x Det 8)
`#[doc(hidden)]` changes only rustdoc visibility, not Rust reachability: all 7
authority-chain types remain `pub` inside `pub mod allocator` (`mod.rs:784-931`), and
6 of 7 constructors (`admit_learning`, `admit_control_state`, `admit_certificate`,
`admit_envelope`, `admit_outcome`, `admit_selection_only`) unconditionally return
`Self { ... }` with no validation — confirmed by direct inspection of the constructor
bodies.
- **Severity 9**: this is the gate that decides whether `allocate_in` admits a call
  at all (`digest_err` feeds directly into the `err_val` priority chain, mod.rs
  ~1469-1502). A caller can construct a passing "certified" proof out of thin air and
  the allocator has no way to tell the difference — this is a security-shaped defect
  (rubber-stamped authorization), not a cosmetic one. Not a 10 only because it
  requires the caller to actively construct fake proof objects rather than silently
  corrupting a value nobody asked for.
- **Occurrence 8**: 14 of the crate's own integration test files already import these
  types via the exact external-crate path (`bcinr_cmca::allocator::{...}`) autofde-lab
  would use — this is demonstrated-reachable today, not hypothetical, and any
  integrator following the crate's own test files as a usage guide will construct
  these types the zero-verification way by default.
- **Detection 8**: no compiler warning, no lint, no doc entry flags this; nothing in
  CI or `cargo doc` output currently signals "do not use" to a downstream consumer.
  Not a 9-10 because a careful reader of the module docs (which do explain intended
  gating) could still notice, and the crate's own tests exercise the constructors
  directly, so an internal maintainer diffing test behavior has some chance.

### CMCA-111 — RPN 504 (Sev 8 x Occ 7 x Det 9)
`allocate_single_lens` (mod.rs:2119-2226) takes the caller's raw `weights` and skips
the multiplicative MWU update `allocate_in` applies internally (mod.rs:1650-1679)
before computing `pi_kq`; the doc comment nonetheless states the blend identity holds
unconditionally.
- **Severity 8**: a caller relying on the documented identity (`sum_kq lambda * per-lens
  == pi_combined`) to validate or decompose an allocation gets silently wrong numbers
  for every call with real (non-zero, differentiated) payoffs where the divergence
  guard admits an update — this is a correctness bug wrapped in an explicit doc
  claim, which is worse than an undocumented gap because a caller has affirmative
  reason to trust it. Not a 9-10 because the output is still a valid probability
  distribution, just not the claimed decomposition — no crash, no NaN, no unsound
  state.
- **Occurrence 7**: any call with non-zero, non-degenerate payoffs where
  `kappa > epsilon_kappa` (the common case for real allocator invocations, not an
  edge case) triggers the divergence; the shipped test's all-zero-payoff scenario is
  the special case, not the norm.
- **Detection 9**: the only existing test
  (`blend_equals_the_lambda_weighted_sum_of_single_lens_results`) is structurally
  incapable of exercising this — all-zero payoffs make the MWU update a no-op by
  construction (`exp(0)==1`), so the test passes regardless of whether the doc claim
  is true for real inputs. No other test touches this identity.

### CMCA-113 — RPN 432 (Sev 8 x Occ 6 x Det 9)
`stability_profile.rs` constants (`certificate_digest`, `minimum_dwell_rounds: 461`,
gain/weight/margin values) are hardcoded with no derivation, formula, or named-owner
policy comment — confirmed by reading the file directly; contrast with
`RDF_INPUT_DIGEST`/`GENERATOR_SOURCE_DIGEST` which are real computed sha256 values.
- **Severity 8**: these are load-bearing production gates — `certificate_digest` is
  compared byte-for-byte to authorize allocation, `minimum_dwell_rounds` gates
  dwell-time refusal, the gain/weight/margin trio feeds a stability inequality. A
  wrong edit during "production tuning" (explicitly imminent per the ticket) changes
  allocator behavior with no test able to say whether the new value is correct or a
  typo. Not a 9-10 because the values are currently internally consistent and shipped
  — the risk is a future edit, not a present defect in behavior.
- **Occurrence 6**: this only fires when someone actually edits these constants for
  deployment tuning — a real, near-term, but not every-call event.
- **Detection 9**: there is no test, lint, or CI check that would catch a "plausible
  but wrong" edit to any of these constants; unlike the ontology-generated constants,
  there's no re-derivation path to diff against.

### CMCA-116 — RPN 336 (Sev 7 x Occ 6 x Det 8)
`escort_distribution`'s only domain check is a flat `|q| > MAX_LENS_MAGNITUDE` cutoff
(escort.rs:198); the module's own doc (escort.rs:71-90) states ~36-37% relative error
near `|q|=16` and explicitly declines to gate on it.
- **Severity 7**: a caller gets a well-typed `Ok(...)` result that is quietly ~36%
  off from the true value with no accompanying signal — this is the "silently wrong
  answer" failure shape, but bounded to numeric magnitude (not a security or
  logic-branch failure) and the crate's own docs do disclose the bound (just not at
  the call site), so a diligent reader has a chance to know. Not higher because the
  crate is honest about the number in module docs, even though the API doesn't
  surface it.
- **Occurrence 6**: any fractional-`q` call in the `|q|` mid-to-high range hits this;
  this is a real, non-adversarial usage pattern for a caller sweeping `q`, not a
  contrived corner case.
- **Detection 8**: no test asserts a caller can distinguish high- from low-error
  results; the error is measured (CMCA-106) but that measurement never becomes a
  runtime-checkable signal.

### CMCA-122 — RPN 210 (Sev 6 x Occ 5 x Det 7)
The `err_val` priority chain (mod.rs, `allocate_in`) buckets `eta_err` into the
`LearningRateOutsideEnvelope` reason alongside `lr_err`/`beta_err`, and `price_err`
has no dedicated branch, surfacing only via the fallback `PriceGainUnsafe` — so when
both `eta_err` and `price_err` are true, the reported reason is
`LearningRateOutsideEnvelope`, masking `PriceGainUnsafe`.
- **Severity 6**: this misdirects debugging effort (wrong subsystem) rather than
  producing wrong allocator output — the call still correctly refuses, so no bad
  numbers reach the caller, just a wrong diagnostic. Real but bounded impact.
- **Occurrence 5**: requires both conditions co-occurring, which is plausible under
  real stress/degenerate inputs but not the common single-condition refusal case.
- **Detection 7**: no regression test constructs this joint condition, so it would
  ship unnoticed, but the failure is self-limiting (it's a `Debug`-only enum
  mislabel, easy to spot once someone hits it and reads the surrounding code).

### CMCA-117 — RPN 200 (Sev 5 x Occ 5 x Det 8)
Four compounding gaps: `masses_tied`'s `1e-9` threshold isn't calibrated to the
Q16.16 grid; `DIFFERENTIAL_TOLERANCE=0.40`'s measurement basis predates the CMCA-107
fix (same squashed commit `835c7945`, confirmed via `git log`); `inside_envelope`
still has only 2 of the 3 conditions CMCA-107 itself anticipated; `PROPTEST_CASES`
defaults to 1 and no CI workflow overrides it for this suite (confirmed by grep
across all 4 workflow files).
- **Severity 5**: this is a test-rigor gap, not a code defect reaching a caller
  directly — the risk is that a real allocator regression could land undetected
  because the differential test's safety margin is stale/uncalibrated and its
  effective coverage is ~1 fresh case per CI run. Moderate, indirect severity.
- **Occurrence 5**: every CI run is affected (1 proptest case is the norm, not the
  exception), but the downstream harm only materializes if a real regression is
  introduced while this gap persists — conditional on a second event.
- **Detection 8**: this is a rare case where the ticket itself grades the detection
  mechanism directly — confirmed by grep that no workflow sets `PROPTEST_CASES` for
  this suite while other suites in the same repo do (so it's a known, applied
  pattern just missing here), meaning the gap would not be caught by CI as currently
  configured.

### CMCA-120 — RPN 192 (Sev 4 x Occ 8 x Det 6)
The CMCA-107 regression test only exercises the negative (kappa=0, update withheld)
path; no test asserts the positive path (kappa > epsilon_kappa, weights actually
update and match the f64 oracle). Separately, `compute_kappa`'s `mass_pow` is
recomputed 8x redundantly per call (256 `fixed_pow` calls vs. an achievable 32) —
this is a real perf-only finding, not folded into the FMEA score since it has no
correctness effect.
- **Severity 4**: if the positive (admit) branch of the MWU update were broken, the
  effect is wrong allocator weights on every call that actually updates — but
  "broken" here would need to be a fresh regression, not a currently-known live bug
  (the code path is exercised by other tests, e.g. differential.rs's broader
  proptest sweep, just not asserted against a deterministic oracle in this specific
  regression test). Scored as moderate, not high, because no ticket or review claims
  this branch is currently wrong — this is a coverage gap, not a confirmed defect.
- **Occurrence 8**: the positive-update branch is the common case in real usage (most
  multi-child nodes with real payoffs have kappa > epsilon_kappa) — this is the
  normal path, not an edge case.
- **Detection 6**: the broader `differential.rs` proptest sweep would likely catch a
  sufficiently large regression via its f64-vs-fixed tolerance check (moderating
  detection down from a pure 8-9), but that sweep runs at `PROPTEST_CASES=1`
  effectively (see CMCA-117), so real per-run coverage of this exact branch is
  weak-to-absent for a deterministic, targeted check.

### CMCA-119 — RPN 135 (Sev 3 x Occ 9 x Det 5)
No `examples/` directory exists (deleted this session); the sole doc-tested example
walks `observatory::evaluate_calibration`, never `allocate`/`allocate_single_lens`/
`escort_distribution`; 5 overlapping entry points and 9+ independent `...Refusal`
enums with no shared `Error`/`Display` impl (confirmed: `Debug`-only, no
`std::error::Error`).
- **Severity 3**: onboarding friction and integration slowdown, not a correctness or
  security defect — a careful integrator (reading module docs) can still get it
  right, just slower and with more risk of picking a suboptimal entry point.
- **Occurrence 9**: every new integrator (autofde-lab, by name) hits this
  immediately on first contact with the crate — this is the highest-occurrence
  finding in the set precisely because it's not conditional on any particular call
  pattern.
- **Detection 5**: this is self-evident to any human reading the crate (not a hidden
  runtime defect), so "detection" in the FMEA sense (would tooling/tests catch it
  before it reaches a caller) is moderate — a caller discovers it immediately by
  trying to use the crate, which is a form of self-detection, but no CI/lint flags
  missing examples or ungrouped error types.

### CMCA-121 — RPN 84 (Sev 3 x Occ 4 x Det 7)
`dwell_time_lock_holds_switch_until_tau_d_then_switches` drives exactly one mode
transition and stops; the spec (`stability_proof_draft.md:101-106`) describes a bound
over a *sequence* of switches. Separately, node 1 in the "fixed" test tree still has
kappa=0 (same degeneracy class as CMCA-107, one level down) — doc-comment overclaim,
not a functional bug.
- **Severity 3**: a stale-baseline bug on switch #2 (the hypothesized failure mode)
  would produce incorrect dwell-time enforcement on repeated mode switches — real if
  it existed, but this ticket does not claim such a bug is confirmed to exist, only
  that the test couldn't catch it if it did. Scored on the coverage gap, not a
  confirmed defect.
- **Occurrence 4**: repeated switching is a real production pattern (mode-switching
  systems rarely switch exactly once) but requires the underlying bug to actually
  exist, which is unconfirmed.
- **Detection 7**: no test currently exercises a second switch at all, so a bug there
  would go fully undetected; not higher because the single-switch test does establish
  correct `t=0` baseline handling, giving partial confidence by extension.

### CMCA-118 — RPN 32 (Sev 2 x Occ 2 x Det 8)
`generator.py`'s `parse_ttl` strips everything after `#` per line without tracking
quote state, corrupting any TTL literal containing `#` — reproduced directly
(`"value # not a comment"` -> `'"value'`, silently, no exception). Confirmed no
current ontology file (`cmca-rdf.ttl`, `generalization.ttl`) uses `#` inside a
literal.
- **Severity 2**: if triggered, this silently corrupts a generated production
  constant with no error — that's a serious failure shape in the abstract, but it is
  confirmed **not live** against any current input, so today's actual risk to a
  caller is near zero; scored low because "would be bad if it happened" is separated
  from "is currently happening or imminent."
- **Occurrence 2**: requires someone to add a `#`-containing literal to the ontology
  source in the future — possible but not indicated as planned, and not something
  autofde-lab integration triggers.
- **Detection 8**: zero test coverage exists for `generator.py` at all (confirmed by
  grep), so if/when this is triggered, nothing would catch it before it reaches
  compiled constants.

## Recommended fix order (by RPN)

CMCA-114, CMCA-111, CMCA-113 dominate the ranking and share a theme: production
consumers trusting something the crate calls "certified" or "the blend identity"
that isn't actually enforced or true under real inputs. These three should be
prioritized ahead of the numeric-error-visibility (CMCA-116) and
diagnostic-accuracy (CMCA-122) findings, which are real but lower-severity. The
test-rigor cluster (CMCA-117, 120, 121) and the onboarding/latent-bug tail
(CMCA-119, 118) can follow.
