# bcinr-powl

## What bcinr-powl Is

bcinr-powl is a proof-carrying POWL (Partially Ordered Workflow Language) execution engine built on the branchless calculus primitives of the bcinr library. It executes workflow graphs defined as POWL AST nodes, co-produces audit receipts alongside every execution, and exports event logs in OCEL 2.0 format for process-mining conformance validation — all in a `no_std`-compatible, panic-free runtime with deterministic latency.

## What It Is Not

bcinr-powl is currently a proof-carrying POWL research artifact. It is not yet a full Camunda/Temporal replacement. It is not yet independently SOTA-crowned. It is designed to make workflow execution, conformance evidence, and audit receipts co-produced by the runtime.

## How to Install

```toml
[dependencies]
bcinr-powl = "26.6.25"
```

## How to Run Tests

```bash
cargo test -p bcinr-powl --lib
cargo test -p bcinr-powl --lib --features std
```

## How Receipts Work

Receipts carry `op_trace` (bitmask of operations executed), `topo_order` (firing order of nodes), `event_count`, `chain_hash` (BLAKE3 hash of the execution chain), and an `overflow` flag indicating dropped events. Call `verify_topo_order()` on a receipt to check that all predecessor constraints were satisfied in the recorded firing order.

## How OCEL Export Works

Call `log.to_ocel_2_0()` to obtain an OCEL 2.0 struct or `log.to_ocel_json()` for a JSON string. Both require `--features std`. Export is off the hot path and does not affect execution latency.

## How Conformance Validation Works

Call `validate_against_tape(log, tape)` with an event log and a reference execution tape. Returns `ConformanceResult::Conforms` if the log satisfies all declared ordering and participation constraints, or `ConformanceResult::Violation` with a structured description of the deviation.

## How Loop Bounds Work

`PowlAstNode::Loop { max_iters: u8 }` declares the maximum iteration count for a loop node. `max_iters = 0` means unlimited. The bound is enforced branchlessly in the scheduler using saturating arithmetic — no branch misprediction on the hot path.

## How XOR Boundaries Work

`XorChoice` inside a `Loop` body or redo branch is rejected at compile time with `CompileError::XorInsideLoop`. This prevents unbounded non-determinism inside loops and ensures the scheduler can produce a deterministic receipt.

## How Overflow Is Reported

`PetriTickResult.event_overflow_count` counts the number of events dropped in a single scheduler tick when the event buffer is full. `ReceiptWorker.overflow()` returns the cumulative overflow count across all ticks for a workflow execution.

## Known Exclusions

- Distributed persistence
- Human task UI
- Full BPMN import/export
- Camunda/Temporal feature parity
- Peer-reviewed SOTA
