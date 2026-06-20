<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (quest_step_advanced). Re-scaffold: `ggen sync`. -->

# Pattern: QuestStepAdvanced

> **Family:** Narrative / Dialogue · **Kernel:** `quest_step_advanced` · **Lowering:** `Dfa` · **Id:** 17

Advance a linear quest one step; complete moves forward, abandon resets.

---

## Context

Linear quest progression in RPGs and adventure games follows a well-defined sequence of steps: accept, complete step 1, complete step 2, complete step 3, reach Done. Without a DFA, quest state is managed with a nested `match` or `switch` on the current step and event, adding one branch per legal transition per quest event. Over many simultaneous quests and replay-log entries, the branch overhead accumulates, and the absence of an explicit Done absorbing state allows bugs where a completed quest can be re-triggered. Replay log analysis also becomes difficult when state transitions are scattered across multiple match arms rather than captured in a single auditable table.

## Forces

- **Branch misprediction** — a nested `match (step, event)` over 15 combinations adds mispredictable branches per quest event, compounding when many quests are active simultaneously.
- **Deterministic latency** — the Dfa lowering resolves the transition in O(1) via `TABLE[step * 3 + symbol]`, with no branch.
- **Absorbing Done state** — once a quest reaches Done (step 4), it must be structurally impossible to advance further; the Done row absorbs all symbols (`idle`, `complete`, `abandon` all loop to Done).
- **Reset on abandon** — the `abandon` symbol must reset any in-progress step back to step 0 (not to Done), modeling quest cancellation cleanly.
- **Out-of-alphabet safety** — invalid symbols from a buggy caller must not index outside the table; OOB symbols are masked to `idle` (hold) branchlessly.
- **OCEL auditability** — OCEL event code 69 ties every quest step transition to an auditable `player`/`quest` object trace, enabling quest replay forensics.

## Solution

The kernel packs state as bits[0..8] = current step (0 = step0, 1 = step1, 2 = step2, 3 = step3, 4 = Done) and input as bits[0..8] = quest symbol (0 = idle, 1 = complete, 2 = abandon). The 5×3 transition table (15 entries) is flat; `complete` advances each in-progress step forward, `abandon` resets to step0, `idle` holds. The Done row maps all symbols to Done (absorbing). OOB symbols are masked to `idle` (0) via `sym_raw & 0.wrapping_sub(in_alphabet)`. `dfa_advance` performs the table lookup. The `Dfa` lowering is correct because quest progression is inherently a finite state machine where all legal transitions can be enumerated explicitly.

## Consequences

**Gains:** the Done absorbing state prevents re-triggering a completed quest; abandon resets correctly in all in-progress states; OOB symbols hold the current step; OCEL event 69 provides a per-quest-step audit. **Costs:** the alphabet is fixed at 3 symbols; adding a `pause` or `timeout` event requires widening the table. **Compositions:** quest completion in the Done state triggers [DialogueNodeAdvanced](dialogue_node_advanced.md) to unlock NPC dialogue; the DFA pattern is the same lowering as [EntityStateTransitioned](entity_state_transitioned.md); quest step events fold into [ReceiptAppended](receipt_appended.md) for receipt chain integrity.

---

## Structure Diagram

```mermaid
---
title: QuestStepAdvanced — DFA (5 states, alphabet: idle/complete/abandon)
---
stateDiagram-v2
    [*] --> step0
    step0: step0 (0)
    step1: step1 (1)
    step2: step2 (2)
    step3: step3 (3)
    Done: Done (4) [absorbing]

    step0 --> step0 : idle / abandon
    step0 --> step1 : complete

    step1 --> step1 : idle
    step1 --> step0 : abandon
    step1 --> step2 : complete

    step2 --> step2 : idle
    step2 --> step0 : abandon
    step2 --> step3 : complete

    step3 --> step3 : idle
    step3 --> step0 : abandon
    step3 --> Done : complete

    Done --> Done : idle / complete / abandon
```

---

## Evidence Model

| Field | Value |
|---|---|
| OCEL activity | `QuestStepAdvanced` |
| Event code | `69` |
| OTEL span | `69` |
| Object kinds | `player`, `quest` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches quest_step_advanced_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 17 |
| Family | Narrative / Dialogue |
| Lowering | `Dfa` |
| State cardinality | 5 |
| Primitive | `bcinr_logic::dfa::dfa_advance` |
| Kernel signature | `quest_step_advanced(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/quest_step_advanced.rs` |

---

## How to Use

```rust
use wasm4games::patterns::quest_step_advanced;

// Pack state and input into u64 fields as documented in the kernel source.
let result = quest_step_advanced(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = quest_step_advanced(state, input);
otel::emit(69);
let ev = OcelEvent::new(69, logical_tick, admission_status);
```

---

## Related Patterns

- [DialogueNodeAdvanced](dialogue_node_advanced.md) — quest completion in the Done state triggers NPC dialogue advancement.
- [EntityStateTransitioned](entity_state_transitioned.md) — uses the same DFA lowering idiom for general entity FSMs.
- [ReceiptAppended](receipt_appended.md) — quest step events fold into the receipt chain for audit integrity.
