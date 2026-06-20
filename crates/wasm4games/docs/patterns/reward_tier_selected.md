<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (reward_tier_selected). Re-scaffold: `ggen sync`. -->

# Pattern: RewardTierSelected

> **Family:** Economy / Progression · **Kernel:** `reward_tier_selected` · **Lowering:** `Bitset` · **Id:** 35

Select a reward tier (0..3) by counting prestige flags set in a bitset, then bucketing the count against three thresholds.

---

## Context

<!-- TODO: Describe the game situation that makes RewardTierSelected necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: oracle predicate: matches reward_tier_selected_reference for all inputs -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every RewardTierSelected call that branches adds jitter
     - Deterministic latency: the Bitset lowering gives O(1) constant time
     - Bounded state: stateCard = 16 (16 distinct states)
     - Auditability: the OCEL event code 97 ties the transition to an object trace
     Authority to defend: oracle predicate: matches reward_tier_selected_reference for all inputs -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how reward_tier_selected resolves the forces.
     It lowers onto `bcinr_logic::int::popcount_u64` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Bitset` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::int::popcount_u64`

_Replace this placeholder with the solution description._

## Consequences

<!-- TODO: What trade-offs follow from applying this pattern?
     Gains: predictable latency, side-channel resistance, OCEL audit trail.
     Costs: fixed bit-field ABI, state space bounded to 16 classes.
     What patterns naturally compose with this one (see Related Patterns below)? -->

_Replace this placeholder with consequences and trade-offs._

---

## Structure Diagram

```mermaid
graph LR
    state["state\n(u64)"]
    input["input\n(u64)"]
    kernel["reward_tier_selected\nBitset: bcinr_logic::int::popcount_u64"]
    result["result\n(u64)"]
    state --> kernel
    input --> kernel
    kernel --> result
    ocel_0["OCEL: player"]
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
| OCEL activity | `RewardTierSelected` |
| Event code | `97` |
| OTEL span | `97` |
| Object kinds | `player` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches reward_tier_selected_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 35 |
| Family | Economy / Progression |
| Lowering | `Bitset` |
| State cardinality | 16 |
| Primitive | `bcinr_logic::int::popcount_u64` |
| Kernel signature | `reward_tier_selected(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/reward_tier_selected.rs` |

---

## How to Use

```rust
use wasm4games::patterns::reward_tier_selected;

// Pack state and input into u64 fields as documented in the kernel source.
let result = reward_tier_selected(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = reward_tier_selected(state, input);
otel::emit(97);
let ev = OcelEvent::new(97, logical_tick, admission_status);
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
