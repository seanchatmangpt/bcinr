# Governed Decision Kernel

A general, reusable pattern for making a ranking/allocation decision auditable and
hard to silently corrupt: a deterministic multi-criteria core produces a typed
refusal or a result; an audit-trail receipt binds that result to the exact inputs
that produced it; an *optional*, narrowly-calibrated LLM plausibility check sits on
top as a coarse tamper detector, never as the thing that decides. This document
describes the pattern as it actually exists in this codebase today: one reference
implementation, one real external consumer, both built and exercised this session.
It is not a design proposal — every claim below is grounded in a cited path or a
cited, real measurement.

## The four layers

### 1. Deterministic core — `crates/bcinr-cmca/src/allocator/mod.rs`

`allocate()` (`crates/bcinr-cmca/src/allocator/mod.rs:2434`) is the load-bearing
function: given `N` candidates' `PackedSemanticState`s, `K` measures, `Q` policy
lenses, a lambda weighting, and a parent forest describing hierarchy (flat, i.e.
`parent = [-1; N]`, is a supported degenerate case), it computes a single combined
allocation-share vector over the `N` candidates via a fixed-point (`NonNegativeFixed`,
Q16.16), branchless, exactly-`N`-iteration propagation (`flow_step`). No
input-dependent loop bounds, no floating point, no panics — out-of-envelope or
invalid state (e.g. a cyclic `parent`) is a typed `Result::Err`, not a crash or a
silent best-effort answer. `allocate_in` (line 1772) is the inner routine `allocate`
wraps; `allocate_single_lens` (line 2589) is a related single-lens variant used
elsewhere in the crate. This is the thing every other layer exists to make legible
and safe to call from outside the process — it is not itself new work from this
session.

### 2. Audit-trail receipt — `crates/bcinr-cmca/src/allocation_receipt.rs`

`allocation_receipt.rs` answers a question `allocate()` alone cannot: after a call
returns only the combined vector, "why did candidate X get share Y under measure K
at lens Q" is otherwise unrecoverable without re-running the whole allocation and
diffing internal state by hand. `seal_allocation_receipt()` (line 274) mints an
`AllocationReceipt` binding one candidate's per-`(measure, lens)` share to a typed
`AllocationBindings` struct (candidate index, measure, lens index, lens `q`, and a
digest of the semantic states / parent forest / MWU weights that fed the
computation). `verify_allocation_receipt()` (line 328) independently
**recomputes** the share from the bindings plus caller-supplied actual inputs,
using the crate's own topology and per-lens kernel functions, and refuses
(`AllocationRefusal`, one variant per check, including `Cyclic` for a cyclic
`parent`) on any mismatch rather than trusting the receipt's recorded share.

One correction to state precisely, because it is easy to round up: the binding
digest this module uses is a 64-bit non-cryptographic splitmix64-style finalizer
(`mix64`) — the module's own doc comment (`allocation_receipt.rs:29-33`) is
explicit that this is **not** a collision-resistant hash like BLAKE3, and that it
is "adequate to catch accidental/incidental input drift and to structure an audit
trail," not a security boundary: "a party who can freely choose both a receipt and
its claimed inputs could construct a digest collision." The crate's own comment
points at `bcinr-powl`'s BLAKE3-backed `OcelCausalReceipt` as the actual
cryptographic receipt mechanism in this codebase, for contrast — that is a
different, stronger primitive than `allocation_receipt.rs` provides, and the two
should not be conflated. `allocation_receipt.rs` is an audit aid; treat it as such.

### 3. General-purpose entry point — `crates/bcinr-cmca/src/bin/cmca_rank_cli.rs`

Built this session (`crates/bcinr-cmca/src/bin/cmca_rank_cli.rs`, 281 lines). A
small stdin/stdout JSON CLI wrapping the real `allocator::allocate()` as a
**general N-candidate multi-criteria ranking entry point**, so out-of-process
callers can rank arbitrary candidate data by the compiled `case_studies`
lens-weighting policy without reimplementing the allocator, and without being
tied to `cmca_allocate_cli`'s single fixed fixture (`cmca_allocate_cli` is the
other, pre-existing binary in the same `bin/` directory; `cmca_rank_cli` is new
and generic, not a replacement).

