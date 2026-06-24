# POWL v2 as Chess-Factory Manufacturing Architecture

**Version 2 — corrected framing**

The first version of this document argued POWL v2 enables "branchless alpha-beta." That claim is wrong.

**A CPU branch and a logical branch are different things.**

You can replace `if alpha >= beta` with a mask operation, but the search tree still bifurcates. The decision still exists. You have merely changed representation.

```text
POWL removes control-flow representation branches.
POWL does not remove search-space branching.
```

That distinction is critical. This document rebuilds from the correct claim.

---

## The Correct Claim

```text
POWL → topology-derived concurrency
```

Not:

```text
POWL → branchless search
```

These are radically different propositions with radically different competitive implications.

---

## Why Stockfish Cannot Do This

Stockfish's concurrency model is embedded in its implementation:

```text
Search Architecture
    ↓
Programmer
    ↓
Parallel Search Design (YBWC, lazy SMP, etc.)
    ↓
Threads
```

The developer decides: split here, don't split there, share TT, young brothers wait. Concurrency is written, not derived. Changing the concurrency model requires changing the code.

POWL introduces a scheduler between the graph and the workers:

```text
Search Architecture
      ↓
POWL Graph (activities + pred_mask + succ_mask)
      ↓
Scheduler (finds runnable ops automatically)
      ↓
Workers (CPU, GPU, WASM, remote — same graph)
```

The scheduler discovers concurrency. Nobody writes `spawn_thread`. Instead: `pred_mask satisfied → runnable`. The target substrate — CPU thread, GPU kernel, WASM worker — is a scheduler concern, not a graph concern.

---

## Two Graphs at Different Scales

There are two distinct POWL graphs in this architecture. Confusing them is the central error of the previous draft.

### Graph 1: Search Graph (microseconds)

Models search phases, not individual moves and not runtime depth:

```text
powl PvSearchNode {

  xor TerminalGate {
    MateOrDrawReturn;
    DepthZeroToQSearch;
    ContinueSearch;
  }

  sequence {
    ProbeTT;
    StaticEvalWithNNUE;
    ApplyPruningGates;
    GenerateMoves;
    OrderMoves;

    loop MoveLoop {
      sequence {
        MakeMove;

        xor SearchMode {
          FullWindowPVSearch;
          NullWindowScoutSearch;
          ReSearchIfImproved;
        }

        UnmakeMove;

        xor ResultGate {
          BetaCutoff;
          AlphaImprovement;
          NoImprovement;
        }

        UpdateNodeStats;
      }
      until Done;
    }

    StoreTTBound;
    ReturnScore;
  }
}
```

**What POWL models:** search phases, cutoff gates, TT gates, iterative deepening loops, aspiration loops, qsearch transitions.

**What runtime still owns:** move generation, dynamic depth, alpha/beta values, NNUE accumulators, TT mutation.

**What POWL explicitly does not model:** individual move nodes, runtime depth as topology (`D0`, `D1`, `D17`). Depth is runtime data, not topology. `SearchState<D17>` does not scale. `SearchState<TtProbed>` does.

### Graph 2: Manufacturing Graph (minutes to hours)

```text
powl TopologyManufacturing {

  sequence {
    GenerateTopologyVariants;

    partial-order BenchmarkMatrix {
      BenchmarkVariantA;
      BenchmarkVariantB;
      BenchmarkVariantC;
      BenchmarkVariantD;
    }

    CollectWDLReceipts;
    RankByScore;
    PromoteWinner;
    EmitSearchSpec;
  }
}
```

The benchmark ops all have `pred_mask: {GenerateTopologyVariants complete}` and `succ_mask` pointing to collection. The scheduler fans them out automatically — no explicit parallelism written anywhere.

This graph operates at the level of search topologies, not search nodes. It is worth more competitive value than Graph 1.

---

## The Phase TypeState Pattern (Correct Version)

The previous draft proposed depth types (`D0`, `D1`, `D2`, ...). This was wrong.

