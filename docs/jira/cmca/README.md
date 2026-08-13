# bcinr-cmca tickets

Tracked gaps found during the `bcinr-cmca` lens-selection fix, production-readiness
audit, and a maximum-concurrency adversarial code review (2026-08-13). Each ticket is
a standalone file; open each for full context, acceptance criteria, and files likely
touched.

## Round 1 (CMCA-101–107): the original hardening pass

| ID | Title | Type | Priority | Status |
|----|-------|------|----------|--------|
| [CMCA-101](CMCA-101.md) | Dwell-time hysteresis (`dom_mode`/`prev_mode`) computed but never wired to output | Tech Debt | Medium | **Done** — see CMCA-121 for a residual test gap found on re-review |
| [CMCA-102](CMCA-102.md) | Authority chain (`CertifiedLearning` et al.) held unexported pending Hoare-logic verification | Tech Debt / Verification | Low | **Done (Branch B)** — see CMCA-114, its enforcement was found to be non-existent |
| [CMCA-103](CMCA-103.md) | `allocate_in` silently accepts out-of-range `q` when `proof=None` | Bug | High | **Done** — see CMCA-110/122, the same bug class was found still open for `eta`/`numeric_has_err` |
| [CMCA-104](CMCA-104.md) | `DIFFERENTIAL_TOLERANCE=0.22` placeholder doesn't gate CI | Test Rigor / Correctness | High | **Done** — see CMCA-117 for calibration/coverage gaps found on re-review |
| [CMCA-105](CMCA-105.md) | No CI check that `generated/` matches ontology source | Infra / CI Hardening | High | **Partially done** — see CMCA-118/113 for concrete bugs found in the generator and its downstream constants |
| [CMCA-106](CMCA-106.md) | `power()`'s error bound unmeasured beyond `\|q\|=16` | Correctness / Numerical Verification | High | **Done** — see CMCA-109/116, a real bug and a real gap found adjacent to this |
| [CMCA-107](CMCA-107.md) | Fixed-vs-f64 `allocate()` disagreement of ~0.65 | Bug | High | **Done** — see CMCA-112/120, the fix has its own precision/coverage gaps |

## Round 2 (CMCA-108–122): adversarial review of round 1's own output

Nine agents reviewed the round-1 work concurrently and adversarially — each
instructed explicitly to find real defects, not to confirm the work was good.
30 distinct findings surfaced; the ones below are the real, ticket-worthy ones
(minor/no-finding items are folded into the ticket bodies above or omitted).

| ID | Title | Type | Priority |
|----|-------|------|----------|
| [CMCA-108](CMCA-108.md) | `allocate`/`allocate_single_lens` hard-locked to compile-time N=8/K=4/Q=4 | Bug / Design Gap | **Critical — blocking for autofde-lab** |
| [CMCA-109](CMCA-109.md) | `power(0, negative q)` silently returns saturated-max tagged as "no fault" | Bug | **Critical** |
| [CMCA-110](CMCA-110.md) | `eta_err` has no upper bound; `numeric_has_err` still proof-gated — CMCA-103's bug class left half-fixed | Bug | **Critical** |
| [CMCA-111](CMCA-111.md) | `allocate_single_lens`'s "reproduces the blend" claim is false under real (non-zero-payoff) MWU updates | Bug / Doc Overclaim | High |
| [CMCA-112](CMCA-112.md) | `compute_kappa`'s `fixed_pow` saturates for in-domain masses; `0/0` diverges from f64 fail-safe | Bug | High |
| [CMCA-113](CMCA-113.md) | `stability_profile.rs`'s production constants are opaque and untraceable | Design Gap | High |
| [CMCA-114](CMCA-114.md) | `#[doc(hidden)]` gives zero compile-time enforcement; authority chain is a reachable rubber-stamp | Bug / Security Design Gap | High |
| [CMCA-115](CMCA-115.md) | `allocation_receipt` has no cyclic-parent check; digest oversold as tamper-evident | Bug / Security Design Gap | High |
| [CMCA-116](CMCA-116.md) | `escort_distribution` gives zero signal for ~36% error results | Bug / Design Gap | High |
| [CMCA-117](CMCA-117.md) | Differential test's envelope classification miscalibrated; CI never actually fuzzes it | Test Rigor | Medium-High |
| [CMCA-118](CMCA-118.md) | `generator.py` silently corrupts TTL literals containing `#`; zero test coverage | Bug | Medium |
| [CMCA-119](CMCA-119.md) | No runnable example; 5 overlapping entry points; no unified error trait | Design Gap / Onboarding | Medium |
| [CMCA-120](CMCA-120.md) | CMCA-107 regression test only covers the negative path; 8x redundant `fixed_pow` computation | Test Gap / Performance | Medium |
| [CMCA-121](CMCA-121.md) | Dwell-time test proves single-switch only; residual kappa=0 node in the "fixed" tree | Test Gap / Documentation | Low-Medium |
| [CMCA-122](CMCA-122.md) | `eta_err` reports the wrong refusal reason when combined with `price_err`; `clip()` now dead code | Bug / Test Gap | Medium |