Request/response shape:

```jsonc
// stdin
{ "candidates": [ { "name": "solver_a", "measures": [0.8, 0.6, 0.9, 0.5] }, ... ] }

// stdout (success)
{ "ranking": [ { "name": "solver_a", "share": 0.31 }, ... ] }  // sorted descending by share

// stderr (failure, exit 1)
{ "error": "too many candidates: got 9, allocate() is compiled for at most N=8 (CMCA-108)" }
```

Each candidate's 4 `measures` are mapped onto the compiled `case_studies` fixture's
4 measure formulas (`MeasureCache`/`MeasureSearch`/`MeasureRetrieval`/
`MeasureScheduling`) by pinning the shared factors (`businessValue`, `standing`,
`accessFrequency`) to `1` and driving each measure's one free factor from exactly
one `measures[i]` — see the file's own module doc comment for the per-measure
derivation. This is a semantic reinterpretation of a fixture originally compiled
for a different domain, not a claim that ranked candidates are literally caches or
searches. Fewer than 8 real candidates are padded with explicit all-zero
`phantom_state()` entries (excluded from the response by name); more than 8 is
refused with a typed JSON error, never silently truncated.

## Why this pattern, not an LLM router

The alternative this pattern deliberately avoids is asking an LLM to decide the
ranking directly. The reason is grounded in this session's own measured DSPy
results, not a general LLM-skepticism claim:

- `crates/bcinr-cmca/docs/DSPY_THIRD_PARTY_VALIDATION.md`: a live call to
  `groq/openai/gpt-oss-20b` (never mocked) correctly flagged an obvious +0.3
  tamper (score `90/100`) and passed a genuine allocation (score `0/100`) — on
  **one** untampered case and **one** tampered case, one run.
- `crates/bcinr-cmca/docs/DSPY_MAGNITUDE_CALIBRATION.md` widened that to a real
  10-point sweep (`delta_millionths` from `300000` down to `10`, 3 live repeats
  per magnitude, 5 live repeats on the untampered baseline, 35/35 calls
  succeeded — raw data in `crates/bcinr-cmca/scripts/sweep_results.json`):
  - **False-positive rate on genuine allocations: `0/50` (0.00)** across all
    baseline calls in both documents combined (5 in the sweep, plus the earlier
    single-run baseline).
  - **Reliable detection at magnitudes `>= ~10000`** (an absolute allocation-share
    shift `>= 0.01`): `300000`, `100000`, `30000`, and `10000` all scored a mean
    of `80`-`100` with a `1.00` detection rate.
  - **Explicitly non-monotonic below that**, and not to be trusted as a smooth
    falloff: magnitude `3000` and `1000` scored `20.0`/`5.0` (undetected), but
    magnitude `300` scored `70.0` (detected) — a single-point re-detection
    sandwiched between two misses on only 3 repeats per point. The calibration
    doc's own conclusion: "Trusting this DSPy layer to catch a subtle,
    small-magnitude tamper would be unfounded based on this run."

The honest reading of that data is: a calibrated LLM judge is a real, cheap,
useful **coarse plausibility check** for gross tampering, with a measured envelope
of "never false-positives on genuine output, reliably catches shifts at or above
roughly 1% of total allocation share" — and an explicitly unreliable, noisy
detector below that floor. That is not the profile of something that should ever
be the sole or primary decision-maker for a ranking. It is the profile of a good
second opinion sitting on top of a mechanism (the deterministic allocator +
receipt) that is exact, not probabilistic, and whose refusals are typed rather
than a suspicion score. The pattern this document describes puts the LLM layer
exactly there — optional, additive, narrow — never as a replacement for
`allocate()`'s determinism or `allocation_receipt.rs`'s recomputation check.

## Known real limitation: N/K/Q are compile-time fixed

