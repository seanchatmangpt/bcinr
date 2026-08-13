# bcinr-cmca tickets

Tracked gaps found during the `bcinr-cmca` lens-selection fix and production-readiness
audit (2026-08-13). Each ticket is a standalone file; open each for full context,
acceptance criteria, and files likely touched.

| ID | Title | Type | Priority | Status |
|----|-------|------|----------|--------|
| [CMCA-101](CMCA-101.md) | Dwell-time hysteresis (`dom_mode`/`prev_mode`) computed but never wired to output | Tech Debt | Medium | **Done** |
| [CMCA-102](CMCA-102.md) | Authority chain (`CertifiedLearning` et al.) held unexported pending Hoare-logic verification | Tech Debt / Verification | Low | **Done (Branch B)** |
| [CMCA-103](CMCA-103.md) | `allocate_in` silently accepts out-of-range `q` when `proof=None` | Bug | High | **Done** |
| [CMCA-104](CMCA-104.md) | `DIFFERENTIAL_TOLERANCE=0.22` placeholder doesn't gate CI | Test Rigor / Correctness | High | **Done** |
| [CMCA-105](CMCA-105.md) | No CI check that `generated/` matches ontology source | Infra / CI Hardening | High | **Partially done** — generator fixed, CI step still deferred, see below |
| [CMCA-106](CMCA-106.md) | `power()`'s error bound unmeasured beyond `\|q\|=16` | Correctness / Numerical Verification | High | **Done** |
| [CMCA-107](CMCA-107.md) | Fixed-vs-f64 `allocate()` disagreement of ~0.65 | Bug | High | **Done — root-caused and fixed** |

**6 of 7 tickets fully closed, 1 partially closed with an honest remainder.** Verified
end-to-end this session: `cargo test -p bcinr-cmca --features std` → **192 passed, 0
failed**; `cargo clippy -p bcinr-cmca --features std --all-targets -- -D warnings` →
clean; `cargo check --workspace` → clean; `cargo publish -p bcinr-cmca --dry-run` →
correctly refused.

## What shipped, and how the tickets interacted with each other

This work was done by parallel agents in isolated git worktrees, each self-verifying
before returning a diff, then integrated one at a time into the real tree with a full
test run after each — because several of these fixes touch the same code
(`allocator/mod.rs`) and genuinely interact:

- **CMCA-107's divergence-guard fix changed real allocator behavior** (which weight
  updates get admitted), and this **broke CMCA-101's originally-written test** once both
  were combined: CMCA-101's test tree was a flat star (root + 7 leaf children), and
  `compute_kappa` (CMCA-107's fix) is identically zero for any node whose direct children
  are all leaves — a real mathematical property, not a bug in either fix. CMCA-101's test
  was changed to a genuine two-level tree to make the weight update it's testing actually
  admissible. **Neither ticket was wrong; they were both right about different things, and
  combining them surfaced a real edge case in the test setup, not the production code.**
- **CMCA-103's `q_err`/`price_err`/`eta_err` enforcement fix** was scoped narrower than
  its own ticket's default framing after investigation found some `has_error` components
  are legitimately proof-gated (documented by an existing Chicago-TDD test) and some are
  not — the fix distinguishes them rather than making everything unconditional.
- **CMCA-104's tolerance fix directly found CMCA-107** — tightening a placeholder
  assertion surfaced a real, previously-invisible bug. This is the system working as
  intended: real rigor finds real bugs.

## Per-ticket notes

- **CMCA-101**: `tests/dwell_time_hysteresis.rs`, 2 new tests. Correction to the
  ticket's own premise: the mechanism isn't fully inert (`switch_wanted`/`can_switch`
  gate the weight update that feeds subsequent calls) — it's just never exercised by any
  real caller, exactly as the ticket's title says.
- **CMCA-102**: Branch B only (explicit `#[doc(hidden)]` + doc comments naming the real
  blocker). Branch A (an actual Hoare-logic proof) remains open, out of reach here.
- **CMCA-103**: fixed, with one flagged follow-up — `hostile_mutants.rs`'s mutant_5 kill
  test can no longer distinguish that mutant through the public API post-fix; left with a
  comment rather than a unilateral mutation-harness redesign.
- **CMCA-104**: implemented; directly found CMCA-107 as a side effect of doing its job
  correctly.
- **CMCA-105**: `generator.py`'s default output path fixed; the `macro_rules!` drift
  reconciled with a confident git-history answer (dead code, correctly removed from the
  committed tree, generator just never caught up — fixed the generator, not the tree).
  **Still open**: `stability_profile.rs` has no ontology/generator backing at all and
  never did (present hand-written since the crate's first commit) — bringing it under
  real generation needs domain knowledge this session doesn't have, so the CI diff-check
  step itself was correctly not added yet. Also corrected: the ticket's original
  characterization of this file's header as a false "ggen sync... DO NOT EDIT" claim was
  itself inaccurate — that exact phrasing belongs to a different file
  (`generated_profile.rs`), not `stability_profile.rs`.
- **CMCA-106**: full-domain sweep implemented and verified independently — the numbers
  in the ticket match a direct re-run exactly.
- **CMCA-107**: root-caused (missing divergence guard, confirmed via the tell-tale
  previously-unused `_epsilon_kappa` parameter) and fixed. Original failing seed
  unrecoverable; the defect class is pinned via a deterministic regression test instead.

## Verification

Every change above was independently re-verified in this session, not taken on an
agent's word alone: diffs read line-by-line before applying, `cargo build`/`test`/
`clippy` re-run after each integration step (not just once at the end), and the two
places integration actually broke something (CMCA-101 × CMCA-107 interaction, and
lib.rs/allocator.rs clobbering from stale-worktree base commits) were caught by that
per-step verification and fixed before moving on.
