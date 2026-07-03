# DfCM Crown Bench: what the numbers mean, and how they relate to SOTA planning

This explains the `dfcm_crown_bench`/`dfcm_crown_suite` results and is honest
about what they do and do not prove relative to state-of-the-art automated
planning. **Short version: this benchmark is not measuring planning quality
against IPC-class planners, and a direct number-to-number comparison would be
misleading.** It measures something different — and narrower — on purpose.

**Note on the numbers below**: the "What was actually measured" section
records a historical snapshot (243ms total) from before the `may_fire`
fact-deduplication fix. See "Fixed: `may_fire` fact deduplication" further
down for the current numbers (41ms total, ~6.1x faster) — kept as a
historical record of the investigation sequence, not silently overwritten.

## What was actually measured

`run_dfcm_crown_suite()` runs a fixed 16-cell matrix (worker count ∈
{8,16,32,64} × capacity ∈ {1,2,4,8}) of a tiny synthetic `assign-worker`
durative-action domain, and for every cell times: PDDL→POWL topology
derivation, temporal planning, bounded schedule analysis, Prolog8 admission,
BLAKE3 receipt-chain computation, and replay. Release-mode result:

```text
wall_clock_ms   = 243   (post-optimization; was 245 before the analysis_ns fix below)
topology_ns     = 85,125      (~5.3 µs/cell)
planning_ns     = 9,227,542   (~577 µs/cell)
analysis_ns     = 25,389,250  (~1.59 ms/cell, was 26,557,041 before find_temporal_plan_with_fn_overrides)
admission_ns    = 102,927,749 (~6.43 ms/cell, unchanged — see "what was tried and rejected")
receipt_ns      = 77,376      (~4.8 µs/cell)
replay_ns       = 102,968,961 (~6.44 ms/cell, unchanged)
max_ops         = 64
max_parallelism = 8
passed_5s_gate  = true (243ms vs. a 5,000ms budget — ~20x headroom)
```

The debug-mode wall-clock gate test (`dfcm_crown_suite_completes_under_5_seconds`)
runs the identical suite unoptimized and lands at ~3.0s — still comfortably
under the 5s contract, with the gap between 243ms (release) and 3.0s (debug)
being entirely normal optimization-level variance, not suite instability.

### Where the time actually goes

`admission_ns` and `replay_ns` dominate (≈84% of total combined), and they're
nearly identical — both call `execute_temporal_plan`. An earlier version of
this document hypothesized that per-call Prolog8 `Kernel`/`Catalog`
*construction* was the dominant cost. **That hypothesis was checked directly
against the `prolog8` crate's source and found wrong**: `Kernel::new`/
`Catalog::new` just allocate empty `Vec`/`BTreeMap`s — trivially cheap,
regardless of how often they're called. The real cost is legitimate,
proportional work inside `Kernel::query()`: `assemble_fact_answer`/
`assemble_negative` build a real proof node and a hashed `Receipt` on every
single query (see `prolog8::kernel::finalize_proof`/`assemble_receipt`). That
is bounded, cheap-by-design work — one proof node, fixed-size hashing — but
it is genuinely proportional to the number of admission checks performed
(once per plan step, times two calls to `execute_temporal_plan` for the
admission + replay stages), not overhead from object construction. It is not
"waste" to optimize away; it is the actual cost of the receipt guarantee
itself. No further optimization was applied here — see "What was tried and
rejected" below.

`planning_ns` and `analysis_ns` are next, both well under 2ms/cell even at
the 64-worker, capacity-8 cell. `topology_ns` and `receipt_ns` are
negligible — both are pure, cheap, bounded computations (mask derivation,
BLAKE3 hashing) exactly as the architecture predicts.

## What was tried and rejected

A follow-up 80/20 optimization pass checked two hypotheses from a code audit:

1. **"Reuse the Prolog8 Kernel/Catalog across calls instead of rebuilding it
   per `execute_temporal_plan` call."** Rejected after reading the actual
   `prolog8` source (no assumptions): `Kernel`/`Catalog` expose no
   reset/clear API, and — more importantly — their construction cost is
   already near-zero, so there was nothing to save. The admission/replay
   cost is real per-query proof/receipt work, not object-construction
   overhead. Implementing a "kernel reuse" fix here would have been
   optimizing something that was never the bottleneck.
