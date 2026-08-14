# Chatman Ecosystem ERRC Refresh — 2026-08-13

This document extends (does not replace) `/Users/sac/gymact/docs/2026-08-12-chatman-ecosystem-errc-8020-1000x.md`
("the original doc"). It synthesizes 7 parallel repo surveys run this session — bcinr, mfw, ggen,
ggen-marketplace, ggen-legacy, gymact, autofde-lab — against the original doc's method and claims.
`praxis` was **not surveyed this round**; every figure the original doc reported for `praxis` is
carried forward unchanged and unverified by this refresh, not re-measured.

Each survey ran independently (medium breadth, single pass), so per-repo confidence here is lower
than the original doc's single-repo-deep-dive standard. Where a survey said UNVERIFIED, this
synthesis keeps it UNVERIFIED — it does not resolve any open question optimistically.

## 1. What's changed since 2026-08-12

Real, dated, checkable deltas from this session, by repo:

- **bcinr**: 34 commits on `feat/powl-soundness-cli` since 2026-08-12, two real lines of work —
  (a) CMCA hardening (tickets CMCA-108 through CMCA-122+, `cmca_rank_cli`, `cmca_allocate_cli`,
  `allocation_receipt.rs`, a DSPy-based third-party plausibility/tamper-magnitude validation
  layer, `docs/cmca/` now 29 files) and (b) commit `813cbe2b`,
  `feat(bcinr-powl): add soundness_cli stdin/stdout JSON bridge to WfNet::check_soundness`. One
  uncommitted change remains in the working tree (`crates/bcinr-pddl/src/validate.rs`, +122
  lines). Latest commit's subject references *ggen-legacy* reconstitution — cross-repo analysis
  content appears to be living in bcinr's doc tree, flagged as a doc-placement issue, not a
  capability.
- **mfw**: **no new commits since 2026-08-12** (last commit still 2026-07-29, matching the
  original doc exactly). The real change is in what got *read*, not what got *written*:
  `tools/cmca-generator/VERIFICATION_REPORT.md` (a producer-side report the original doc did not
  cite) shows mfw's CMCA generator's two previously-blocking issues resolved and zero import
  coupling to bcinr, confirmed by a real `grep`/`cmp`/Q16.16 bit-for-bit check. This is new
  *evidence* about mfw's CMCA producer role, not new mfw work this session.
- **autofde-lab**: real, live-verified feature landed this session — `match_solvers(ranked=True)`
  now calls out to bcinr's compiled `cmca_rank_cli` binary end-to-end (12/12 tests pass, direct
  subprocess invocation reproduced, commit sequence `571e834f` → `7dbdc48c` → `9cfbfdf7` →
  `a6dd0523`, with `9cfbfdf7` a real self-caught bug fix: the first cut of the feature "always hit
  the refusal and silently fell back to unranked order," per the code's own comment). Two new
  governance docs added, both explicitly self-labeled as non-execution (no repo state mutated).
- **ggen**: ran a real ERRC Eliminate pass on itself this session/period (`2c579a616`, "ERRC
  eliminate — dead crates + superseded packs, RCA-corrected") — with a documented self-correction:
  the elimination survey initially missed live consumers for two pack candidates, caught before
  merge. Three more ERRC-named commits in the same window.
- **ggen-marketplace**: new work today (`c543bb8`, 2026-08-13 09:55), including
  `feat(autofde-semantic-registry-pack)` and two real bug fixes to `github-actions-pack` gate 020
  (previously "never fired against real consumer data"). Whether any of this maps to the other six
  repos' work this session is UNVERIFIED — no cross-referencing commit found.
- **ggen-legacy**: 7 new commits after the original doc's cutoff, most recent `cbf4e5e` (2026-08-12
  17:05) and `9940eb1` ("add real Groq-backed disposition-proposer example").
- **gymact**: continuous same-day commits through 2026-08-13 17:48 — new gyms, an FMEA+RCA round
  fixing 2 real bugs, closure of 6 gyms to "full two-proof standing," and `dspy_agent` wrapping
  read-only capabilities as ReAct tools (`31c00f8`), which lines up with `dspy_verifier.py`'s Aug
  10 mtime in bcinr's own CMCA validation-layer work — plausible same-family activity, not
  confirmed as coordinated.

