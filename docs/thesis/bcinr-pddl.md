# bcinr-pddl: A Bounded Receipted Planning Execution Engine for Process Intelligence

## Abstract

We present **bcinr-pddl**, a Rust crate implementing the full Bounded Receipted Chatman Equation (BRCE) execution loop for planning problems expressed in PDDL. The crate translates PDDL 3.1 domain and problem descriptions through four sequential layers — parsing, grounding, Prolog8 admission, and BLAKE3 receipting — into cryptographically auditable OCEL event logs. The design enforces decidability at every layer: STRIPS-8 bounds (arity ≤ 8, conjuncts ≤ 8, params ≤ 8, plan depth ≤ 64) ensure the planning space is finite and enumerable. Prolog8's `may_fire(label)` gate enforces that no execution step is taken without standing from a Horn-rule policy. BLAKE3 chaining produces a tamper-evident receipt for the entire execution. The result is a planning engine that is not merely correct but *receipted* — every admitted plan can be replayed, audited, and proved.

---

## 1. Introduction

Planning systems have been studied since STRIPS (1971). The field has produced powerful planners — Fast Downward, Madagascar, Optic — capable of solving large benchmark problems. Yet these systems share a common gap: they produce *plans*, not *proofs*. A plan is a sequence of actions. A proof is a sequence of actions together with evidence that each action was admitted, that the execution occurred, and that the goal was reached — all chained into a record that can be independently verified.

This gap matters for process intelligence. In manufacturing, logistics, healthcare, and software deployment, the audit question is not "did a plan exist?" but "did the right things happen for the right reasons, and can we prove it?" Current planners answer the first question. bcinr-pddl answers the second.

The design follows the Bounded Receipted Chatman Equation (BRCE) — a formal framework in which actuating any action `A` requires standing from a proof system `R`:

```
actuate(A) ⟺ R ⊢ A
```

The BRCE stack for bcinr-pddl is:

```
PDDL8 (domain + problem)
  │
  ▼  parse.rs
Pddl8Domain + Pddl8Problem
  │
  ▼  ground.rs
GroundProblem (all action instances + BFS plan)
  │
  ▼  Pddl8Tape (ordered ops with pred_mask edges)
  │
  ▼  execute.rs
Prolog8 kernel (may_fire/1 gate per op)
  │
  ▼  BLAKE3 chain (per-step receipt hashes)
  │
  ▼  OCEL event log + Pddl8ExecutionReceipt
```

Each layer is a bounded transformation. No layer can produce infinite output. The entire stack terminates.

---

## 2. The PDDL8 Bound

Classical PDDL supports unrestricted arity, arbitrarily deep conjunctions, and universal quantification over object sets of any size. These features make planning PSPACE-complete in general, and the planning space undecidable for some problem families under infinite domains.

PDDL8 is the subset we admit. It is defined by five constants:

| Constant | Value | Meaning |
|---|---|---|
| `PDDL8_MAX_ARITY` | 8 | Maximum predicate arity |
| `PDDL8_MAX_PARAMS` | 8 | Maximum action parameters |
| `PDDL8_MAX_CONJUNCTS` | 8 | Maximum precondition atoms |
| `PDDL8_MAX_PLAN_DEPTH` | 64 | BFS depth ceiling |
| `PDDL8_MAX_GROUND` | 4096 | Maximum ground action instances |

These constants align with Prolog8's own bounds (`ARITY_CAP = 8`, `BODY_CAP = 8`, `VAR_CAP = 8`). The two systems share the same bound family, which is not coincidental: Prolog8 is the semantic gate for PDDL8 execution. A PDDL8 action schema maps directly to a Prolog8 rule without lifting — parameters become variables, preconditions become body atoms, effects become head conclusions.

### 2.1 Why These Bounds?

The bound `n ≤ 8` fits in a single byte. This is the first guarantee: every binding pattern for every predicate can be represented as a `u8` bitmask. With 256 binding patterns available per predicate, the Prolog8 kernel can build full lookup tables (LUTs) over all possible query shapes at load time. The BRCE admission property follows:

> **Lemma 1.** For any PDDL8 action schema with parameters ≤ 8 and preconditions ≤ 8, the corresponding Prolog8 rule has body length ≤ 8 and arity ≤ 8. The kernel can evaluate `may_fire(label)` in O(1) via LUT dispatch.