## Round 3 (FMEA/RCA-driven fixes)

`docs/jira/cmca/FMEA.md` scored the ten open round-2 tickets by `RPN = Severity x
Occurrence x Detection`; the six highest-ranked were fixed this round, each with a
root cause and a regression test or compile-time signal (full detail in
`DFLSS_CONTROL.md`):

| ID | RPN | Status | Commit |
|----|-----|--------|--------|
| [CMCA-114](CMCA-114.md) | 576 | **Done** — `#[deprecated]` compile-time signal, trybuild UI regression test | `cc4ab3ad` |
| [CMCA-111](CMCA-111.md) | 504 | **Done** — corrected blend-identity precondition doc, non-degenerate regression test | `1900ea8d` |
| [CMCA-113](CMCA-113.md) | 432 | **Done** — constant provenance documented, contraction-invariant regression tests | `b03c139a` |
| [CMCA-116](CMCA-116.md) | 336 | **Done** — `PathConfidence` signal added to `escort_distribution` | `7c849f92` |
| [CMCA-122](CMCA-122.md) | 210 | **Done** — dedicated refusal reasons for `eta_err`/`price_err` | `26692f1a` |
| [CMCA-117](CMCA-117.md) | 200 | **Done** — `PROPTEST_CASES: "4096"` in CI, grid-derived `masses_tied` threshold | `ef3896dc` |
| [CMCA-120](CMCA-120.md) | 192 | **Done** — positive-path kappa regression test, `mass_pow` hoisted out of the `v` loop (benchmark AC descoped, no harness exercises this path) | `d141ddce` |
| [CMCA-119](CMCA-119.md) | 135 | **Done** — runnable example, entry-point nav doc, `Display`/`Error` on 5 named `...Refusal` enums (other enums out of scope) | `d674189b` |
| [CMCA-121](CMCA-121.md) | 84 | **Done** — second-switch dwell-time phase added, node-1 residual kappa=0 doc-comment corrected (tree-extension alternative descoped) | `d49c3e7f` |
| [CMCA-118](CMCA-118.md) | 32 | **Done** — literal-aware TTL comment stripping, subprocess regression test (broader generator test-suite AC left as follow-up) | `1344156e` |

Round 3's four lowest-RPN tickets (CMCA-118/119/120/121) were accepted as residual
risk in this round's initial close-out, then closed in a follow-up pass — see
`docs/jira/cmca/DFLSS_CONTROL.md` for the full Control-phase summary: what was
measured, what was fixed (all ten tickets now), and the specific test/CI gates now
guarding each fixed failure mode.

## The pattern worth naming

Every round-1 ticket marked "Done" had at least one real gap found on
adversarial re-review — not because the round-1 work was sloppy (it was
independently verified, tested, and green at the time), but because "done"
for a ticket scoped narrowly to one defect is not the same as "the
surrounding code is now fully correct." CMCA-103's own fix left the identical
bug class open one flag away (`eta_err`'s missing upper bound, CMCA-110).
CMCA-107's own fix has a precision gap in the exact function it added
(`fixed_pow`, CMCA-112). CMCA-106's own sweep found a real bug adjacent to
what it was measuring (`power(0, ...)`, CMCA-109) without noticing it, because
it wasn't in the swept grid. This is the value of adversarial review as a
distinct pass from implementation-and-verification: the second pass is
looking for what the first pass's own frame of reference couldn't see.

**Highest priority for the stated goal (autofde-lab production integration):**
CMCA-108 (compile-time size lock-in) must be resolved or explicitly
acknowledged before integration work starts — it determines whether the
certified allocation path is usable at all. CMCA-109/110 are real, silent
correctness bugs reachable via the common `proof=None` call path autofde-lab
would use by default. CMCA-113/114 are both about production-readiness gaps
in exactly the "certified"/"stability" machinery a real production consumer
would reasonably assume is load-bearing.