Stockfish searches to depth 20, 30, 40+. Manufacturing 64 TypeState depth types is not useful.

**Correct:** Phase types within a single search node:

```rust
// Zero-sized phase tokens — compile-time proof of ordering
pub struct Initial;
pub struct TtProbed;
pub struct MovesGenerated;
pub struct MovesOrdered;
pub struct MovesSearched;
pub struct Resolved;

// Engine typed by phase, not depth
pub struct SearchState<Phase> {
    board: Board,
    alpha: i32,
    beta: i32,
    depth: usize,        // runtime data — not a type parameter
    _phase: PhantomData<Phase>,
}

// Transitions enforce ordering at compile time
impl SearchState<Initial> {
    pub fn probe_tt(self) -> Either<(i32, SearchState<Resolved>), SearchState<TtProbed>> {
        // TT hit → immediately Resolved (skip remaining phases)
        // TT miss → TtProbed (continue pipeline)
    }
}

impl SearchState<TtProbed> {
    pub fn generate_moves(self) -> SearchState<MovesGenerated> { ... }
}

impl SearchState<MovesGenerated> {
    pub fn order_moves(self) -> SearchState<MovesOrdered> { ... }
}

impl SearchState<MovesOrdered> {
    pub fn search_all(self) -> SearchState<MovesSearched> { ... }
}

impl SearchState<MovesSearched> {
    pub fn resolve(self) -> SearchState<Resolved> { ... }
}
```

**Type-system guarantee:** You cannot call `.order_moves()` before `.generate_moves()`. You cannot skip `TtProbed` to reach `MovesGenerated`. TT hits short-circuit to `Resolved` through a typed path — no unchecked returns.

**Compile-time proofs:**

- ❌ `SearchState<Initial>.order_moves()` — `order_moves` requires `MovesGenerated`
- ❌ `SearchState<TtProbed>.search_all()` — `search_all` requires `MovesOrdered`
- ❌ `SearchState<MovesOrdered>.probe_tt()` — `probe_tt` is only available on `Initial`

**What this gives you:** The search phase pipeline is machine-checked. Phase violations are compile errors, not runtime bugs.

---

## The Correct POWL Search Topology

### Full Stockfish-as-POWL

```text
powl StockfishSearch {

  sequence {
    AdmitUciCommand;
    AdmitPosition;
    ConfigureLimits;
    MaybeProbeOpeningBook;

    loop IterativeDeepening {
      sequence {
        StartDepth;
        MaybeAdjustAspirationWindow;

        xor AspirationSearch {
          SearchWithinWindow;
          ReSearchLow;
          ReSearchHigh;
        }

        UpdateBestMove;
        UpdateTimeManager;
        EmitInfo;
      }
      until StopCondition;
    }

    EmitBestMove;
  }
}
```

### Root Search

```text
powl RootSearch {

  sequence {
    ProbeTranspositionTable;
    GenerateLegalMoves;
    OrderMoves;

    loop RootMoveLoop {
      sequence {
        MakeMove;
        SearchChild;
        UnmakeMove;

        xor ScoreOutcome {
          AlphaRaise;
          BetaCutoff;
          IgnoreMove;
        }

        UpdatePV;
        UpdateTT;
        UpdateHistoryStats;
      }
      until MovesExhaustedOrStopped;
    }

    StoreRootResult;
  }
}
```

### QSearch

```text
powl QSearch {

  sequence {
    StandPatEvalWithNNUE;

    xor StandPatGate {
      BetaCutoff;
      AlphaRaise;
      ContinueCaptures;
    }

    GenerateCapturesAndChecks;
    OrderBySEE;

    loop TacticalMoveLoop {
      sequence {
        MakeMove;
        QSearchChild;
        UnmakeMove;
        UpdateAlphaOrCutoff;
      }
      until TacticalMovesExhausted;
    }

    ReturnQuietScore;
  }
}
```

### NNUE as Subworkflow

