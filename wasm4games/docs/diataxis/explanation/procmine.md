# wasm4games: The Offline Conformance Pipeline

> Explanation — the *why* and *shape* of `src/procmine`. Not *how you run it* (see the
> API in `procmine::conformance`) and not *what the JTBD chains are* (see the
> [foundry overview](wasm4games-overview.md)), but the harder question: **how does an
> offline, `no_std`, allocation-free crate do what process miners do with databases,
> event logs, and discovery algorithms?**

## Why process mining belongs here

The wasm4pm doctrine is:

> Engines project worlds; wasm4games operates patterns; **wasm4pm admits evidence**; ggen
> manufactures the law.

The external `wasm4pm` authority is the final admissibility gate. But "final" means
*after* the evidence arrives. There is a prior question every system needs to answer
offline, deterministically, and without a network: **is the kernel chain I executed
structurally coherent with the model I claimed to run?**

That question is process mining. Specifically, it is the sub-discipline Wil van der Aalst
calls *conformance checking*: given a normative process model (what *should* happen) and an
observed event trace (what *did* happen), measure how far apart they are. `procmine` answers
that question for the 8 JTBD chains, using the same concepts van der Aalst formalized for
enterprise process logs, but without heap, floats, or standard library.

## The three doubts it answers

Like [The Honest Kernel](wasm4games-the-honest-kernel.md), the conformance pipeline answers
three questions an honest system must face:

1. **Did the chain execute in the declared order?** — a teleported or skipped step breaks
   the sequential workflow model; token-based replay detects it.
2. **How far did observed behavior diverge from declared behavior?** — fitness in basis
   points gives an integer distance that can gate admission.
3. **Has the conformance algorithm itself drifted?** — the `GOLDEN_CONFORMANCE_DIGEST`
   detects drift in the math, the model, or the chain ontology.

## The normative model: sequential workflow nets

Each JTBD chain has a *de jure* model — the declared activity order, extracted from
`chains.ttl` and committed as `CHAIN_MODELS` in `src/procmine/model.rs` (a GENERATED file).

For a chain of n activities, the normative model is a **sequential workflow net**: n+1
places (p₀ = source, p_n = sink) and n transitions, each transition consuming one token from
its input place and producing one to its output place. A perfectly conforming trace executes
all n transitions in order, draining the source and filling the sink exactly once.

```
  p₀ ──[t₁]──► p₁ ──[t₂]──► p₂ ── ··· ──[tₙ]──► pₙ
  (source)                                          (sink)
```

`ChainModel::activities` is the transition label sequence `[t₁, t₂, …, tₙ]`, where each
label is a pattern id. `CHAIN_MODELS` is ordered by chain name, so the position is stable
across regenerations.

## Token-based replay: measuring conformance

**Token-based replay** (van der Aalst, *Process Mining*, 2nd ed., §5.2) is the simplest
conformance algorithm: simulate the normative Petri net on the observed trace and count
tokens produced, consumed, missing, and remaining.

`replay::replay(model: &[u16], trace: &[u16]) -> TokenCounts`:

1. Place one token on the source place (p₀). *produced += 1.*
2. For each observed activity `a` in the trace:
   - Find the transition `t` in the model with label `a`.
   - If `t`'s input place has a token, consume it. *consumed += 1.*
   - If not, **create a missing token** and consume it immediately. *missing += 1, consumed += 1.*
   - Produce a token on `t`'s output place. *produced += 1.*
3. After the trace, consume the sink token (p_n). *consumed += 1.*
4. Count tokens remaining on any non-sink place. *remaining = uncollected producer tokens.*

The four counters `(p, c, m, r)` fully characterize the replay's health:
- `m = 0, r = 0` → perfect replay: every declared activity fired in order, nothing skipped.
- `m > 0` → activities were fired that required creating tokens (wrong order or extra steps).
- `r > 0` → activities were skipped (tokens were produced but never consumed).

### Why integer-only, no floats

`procmine` is `no_std`. There is no float library. The van der Aalst fitness formula:

```
fitness = ½ × (1 − m/c) + ½ × (1 − r/p)
```

is implemented in **basis points** (integer hundredths of a percent, 0..=10_000):

```rust
// From src/procmine/conformance.rs
pub fn fitness_bp(counts: &TokenCounts) -> u32 {
    let c = counts.consumed.max(1);
    let p = counts.produced.max(1);
    let left  = 10_000 * counts.consumed.saturating_sub(counts.missing) / c;
    let right = 10_000 * counts.produced.saturating_sub(counts.remaining) / p;
    (left + right) / 2
}
```

The `.max(1)` guards prevent division by zero without branching. Perfect fitness = 10,000.
The admission threshold is 9,000 bp — below that, `to_verdict` returns `Verdict::Refused`.

## Directly-follows graph discovery

Beyond replay, `dfg::Dfg` answers a different question: not "did the trace match the model"
but "what *directly-follows* relation does this trace (or collection of traces) imply, and
how does it compare to the model's?"

The directly-follows relation `a → b` holds when activity `b` immediately follows `a` in
some trace. Discovering it from a log and comparing to the model's declared relation is van
der Aalst's simplest process discovery method — one that scales to large logs without
alignment computation.

`Dfg` is a bounded, de-duplicated edge set (capacity: 64 edges). Key operations:

- `Dfg::from_model(m)` — the model's declared DFG (7 edges for an 8-step chain)
- `dfg.observe(trace)` — record all `a → b` pairs from one trace
- `discover(traces: &[Trace]) → Dfg` — the union DFG of a multi-trace log
- `discovered.order_divergence(&model_dfg) → DfgDivergence { extra, missing }`:
  - `extra` — edges in the observed log not in the model (unexpected directly-follows)
  - `missing` — edges in the model not seen in the log (skipped directly-follows)

A log of perfect traces discovers a DFG identical to the model (`{extra:0, missing:0}`).
Any reordering introduces `extra > 0` edges; any skip introduces `missing > 0`. DFG
divergence is a fast, allocation-free lens on the structural shape of what actually ran.

## OCEL export: object-centric event logs

The third module, `ocel_log`, bridges between process mining's evidence model and
wasm4pm's object-centric format:

- `events_for_chain(model, kernel_outputs: &[u64; 8]) → [OcelEvent; 8]` — maps each
  kernel step onto an `OcelEvent` with: the pattern id as `activity`, the step index as
  `timestamp`, the kernel output as the linked object id, and the pattern's declared
  `ObjectKind`s as object types. The event is traceable to a real object, not a log label.
- `trace_for_chain(events: &[OcelEvent]) → Trace` — extracts the activity sequence from a
  log, giving the observed trace for replay.
- `log_for_chain(model, kernel_outputs) → OcelLog` *(alloc feature)* — the same 8 events
  in a growable log, with `OcelLog::to_json()` for OCEL-flavored serialization.

The critical property of `events_for_chain`: the `objects` links in each event are
grounded in the kernel output that produced them. A `damage_applied` event links the damage
value it computed; a `waypoint_reached` event links the waypoint object it resolved. This
is what makes the log *object-centric* — events are not a flat sequence of labels, but a
bipartite graph of events and objects, exactly as OCEL-2.0 requires.

## The golden digest: drift detection for the conformance algorithm

`GOLDEN_CONFORMANCE_DIGEST` (pinned in `src/procmine/mod.rs`) is a frozen oracle for the
entire conformance pipeline:

```rust
pub fn conformance_digest() -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;  // FNV-1a seed
    for m in CHAIN_MODELS {
        let r = check_model_self(m);
        // fold: chain_id, fitness_bp, status, p, c, m, r, activity ids, ch:golden
        ...
    }
    h
}
```

It folds every chain's self-conformance result — including the raw token counts, fitness,
verdict status, the model's full activity sequence, and the chain's own `ch:golden` anchor —
into one rolling FNV-1a hash. Any of the following changes the digest:

