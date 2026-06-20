<!-- SCAFFOLDED BY ggen — fill in Context/Forces/Solution/Consequences below, then commit. -->
<!-- Source: ggen/schema/patterns.ttl (receipt_appended). Re-scaffold: `ggen sync`. -->

# Pattern: ReceiptAppended

> **Family:** Evidence & Replay · **Kernel:** `receipt_appended` · **Lowering:** `Receipt` · **Id:** 9

Append one transition to a rolling receipt chain via FNV-1a folding.

---

## Context

<!-- TODO: Describe the game situation that makes ReceiptAppended necessary.
     Why does this pattern exist? What breaks — or becomes unpredictably slow —
     without it? Seed from the authority line below, then expand:
     Authority: oracle predicate: matches receipt_appended_reference for all inputs -->

_Replace this placeholder with 2–3 sentences on the game problem this pattern addresses._

## Forces

<!-- TODO: List the tensions this pattern must hold in balance. Consider:
     - Branch misprediction: every ReceiptAppended call that branches adds jitter
     - Deterministic latency: the Receipt lowering gives O(1) constant time
     - Bounded state: stateCard = 64 (64 distinct states)
     - Auditability: the OCEL event code 32 ties the transition to an object trace
     Authority to defend: oracle predicate: matches receipt_appended_reference for all inputs -->

_Replace this placeholder with the forces — what pulls in opposite directions here._

## Solution

<!-- TODO: Explain how receipt_appended resolves the forces.
     It lowers onto `bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt` to compute without conditional branches.
     Describe the bit-field ABI (how state and input are packed into u64),
     the branchless arithmetic, and why `Receipt` was the right lowering choice. -->

**Branchless primitive:** `bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt`

_Replace this placeholder with the solution description._

## Consequences

<!-- TODO: What trade-offs follow from applying this pattern?
     Gains: predictable latency, side-channel resistance, OCEL audit trail.
     Costs: fixed bit-field ABI, state space bounded to 64 classes.
     What patterns naturally compose with this one (see Related Patterns below)? -->

_Replace this placeholder with consequences and trade-offs._

---

## Structure Diagram

```mermaid
graph LR
    state["state\n(u64)"]
    input["input\n(u64)"]
    kernel["receipt_appended\nReceipt: bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt"]
    result["result\n(u64)"]
    state --> kernel
    input --> kernel
    kernel --> result
    ocel_0["OCEL: receipt_chain"]
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
| OCEL activity | `ReceiptAppended` |
| Event code | `32` |
| OTEL span | `3` |
| Object kinds | `receipt_chain` |
| Required status | `4` |
| Refusal status | `7` |
| Authority | oracle predicate: matches receipt_appended_reference for all inputs |

---

## Metadata

| Field | Value |
|---|---|
| Pattern id | 9 |
| Family | Evidence & Replay |
| Lowering | `Receipt` |
| State cardinality | 64 |
| Primitive | `bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt` |
| Kernel signature | `receipt_appended(state: u64, input: u64) -> u64` |
| Source kernel | `src/patterns/receipt_appended.rs` |

---

## How to Use

```rust
use wasm4games::patterns::receipt_appended;

// Pack state and input into u64 fields as documented in the kernel source.
let result = receipt_appended(state, input);
```

The kernel is a pure `fn(u64, u64) -> u64` — no side effects, no allocation, no branches.
Pair with the evidence layer to emit OCEL events, OTEL spans, and receipt folds:

```rust
use wasm4games::evidence::{ocel::OcelEvent, otel};

let result = receipt_appended(state, input);
otel::emit(3);
let ev = OcelEvent::new(32, logical_tick, admission_status);
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