```text
powl NNUEEval {

  xor AccumulatorState {
    RefreshAccumulator;
    IncrementallyUpdateAccumulator;
  }

  sequence {
    SelectPerspective;
    ApplySparseFeatureDelta;
    ClippedRelu;
    IntegerForwardPass;
    ScaleToCentipawns;
    ReturnEval;
  }
}
```

---

## Auto-Concurrency: Where the Value Lives

Because POWL has explicit dependency masks, the scheduler automatically finds work that is **ready now**. Concurrency is derived from the graph, not written by the programmer.

### Root Move Parallelism

Root moves with `pred_mask: 0` are all immediately runnable:

```text
RootMoveA  pred_mask: 0    → runnable
RootMoveB  pred_mask: 0    → runnable
RootMoveC  pred_mask: 0    → runnable
RootMoveD  pred_mask: 0    → runnable
```

The scheduler fans them out — to CPU threads, GPU kernels, or WASM workers — without the graph knowing which substrate runs them. The same topology graph runs single-threaded, multi-threaded, or distributed.

### Safe vs Dangerous Auto-Concurrency

Not all concurrency is safe for chess strength:

**Safe** (auto-parallelize freely):
- Root moves at depth 1
- Eval stations per position
- Motif detectors per position
- Benchmark matches between topology variants
- Architecture mutation jobs
- Opening book hash generation

**Dangerous** (preserve alpha-beta ordering within each worker):
- Deep sibling nodes before first move proves alpha (ruins move ordering)
- LMR-dependent searches (must see full window first)
- Aspiration re-searches (sequential by definition)
- TT mutation without shards (data race)

**The rule:** POWL handles macro-concurrency. Rust search handles micro-recursion. These do not mix.

---

## Three Levels of Competition

Stockfish is strongest at Level 1. The factory is designed to compete at Level 3.

### Level 1 — Search

Traditional Stockfish. One hand-tuned search tree. Depth, time, heuristics.

### Level 2 — Search Topology

POWL describes the phases, gates, and loops. TypeState enforces ordering. The POWL graph is the **constitutional description** of what the engine does. Rust is one implementation that satisfies the constitution. Later: CUDA, SIMD, WASM, FPGA could all satisfy the same POWL contract.

### Level 3 — Architecture Search

```text
Topology A
Topology B
Topology C
Topology D
```

All manufactured from TTL. Then:

```text
Benchmark Matrix (Manufacturing Graph)
```

runs them in parallel, receipts each W/D/L, promotes the winner.

**The factory is not searching chess positions at this level. It is searching search architectures.**

Stockfish optimizes one architecture by hand. The factory manufactures many lawful architectures, benchmarks them, receipts them, and keeps the winners. That is the actual competitive edge — not execution speed on a single architecture, but manufacturing velocity across an architecture population.

---

## Manufacturing Responsibilities

```text
POWL v2 manufactures:
  search topology (phases, gates, loops)
  phase ordering proofs (TypeState)
  concurrency schedule (pred_mask derivation)
  OCEL replay traces (one event per activity)
  verification receipts (W/D/L per topology)
  LLM implementation specs (per hand-authored plugin)

Rust hand-authors:
  recursive alpha-beta body
  TT mutation
  move generation
  NNUE inference
  qsearch
  time manager

GGEN branchless TTL manufactures:
  eval station fallback (passed_pawn, rook_open_file, etc.)
  tactical motifs
  Q8.8 weights
  oracle proptests
```

---

## The Crown Sentence

```text
Stockfish-in-POWL is not branchless everywhere.
It is admitted control-flow everywhere.
```

**POWL v2 becomes the lawful map of branching** while GGEN remains the lawful manufacturer of branchless kernels. These are different tools for different layers, and neither replaces the other.

The strongest version is:

```text
Stockfish optimizes one architecture.
Chess-factory manufactures architectures.
```

