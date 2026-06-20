<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (fov_adjusted). Re-scaffold: `ggen sync`. -->

# Pattern: FovAdjusted

> **Family:** Camera · **Kernel:** `fov_adjusted` · **Lowering:** `Saturating` · **Id:** 43

Adjust field-of-view by a signed delta, clamping to [min_fov, max_fov].

---

## Context

<!-- TODO: Describe the game situation that makes FovAdjusted necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: oracle predicate: matches fov_adjusted_reference for all inputs -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every FovAdjusted call that branches adds jitter
     - Deterministic latency: the Saturating lowering gives O(1) constant time
     - Bounded state: stateCard = 32 (32 distinct states)
     - Auditability: the OCEL event code 105 ties the transition to an object trace
     Authority to defend: oracle predicate: matches fov_adjusted_reference for all inputs -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how fov_adjusted resolves the forces.
     It lowers onto `bcinr_logic::int::saturating_add_i64` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Saturating` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::int::saturating_add_i64`

_Replace this placeholder with the solution description._

## Consequences

<!-- TODO: What trade-offs follow from applying this pattern?
     Gains: predictable latency, side-channel resistance, OCEL audit trail.
     Costs: fixed bit-field ABI, state space bounded to 32 classes.
     What patterns naturally compose with this one (see Related Patterns below)? -->

_Replace this placeholder with consequences and trade-offs._

---

## Structure Diagram

```mermaid
graph LR
    state["state\n(u64)"]
    input["input\n(u64)"]
    kernel["fov_adjusted\nSaturating: bcinr_logic::int::saturating_add_i64"]
    result["result\n(u64)"]
    state --> kernel
    input --> kernel
    kernel --> result
    ocel_0["OCEL: camera"]
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
| OCEL activity | `FovAdjusted` |
| Event code | `105` |
| OTEL span | `105` |
| Object kinds | `camera` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches fov_adjusted_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 43 |
| Family | Camera |
| Lowering | `Saturating` |
| State cardinality | 32 |
| Primitive | `bcinr_logic::int::saturating_add_i64` |
| Kernel signature | `fov_adjusted(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/fov_adjusted.rs` |

---

## How to Use

```rust
use wasm4games::patterns::fov_adjusted;

// Pack state and input into u64 fields as documented in the kernel source.
let result = fov_adjusted(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = fov_adjusted(state, input);
otel::emit(105);
let ev = OcelEvent::new(105, logical_tick, admission_status);
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