**Does the original doc's central finding still hold?** Yes, unchanged. The original finding was
that *mass* (file/LOC count) and *authority* (production-actuation capability) point in opposite
directions across the portfolio — the biggest repos deny themselves actuation authority, and no
repo claims the `autofde`/`BRCE.DO` production-consequence path. Every survey this round that
checked authority language (bcinr, mfw, ggen, ggen-legacy, gymact, autofde-lab) found the same
self-denial pattern, restated in each repo's own doctrine, not contradicted anywhere. mfw's report
adds one real nuance (see §4) — a bounded, self-claimed actuation gate (`broker`) that is
narrower than but sits between autofde-lab's full self-denial and the still-nonexistent `autofde`
repo — but this refines rather than reverses the finding: no repo surveyed this round claims to
*be* `autofde` or to own `BRCE.DO`.

## 2. Updated mass table

The original doc's headline mass claim: `autofde-lab` + `praxis` ≈ 75% of an ~11-repo,
~40,206-file total, using raw/uncorrected file counts. This round's surveys, where they redid the
count, corrected several of those raw counts by excluding agent-worktree contamination
(`.claude/worktrees/*`) that inflates naive `find` output by 3–12x. `praxis` was not re-measured
this round; its original-doc figure (13,763) is carried forward as-is, unverified here.

| Repo | This session's real count (method) | vs. original doc | Notes |
|---|---|---|---|
| bcinr | 689 `.rs` files, ~169,540 LOC | doc had 962 (stale/likely uncorrected) | new crate `bcinr-cmca` (53 files) didn't exist in the 08-09 snapshot |
| mfw | 1,233 files (850 `.rs`, 181 `.py`, 238 `.ttl`, 94 hand-authored `.lean`, `.lake` vendor cache excluded) | doc had 1,593 | same order of magnitude; methodology differs, not a real change (no new commits) |
| ggen | 2,196 real files (`.rs`; excl. `.claude/worktrees/` which held 11,141 stale duplicate files, 83% of raw count) | doc had 2,538 | no dramatic mass growth; raw naive count would have overstated by ~6x |
| ggen-marketplace | 1,802 files, ~289,670 LOC | not in original doc's table | new coverage, not a revision |
| ggen-legacy | 275 files (22 `.rs`, 55 `.py`, 182 `.md`), 4,766 Rust LOC | doc had 65 | 4.2x growth; likely a narrower original filter, confirmed via commit history that real growth occurred |
| gymact | ~327 hand-authored files (265 Python, 18 Rust — new subsystem `rust/crown`+`rust/protocol_gym`, 43 `.md`, 19 `.ttl`); excludes `ggen/` vendored packs, 21 stale worktrees | doc had 240 | +36% real growth; Rust subsystem is entirely new since the original doc recorded 0 Rust in gymact |
| autofde-lab | 1,358 real Python files (~270,000 LOC), excluding 41 `.claude/` worktrees (120,212 of 121,788 naive-counted files were worktree scratch) | doc had 16,488 | ~12x correction; the original doc's own caveat ("true hand-authored figure... was not separated out here") is confirmed accurate by this direct check |
| praxis | not surveyed this round | doc had 13,763 | carried forward unchanged, unverified this round |

**Net effect on the 75% mass claim**: not recomputed as a portfolio percentage this round (no
survey re-derived the original doc's ~40,206-file denominator, and praxis wasn't touched). What
*is* now directly confirmed, where the original doc had flagged it as an open caveat: `autofde-lab`'s
true hand-authored mass is roughly 12x smaller than its raw count (1,358 vs. 16,488), which
weakens the specific 16,488-file figure underpinning the original 75% claim without this refresh
being able to state a corrected portfolio percentage — that would require re-running the same
correction on `praxis` and re-summing the denominator, which is out of scope here. UNVERIFIED:
whether the corrected ~75% claim still holds numerically once all repos are counted the same way.

## 3. Cross-repo redundancy findings

Concrete overlaps found this session, each cited to its source survey:

- **bcinr ↔ ggen** (from ggen's survey): ggen vendors `bcinr-pddl` and `bcinr-mfw-ir` as
  workspace crates, copied (not registry-pulled) from bcinr, deliberately, because the published
  crates.io release (26.6.26) lacked an API (`Pddl8Error::PlanningFailed`/`.into_result()`) that
  ggen needed. This is a real, documented content divergence between two on-disk copies of the
  same code — a live drift risk, not resolved by either repo's doctrine.
- **bcinr ↔ ggen-legacy** (from ggen-legacy's survey): ggen-legacy uses `bcinr-pddl` directly as
  its planner boundary (`docs/v26.8.1/10-system/12-workspace-crate-map.md`,
  `planning/v26.8.1/README.md`) rather than reimplementing PDDL parsing — a genuine, previously
  uncaught dependency edge (the original doc's redundancy pass only checked ggen-legacy against
  `ggen`/`ggen-create`, not bcinr).
- **mfw ↔ bcinr** (from mfw's survey): mfw's CMCA generator (`tools/cmca-generator/generator.py`)
  and bcinr's `bcinr-cmca` crate work over the same ontology (`cmca-rdf.ttl`, byte-identical
  post-header) and the same Q16.16 value space — confirmed producer/consumer, not duplicated
  capability; the verification report explicitly proves zero import coupling.
- **ggen-marketplace ↔ mfw** (from ggen-marketplace's survey): `mfw-pack` generates a Rust catalog
  of mfw's structure, including `mfw:standingAuthority` per surface, directly modeled on
  `~/mfw/AGENTS.md`'s 10-stage cycle — a tight coupling (catalog-of vs. owns-the-runtime), not
  duplication.
- **gymact ↔ bcinr-powl** (from gymact's survey): both independently implement BLAKE3-based
  receipt/evidence primitives (RFC8785 JCS canonicalization + BLAKE3-256 hash chains in gymact;
  "POWL runtime + receipt verification (BLAKE3)" in bcinr) with **no dependency relationship found
  in either repo's doctrine** — a real, unresolved overlap-candidate at the primitive level,
  flagged but not adjudicated as redundant-vs-convergent-by-coincidence.
- **No overlap found** this round between bcinr and ggen-marketplace (name-grep only, not deep);
  between gymact and mfw/ggen family (gymact's own doctrine explicitly denies overlap, e.g.
  `AllowListAuthorityResolver` "is not a substitute for BRCE," and a documented "ggen-ownership
  boundary" commit); between autofde-lab and mfw/ggen-marketplace/ggen-legacy this session.

## 4. Updated ERRC grid

**Eliminate**

- *(ggen survey)* Confirmed real, already executed by the repo itself: `cpmp`,
  `openapi-cnv-reflect` (zero reverse deps via `cargo tree -i`), `genesis-types-v2`/
  `genesis-core-v2`, two wasm4pm packs — deleted in commit `2c579a616`, with a documented
  self-correction (two candidates initially misjudged safe to delete, caught before merge).
- *(bcinr survey, new finding)* The CMCA-doc stream inside bcinr currently contains content that
  is actually about `ggen-legacy` (latest commit `061106fc`, "re-verify ggen-legacy
  reconstitution..."). Candidate Eliminate/relocate: move cross-repo analysis out of bcinr's doc
  tree into the repo it's actually about.
- *(ggen survey)* `just sync`/`just sync-dry` convenience recipes remain known-broken
  (`--audit` isn't a live flag) — a narrow, named, still-open Eliminate-or-fix candidate, unchanged
  since 2026-07-17.

**Reduce**

- *(autofde-lab survey)* `.claude/worktrees/` scratch state across at least bcinr (10), ggen (worktree
  contamination inflated a naive count 6x), gymact (21), and autofde-lab (41, 26 GB) is real,
  measured bloat inflating naive file counts by 3–12x repo to repo. Not a new finding in kind (the
  original doc likely hit some of this) but newly quantified per-repo this round — a concrete
  Reduce target if any of these repos want accurate self-measurement going forward.
- *(ggen-marketplace survey)* 3 stray `.claude/worktrees/wf_*` duplicating pack content, same
  pattern.

**Raise**

- *(mfw survey)* mfw's admission/broker/receipt/replay claim (`AGENTS.md`: "the broker is the only
  lawful actuation path") is real, in-repo, and doctrinally hedged — a candidate for being raised
  in ecosystem-wide visibility, since it's a genuine middle rung between autofde-lab's full
  self-denial and the still-missing `autofde` repo, previously under-cited (the original doc only
  reached mfw's doctrine via a quote *from* autofde-lab's `CLAUDE.md`, not mfw's own docs).
- *(gymact survey)* gymact's receipt/evidence machinery (RFC8785 JCS + BLAKE3-256 hash-chained
  ledgers, SQLite WAL + restart verification) is the densest real receipt implementation among the
  smaller repos surveyed — candidate to raise as a shared primitive rather than let it stay
  independently reinvented alongside bcinr-powl's parallel BLAKE3 work (see §3).

**Create — does anything move the missing fourth stage (`autofde`/`BRCE.DO`) forward?**

The original doc flagged its own headline Create claim (a missing fourth stage owning
`BRCE.DO`/production actuation) as resting on secondhand doctrine, and a follow-up found it
overstated. This round's surveys do not reverse that: **no survey found `autofde` as a repo, and
no survey found any repo claiming the `BRCE.DO` actuation verb for itself.** The closest new
evidence is from ggen-marketplace: the `consequence-ir-pack` (v0.2.0) manifest and ontology
explicitly define vocabulary for "bounded authority, BRCE-only DO," and an `AuthorityGrant` class
— but per that survey's own framing, this is "a semantic vocabulary artifact *about* authority/
BRCE.DO... not an actuation path itself." It is candidate prior art for a future Create stage's
ontology, not the stage itself. Net: **the Raise item ("the missing fourth stage") is still open
and still missing** — this round found a naming/ontology candidate for it, not an implementation,
and that distinction should be preserved rather than rounded up.

## 5. Honest gaps

What this refresh could not verify, given 7 parallel medium-breadth surveys instead of the
original doc's single-repo depth:

- **praxis was not surveyed this round at all.** Every praxis figure here is carried forward
  unchanged from the original doc, not re-measured, not re-verified.
- **No survey re-derived the original doc's ~40,206-file portfolio denominator or the ~75% mass
  claim as a recomputed percentage.** The autofde-lab correction (16,488 → 1,358 real files) is
  confirmed directly; its effect on the portfolio-wide percentage is not computed here.
- **Cross-repo timing coordination is asserted nowhere with certainty.** gymact's `dspy_agent`
  work (Aug 10) and bcinr's `dspy_verifier.py`-adjacent CMCA validation layer are only noted as
  plausibly-related by date proximity — not confirmed as the same effort or even aware of each
  other.
- **The bcinr↔gymact BLAKE3 receipt-primitive overlap (§3) is flagged, not adjudicated.** Whether
  this is deliberate convergence, wasted duplicate effort, or two genuinely different receipt
  formats that happen to share a hash function was not checked at the format level by either
  survey.
- **mfw's "fixed-point API mismatch" P0-blocker**, referenced in this session's own task framing
  as an active blocker, was **not found** in mfw's own repo by mfw's survey — if it's real, it
  lives on the bcinr-cmca consumer side, which was not audited for that specific claim this round.
  UNVERIFIED, not resolved.
- **ggen-marketplace's redundancy check against bcinr specifically** was a name-grep only ("no
  `bcinr-pack` found") — not a deep check.
- **Whether ggen-marketplace's `consequence-ir-pack` ontology is used by, or influences, any other
  surveyed repo's actual authority-decision code** (vs. existing only as a marketplace-hosted
  artifact) was not checked.
- A next pass, to close these, would need: (a) a praxis survey run to the same standard as the
  other six, (b) a single consistent file-counting methodology applied to all 8 repos to produce a
  real recomputed mass percentage, (c) a direct code-level (not doctrine-level) diff of bcinr-powl's
  and gymact's receipt implementations, and (d) a direct check of bcinr-cmca's consumer-side code
  for the claimed fixed-point API mismatch.

## 6. Follow-up: the four §5 gaps, closed with real evidence

Run as three independent, real investigations (not another full 8-repo sweep). All four gaps
from §5 addressed; results below, including one new finding not in scope of the original ask.

### 6.1 praxis — surveyed for the first time this session

**Build: BLOCKED**, and worse than a compile error — `cargo check --workspace` and
`cargo metadata` both fail at **manifest resolution**, before any code is even read, because
`Cargo.toml`'s hardcoded local `path = "/Users/sac/..."` dependencies include one
(`ggen-core` → `/Users/sac/ggen/crates/ggen-core`) that **does not exist** — `~/ggen`'s current
crate layout has no `ggen-core` (it has `ggen-engine`, `ggen-cli`, `ggen-graph`, etc.). The whole
praxis workspace is currently un-inspectable by any cargo tool on this machine.

**Mass, corrected**: naive raw count 33,568 files → **5,858 corrected total / 1,027 hand-authored
source files** (`.rs`/`.py`/`.js`/`.ts`/`.erl`), after excluding `.claude/worktrees` (empty here —
unlike every other repo, praxis's inflation source is different), `target`, `node_modules`,
`.venv`, and — the actual culprit — **`vendors/`, 24,262 files (72% of the naive count)**. The
carried-forward 13,763 figure from the original 2026-08-12 doc is **~13x** the corrected
hand-authored count and was already itself a partial undercount relative to today's naive total —
both numbers were dominated by non-hand-authored vendor content, never a real code-mass signal.

**Authority**: explicit self-denial, consistent with every other repo surveyed. `README.md:141`:
*"Output is proposal (`O`), never authority (`O*`) — every candidate still passes `law`/`plan`
admission (AR-9)."* `docs/VISION_2030_PRD.md:65`: *"No physical enforcement claims... Admission
is software-binding, not physics-binding."* No `BRCE.DO` or "standing verdict" string found
anywhere in the repo.

**Redundancy**: real, direct — not conceptual — dependency edges: `Cargo.toml` hard-paths
`bcinr-logic`/`bcinr-pddl` from `/Users/sac/bcinr` and (transitively, currently broken)
`multifractal-workflow` from mfw. Praxis sits as a **consumer** at the top of the local dependency
graph, not a duplicate-capability island — which is exactly why one missing path cascades into a
full workspace failure. No evidence found of overlap with ggen-marketplace, ggen-legacy, gymact,
or autofde-lab in praxis's own docs (checked one direction only — praxis's docs, not the other
four repos' docs for praxis mentions).

**Since 2026-08-06**: zero commits. The repo is frozen at the exact commit the original 13,763
figure was drawn from — the number's inaccuracy is 100% a counting-methodology artifact, not
stale-vs-current code.

### 6.2 bcinr-powl vs. gymact BLAKE3 receipts — resolved, not just flagged

Real code-level comparison (not doctrine): **same core idea (hash-chained receipts over BLAKE3),
independently and differently engineered — not interoperably verifiable, and not equally rigorous.**

| | bcinr-powl (`causal_receipt.rs`) | gymact (`evidence.py` + `sqlite_ledger.py`) |
|---|---|---|
| Canonicalization | Hand-packed fixed 99-byte LE buffer, no general spec — "canonical" means only "this exact Rust struct's field order," undocumented outside a comment | Real RFC 8785 JCS (`rfc8785.dumps`) applied uniformly to arbitrary payloads |
| Chaining | `BLAKE3(chain_hash \|\| frame_bytes)`, streamed through one `Hasher`; redundantly also stores `prior_hash` per-frame | `BLAKE3(JCS({sequence, previous_digest, receipt_digest}))`, one-shot per record |
| Persistence | None (in-memory only) | Two ledgers — in-memory and SQLite-WAL, durable |
| Restart verification | None | On every SQLite ledger open, `verify()` re-walks and re-derives the *entire* chain from stored data before trusting it, refusing to serve a mismatched ledger |
| Origin authentication | None | HMAC-SHA256 checkpoint signing over the JCS-canonical payload |

**Direct answer to "is this real duplication or convergent-but-different":** genuinely different
designs for different problems. bcinr's frames are fixed-size, `#[repr(C, align(64))]`,
cache-line-sized structs for a manufacturing/execution trace where allocation-free streaming
throughput matters (an explicit performance comment in the code confirms this design intent).
gymact's records are variable-shape, durably persisted, externally-auditable evidence with
crash-safety and signed checkpoints for third-party handoff. Forcing a shared verifier would mean
rewriting one side to fit the other's shape — not a real reuse opportunity.

**One concrete, low-cost, worth-doing action, asymmetric (bcinr side only)**: bcinr's canonical
form is currently *implicit* — a byte layout documented only in a comment, with no version field
and no compile-time signal if the struct's field order ever changes beyond a size assertion. That
is exactly the fragility JCS-style canonicalization exists to prevent. The fix isn't "adopt JCS"
(wrong tool for a fixed-size, allocation-free hot path) — it's turning the implicit layout into an
**explicit, versioned wire-format spec**, so a receipt stays verifiable outside the exact Rust
struct that produced it. Not done this round; a real, scoped, low-risk follow-up if bcinr's
receipts are ever meant to be checked by anything other than the originating process.

### 6.3 The mfw ↔ bcinr-cmca fixed-point mismatch — real, but backwards from how it was framed, plus a second independent instance found in bcinr itself

**The blocker is real and currently live** — but the direction stated in this session's original
task framing (mfw emits `from_bits`, consumer wants `from_value_bits`) is **wrong**. The actual
state, verified by reading both sides directly:

- `bcinr-cmca`'s `NonNegativeFixed`/`SignedFixed` (`crates/bcinr-cmca/src/fixed.rs`) expose
  **only** `from_bits(bits: u32/i32)`. No `from_value_bits` exists anywhere in the type.
- mfw's real, currently-committed generator (`~/mfw/tools/cmca-generator/generator.py`, all 4
  emission sites) emits `NonNegativeFixed::from_value_bits(...)` / `SignedFixed::from_value_bits(...)`.
- The live ticket that caused this, `~/mfw/docs/jira/v26.7.18/mfw-cmca-producer/task-fix-generator-fixed-point-api_v26.7.18_final_2026-07-17_AgentSwarm.md`
  (`MFW-261718-002`, P0), asserted the *opposite* of what's actually in `fixed.rs` — it claimed
  "the consumer fixed-point API only exposes `SignedFixed::from_value_bits()`" and directed the
  generator to match that false premise. Its acceptance criteria reference
  `bcinr-cmca/src/generated_artifact.rs`, **which does not exist anywhere in the bcinr tree** — so
  the fix was applied and never actually verified against a real artifact.
- **Net effect**: any freshly-generated artifact from mfw's real generator would fail to compile
  against `bcinr-cmca` today, on an unresolved-method error, in the opposite direction the original
  framing described. mfw's own `VERIFICATION_REPORT.md` claim of an 89/89 match used a regex for
  `from_bits(` that would not match `from_value_bits(` — that report is stale relative to the
  generator's current source, not evidence the mismatch is resolved.

**Second, independent instance found in bcinr's own tree, not part of the original ask**:
`crates/bcinr-cmca/src/proposal.rs` calls `SignedFixed::from_value_bits(...)` nine times in its
own `#[cfg(test)]` module — the identical stale name, left over from before a real rename
(`from_value_bits` → `from_bits`, commit `7197c91f`, "Closure Ticket C1: Repair fixed-point
semantics") that never propagated to this file. **This does not currently break the build**,
verified directly (`cargo test -p bcinr-cmca --features std --lib` passes clean) — because
`proposal.rs`, despite being a real, tracked, doc-commented file ("Authority hop 1 of the C3
chain," last touched 2026-07-29), is **never declared as a module anywhere** (`grep -rn "mod
proposal"` across the whole crate returns nothing). It's orphaned dead code, silently excluded
from compilation, which is the only reason its own stale API reference hasn't already surfaced as
a build failure. Two separate, real problems worth their own follow-up, not addressed this round:
(a) decide whether `proposal.rs` should be wired back into `lib.rs` (and if so, fix its
`from_value_bits` calls first) or removed if superseded, and (b) mfw's generator needs to go back
to emitting `from_bits` to match the real consumer API, with `MFW-261718-002` corrected or closed
as based on a false premise.

### 6.4 Portfolio mass — best-effort recomputation, with honest unit caveats

Corrected counts are now available for 8 of the original doc's repos. They are **not
directly comparable to each other** — some are ".rs files only" (bcinr 689, ggen 2,196), some are
"hand-authored files across all languages" (mfw 1,233; gymact 327; praxis 1,027 or 5,858 total),
and autofde-lab's 1,358 is Python-only (its real C++ solver core, `cpp/`, isn't counted). This
recomputation uses each repo's most-inclusive corrected figure and states that choice per repo —
treat the resulting percentage as directional, not authoritative, and do not cite it as more
precise than the inputs allow:

| Repo | Figure used | Basis |
|---|---|---|
| bcinr | 689 | `.rs` only |
| mfw | 1,233 | all hand-authored types |
| ggen | 2,196 | `.rs` only |
| ggen-marketplace | 1,802 | corrected total |
| ggen-legacy | 275 | all hand-authored types |
| gymact | 327 | all hand-authored types |
| autofde-lab | 1,358 | Python only (undercounts `cpp/`) |
| praxis | 5,858 | corrected total, all types |
| **Sum (8 repos)** | **≈ 13,738** | |

`autofde-lab + praxis` share of this 8-repo corrected sum: **(1,358 + 5,858) / 13,738 ≈ 52.6%** —
still the largest two repos, still a majority, but nowhere near the original doc's raw-count 75%
claim, and using praxis's stricter hand-authored figure (1,027) instead: (1,358 + 1,027) / 8,907
≈ 26.8% — a very different picture depending on which praxis number is used. **This spread itself
is the honest finding**: the corrected percentage is somewhere between ~27% and ~53% depending on
methodology choices this refresh cannot fully resolve, a real narrowing from "unverified 75%" but
not a clean replacement number. This also only covers 8 of the original 11 repos — which 3 are
missing was not re-established this round (the original doc's full repo list wasn't re-read for
this recomputation).