The bound `depth ≤ 64` bounds the BFS frontier. The total number of states visited is at most `|ground_actions|^64`, but in practice BFS terminates on realistic problems within the reachable state space, which is bounded by `|ground_atoms|^2`. With `PDDL8_MAX_GROUND = 4096` actions and object domains typical in tactical planning problems (≤ 20 objects), the state space is fully enumerable on commodity hardware.

---

## 3. Architecture

### 3.1 parse.rs — PDDL 3.1 Text to Canonical Types

The `pddl` crate (v0.2.0) provides a nom-based PDDL 3.1 parser with a strongly-typed AST. `parse.rs` implements two public functions:

```rust
pub fn domain_from_pddl(text: &str) -> Result<Pddl8Domain, Pddl8Error>
pub fn problem_from_pddl(text: &str) -> Result<Pddl8Problem, Pddl8Error>
```

The lowering traversal handles the full pddl AST hierarchy:

- `Domain::structure().values()` → filter `StructureDef::Action(ActionDefinition)` → `Pddl8ActionSchema`
- `ActionDefinition::precondition()` → `PreconditionGoalDefinitions` → walk `PreconditionGoalDefinition::Preference(PreferenceGoalDefinition::Goal(GoalDefinition::...))` chains
- Effects: `Effects` → `ConditionalEffect::Effect(PrimitiveEffect::AtomicFormula/NotAtomicFormula)` → `add_effects / del_effects`
- `Problem::init()` → `InitElements` → filter `InitElement::Literal(Literal::AtomicFormula)` → `Pddl8Atom`
- `Problem::goals()` → same `PreconditionGoalDefinitions` traversal as preconditions

The output types are defined in `wasm4pm_compat::pddl` — the canonical cross-crate standard. This separation is deliberate: the `pddl` parser crate (nom, bytecount, nom_locate dependencies) only compiles inside bcinr-pddl. All other crates in the ecosystem that need PDDL types import from `wasm4pm_compat::pddl` directly, without pulling in a PDDL parser.

**Bound enforcement at parse time:** arity, parameter count, and conjunct count are checked during lowering. A parse that would produce a schema violating PDDL8 bounds returns `Pddl8Error::BoundExceeded` before any grounding occurs.

### 3.2 ground.rs — Grounding and Forward Search

`GroundProblem::build()` instantiates every action schema over the problem's object set using odometer-style enumeration — a Cartesian product over `|objects|^|params|` assignments. All assignments are tested; no typing filter is applied (the PDDL8 subset targets untyped or loosely-typed problems; typed problems are supported by the pddl parser but PDDL8 grounds without type pruning). The ground action count is bounded by `PDDL8_MAX_GROUND`.

`GroundProblem::find_plan()` runs BFS over the state space:

```
queue: VecDeque<(BTreeSet<Pddl8GroundAtom>, Vec<action_index>)>
visited: HashSet<Vec<Pddl8GroundAtom>>   // keyed on sorted atom list
```

For each state dequeued:
1. Check goal satisfaction — all goal atoms present in state.
2. For each ground action: check all preconditions against state.
3. If applicable: compute successor state (apply add/del effects), check visited, enqueue.

BFS guarantees the shortest plan in terms of step count. Depth is capped at `PDDL8_MAX_PLAN_DEPTH = 64`. If BFS exhausts the reachable space without reaching the goal, `Pddl8Error::NoAdmittedPlan` is returned.

**Output:** `Pddl8Tape` — an ordered list of `Pddl8TapeOp` values, each carrying:
- `index: u8` — position in the plan
- `label: String` — grounded action name, e.g. `"load-truck(pkg1,truck1,loc_a)"`
- `pred_mask: u64` — bit mask over predecessor op indices (currently sequential: op `i` depends on op `i-1`)
- `action: Pddl8GroundAction` — the full ground action with effects

The tape is the POWL geometry of the plan: a partial-order structure over ground actions, expressed as a u64 bitmask per op. This is the same representation used by bcinr-powl for process execution, which means plans produced by bcinr-pddl can be fed directly into bcinr-powl for conformance checking.

### 3.3 execute.rs — The BRCE Loop

`execute_tape()` is the core of the crate. Its signature:

```rust
pub fn execute_tape(
    tape: &Pddl8Tape,
    initial_state: &BTreeSet<Pddl8GroundAtom>,
    goal: &[Pddl8GroundAtom],
    case_id: &str,
    policy_rules: &[(&str, Vec<&str>)],
) -> Result<(Pddl8ExecutionLog, Pddl8ExecutionReceipt, OCEL), Pddl8Error>
```

The function manages a `Ctx` — an in-process Prolog8 kernel with predicate/term interning tables. The loop is:

```
for each op in tape.ops:
  1. query may_fire(label) against Prolog8 kernel
  2. if denied → return Pddl8Error::StepDenied
  3. load add_effects as facts into kernel
  4. tick epoch (del_effects are scoped by epoch in queries)
  5. update BLAKE3 chain: hash(prev_chain ‖ label ‖ op.index ‖ epoch)
  6. push Pddl8StepResult to log
  7. push OCELEvent to ocel_events

after all ops:
  8. check goal satisfaction in current state
  9. if met: load fact goal_reached(__goal__) into kernel, query it
  10. finalize chain: hash(chain ‖ "GOAL_MET"/"GOAL_MISS")
  11. compute plan_root = BLAKE3(op labels)
       state_root = BLAKE3(init atoms)
       goal_root  = BLAKE3(goal atoms)
  12. return (Pddl8ExecutionLog, Pddl8ExecutionReceipt, OCEL)
```

**Admission policy.** When `policy_rules` is empty, every op on the tape is pre-admitted via a `may_fire(label)` fact loaded before execution. This is the "trusted planner" mode — the BFS plan is admitted as given. When `policy_rules` is non-empty, each rule is loaded into the Prolog8 kernel as a Horn clause:

```prolog
may_fire(head_label) :- may_fire(body_label_1), may_fire(body_label_2), ...
```

This enables dynamic admission: an op is admitted only if its label is derivable under the policy. A step whose label is not derivable returns `Pddl8Error::StepDenied` immediately — execution halts at the first denied step. This is the BRCE gate:

```
actuate(op) ⟺ Prolog8 ⊢ may_fire(op.label)
```

**Epoch management.** The Prolog8 kernel uses `EpochId` to scope fact visibility. After each op, the epoch is incremented via `tick_epoch()`. Del-effects are not actually retracted from the kernel (immutable append-only fact store); instead, query epochs ensure that queries reflect the current logical state. This is the same mechanism prolog8 uses for temporal stratification.

**BLAKE3 chain.** Each step contributes to a running chain hash:

```
chain_0 = [0u8; 32]
chain_{i+1} = BLAKE3(chain_i ‖ label_i ‖ index_i ‖ epoch_{i+1})
chain_final = BLAKE3(chain_n ‖ "GOAL_MET" | "GOAL_MISS")
```

This produces a per-execution fingerprint where any change to the plan order, any change to an action label, or any change to the goal outcome produces a different `chain_hash`. The receipt thus certifies:

1. **What was planned** (plan_root)
2. **What initial state was assumed** (state_root)
3. **What goal was targeted** (goal_root)
4. **That the goal was reached** (goal_reached: bool)
5. **The full execution chain** (chain_hash)

### 3.4 OCEL Output

Every admitted step produces an `OCELEvent` with attributes:

| Attribute | Value |
|---|---|
| `activity` | Op label, e.g. `"drive-truck(truck1,loc_a,loc_b)"` |
| `epoch` | Kernel epoch after this step |
| `adds` | Semicolon-delimited add effects |
| `dels` | Semicolon-delimited del effects |
| `receipt` | BLAKE3 chain hash at this step (hex) |

Events are linked to a single `OCELObject` identified by `case_id` — the case object representing this execution instance. The resulting OCEL log is a standard Object-Centric Event Log conforming to `wasm4pm_compat::ocel::OCEL`, directly ingestible by all wasm4pm conformance and discovery algorithms.

This means a PDDL8 execution in bcinr-pddl produces an OCEL trace that wasm4pm can mine. The full loop closes:

```
Plan (bcinr-pddl) → Execute (Prolog8 gate) → Trace (OCEL) → Mine (wasm4pm) → Model → Plan again
```

---

## 4. Decidability Properties

**Claim.** For any PDDL8 domain and problem satisfying the five bound constants, `execute_tape()` terminates.