POWL makes this possible because it turns hidden recursive control flow into explicit graph topology that can be inspected, mutated, compiled to multiple substrates, and benchmarked automatically. The Manufacturing Graph — not the Search Graph — is where the factory gains its structural advantage over hand-authored engines.

---

## Roadmap

### Phase 1: TypeState on search.rs (2 weeks)

- [ ] Define phase tokens: `Initial`, `TtProbed`, `MovesGenerated`, `MovesOrdered`, `MovesSearched`, `Resolved`
- [ ] Rewrite `search.rs` public API to use `SearchState<Phase>` in signatures
- [ ] Verify the type system catches phase-ordering violations
- [ ] Behavior unchanged; only the type-level proof is new

### Phase 2: POWL Search Graph (3 weeks)

- [ ] Compile phase transitions to flat `Powl64Op` array
- [ ] Replace recursive `ab()` with `powl64_execute_step` loop
- [ ] Benchmark vs baseline — expect ~same or small improvement
- [ ] Root-move ops get `pred_mask: 0` → auto-parallel on free workers

### Phase 3: Manufacturing Graph (4 weeks)

- [ ] Define `cf:SearchTopology` TTL class
- [ ] GGEN generates topology variants from TTL (gate ordering, pruning thresholds)
- [ ] Manufacturing Graph fans out benchmark jobs from `pred_mask: 0`
- [ ] Collect W/D/L receipts per topology
- [ ] Promote winner via `ggen sync` updating weights

### Phase 4: Architecture Search Loop (ongoing)

- [ ] Mutate POWL Search Graphs legally (preserve soundness)
- [ ] Benchmark each mutation automatically
- [ ] Archive receipts; evolve the winning lineage
- [ ] Specialize for 100µs window, then 250µs, then 1ms

---

---

## Phase-Adaptive Topology: The 100µs Differentiator

Chess has distinct phase groups. The optimal search control-flow is **different per phase, time budget, and hardware profile**. Running one universal search loop is a structural loss at ultra-short time controls.

### The Five Phase Groups

| Phase | POWL Graph Bias | Why |
|-------|-----------------|-----|
| **Opening** | book probe → theory preference → shallow verify | Don't spend 100µs rediscovering known moves |
| **Early middlegame** | root-parallel topology variants | Many plausible plans; evaluate alternatives concurrently |
| **Tactical crisis** | SEE → qsearch → check/capture forcing graph | Forcing lines matter more than broad search |
| **Quiet middlegame** | plan/eval-heavy graph | More value from positional eval, king safety, pawn structure |
| **Endgame / tablebase** | tablebase probe first, material rule graph | Do not approximate solved positions |

### Phase-Aware Control Flow

The factory can make the **entire control-flow graph phase-aware**:

```text
Position
→ PhaseClassifier (game phase scalar, material count, pawn structure)
→ Select POWL Graph
→ Execute phase-specific scheduler
→ Receipt result
```

Concrete topologies:

```text
powl OpeningPOWL {
  sequence {
    BookProbe;
    xor BookGate { TheoryMoveVerify; FallbackSearch; }
    FastReturn;
  }
}

powl TacticalPOWL {
  sequence {
    InCheckGate;
    GenerateForcingMoves;
    SEEFilter;
    QSearchDeepen;
    ReturnTacticalScore;
  }
}

powl QuietMiddlegamePOWL {
  sequence {
    ProbeTT;
    StaticEvalStations;   // eval station batch, all pred_mask: 0 → parallel
    KingSafetyWeight;
    PawnStructureWeight;
    PVS;
    StoreTT;
  }
}

powl EndgamePOWL {
  sequence {
    TablebaseProbe;
    xor TablebaseGate {
      ReturnDTZ;
      MaterialRuleGraph;
    }
    PassedPawnRace;
    PreciseSearch;
  }
}
```

### Why This Matters at 100µs

At 100µs per move, the wrong graph is lethal:

```text
Opening position + TacticalPOWL = wasted work on captures that don't exist
Endgame position + OpeningPOWL  = book probe on a position with 3 pieces
Tactical crisis  + QuietPOWL    = eval stations while opponent has a queen fork
```