`allocate()` is generic over const parameters `N`, `K`, `Q`, but `cmca_rank_cli`
links against one already-compiled instantiation: `crates/bcinr-cmca`'s
`case_studies` fixture, `N = 8`, `K = 4`, `Q = 4` (CMCA-108). This is a real,
load-bearing constraint of the current binary, not a hidden one:

- **8-candidate cap.** `cmca_rank_cli` refuses (typed JSON error, exit 1) any
  request with more than 8 real candidates — "there is no correct way to drop a
  caller's candidate without their knowledge" (module doc comment). Fewer than 8
  are padded with phantom zero-measure candidates, not silently dropped.
- **4-measure reinterpretation, not 4 free-form axes.** The CLI's `measures[i]`
  values are algebraically forced through `case_studies`'s specific, pre-compiled
  measure formulas (`MeasureCache`/`Search`/`Retrieval`/`Scheduling`) by pinning
  shared factors to their multiplicative identity. This works because those four
  formulas happen to reduce to `measures[i]` exactly under that pinning — it does
  not generalize to an arbitrary number of criteria or an arbitrary formula shape
  without either forking the generated fixture or completing the "genuinely
  const-generic rewrite" the CLI's own doc comment flags as "scoped and deferred
  as future work, not attempted here."

A caller needing more than 8 candidates or more than 4 independent criteria is not
served by the current binary as-is; that would require either a second compiled
fixture with larger `N`/`K`/`Q`, or the deferred const-generic rewrite.

## The first real external consumer: autofde-lab

`/Users/sac/autofde-lab`'s `match_solvers(ranked=True)` is the first, and
currently only, real out-of-process consumer of `cmca_rank_cli`, wired this
session. From the real wiring report:

- **Branch/commit:** `feat/cmca-ranked-solver-matching`, commit `571e834f` —
  "feat(solvers): implement ranked=True via optional cmca_rank_cli governed
  ranking." Not pushed.
- **What changed:** `src/autofde_lab/utils.py` (+205/-2) gained
  `_solver_measures()` (4 real class-level attres used as the CLI's 4 measures:
  domain-requirement count, hyperparameter count, a `_check_domain_additional`
  override flag, and MRO depth), `_resolve_cmca_rank_cli_bin()`
  (`BCINR_HOME`/`CMCA_RANK_CLI_BIN`-probed, never raises), and
  `_rank_via_cmca_rank_cli()` (a real subprocess call to the built binary,
  parsing its JSON response, returning `None` on any failure). `match_solvers()`
  uses these when `ranked=True`, falling back to `[(solver, 1-based-rank), ...]`
  on any failure — `ranked=False` is unchanged.
- **What was actually exercised, stated plainly:** the binary-present path (a
  real subprocess call to `~/bcinr/target/debug/cmca_rank_cli`) **did run for
  real** in that environment, and it hit the real >8-candidate refusal path —
  the domain in question matches 46 real registered solvers, over the compiled
  `N=8` cap — which correctly triggered the graceful-fallback branch. That was
  asserted in the test as the correct, expected behavior of the cap, not treated
  as a bug. It is not, therefore, a demonstration of a successful *governed*
  ranking response end-to-end in that specific test (a <=8-candidate domain would
  be needed to observe that); it is a real demonstration of the refusal-to-fallback
  path working correctly under the binary's actual compiled limit. The
  binary-absent fallback test (`CMCA_RANK_CLI_BIN` pointed at a real nonexistent
  path, `BCINR_HOME` unset) is unconditionally real, not skipped.
- **Real test output:** `tests/fabric/test_phi_dispatch_chicago.py`: `8 passed,
  4 warnings in 11.14s`. Full module `tests/fabric/`: `198 passed, 1 skipped`
  (the skip is a pre-existing, unrelated `a2a` import-optional test, not touched
  by this change).

## How to add a second consumer

The actual integration surface a third consumer should target is
`cmca_rank_cli`'s stdin/stdout JSON contract shown above — not a library import,
not FFI, not the crate's Rust API directly. Concretely:

1. **Build the binary once:**
   `cargo build -p bcinr-cmca --features std --bin cmca_rank_cli` and locate it at
   `target/debug/cmca_rank_cli` (or `target/release/...`).
