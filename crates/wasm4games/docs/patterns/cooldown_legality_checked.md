<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (cooldown_legality_checked). Re-scaffold: `ggen sync`. -->

# Pattern: CooldownLegalityChecked

> **Family:** Anti-Cheat · **Kernel:** `cooldown_legality_checked` · **Lowering:** `Bitset` · **Id:** 73

Verdict bit0 mirrors the i16 sign bit of the remaining cooldown; a negative cooldown (fast-fire cheat) is refused.

---

## Context

<!-- TODO: Describe the game situation that makes CooldownLegalityChecked necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: legality spec: remaining_cooldown >= 0 (i16 sign bit clear) -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every CooldownLegalityChecked call that branches adds jitter
     - Deterministic latency: the Bitset lowering gives O(1) constant time
     - Bounded state: stateCard = 2 (2 distinct states)
     - Auditability: the OCEL event code 135 ties the transition to an object trace
     Authority to defend: legality spec: remaining_cooldown >= 0 (i16 sign bit clear) -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how cooldown_legality_checked resolves the forces.
     It lowers onto `bcinr_logic::bitset::rank_u64` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Bitset` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::bitset::rank_u64`

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
    kernel["cooldown_legality_checked\nBitset: bcinr_logic::bitset::rank_u64"]
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
| OCEL activity | `CooldownLegalityChecked` |
| Event code | `135` |
| OTEL span | `135` |
| Object kinds | `player` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | legality spec: remaining_cooldown >= 0 (i16 sign bit clear) |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 73 |
| Family | Anti-Cheat |
| Lowering | `Bitset` |
| State cardinality | 2 |
| Primitive | `bcinr_logic::bitset::rank_u64` |
| Kernel signature | `cooldown_legality_checked(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/cooldown_legality_checked.rs` |

---

## How to Use

```rust
use wasm4games::patterns::cooldown_legality_checked;

// Pack state and input into u64 fields as documented in the kernel source.
let result = cooldown_legality_checked(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = cooldown_legality_checked(state, input);
otel::emit(135);
let ev = OcelEvent::new(135, logical_tick, admission_status);
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