Stockfish has phase-aware heuristics, but they are embedded **inside** the hand-authored search. The graph shape itself does not change. The factory's structural advantage: **the graph is the thing that changes.** Different phases select different POWL graphs; the Rust implementation inside each graph can remain standard alpha-beta.

### Phase Classification as a POWL Activity

The phase classifier is itself a POWL Activity with `pred_mask: 0`:

```text
PhaseClassify
  pred_mask: 0
  succ_mask: {OpeningBit | TacticalBit | QuietBit | EndgameBit}
```

Its `succ_mask` gates the entire search graph selection. A `ChoiceGate` downstream selects exactly one topology. This is not a branch — it is an **admitted gate**: the decision exists, is explicit in the graph, and produces a receipt.

### Manufacturing Phase Topologies

The Manufacturing Graph searches the full product space:

```text
Phase × Hardware × Time Budget × Topology

  OpeningPOWL-A_1Core_100µs
  OpeningPOWL-A_4Core_100µs
  OpeningPOWL-A_16Core_100µs
  OpeningPOWL-B_1Core_100µs
  OpeningPOWL-B_4Core_100µs
  ...
  TacticalPOWL-A_1Core_100µs
  TacticalPOWL-A_4Core_100µs
  ...
```

Each combination is an Activity in the Manufacturing Graph with `pred_mask: 0`. All are independent — no alpha-beta coupling, no TT sharing, no ordering dependencies. This is **embarrassingly parallel**: the scheduler fans every combination across available workers simultaneously, at no implementation cost.

Contrast with root-move parallelism inside search, which has ordering constraints (early cutoffs reduce siblings worth searching). Manufacturing-graph concurrency has none of those constraints. It is the larger gain.

### The AutoML Observation

After 100,000 benchmark games, the Manufacturing Graph may promote a topology nobody understands:

```text
OpeningGraph_F  wins  +2 Elo  over  OpeningGraph_A
```

The receipt proves it:

```text
WDL: W=52.3% D=18.1% L=29.6%
cutoff_rate: 0.71
node_count: 4,312 avg
TT_hit_rate: 0.43
phase_bucket: opening
hardware: 4core_100µs
```

Nobody hand-designed this. Nobody knows why it wins. The receipt is the proof. This is where the architecture stops being a chess engine and starts behaving like AutoML for search topology.

Stockfish's advantage is the accumulated human intuition in its heuristics. The factory's advantage is that it can replace human intuition with receipted evidence, and it can do so automatically across the full manufacturing search space.

This is the full combinatorial advantage:

```text
Stockfish: one architecture, one topology, hand-tuned by humans over decades.
Factory:   Phase × Hardware × Time Budget × Topology product space,
           manufactured, benchmarked, and receipted automatically.
```

---

---

## The 100µs Law: Compile Prior Intelligence, Not Runtime Intelligence

At 100µs per move, the factory does not discover the right architecture during the move. It pre-manufactures the architecture before the game.

```text
Do not search for the right architecture during the move.
Compile the winning architecture before the game.
```

This is the manufacturing principle applied to chess. The factory's edge at 100µs is **less runtime intelligence, more manufactured prior intelligence.**

### O* Is Richer Than Board + Phase

The Chatman Equation's admitted state `O*` is not just `(board, phase)`. The full admission includes:

```text
O* = {
  board,               // legal position
  phase,               // opening / tactical / quiet / endgame
  time_budget,         // 100µs vs 1ms vs 10s
  hardware_profile,    // 1 core vs 4 core vs 16 core
  worker_count,        // available POWL workers
  tt_occupancy,        // TT fill fraction (affects probe cost)
  thermal_state,       // sustained clock rate available
}
```

This matters because:

```text
Opening + 100µs + 16 cores  ≠ same graph as  Opening + 100µs + 1 core
```

