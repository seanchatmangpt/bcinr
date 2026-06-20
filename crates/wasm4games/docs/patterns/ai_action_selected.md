<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (ai_action_selected). Re-scaffold: `ggen sync`. -->

# Pattern: AiActionSelected

> **Family:** AI Agent / Benchmark · **Kernel:** `ai_action_selected` · **Lowering:** `Mask` · **Id:** 13

Select an AI action by utility argmax over four scores (first index wins ties).

---

## Context

<!-- TODO: Describe the game situation that makes AiActionSelected necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: oracle predicate: matches ai_action_selected_reference for all inputs -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every AiActionSelected call that branches adds jitter
     - Deterministic latency: the Mask lowering gives O(1) constant time
     - Bounded state: stateCard = 4 (4 distinct states)
     - Auditability: the OCEL event code 64 ties the transition to an object trace
     Authority to defend: oracle predicate: matches ai_action_selected_reference for all inputs -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how ai_action_selected resolves the forces.
     It lowers onto `bcinr_logic::mask::select_u32` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Mask` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::mask::select_u32`

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
graph LR
    state["state\n(u64)"]
    input["input\n(u64)"]
    kernel["ai_action_selected\nMask: bcinr_logic::mask::select_u32"]
    result["result\n(u64)"]
    state --> kernel
    input --> kernel
    kernel --> result
    ocel_0["OCEL: agent"]
    result --> ocel_0
```

<!-- TODO: Improve this structural data-flow diagram:
     - Annotate bit-field layout on the state/input/result nodes
     - Label the arithmetic operation on the kernel node
     - Add state machine nodes if this pattern has meaningful internal states
     - Or replace with a more specific diagram tailored to this pattern -->

---

## Evidence Model

| Field | Value |
|---|---|
| OCEL activity | `AiActionSelected` |
| Event code | `64` |
| OTEL span | `64` |
| Object kinds | `agent` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches ai_action_selected_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 13 |
| Family | AI Agent / Benchmark |
| Lowering | `Mask` |
| State cardinality | 4 |
| Primitive | `bcinr_logic::mask::select_u32` |
| Kernel signature | `ai_action_selected(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/ai_action_selected.rs` |

---

## How to Use

```rust
use wasm4games::patterns::ai_action_selected;

// Pack state and input into u64 fields as documented in the kernel source.
let result = ai_action_selected(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = ai_action_selected(state, input);
otel::emit(64);
let ev = OcelEvent::new(64, logical_tick, admission_status);
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