**Proof sketch.**
- `find_plan()` terminates because the state space is finite (bounded by `|ground_atoms|^{PDDL8_MAX_GROUND}`) and BFS visits each state at most once.
- The tape returned by `find_plan()` has at most `PDDL8_MAX_PLAN_DEPTH = 64` ops.
- `execute_tape()` iterates over the tape once; each iteration performs a Prolog8 query (O(1) LUT dispatch for ground queries with all arguments bound) and a BLAKE3 hash (O(1) for fixed-length inputs).
- Total time complexity: O(|tape| × max_depth × |body|) ≤ O(64 × 32 × 8) = O(16384) — a constant.

The same decidability argument applies to the full BRCE stack: because the admission gate is Prolog8 with Horn rules over bounded atoms, the `R ⊢ A` check is decidable by the Prolog8 proof engine's depth-bounded SLD resolution (MAX_DERIVE_DEPTH = 32, as established in the prolog8 engine specification).

This is the BRCE escape from Rice's theorem: we do not ask whether an arbitrary program terminates. We ask whether a ground atom `may_fire(label)` is derivable in a Horn database with arity ≤ 8, body ≤ 8, depth ≤ 32. That question is always decidable.

---

## 5. Relationship to the BRCE Stack

bcinr-pddl occupies a specific position in the BRCE stack:

```
┌─────────────────────────────────────────────────────────────────┐
│  G_F^B  (bounded future graph construction)                     │
│  bcinr-pddl: PDDL8 domain + BFS = candidate future grammar     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  μ_B  (bounded admission morphism)                              │
│  Prolog8: may_fire/1 Horn gate                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  ρ  (receipting function)                                       │
│  BLAKE3 chain → Pddl8ExecutionReceipt                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  O*  (observed trace)                                           │
│  OCEL event log, ingestible by wasm4pm                          │
└─────────────────────────────────────────────────────────────────┘
```

**PDDL8 as G_F^B.** In BRCE, `G_F^B` is the bounded future graph — the set of candidate actions the system *could* take, constrained by the domain model. PDDL8 fills this role exactly: the domain defines what actions exist and their preconditions/effects; the problem defines the initial state; BFS finds the shortest path from present to goal. The tape is `G_F^B` instantiated into a concrete candidate sequence.

**Prolog8 as μ_B.** The admission morphism `μ_B` decides which elements of `G_F^B` are admitted for execution. In bcinr-pddl, this is the `may_fire(label)` gate. In the default (no policy rules) mode, `μ_B` = identity: all planned ops are admitted. With policy rules, `μ_B` filters: only ops derivable under the Horn policy may fire. This is the mechanism by which real-world constraints (safety policies, resource limits, authorization rules) can be encoded into the execution gate as Prolog8 rules, without modifying the PDDL domain.

**BLAKE3 as ρ.** The receipting function `ρ` maps `(O*, μ_B, A)` to a receipt `R`. In bcinr-pddl, `ρ` is the BLAKE3 chain function: each admitted step extends the chain, and the final receipt certifies the complete execution. The receipt is not a log — it is a commitment. Given the receipt and the plan, the execution can be replayed and the receipt recomputed; any divergence is detectable.

---

## 6. Integration Surfaces

### 6.1 wasm4pm-cognition breeds

The `schema_from_rule()` bridge in `wasm4pm_compat::pddl` translates a wasm4pm-cognition `Rule` (with `premise: Vec<String>` in `predicate=value` encoding and `conclusion: String` in `!del;add` encoding) into a `Pddl8ActionSchema`. This enables cognition breeds that define STRIPS-like rule chains to have their rules exported as PDDL8 schemas, grounded, planned over, and executed through the BRCE gate.

The implication: any cognition breed that can be expressed as Horn rules over state predicates gains a planning surface for free. The breed defines the domain; bcinr-pddl plans and receipts.

### 6.2 bcinr-powl

The `Pddl8Tape`'s `pred_mask: u64` per op is structurally identical to bcinr-powl's `AdmissionContext` bitmask — a u64 over predecessor ops. A tape from bcinr-pddl can be fed directly into bcinr-powl's LUT-based admission engine for process-order conformance checking, at O(1) per step.

The tape is therefore simultaneously:
- A PDDL plan (ordered action sequence)
- A POWL partial-order trace (bitmask-encoded precedence)
- A Prolog8 query sequence (may_fire/1 per label)
- A BLAKE3 receipt chain input
- An OCEL event sequence

One data structure serving four protocol layers is the architectural signature of BRCE design.

### 6.3 MCP Agents / Claude Code Ecosystem

