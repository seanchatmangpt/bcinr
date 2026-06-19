# wasm4games: The Game-Pattern Foundry

> Explanation — the *why* and *shape* of `crates/wasm4games`. For the rule that governs
> how you change it, read the companion [GGEN-Only User Surface](ggen-only-user-surface.md)
> covenant. To add a pattern, follow the
> [tutorial](../tutorials/wasm4games-add-a-pattern.md).

## What it is

`wasm4games` is a **branchless, `no_std`, engine-agnostic game-pattern foundry**. It does
not draw frames, own an entity-component store, or schedule a render loop. It does one
thing: it takes recurring *game behaviors* — input admission, fixed ticks, entity state
transitions, collision resolution, combat and status effects, physics rendering,
evidence sealing, and player-promotion gating — and reduces each one to a **bounded
byte-class state transformed by a branchless kernel** that emits **object-centric
evidence**.

It is a *foundry* rather than an engine because it manufactures the reusable, verifiable
core of a behavior — the part that must be deterministic, latency-stable, and auditable —
and leaves world projection to whatever engine you bolt it to (Bevy, a custom ECS, a
WASM canvas, a headless server). The foundry stamps out patterns; engines decide where
the patterns live.

## The three-part model

Every pattern in `wasm4games` is built from the same three ideas.

### 1. Bounded byte-class state