A root-parallel topology wastes time on 1 core. A single-thread deep topology wastes cores on 16. The winning graph is a function of the full `O*`, not just the board.

Admission is the lawful routing layer:

```text
Position does not automatically enter a graph.

First: admit board, phase, budget, hardware, evidence.
Then: μ selects topology.
```

This prevents the engine from running the wrong architecture for its environment — a class of error that has no name in traditional chess engines because there is only one architecture.

### The 100µs Pipeline

```text
AdmitO*
→ HashProbe                (TT: O(1))
→ PhaseClassify            (material count: O(1))
→ SelectCompiledPOWLGraph  (table lookup by (phase, budget_bucket, hardware_class): O(1))
→ ProbeBook/Tablebase/TT   (sorted hash: O(1))
→ RunTinySearch            (remaining budget)
→ EmitMoveReceipt
```

Almost all budget goes to search. Every other step is a compile-time artifact selected in O(1) time. The specific nanoseconds are implementation details — the architectural claim is O(1) topology selection, which holds regardless of substrate.

### Compile-In Advantages

| Advantage | Compile-time artifact | Runtime benefit |
|-----------|----------------------|-----------------|
| **Phase-specific POWL graphs** | `OpeningGraph`, `TacticalGraph`, `EndgameGraph` | No universal search overhead |
| **Opening book / reply table** | sorted hash → move table | Zero-cost theory move |
| **Material-class graphs** | graph per material signature | Better endgame routing |
| **Tactical crisis graph** | forcing-move topology | Checks/captures prioritized immediately |
| **Precomputed LMR table** | `[[u8; 64]; 64]` | No floating-point math in hot loop |
| **SEE tables / piece values** | fixed capture classifier | Avoids losing captures without search |
| **Phase-specific move ordering** | ordering formula per graph | Better first move → earlier cutoffs |
| **Station weights** | Q8.8 constants | No dynamic tuning at runtime |
| **Root parallel graph** | independent root ops with `pred_mask: 0` | Auto-concurrency at root level |
| **Benchmark-promoted topologies** | only winners compiled in | No weak variants waste budget |

### What Stockfish Carries at 100µs

Stockfish at 100µs still carries a general elite engine. It pays overhead for features designed for 10-second time controls: deep iterative deepening infrastructure, sophisticated time management, multi-PV support, syzygy tablebase infrastructure. At 100µs most of this overhead buys nothing.

### What the Factory Carries at 100µs

```text
opening micro-engine     (book → verify → return)
tactical micro-engine    (SEE → qsearch → forcing)
quiet micro-engine       (station batch → PVS)
endgame micro-engine     (tablebase → material rules)
low-time micro-engine    (ultra-shallow + high-quality ordering)
```

POWL selects the right one in ~2ns. The remaining 98µs goes entirely to search inside the selected micro-engine with no general infrastructure tax.

### Priority Order for 100µs

```text
Priority 1: Opening hash book
            Cost: ~5ns per probe, ~50MB for 100K positions.
            Return: full 100µs to search after book exit.

Priority 2: Tactical crisis classifier
            Detects: checks, captures, threats before search begins.
            Return: TacticalGraph avoids quiet search on forcing positions.

Priority 3: SEE + qsearch
            Avoids: losing captures without recursive search.
            Return: better move ordering → earlier cutoffs → more nodes in budget.

Priority 4: Phase-specific move ordering
            Per-graph ordering formula (opening: theory bias, tactical: SEE-first, endgame: king-active-first).
            Return: first move searched is better → more cutoffs → more effective depth.

Priority 5: Root-parallel POWL graph
            Root moves with pred_mask: 0 → all runnable → scheduler fans to available cores.
            Return: hardware-level parallelism with zero thread management code.

Priority 6: Benchmark-promoted topology table
            Manufacturing Graph promotes winners per phase bucket.
            Return: topology selection improves over games automatically.
```

### The Compile-In Manufacturing Loop