bcinr-pddl can be used as a planning backend for MCP-connected AI agents operating in Claude Code:

1. **Agent generates PDDL:** an LLM agent drafts a PDDL domain and problem describing a software task (deploy service, run migration, rotate credentials).
2. **bcinr-pddl plans:** `domain_from_pddl` + `problem_from_pddl` + `GroundProblem::build` + `find_plan` produces a tape.
3. **Prolog8 gates execution:** a policy loaded from the agent's authorization rules determines which steps may fire.
4. **Receipt certifies the run:** the agent returns the `Pddl8ExecutionReceipt` as evidence that the task completed under the policy.
5. **OCEL feeds discovery:** the OCEL log is ingested by wasm4pm to mine the actual process model from the execution trace.

This gives Claude Code agents a bounded, receipted, auditable planning loop. The agent cannot act without standing (`R ⊢ A`). The receipt proves it acted. The OCEL proves what it did. The wasm4pm model proves whether the process is conformant.

---

## 7. Falsification and Test Design

The test suite (`tests/logistics.rs`, `tests/brce_loop.rs`) is designed around falsifiability — each test has a specific counterfactual that would cause it to fail.

### 7.1 Logistics domain

The logistics domain (load package onto truck, drive, unload) tests:

- **`logistics_plan_found_and_executed`**: goal `at(pkg1, loc_b)` is reached; receipt fields are non-empty; chain hash matches between log and receipt; OCEL events are present with correct structure.
  - *Falsification*: removing the drive-truck action would make the goal unreachable; `execute_tape` would return `NoAdmittedPlan` or `GoalNotReached`.

- **`logistics_step_receipt_chain_is_deterministic`**: running the same plan twice produces identical `plan_root`, `state_root`, `chain_hash`.
  - *Falsification*: if any non-deterministic input (timestamp, random) entered the hash function, this test would flake. The hash inputs are `(label, index, epoch)` — all deterministic.

- **`logistics_plan_fails_without_truck_at_pickup`**: with the truck at the wrong location, the planner must either find a longer route or fail. It does not panic.
  - *Falsification*: a crash here would indicate the BFS does not handle impossible initial states safely.

### 7.2 Blocksworld BRCE loop

The blocksworld domain (pick up block A, stack on block B) tests:

- **`blocksworld_brce_full_loop`**: full end-to-end from PDDL text to receipt. Goal `on(a,b)` is reached; OCEL events equal tape length; all events link to the case object.
  - *Falsification*: if Prolog8 rejected any step, `StepDenied` would be returned and `goal_reached` would be false.

- **`empty_tape_goal_not_reached`**: an empty tape produces `goal_reached = false` for a non-trivially-satisfied goal.
  - *Falsification*: if the goal-gate logic incorrectly declared the goal met without any steps, this test would catch it.

- **`receipt_differs_by_case_id`** / **`receipt_chain_is_deterministic`**: structural properties of the BLAKE3 chain — same plan produces same chain; different case IDs are isolated (case_id enters OCEL, not the chain).
  - *Falsification*: leaking case_id into the chain would break the second assertion; using system time would break determinism.

---

## 8. Dependency Isolation

The dep graph for bcinr-pddl is intentionally narrow:

```
bcinr-pddl
├── pddl 0.2.0          ← parser (nom, bytecount, nom_locate)
├── blake3 1.x          ← receipting
├── prolog8 (path)      ← admission gate
├── wasm4pm-compat 26.6.28  ← canonical types (OCEL + PDDL8)
├── serde + serde_json  ← serialization
└── chrono              ← OCEL timestamps
```

The `pddl` parser crate is the only heavyweight dep that does not belong in the broader wasm4pm ecosystem. By isolating it here, we ensure:

- `wasm4pm` (Rust/WASM core) does not compile a PDDL parser.
- `bcinr-powl` does not compile a PDDL parser.
- `packages/` (TypeScript monorepo) does not see any PDDL parser artifacts.
- `wasm4pm-cognition` WASM build does not include PDDL parsing.

Canonical types (`Pddl8Domain`, `Pddl8GroundAtom`, etc.) live in `wasm4pm-compat`, which has no parser dep. Any crate that needs to *represent* PDDL8 structures imports from wasm4pm-compat. Only bcinr-pddl needs to *parse* PDDL text.

This is the dep isolation principle: **parser where parsing happens, types everywhere types are needed.**