State is not arbitrary structs. It is small, bounded `u8` alphabets — *byte classes* —
of known cardinality. The canonical example is the admission status lattice in
[`class::status`](../reference/wasm4games-patterns.md#the-classstatus-vocabulary):
`UNKNOWN(0) → BLOCKED(1) → PARTIAL(2) → PENDING(3) → ADMITTED(4) → PROJECTED(5) →
RECEIPTED(6)`, with the terminal/abnormal codes `REFUSED(7)` and `RESIDUAL(8)` sorting
*above* the normal lifecycle, and `COUNT = 9` as the cardinality.

The ordering is deliberate. Because abnormal codes sort high, a *worst-of* join over many
sub-statuses is just a branchless `max`:

```rust
use wasm4games::prelude::{status, Status};

let admitted = Status(status::ADMITTED);
let refused = Status(status::REFUSED);
// No `if`, no `match`: max-code lattice join, computed branchlessly.
assert_eq!(admitted.join(refused).raw(), status::REFUSED);
```

`ByteClass` carries the same discipline for any other alphabet: it clamps into `[0, card)`
via `bcinr_logic::fix::clamp_u32`, so an out-of-domain code is corralled, never panics, and
never branches.

### 2. Branchless kernels on `bcinr-logic`

A *kernel* is the function that advances a pattern's byte-class state. Kernels are
branchless by construction: conditions become masks, saturating arithmetic, lookup
tables, bitset/popcount, DFA transitions, or sorting networks. They do not contain their
own arithmetic tricks — they **lower onto the verified primitives in `bcinr-logic`**:

| Lowering (`ir::LoweringKind`) | bcinr-logic substrate |
|-------------------------------|-----------------------|
| `Mask`       | `bcinr_logic::mask` (branchless select / `max_u32`) |
| `Saturating` | `bcinr_logic::int` / `fix` (saturating + clamp) |
| `Lut`        | table-driven dispatch |
| `Bitset`     | `bcinr_logic::bitset` (popcount, set ops) |
| `Dfa`        | `bcinr_logic::dfa` (deterministic transitions) |
| `Network`    | `bcinr_logic::network` (sort / permutation networks) |
| `Receipt`    | `bcinr_logic::patterns::integrity_receipt` (FNV-1a rolling hash) |

This is what lets `wasm4games` inherit bcinr's guarantees for free: constant-time
behavior, `#![forbid(unsafe_code)]`, side-channel resilience, and the formal-verification
posture described in the project's [PhD Gates](../reference/phd_gates.md). The foundry adds
*game semantics*; the substrate supplies the *branchless calculus*.

### 3. Object-centric evidence

A kernel does not just mutate state — it can *witness* what it did. Each meaningful
transition emits evidence in four mirrored channels (see
[`evidence`](../reference/wasm4games-patterns.md#evidence-types)):

- **OCEL events** ([`evidence::ocel::OcelEvent`]) — an *activity at a logical tick linked
  to real objects*. The links live in the event itself (`ObjectRefs`, bounded to 4),
  which is what makes the log *object-centric* rather than a flat trace. The default build
  is allocation-free and `Copy`; an `alloc`-gated `OcelLog` adds a growable stream and a
  hand-rolled, dependency-free OCEL-flavored JSON export.
- **OTEL span codes** ([`evidence::otel::SpanCode`]) — 16-bit runtime span identifiers so
  hot paths never carry strings; names are resolved only at the boundary.
- **Receipt chains** ([`evidence::receipt::ReceiptChain`]) — a tamper-evident rolling
  FNV-1a hash over emitted events. This is a *telemetry* receipt (witnesses execution
  order so replays can be compared), not a cryptographic signature.
- **Replay frames** ([`evidence::replay::ReplayFrame`]) — `(tick, input, state_digest)`
  triples. Re-folding the same frames must reproduce an identical digest; that determinism
  is precisely what gives a replay evidentiary value.

Because the four channels share the same byte-class status codes and the same logical
tick, they corroborate one another: the OCEL event, its span, its receipt fold, and its
replay frame all describe the *same* transition.

## How it sits on bcinr-logic

```
                ┌─────────────────────────────────────────────┐
                │  wasm4games  (crates/wasm4games)             │
                │  byte-class state + branchless kernels +     │
                │  object-centric evidence                     │
                │  class · ir · patterns · evidence · verify · │
                │  compat                                       │
                └───────────────────────┬─────────────────────┘
                                        │ lowers onto
                ┌───────────────────────▼─────────────────────┐
                │  bcinr-logic  (verified branchless calculus) │
                │  mask · int · fix · bitset · dfa · network · │
                │  scan · reduce · integrity_receipt           │
                └─────────────────────────────────────────────┘
```

`wasm4games` re-exports `bcinr_logic::Branchless` so downstream code uses a *single*
notion of "branchless," and it carries the same `#![no_std]` / `#![forbid(unsafe_code)]`
header. There are no runtime dependencies: the logic substrate is pulled in by path, and
`criterion`/`proptest` are testing-only. Feature flags are additive — `alloc` enables the
growable `OcelLog`, and `std` (which implies `alloc`) enables the full test suite.

## The layered doctrine

`wasm4games` only makes sense inside a four-layer separation of concerns. Each layer has
exactly one job, and no layer reaches into another's:

> **Engines project worlds; wasm4games operates patterns; wasm4pm admits evidence; ggen
> manufactures the law.**

- **Engines project worlds.** Rendering, ECS, scheduling, audio, input devices. The
  foundry never touches this layer; it is engine-agnostic on purpose.
- **wasm4games operates patterns.** The bounded-state, branchless, evidence-emitting core
  of each behavior. This is the crate you are reading about.
- **wasm4pm admits evidence.** The external authority that *decides* whether emitted
  evidence is admissible or must be refused. wasm4games proposes; wasm4pm disposes.
- **ggen manufactures the law.** The kernels are not hand-written. They are *generated*
  from an RDF ontology by ggen. The ontology is the law; the generator stamps it into
  Rust. This is the subject of the [covenant](ggen-only-user-surface.md).

## The wasm4pm relationship: referenced, not vendored

`wasm4games` is *built to be admitted by* `wasm4pm`, but it does **not** vendor it. The
crate ships only **dependency-free mirror types** in [`compat`] — `EvidenceState`,
`Verdict`, and `ReceiptChain`/`ReceiptEnvelope` shapes — modeled 1:1 on the canonical
`wasm4pm-compat` types. This keeps the offline build free of any git dependency, which is
what lets bcinr's strict, network-free `-D warnings` CI compile `wasm4games` at all.

Real admission is *not* claimed offline. The in-crate [`verify`] module is an explicit
**offline proxy**: it confirms status codes stay in-bounds, replays are deterministic, and
it constructs **negative fixtures** (`verify::negative_fixture`) — events stamped with a
pattern's refusal status that a correct authority *must* refuse. Turning that proxy into a
real verdict is the job of the **workspace-excluded** `wasm4games-wasm4pm` bridge crate,
which depends on the actual repositories and is built online (see the how-to:
[Run real wasm4pm admission](../how-to/wasm4games-run-wasm4pm-admission.md)).

The authoritative external surfaces are:

- wasm4pm — <https://github.com/seanchatmangpt/wasm4pm>
- wasm4pm-compat — <https://github.com/seanchatmangpt/wasm4pm-compat>

## Pattern families

The catalog (mirrored by `patterns::PATTERN_REGISTRY` and enumerated in the
[pattern reference](../reference/wasm4games-patterns.md)) holds **75 patterns**: ids 1–70
across 14 families, plus a 15th **Anti-Cheat** family (ids 71–75). The original spine is
four families:

- **Core sim & combat** — the deterministic spine of a game loop: input admission, fixed
  ticks, entity state machines, AABB collision resolution, damage application, and status
  effects.
- **Evidence & replay** — turning transitions into audit trails: receipt appends, OCEL
  event linking, OTEL span emission, and deterministic replay-frame recording.
- **Physics & meaning** — translating raw simulation into renderable, level-of-detail-aware
  signals: physics-value rendering, semantic LOD selection, and projectile advancement.
- **Promotion & NPS** — closing the loop with the player: detecting mastery moments,
  generating shareable artifacts, and gating Net-Promoter-Score prompts so they fire only
  at admissible moments.

Ten more families extend the catalog to id 70 — **Pathfinding, Procedural Generation,
Economy / Progression, Narrative / Dialogue, Camera, Audio, Multiplayer / Network, DfLSS /
Quality, Engine Bridge,** and **AI Agent / Benchmark** — and the **Anti-Cheat** family
(ids 71–75) adds the first *detector* kernels (covered below).

Each family is just a tag on the underlying `PatternSpec` data; the families exist so the
catalog can be browsed, tested, and benchmarked by concern. Every pattern, in every
family, obeys the same model — bounded byte-class state, a branchless kernel on
`bcinr-logic`, object-centric evidence — and is **generated, never hand-coded**.

## The breed-rigor battery and the anti-cheat proving case

A pattern's *value* is its claim to be deterministic, branchless, and correct — so every
generated kernel ships with an in-file battery designed to make that claim falsifiable
rather than merely asserted. The standard, applied to all 75 patterns, is:

- **A branchful `_reference` oracle** next to the branchless body, with an `equivalence`
  proptest pinning the two together — the fast form is judged against a readable form, not
  against itself.
- **Three value-mutants (`cf1`/`cf2`/`cf3`)** — negate / off-by-one / bit-flip variants the
  suite must be able to tell apart from the correct answer, proving the tests are sharp
  enough to *catch* a wrong result.
- **A two-sided fixture corpus (`must_admit` / `must_refuse`)** — the kernel must accept the
  legal cases *and* reject the illegal ones; the refuse side is what stops a rubber-stamp
  kernel from passing.
- **A `weakened` variant with `weakened_fails_corpus`** — a deliberately broken kernel that
  the corpus must catch (it admits ≥1 case it should refuse), proving the corpus itself has
  teeth: removing the check is *observable*.
- **`boundaries`, a dormant `bench` module, and two Hoare-logic lines** rounding out each
  file.

All of it folds into one **`GOLDEN_CORPUS_DIGEST`** (pinned in
`crates/wasm4games/src/corpus.rs`) that every projection target must reproduce. The full
epistemic argument — how these compose to answer "how do I know it's right / the test
isn't lying / what I ran is what you ran" — is laid out in
[The Honest Kernel](wasm4games-the-honest-kernel.md).

The **Anti-Cheat** family is the proving case for this battery. Its kernels do not advance
state; they render a **verdict bitmask** — `0` = ADMITTED (legal), nonzero = refused, with
each set bit naming a refusal reason — computed branchlessly so an illegal input takes the
same path as a legal one. Each detector states its own **authority** in its doc comment
(the local legality rule it enforces: e.g. "a move is admissible iff the displacement does
not exceed the actor's max speed"), and that authority is exactly what its two-sided corpus
admits and refuses. A detector is the clearest case where a one-sided corpus would lie —
"admit everything" passes any all-legal suite — so the `must_refuse` corpus plus the
`weakened` falsifier are what give the anti-cheat claim its force.