2. **Resolve the binary path the way autofde-lab's wiring already does** — this is
   the established convention a third consumer should match, not invent
   independently: check an explicit override env var
   (autofde-lab's is `CMCA_RANK_CLI_BIN`) first, then fall back to probing
   `$BCINR_HOME/target/{debug,release}/cmca_rank_cli`, and never raise if neither
   resolves — return `None`/absent so the caller can fall back to its own
   non-governed default behavior.
3. **Call it as a subprocess**, writing the `{"candidates": [...]}` JSON to stdin
   and reading the `{"ranking": [...]}` JSON from stdout on exit code `0`; treat
   any non-zero exit, any stderr `{"error": ...}` envelope, or any JSON parse
   failure as "governed ranking unavailable this call," not as a fatal error —
   fall back to whatever the caller's ungoverned default ranking was, exactly as
   `match_solvers()`'s `ranked=True` path does.
4. **Respect the compile-time limits above.** A caller with more than 8
   candidates per ranking call, or with a criterion that doesn't reduce to one of
   the 4 measure axes, is not currently servable by this binary and should not be
   silently truncated to fit — either batch/pre-filter down to 8 before calling,
   or treat the >8 case as "governed ranking not applicable here," matching how
   `cmca_rank_cli` itself refuses rather than truncates.
5. If the consumer needs the "why did this candidate get this share" audit trail,
   pair the CLI call with `allocation_receipt.rs`'s `seal_allocation_receipt`/
   `verify_allocation_receipt` inside the Rust process that built the ranking
   response — the CLI itself does not currently expose receipt sealing over
   stdout; that would be a natural, currently-unbuilt extension for a consumer
   that specifically needs tamper-evidence beyond "the process didn't crash."

## Summary of what is, and is not, established

**Established, with cited artifacts:** a deterministic, typed-refusal allocator
(`allocator::allocate`); a non-cryptographic-but-real audit-trail receipt
(`allocation_receipt.rs`) layered on top of it; one general-purpose JSON CLI over
that allocator (`cmca_rank_cli`); one real external consumer wired against that
CLI with a real, non-mocked subprocess test path (autofde-lab's
`match_solvers(ranked=True)`); and one calibrated, honestly-scoped optional LLM
plausibility layer (the DSPy verifier/sweep) with a measured reliability envelope
(0/50 false positives, reliable detection roughly `>= 0.01` absolute share shift,
explicitly unreliable below that).

**Not established:** broader validation across more than one judge model, more
than one tamper index, or more than the 3-5 repeats per magnitude already run; a
second real external consumer beyond autofde-lab; a demonstrated successful
*governed* (non-fallback) ranking response inside autofde-lab's own test suite
specifically (the exercised binary-present test path hit the >8-candidate refusal
branch, not a populated ranking response); and a const-generic rewrite removing
the `N=8`/`K=4`/`Q=4` compile-time limits.

## See also

- `crates/bcinr-cmca/src/allocator/mod.rs` — the deterministic core (`allocate`,
  `allocate_in`, `allocate_single_lens`)
- `crates/bcinr-cmca/src/allocation_receipt.rs` — the audit-trail receipt layer
- `crates/bcinr-cmca/src/bin/cmca_rank_cli.rs` — the general-purpose ranking CLI
- `crates/bcinr-cmca/src/bin/cmca_allocate_cli.rs` — the earlier, single-fixture
  sibling binary this CLI's conventions mirror
- `crates/bcinr-cmca/docs/DSPY_THIRD_PARTY_VALIDATION.md` — the first live DSPy
  judge run (one untampered, one tampered case)
- `crates/bcinr-cmca/docs/DSPY_MAGNITUDE_CALIBRATION.md` — the 10-point tamper
  magnitude sweep this document's reliability envelope is drawn from
- `crates/bcinr-cmca/scripts/dspy_cmca_verifier.py`,
  `crates/bcinr-cmca/scripts/dspy_cmca_sweep.py` — the scripts behind both docs
  above