---

## 9. Implementation Notes

### 9.1 Grounding without typing

PDDL typing (`:typing` requirement) is parsed correctly by the `pddl` crate, but bcinr-pddl grounds without type-pruning: all objects are substituted into all parameter positions. This produces more ground actions than a typed planner would generate, but ensures correctness for untyped problems and simplicity of implementation. For typed problems, the excess ground actions are typically unreachable in BFS (preconditions fail on mistyped objects) so the plan found is still valid.

Future work: type-aware grounding using the domain's type hierarchy to prune the Cartesian product.

### 9.2 Epoch-based deletion

The Prolog8 kernel is an append-only fact store. Del-effects do not retract facts; instead, the epoch counter advances after each step, and queries are evaluated under the current epoch. This means facts added in epoch `e` are visible in all queries with `epoch >= e`. Del-effects are currently handled at the state-tracking level in execute.rs (the Rust `BTreeSet<Pddl8GroundAtom>` is updated) but are not enforced in the Prolog8 kernel itself.

This is sufficient for the `may_fire` gate (which reasons over action labels, not world state) but would need extension for a policy that reasons over world-state predicates (e.g., "may_fire only if `at(truck1, loc_a)` holds"). Full world-state reasoning through Prolog8 is left as future work.

### 9.3 Sequential pred_mask

`Pddl8Tape::from_plan()` assigns `pred_mask = (1 << (index-1))` for index > 0, encoding a total order. This is conservative — many real plans have independent steps that could be parallelized. A future extension: static dependency analysis over effect/precondition intersections to build a true partial order, where two ops with no data dependency (no add-effect of op A is a precondition of op B) receive independent pred_masks.

---

## 10. Conclusion

bcinr-pddl is a planning-to-receipting pipeline that takes PDDL 3.1 text and produces cryptographically auditable execution evidence. Its design enforces three properties that classical planners do not:

1. **Bounded**: every computation terminates by construction (PDDL8 constants, BFS depth cap, Prolog8 horn-rule bounds).
2. **Admitted**: no execution step fires without standing from a Prolog8 Horn-rule policy (the BRCE gate).
3. **Receipted**: every execution produces a BLAKE3 chain receipt and OCEL log that together certify what happened, in what order, under what policy, and whether the goal was reached.

These properties make bcinr-pddl appropriate for process intelligence use cases where the question is not "can a plan exist?" but "did the right things happen, in the right order, for the right reasons, and can we prove it?"

The answer is now: yes.

---

## Appendix A: Bound Constants

```rust
// wasm4pm_compat::pddl
pub const PDDL8_MAX_ARITY: usize = 8;
pub const PDDL8_MAX_CONJUNCTS: usize = 8;
pub const PDDL8_MAX_PARAMS: usize = 8;
pub const PDDL8_MAX_PLAN_DEPTH: usize = 64;
pub const PDDL8_MAX_GROUND: usize = 4096;
```

## Appendix B: Error Taxonomy

```rust
pub enum Pddl8Error {
    ParseError(String),           // pddl crate failed to parse text
    BoundExceeded { what, limit, got }, // PDDL8 constant violated
    UnknownPredicate(String),     // referenced but not declared
    EmptyGrounding,               // no ground actions produced
    NoAdmittedPlan,               // BFS exhausted without goal
    AdmissionLoadError(String),   // Prolog8 rejected a rule load
    StepDenied { op_index, reason }, // may_fire query returned false
    GoalNotReached,               // all steps admitted but goal not met
    ReceiptIntegrity(String),     // hash chain validation failed
}
```

## Appendix C: BRCE Formal Correspondence

| BRCE term | bcinr-pddl implementation |
|---|---|
| `O` (observations) | `Pddl8Problem::init` (initial world state) |
| `O*` (enriched observations) | OCEL event log after execution |
| `G_F^B` (bounded future graph) | `Pddl8Tape` from BFS on `GroundProblem` |
| `μ_B` (admission morphism) | Prolog8 `may_fire/1` gate in `execute_tape` |
| `A` (admitted actions) | Steps with `Pddl8StepResult::admitted = true` |
| `ρ` (receipting function) | BLAKE3 chain in `execute_tape` |
| `R` (receipt) | `Pddl8ExecutionReceipt` |
| `actuate(A) ⟺ R ⊢ A` | Step fires iff Prolog8 derives `may_fire(label)` |
