<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (aabb_collision_resolved). Re-scaffold: `ggen sync`. -->

# Pattern: AabbCollisionResolved

> **Family:** Core Sim & Combat · **Kernel:** `aabb_collision_resolved` · **Lowering:** `Mask` · **Id:** 5

Resolve AABB overlap via the separating-axis test, ANDing four lt-masks.

---

## Context

<!-- TODO: Describe the game situation that makes AabbCollisionResolved necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: oracle predicate: matches aabb_collision_resolved_reference for all inputs -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every AabbCollisionResolved call that branches adds jitter
     - Deterministic latency: the Mask lowering gives O(1) constant time
     - Bounded state: stateCard = 2 (2 distinct states)
     - Auditability: the OCEL event code 17 ties the transition to an object trace
     Authority to defend: oracle predicate: matches aabb_collision_resolved_reference for all inputs -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how aabb_collision_resolved resolves the forces.
     It lowers onto `bcinr_logic::mask::lt_mask_u32` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Mask` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::mask::lt_mask_u32`

_Replace this placeholder with the solution description._

## Consequences

<!-- TODO: What trade-offs follow from applying this pattern?
     Gains: predictable latency, side-channel resistance, OCEL audit trail.
     Costs: fixed bit-field ABI, state space bounded to 2 classes.
     What patterns naturally compose with this one (see Related Patterns below)? -->

_Replace this placeholder with consequences and trade-offs._

---

## Structure Diagram

```mermaid
graph LR
    state["state\n(u64)"]
    input["input\n(u64)"]
    kernel["aabb_collision_resolved\nMask: bcinr_logic::mask::lt_mask_u32"]
    result["result\n(u64)"]
    state --> kernel
    input --> kernel
    kernel --> result
    ocel_0["OCEL: entity_a"]
    result --> ocel_0
    ocel_1["OCEL: entity_b"]
    result --> ocel_1
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
| OCEL activity | `AabbCollisionResolved` |
| Event code | `17` |
| OTEL span | `17` |
| Object kinds | `entity_a`, `entity_b` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches aabb_collision_resolved_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 5 |
| Family | Core Sim & Combat |
| Lowering | `Mask` |
| State cardinality | 2 |
| Primitive | `bcinr_logic::mask::lt_mask_u32` |
| Kernel signature | `aabb_collision_resolved(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/aabb_collision_resolved.rs` |

---

## How to Use

```rust
use wasm4games::patterns::aabb_collision_resolved;

// Pack state and input into u64 fields as documented in the kernel source.
let result = aabb_collision_resolved(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = aabb_collision_resolved(state, input);
otel::emit(17);
let ev = OcelEvent::new(17, logical_tick, admission_status);
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
