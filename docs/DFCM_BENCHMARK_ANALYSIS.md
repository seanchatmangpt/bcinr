# DfCM Crown Bench: what the numbers mean, and how they relate to SOTA planning

This explains the `dfcm_crown_bench`/`dfcm_crown_suite` results and is honest
about what they do and do not prove relative to state-of-the-art automated
planning. **Short version: this benchmark is not measuring planning quality
against IPC-class planners, and a direct number-to-number comparison would be
misleading.** It measures something different — and narrower — on purpose.

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