- A kernel's output changes (kernel logic drift)
- The model's declared activity order changes (chains.ttl edit)
- The token replay algorithm changes (replay.rs edit)
- The fitness formula changes (conformance.rs edit)
- A chain's `ch:golden` changes (chains.ttl edit)

This mirrors `GOLDEN_CORPUS_DIGEST` in `corpus.rs`: one frozen number that would change if
any claimed behavior were silently altered. `assert_conformance_stable()` asserts the live
digest equals the pinned one; it is the conformance equivalent of `verify_corpus()`.

## The pipeline at a glance

```
chains.ttl ─ggen─► CHAIN_MODELS  ←── normative model (de jure)
                         │
                    ocel_log.rs
                         │  events_for_chain(model, kernel_outputs)
                         │  trace_for_chain(events)
                         ▼
                    Trace { acts: [u16; 8], len: u8 }   ←── observed (de facto)
                         │
                    replay.rs
                         │  replay(model.activities, trace.as_slice())
                         ▼
                    TokenCounts { produced, consumed, missing, remaining }
                         │
                    conformance.rs
                         │  fitness_bp(&counts)  →  u32 in 0..=10_000
                         │  to_verdict(&result)  →  Verdict
                         ▼
                    ConformanceResult { chain_id, fitness_bp, counts, trace_fits_order, status }
                         │
                    ┌────▼────┐        dfg.rs
                    │ Verdict │   ←── Dfg::order_divergence (structural comparison)
                    └─────────┘
                         │
                    GOLDEN_CONFORMANCE_DIGEST  (drift detector, frozen oracle)
```

## What this pipeline does NOT cover (the fences)

Like [The Honest Kernel](wasm4games-the-honest-kernel.md), the honest conformance pipeline
names its own edges:

- **Trace alignment is not implemented.** Token replay is the cheapest, most robust
  conformance metric; it handles noise and partial traces gracefully. Trace alignment
  (computing the edit distance between trace and model with move-on-log / move-on-model
  costs) gives more diagnostic detail but requires allocation and is significantly more
  expensive. It is not implemented here.

- **This is not the admission authority.** A fitness score of 10,000 bp and
  `Verdict::Admitted` from `to_verdict` is an *offline conformance* result — it says the
  trace reproduced the declared order. It does not say the external wasm4pm authority
  accepts the evidence. That verdict comes only from the workspace-excluded
  `wasm4games-wasm4pm` bridge (see the how-to:
  [Run real wasm4pm admission](../how-to/wasm4games-run-wasm4pm-admission.md)).

- **Multi-object conformance is not modeled.** The OCEL format links events to objects, but
  the current replay algorithm treats each trace as a flat sequence of activity labels. A
  full object-centric conformance check (one net per object type, interleaved replay) is
  future work.

- **The DFG does not scale to large logs without re-init.** The `Dfg` capacity is 64 edges.
  A single 8-step chain contributes at most 7 edges; the cap is not a concern for 8 chains.
  For a log of many distinct chains or shuffled traces that generate novel pairs, the cap
  saturates silently. This is a `no_std` constraint, not an oversight.

- **`GATE-W4G-PM-001` (regeneration receipt) is open.** `src/procmine/model.rs` carries the
  GENERATED banner and is reproducible by `ggen sync`, but the gate that would *receipt*
  byte-for-byte reproducibility is unverified because `ggen` is not installed in the CI
  environment. The covenant requires regeneration; the gate is still open.

## Why this is the right complexity

Token-based replay, integer fitness, a bounded DFG, and one rolling digest are exactly
enough to answer the three questions above. They are implemented in under 300 lines of
bounded, WCET-provable Rust. They run in a `#![no_std]` context with no allocator, on any
target `wasm4games` supports — including wasm32. Every function is bounded by the chain
length (8 steps) and the DFG capacity (64 edges); there are no unbounded loops.

The principle: adopt the minimum van der Aalst mechanism that makes the claim falsifiable.
Token replay does that. Everything else in the procmine literature is available to add later,
but only when there is a specific claim to support that the simpler metric cannot answer.
