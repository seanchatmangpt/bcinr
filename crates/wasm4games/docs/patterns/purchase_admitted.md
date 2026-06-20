<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (purchase_admitted). Re-scaffold: `ggen sync`. -->

# Pattern: PurchaseAdmitted

> **Family:** Economy / Progression · **Kernel:** `purchase_admitted` · **Lowering:** `Dfa` · **Id:** 34

Advance a purchase flow FSM one transition over a flat 5x5 transition table.

---

## Context

<!-- TODO: Describe the game situation that makes PurchaseAdmitted necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: oracle predicate: matches purchase_admitted_reference for all inputs -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every PurchaseAdmitted call that branches adds jitter
     - Deterministic latency: the Dfa lowering gives O(1) constant time
     - Bounded state: stateCard = 4 (4 distinct states)
     - Auditability: the OCEL event code 96 ties the transition to an object trace
     Authority to defend: oracle predicate: matches purchase_admitted_reference for all inputs -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how purchase_admitted resolves the forces.
     It lowers onto `bcinr_logic::dfa::dfa_advance` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Dfa` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::dfa::dfa_advance`

_Replace this placeholder with the solution description._

## Consequences

<!-- TODO: What trade-offs follow from applying this pattern?
     Gains: predictable latency, side-channel resistance, OCEL audit trail.
     Costs: fixed bit-field ABI, state space bounded to 4 classes.
     What patterns naturally compose with this one (see Related Patterns below)? -->

_Replace this placeholder with consequences and trade-offs._

---

## Structure Diagram

```mermaid
---
title: PurchaseAdmitted — DFA (4 states)
---
stateDiagram-v2
    [*] --> S0
    S0: State_0
    S1: State_1
    S2: State_2
    S3: State_3
    S0 --> S0 : TODO_symbol
    S1 --> S1 : TODO_symbol
    S2 --> S2 : TODO_symbol
    S3 --> S3 : TODO_symbol
```

<!-- TODO: Replace State_N labels and TODO_symbol edges with the actual state names
     and alphabet symbols from src/patterns/purchase_admitted.rs (see the DFA table
     comment and the _reference oracle for the canonical state/symbol vocabulary). -->

---

## Evidence Model

| Field | Value |
|---|---|
| OCEL activity | `PurchaseAdmitted` |
| Event code | `96` |
| OTEL span | `96` |
| Object kinds | `player`, `item` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches purchase_admitted_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 34 |
| Family | Economy / Progression |
| Lowering | `Dfa` |
| State cardinality | 4 |
| Primitive | `bcinr_logic::dfa::dfa_advance` |
| Kernel signature | `purchase_admitted(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/purchase_admitted.rs` |

---

## How to Use

```rust
use wasm4games::patterns::purchase_admitted;

// Pack state and input into u64 fields as documented in the kernel source.
let result = purchase_admitted(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = purchase_admitted(state, input);
otel::emit(96);
let ev = OcelEvent::new(96, logical_tick, admission_status);
```

<!-- TODO: Add a concrete game-loop example showing this kernel in context:
     how is the state packed? what does the caller do with the result?
     what does the OCEL event represent in the game world? -->

---

## Related Patterns

<!-- TODO: Add links to related patterns in this directory. Examples:
     - [PatternName](pattern_name.md) — brief relationship note
     Suggestions: look for patterns in the same family, same lowering, or that
     compose naturally (one pattern's output feeds another's input). -->

_No related patterns linked yet — fill in and remove this placeholder._