2. **"`schedule_analysis::replan_with_perturbed_capacity` clones the entire
   `GroundTemporalProblem` (grounded actions, condition trees, atom sets)
   just to perturb one numeric fluent."** Confirmed real and fixed: added
   `GroundTemporalProblem::find_temporal_plan_with_fn_overrides` (accepts a
   small `HashMap<String,f64>` override merged into a *cloned copy of just
   `initial_fn_values`*, not the whole struct), and switched
   `replan_with_perturbed_capacity` to use it. Result: `analysis_ns` dropped
   from 26.56ms to 25.39ms total (≈4% of the suite's ~26ms analysis cost,
   ≈0.5% of total suite wall clock) — a real, verified improvement, but
   `analysis_ns` was never the dominant term, so total wall clock barely
   moved (245ms → 243ms). This is the honest 80/20 outcome: the one clearly
   wasteful pattern in the codebase was small relative to the suite's actual
   bottleneck (legitimate per-query proof/receipt hashing), which isn't
   "waste" to begin with.

## Cold/warm attribution split (follow-up pass)

A further pass split the suite's timing into cold-once vs. warm-per-cell
buckets, and — separately — measured what a *cheaper* replay path would
actually cost, instead of assuming today's `replay_ns` (full re-execution)
is the only option.

```text
cold_topology_once_ns            = 55,459 ns   (55.5 µs — one domain parse, outside the 16-cell loop)
warm_replay_existing_receipt_ns  = 76,001 ns   (76 µs total across 16 cells — chain-only validation)
replay_ns (full re-execution)    = 103,346,251 ns (103.3 ms total across 16 cells — unchanged)
```

**`cold_topology_once_ns` confirms surface 6's expectation**: domain parsing
is a true one-time compile-time cost (55.5µs total, not per-cell), already
correctly hoisted outside the loop in `dfcm_crown.rs` — nothing to fix here.