```text
Manufacture phase graphs from TTL
    ↓
Benchmark each graph per phase bucket per time window
    ↓
Promote winners to compiled topology table
    ↓
Deploy: runtime selects from table in O(1)
    ↓
Collect new game evidence → repeat
```

This loop runs offline. The 100µs runtime sees only the promoted winners, never the manufacturing overhead.

---

## The Complete Picture: Chatman Equation + Combinatorial Maximalism

The Chatman Equation applied to chess-factory:

```text
A = μ(O*)

O* = admitted position state
     (legal board + phase classified + time budget + TT state + material profile)

μ  = selected POWL topology + Rust search executor

A  = receipted best move
```

Every move becomes a lawful admission cycle:

```text
1. Admit O*
   - board is legal (chess crate validates)
   - phase classified (opening / tactical / quiet / endgame)
   - time budget known (100µs)
   - TT state bounded (1<<17 slots)
   - material profile known (Q8.8 station weights)

2. Manufacture candidate POWL graphs
   - OpeningGraph (book → verify → fast return)
   - TacticalGraph (SEE → qsearch → forcing lines)
   - QuietGraph (station batch → PVS → eval-heavy)
   - EndgameGraph (tablebase → material rules → precise search)
   - LowTimeGraph (shallow + high-quality ordering only)

3. Execute benchmark matrix
   - W/D/L per topology variant
   - nodes searched
   - cutoff rate
   - TT hit rate
   - qsearch explosion rate
   - blunder rate per phase

4. Promote graph
   - only if W/D/L receipt beats receipted baseline
   - promotion evidence required — no intuition, no hand-tuning

5. Replay
   - OCEL trace proves why the graph won
   - conformance check: actual search phases match declared POWL topology
```

This is not "improve the engine." This is:

```text
Manufacture every lawful engine variant.
Prove which one wins.
Keep only receipted improvements.
```

### The Full Stack

```text
Chatman Equation
    ↓
Admission
  (board + phase + budget + hardware + evidence → O*)
    ↓
POWL Graph Selection
  O(1) table lookup by (phase, hardware_class, budget_bucket)
    ↓
Rust Search
  (alpha-beta, TT, NNUE, move gen inside selected graph)
    ↓
Receipt
  (WDL + cutoff_rate + TT_hit_rate + node_count)
    ↓
Manufacturing Graph Feedback
  (receipt → rank → promote winner → update compiled table)
    ↓
↑ next game reads promoted table ↑

Layer responsibilities:
  Chatman Equation       = lawful admission + receipt (no heuristic soup)
  Combinatorial Maximalism = Phase × Hardware × Time Budget × Topology space
  POWL v2               = executable topology (pred_mask / succ_mask / gates)
  GGEN                  = manufacturing system (TTL → Rust stations + oracle tests)
  Rust                  = hot-path implementation (alpha-beta, TT, NNUE, move gen)
```

### Why This Beats Stockfish's Architecture (Not Stockfish's Strength)

Stockfish is stronger today. That is not the claim.

The claim is structural:

```text
Stockfish explores: one point in (Phase × Hardware × Time Budget × Topology) space, very deeply.
Factory explores:   the full product space automatically, with receipts proving which point wins.
```

At ultra-short time controls (100µs), topology selection matters more than heuristic depth. A phase-appropriate topology with shallower search beats a universal deep search running the wrong graph for its environment.

The gap is structural and it widens automatically: every game produces receipts, receipts update the topology table, the table improves without human intervention. Stockfish's heuristics were accumulated by humans over decades. The factory's topology table accumulates by running games.

---

## References

- `playground/src/powl.rs` — POWL v2 reference executor (pred/succ mask algebra)
- `wasm4pm_compat::powl` — POWL v2 TypeState/TypeScript specs
- `crates/chess-factory/ontology/chess.ttl` — GGEN source of truth for eval stations
- `crates/chess-factory/src/search.rs` — current hand-authored search (Phase 1 target)
- Stockfish `search.cpp` — the topology this document maps to POWL