**The replay comparison is the headline finding of this pass.** Chain-only
replay validation (`compute_plan_chain` recomputed and compared against the
receipt's steps) costs **~76µs total across all 16 cells — roughly 1,360x
cheaper than the 103.3ms full re-execution replay** the crown suite
currently measures as `replay_ns`. This is not a small optimization; it is a
genuine semantic fork that the architecture needs to make explicitly rather
than silently default:

- **Full re-execution** (`execute_temporal_plan` called again) proves *"this
  plan would be re-admitted under the same policy today"* — it re-runs every
  Prolog8 `may_fire` query from scratch. This is the right guarantee if
  policy can change between the original execution and the replay, and you
  need to know whether it *still* holds.
- **Chain-only validation** (`compute_plan_chain` alone) proves only *"this
  receipt's chain hash is a pure, reproducible function of these exact
  plan steps"* — it does not re-check admission at all. This is the right
  guarantee if the question is "did this receipt get tampered with / is this
  the same execution that was recorded," not "would this still be legal."

**Neither is a drop-in replacement for the other.** The PRD backlog's
`replay_receipt` MCP tool (task #15, not yet implemented) needs to pick one
of these two guarantees explicitly, and should probably expose *both* as
distinct operations (e.g. `replay_receipt` for the cheap chain check,
`re_admit_plan` or similar for the expensive full re-admission check) rather
than conflating them under one name the way this benchmark's `replay_ns`
metric currently does.

### Surfaces 3-6: inspected, no further code changes made

- **Surface 3 (Prolog8 proof/receipt assembly, one level deeper than
  `Kernel::new`)**: reasoned qualitatively from the already-read `prolog8`
  source rather than adding new instrumentation probes inside
  `execute_temporal_plan`'s loop (which would itself add per-step
  bookkeeping overhead to a hot path). `Kernel::query`'s fact scan is
  `O(fact_blocks × rows)` — bounded by the number of loaded facts, which is
  itself bounded by the 64-op cap — and `assemble_fact_answer`/
  `assemble_negative` build exactly one `ProofNode` plus a hashed `Receipt`
  per query, regardless of how many facts exist. Also newly noted: the
  per-step OCEL event construction inside `execute_temporal_plan`'s loop
  (`OCELEvent`/`OCELEventAttribute` allocation) shares the same `admission_ns`
  timing bucket as the actual Prolog8 query and was never separated from it
  — this is a real, previously-uncounted contributor to `admission_ns`/
  `replay_ns`, though separating it cleanly requires adding timing probes
  inside a hot loop, which was deferred pending surface 7's allocation data
  (a cheap-vs-expensive allocation count is a better signal for whether this
  is worth a dedicated timing split than guessing).
- **Surface 4 (schedule_analysis replan multiplicity)**: confirmed by
  reading the code that `resource_keys` are neither de-duplicated nor
  filtered for relevance (`schedule_analysis.rs`'s loop over
  `resource_keys.iter().enumerate().take(64)` replans unconditionally for
  every entry, twice, regardless of whether the key appears in any grounded
  action's conditions). **Not fixed**: the crown suite itself only ever
  passes one resource key (`["available-workers"]`), so this pattern isn't
  currently exercised as waste by anything in this codebase — flagging it
  for future callers that might pass duplicate/irrelevant keys, not treating
  it as a live 80/20 win today (per the plan's "not optimizing before
  measurement shows it matters" fence).
- **Surface 5 (`ground.rs` planner scan pattern)**: `warm_plan_ns`
  (=`planning_ns`) stayed at ~577µs/cell even at the largest (64-worker)
  cell, both before and after this pass — the full-rescan pattern in
  `find_temporal_plan`'s scheduling loop and its `timed_inits` rescan are
  confirmed to be "ALIVE bounded scan," not a live cost, at the sizes this
  benchmark exercises. Left unchanged.
- **Surface 6 (parse/topology compile-artifact boundary)**: confirmed via
  `cold_topology_once_ns` (55.5µs total, one call) that the domain is
  already correctly parsed once outside the per-cell loop — this
  architectural property already held before this pass; nothing to fix.
  `GroundTemporalProblem::build`'s per-cell `TypeIndex::build` re-derivation
  is correct, not waste, since the crown suite's object/type declarations
  (worker count) genuinely change per cell.

## Surface 7: allocation profiling (real numbers, confirms surface 3)

Implemented as a lightweight feature-gated global counting allocator
(`crates/bcinr-pddl/src/alloc_counter.rs`, `dhat-heap` Cargo feature) rather
than the `dhat` crate — `dhat` produces a JSON file for its own viewer, which
doesn't cleanly map onto `DfcmBenchReceipt`'s structured
`alloc_count_by_stage`/`bytes_allocated_by_stage` fields; a direct atomic
counter wrapping the system allocator gives exact per-stage counts with a
two-line `snapshot()`/subtract around each stage, at the cost of no call-tree
visualization. Run via
`cargo run -p bcinr-pddl --example dfcm_alloc_profile --features dhat-heap --release`.

```text
                allocations   bytes
topology        1,456         58,420
planning        189,678       2,282,409
analysis        521,983       6,343,282
admission       179,560       33,981,740
receipt         560           5,888
replay          179,560       33,981,740
```

**This confirms surface 3's hypothesis with real data, not speculation.**
`admission` and `replay` allocate ~34MB and ~180K allocations each across the
16-cell suite (≈2.1MB, ≈11,200 allocations *per cell*) — a genuinely large
allocation volume that lines up with their dominant time cost far better than
"proof-search algorithmic complexity" does (the actual proof search, per the
`prolog8` source read in the original pass, is O(1) per query). The likely
sources, matching surface 3's qualitative note: per-step `OCELEvent`/
`OCELEventAttribute` construction (`Vec` and `String` allocation per plan
step, per `execute_temporal_plan`'s loop) and `Ctx::pred`/`Ctx::term`'s
`String`-keyed `HashMap` interning inside the same loop. Both are real,
addressable allocation sources — but per the plan's own fence ("do not
optimize before measurement," and this pass's scope was attribution, not a
third optimization round), no code changes were made based on this data.
**This is the concrete next-optimization candidate** for a future pass,
now backed by allocation counts instead of a falsified construction-cost
guess.

`analysis`'s unexpectedly high allocation count (521,983 — more than
`admission`/`replay` combined) despite its small time cost (~25ms) is also
notable: `find_temporal_plan_with_fn_overrides`'s two replans per resource
key (`replan_with_perturbed_capacity`, called twice per cell after the
surface-1-era fix) still clone `self.initial_atoms` (a `BTreeSet`) and
`self.initial_fn_values` fully inside `find_temporal_plan` itself, once per
replan — the earlier fix eliminated the *outer* `GroundTemporalProblem`
clone but not the *inner* per-call state clones that `find_temporal_plan`
always does, regardless of caller. Flagged, not fixed, in this pass.

## L3 substage attribution: admission/replay cost is almost entirely `query_ns`

A follow-up pass added `execute_temporal_plan_instrumented` (a bench-only
duplicate of `execute_temporal_plan` with `Instant::now()` checkpoints
around fact-loading, per-step Prolog8 querying, effect application, chain
hashing, and OCEL trace construction — zero overhead added to the real,
production `execute_temporal_plan` used by every other caller). Real numbers
from the largest cells:

```text
fact_load_ns            = 287,124 ns   (admission) /   262,540 ns (replay)
query_ns                = 103,794,486 ns (admission) / 104,418,206 ns (replay)
effects_apply_ns        = 598,536 ns   (admission) /   639,627 ns (replay)
proof_receipt_build_ns  = 90,840 ns    (admission) /    91,291 ns (replay)
trace_build_ns          = 391,700 ns   (admission) /   392,460 ns (replay)
```

**`query_ns` — the per-step `ctx.query_may_fire` call — is ~98% of the total
`admission_ns`/`replay_ns` cost.** Every other substage combined
(fact-loading, effect application, chain hashing, OCEL construction) is
under 1.3ms out of ~105ms. This corrects the prior pass's speculative note
that OCEL event construction or Prolog8 string interning were likely
culprits — `trace_build_ns` and `fact_load_ns` are both negligible. The real
cost is inside `ctx.query_may_fire`, i.e. inside `prolog8::Kernel::query`.

**A concrete, textually-confirmed root cause was found — a real redundancy,
not proof-algorithm cost.** `Ctx::load_may_fire` (`execute.rs`) creates one
new `FactBlock8` per call, with **no deduplication**:

```rust
fn load_may_fire(&mut self, label: &str) {
    let pred = self.pred("may_fire", 1);
    let term = self.term(label);
    let row = FactRow8::new(pred, 1, &[term], SRC);
    let _ = self.kernel.load_facts(FactBlock8::new(pred, 1, vec![row]));
}
```

`execute_temporal_plan`'s fact-loading loop calls this **once per plan
step** (`for step in &steps { ctx.load_may_fire(&step.action_name); }`).
For durative-action plans, every step of the same schema shares the *same*
`step.action_name` (the bare schema name — see the durative-action label fix
from an earlier pass), so at the 64-worker cell, `may_fire("assign-worker")`
gets loaded as **64 separate, byte-identical `FactBlock8`s**. `prolog8`'s
`Kernel::scan_facts` (source-confirmed in an earlier pass) does not
short-circuit on the first match — it scans **every** fact block, filters,
and collects **all** matches into a `Vec<(FactRow8, [u8;32])>`; `query()`
then builds a proof/receipt answer (`assemble_fact_answer`) for **every**
element of that vector, not just one. So at n=64 identical steps, a single
query scans up to 64 fact blocks and assembles up to 64 redundant proof
answers, and this happens once per step — a real O(n²)-shaped redundancy
inside `query_ns`, not O(1) proof-search cost.

This was flagged as a genuine, evidence-backed optimization candidate
satisfying the project's own bar: *"An optimization candidate is admitted
only if it appears at L3 or deeper with measured ns/alloc impact and a
preserved guarantee boundary."* It was then fixed in a dedicated follow-up
pass — see "Fixed: may_fire fact deduplication" below.

## Fixed: `may_fire` fact deduplication

**The fix**: `Ctx::load_may_fire`'s three call sites (`execute_tape`,
`execute_temporal_plan`, `execute_temporal_plan_instrumented`, all in
`crates/bcinr-pddl/src/execute.rs`) now track a `BTreeSet` of already-loaded
labels and only call `ctx.load_may_fire(label)` the first time a label is
seen, instead of once per op/step unconditionally. **Guarantee preserved**:
`may_fire(label)` is queried by label alone
(`ctx.query_may_fire(&step.action_name)`), never by step index or instance
identity — it is already a set-membership fact under the current admission
model, not a multiset one, so loading it once vs. N times cannot change
what's provable. The fix was scoped narrowly to these three call sites, not
to `prolog8::Kernel::scan_facts`/`assemble_fact_answer` internals, which
remain untouched — a separate crate with its own design authority.

A dedicated regression test,
`duplicate_action_labels_admit_identically_to_a_single_label`
(`crates/bcinr-pddl/tests/capacity.rs`), documents the guarantee directly:
multiple `assign-worker` steps sharing one label all admit and reach their
goal, exactly as before the fix.

**Measured before/after** (release-mode `dfcm_crown_bench`, full 16-cell suite):

```text
                        before          after         speedup
wall_clock_ms           251             41            ~6.1x
admission_ns            105,672,164     2,038,791     ~51.8x
replay_ns               106,282,418     2,025,501     ~52.5x
admission.query_ns      103,794,486     730,961       ~142x
replay.query_ns         104,418,206     729,871       ~143x
admission alloc_count   179,560         32,608        ~5.5x fewer
admission alloc_bytes   33,981,740      1,483,532     ~22.9x fewer
replay alloc_count      179,560         32,608        ~5.5x fewer
replay alloc_bytes      33,981,740      1,483,532     ~22.9x fewer
```

The debug-mode 5s gate test dropped from ~3.0s to ~0.21s (~14x) in the same
change — consistent with the release numbers, just at debug-build constant
factors. `debug_assert_eq!(receipt.chain_hash, replay_receipt.chain_hash)`
in `dfcm_crown.rs`'s loop continued to hold throughout, confirming the fix
doesn't change what's provable, only how much redundant work proving it
took. All 26 `bcinr-pddl` tests (25 prior + 1 new regression test) pass.

**The bottleneck moved, honestly reported, not hidden**: post-fix,
`analysis_ns` (~25.4ms, unchanged by this fix) is now the largest single L1
stage — larger than `admission_ns`+`replay_ns`+`planning_ns` combined
(~13.3ms). Its allocation count (521,983, unchanged) already exceeds
`admission`+`replay` combined (65,216, post-fix) by ~8x. This was flagged in
an earlier pass ("NEXT LEAD: `find_temporal_plan`'s internal per-call
`BTreeSet`/`HashMap` state clones, which survive even after the
`find_temporal_plan_with_fn_overrides` fix eliminated the outer
`GroundTemporalProblem` clone") and was investigated directly in the
following pass — see below.

## Analysis bottleneck after may_fire dedupe

A dedicated attribution pass added `analyze_schedule_instrumented`
(`crates/bcinr-pddl/src/schedule_analysis.rs`) — `analyze_schedule` now
delegates to it directly rather than duplicating it, since `analyze_schedule`
only makes ~3-6 sub-calls total (unlike `execute_temporal_plan`'s per-step
hot loop, which needed a separate bench-only duplicate to avoid measurement
overhead in a tight loop). New `AnalysisSubstageNs` fields:
`resource_key_collect_ns`, `base_plan_ns`, `perturb_minus_ns`,
`perturb_plus_ns`, `sensitivity_compute_ns`, `result_build_ns`.

**Post-dedup baseline** (release, full 16-cell suite, before this pass):

```text
wall_clock_ms   = 41
analysis_ns     = 25,482,211 ns  (~62% of wall clock)
analysis alloc  = 521,983 allocations / 6,343,282 bytes
```

**L3 analysis substage attribution** (real numbers):

```text
resource_key_collect_ns = 416 ns
base_plan_ns            = 9,340,544 ns
perturb_minus_ns        = 6,989,209 ns
perturb_plus_ns         = 9,124,711 ns
sensitivity_compute_ns  = 293 ns
result_build_ns         = 332 ns
```

`base_plan_ns + perturb_minus_ns + perturb_plus_ns ≈ 25.45ms`, matching
`analysis_ns`'s ~25.48ms almost exactly. `resource_key_collect_ns`,
`sensitivity_compute_ns`, and `result_build_ns` are all sub-microsecond,
combined negligible.

**Finding: no hidden extra redundancy — the cost is fully explained by
three proportional `find_temporal_plan`-family calls.** `analyze_schedule`
calls the planner three times per cell (once for the base plan, once each
for the −1/+1 capacity perturbations via
`replan_with_perturbed_capacity`/`find_temporal_plan_with_fn_overrides`),
and each of those three calls costs roughly what one standalone
`find_temporal_plan` call costs (`planning_ns` totals 9.26ms across the
suite for one call per cell; `base_plan_ns`/`perturb_plus_ns` are both
~9.1-9.3ms, `perturb_minus_ns` slightly less at 6.99ms, plausibly because
lower capacity plans terminate sooner). This is the **expected, inherent
cost of the finite-difference sensitivity method** chosen deliberately in
an earlier pass over building a full LP/polytope solver — not a bug, and
not evidence of a *separate* redundancy layered on top of that method's
known cost.

**Phase 2 decision: no optimization candidate admitted in this pass.** Per
the project's own admission rule, a candidate needs a source-confirmed root
cause distinct from the method's inherent cost, with a small,
guarantee-preserving, low-risk fix. The one real remaining lever —
`find_temporal_plan`'s internal per-call `state`/`fn_vals` clones
(`self.initial_atoms.clone()`, `self.initial_fn_values.clone()`) — is real,
but reducing it would mean changing `find_temporal_plan`'s core signature
and loop (e.g. a mutable scratch-state parameter reused across calls), which
is shared by *every* planning call in the codebase, not an analysis-specific
fix. That is a broader, higher-risk change appropriate for its own
dedicated, carefully-tested pass — not a "surgical" fix to squeeze into an
attribution pass, per this project's explicit "one candidate, narrow scope,
no broad rewrites" discipline. Stopping here, as prescribed: *"Stop after
attribution if no candidate satisfies the admission bar."*

**Guarantee boundary preserved**: `TemporalPlan.makespan`, `.steps`,
capacity-sensitivity result shape, `max_ops`/`max_parallelism`, and all
admission/replay behavior are unchanged — this pass added instrumentation
only, no optimization, no semantic change. 26/26 tests pass (unchanged from
before this pass; no new regression test was needed since no behavior
changed).

**Remaining bottleneck after this pass**: `analysis_ns` (~25.4ms) remains
the dominant L1 stage, now understood precisely rather than suspected. The
next evidence-backed candidate, if this becomes a priority, is a
planner-wide (not analysis-specific) scratch-state reuse mechanism inside
`find_temporal_plan` — scoped as its own future pass.

## Resolution ladder: what's implemented vs. deferred

Per the multi-resolution benchmarking framework:

| Layer | Status | Notes |
|---|---|---|
| L0 crown gate | Implemented | `dfcm_crown_suite_completes_under_5_seconds`, ≤5s contract |
| L1 stage timing | Implemented | `topology_ns`/`planning_ns`/`analysis_ns`/`admission_ns`/`receipt_ns`/`replay_ns` |
| L2 lifecycle split | Implemented | `cold_topology_once_ns`, `warm_*_ns`, `warm_replay_existing_receipt_ns` (chain-only) vs. `replay_ns` (full re-execution) |
| L3 substage attribution | Implemented (admission, replay, analysis) | `execute_temporal_plan_instrumented`'s `SubstageNs` (fact_load/query/effects_apply/proof_receipt_build/trace_build); `analyze_schedule_instrumented`'s `AnalysisSubstageNs` (resource_key_collect/base_plan/perturb_minus/perturb_plus/sensitivity_compute/result_build) |
| L4 allocation | Implemented (L1 granularity only) | `dhat-heap` feature, `alloc_counter.rs`; not extended to L3 substage granularity for analysis (the ns-level split alone was sufficient to reach a conclusion — see "Analysis bottleneck after may_fire dedupe") |
| L5 scaling slopes | **Deferred** | Requires new domain/problem generators varying resource-key count, timed-init count, and condition-tree depth independently of worker count — none of which today's `dfcm-crown` domain varies. Needs its own scoping pass. |
| L6 guarantee-mode benchmarks | **Partially implicit, not formalized** | `chain_validate_only` ≈ `warm_replay_existing_receipt_ns`; `replay_against_current_law` ≈ `replay_ns`/`admission_ns`. `trace_compare_only` (needs `compare_traces`, PRD backlog task #15) and `replay_against_recorded_law` (needs a policy-versioning concept absent from `Ctx` entirely) require functionality that doesn't exist yet — not benchmarked against code that doesn't exist. |

## Why this isn't a SOTA planning comparison

Automated planning research measures planners on a different axis entirely:
given a hard, often unbounded combinatorial search problem (IPC benchmark
domains — Blocksworld, Logistics, Rovers, temporal/numeric domains with
hundreds of objects), find a valid (and ideally near-optimal or optimal)
plan, where the search itself is the bottleneck and can be exponential in the
worst case. Published results for planners like Fast Downward, its temporal
extension TFD, OPTIC, and ENHSP report search times ranging from
milliseconds to many seconds or minutes depending on domain difficulty, with
grounding/translation alone reported to take up to 25–30 seconds on larger
IPC tasks ([Fast Downward](https://arxiv.org/pdf/1109.6051),
[FAPE](https://arxiv.org/pdf/2010.13121), recent Fast Downward/ENHSP-20
benchmarking). These systems are solving a fundamentally harder problem class
than this benchmark does — they search a space whose size is not bounded by
construction.

`find_temporal_plan` (the planner exercised here) is the opposite kind of
artifact: a **greedy, non-backtracking, capacity-aware scheduler** over a
domain whose ground-action count is capped at 64 by the benchmark's own
design. It does not search — it does one forward pass per tick, scheduling
every currently-applicable action. That is precisely why it's fast: it has
no exponential search tree to traverse, because the "8/64" discipline
described in `docs/MANIFESTO.md` removes the unbounded part of the problem
before planning even starts. It is not a faster *solution* to the problem
IPC planners solve — it solves a narrower, bounded problem instance by
construction, and IPC-style competitive planners are not being asked to
"also produce a Prolog8 admission proof and a replayable BLAKE3 receipt
chain" the way this benchmark requires every cell to do.

**The honest comparison is therefore not "we're faster than Fast
Downward/OPTIC/ENHSP."** It would only be fair to compare planning runtime
directly if this codebase attempted optimal or near-optimal planning over
the same unbounded IPC domains those systems target — it does not, and the
plan `find_temporal_plan` returns has no optimality guarantee at all (stated
plainly in `docs/MANIFESTO.md` §5).

## What this benchmark *does* establish

1. **The bound is real, not incidental.** `max_ops == 64` at the largest
   cell, hitting the tape ceiling exactly, with the suite still finishing in
   245ms — confirming the architecture's claim that bounded composition
   stays cheap *because* it's bounded, not because the test cases happened
   to be small by accident.
2. **Verification overhead is genuinely low relative to the 5s contract.**
   Even the slowest stage (admission/replay, ~6.43ms/cell — real per-query
   proof/receipt hashing work, not construction overhead) leaves roughly
   20x headroom against the gate in release mode. If this becomes a real
   product surface (the PRD's `admit_schedule`/`replay_receipt` MCP tools)
   and that headroom needs to shrink further, the honest next lever is
   reducing the number of admission queries per plan (e.g. batching, or
   accepting a proof-mode that skips per-step hashing when a caller doesn't
   need step-level receipts) — not "kernel reuse," which this pass confirmed
   isn't where the cost lives.
3. **Composition doesn't degrade.** Topology, planning, analysis, admission,
   receipt, and replay are six structurally different operations (graph
   algorithm, greedy scheduler, finite-difference probe, Horn-clause proof,
   BLAKE3 hash, re-execution) and none of them dominate the suite
   unexpectedly — there's no hidden quadratic blowup as worker count
   quadruples from 8 to 64.

## What would be needed for a genuine SOTA comparison

To make an apples-to-apples claim against Fast Downward/OPTIC/ENHSP, this
codebase would need to: (a) attempt the *same* IPC benchmark domains, not a
synthetic capacity-demo domain sized to fit the 8/64 bound by construction;
(b) report plan quality (makespan/cost relative to optimal), since
`find_temporal_plan` makes no optimality claim today; and (c) decide whether
the comparison should include or exclude the admission-proof and receipt-
chain cost those other planners don't pay at all. None of that exists yet —
this benchmark is scoped to prove the bounded-composition claim in
`docs/MANIFESTO.md`, not a planning-quality claim.

## Sources

- [The Fast Downward Planning System](https://arxiv.org/pdf/1109.6051)
- [FAPE: a Constraint-based Planner for Generative and Hierarchical Temporal Planning](https://arxiv.org/pdf/2010.13121)
- [An Approach to Temporal Planning and Scheduling in Domains with Predictable Exogenous Events](https://arxiv.org/pdf/1110.2728)
- [International Planning Competition 2023](https://ipc2023.github.io/)
