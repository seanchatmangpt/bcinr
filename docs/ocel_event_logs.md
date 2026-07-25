# OCEL 2.0 and Execution Tracing in BCINR

The `bcinr` branchless codebase leverages OCEL 2.0 (Object-Centric Event Logs) in combination with cryptographic BLAKE3 receipts to achieve deterministic, alloc-free execution tracing and verifiable conformance checking.

## 1. Object-Centric Event Logging (OCEL)
At the heart of the execution engine (`bcinr-powl`) lies a zero-allocation, fixed-capacity event log designed around the OCEL 2.0 specification. 

### Branchless SRBCG (Symmetric Run-Bounded Conformance Gating)
- The execution engine records workflow events using the `OcelLog` (located in `crates/bcinr-powl/src/ocel.rs`), which compactly stores up to 512 discrete `OcelEvent` structures per instance.
- To meet Radon Law requirements ($CC=1$, 0 heap allocations, branchless), run assignment utilizes **SRBCG**, a deterministic comparison network that tracks up to 64 concurrent unique run IDs securely without relying on conditional branching or loops with data-dependent termination.
- It records essential execution events such as `op_fired` and `run_sealed`.

### Conformance Validation
The `validate_against_tape` algorithm stringently verifies process conformance against a compiled `PowlTape` in a purely bitwise and mathematical manner:
- **Predecessor Constraints:** Validates that operations do not fire before their required dependencies (bitmask subset validation).
- **Seal Consistency:** Verifies that the accumulated `op_fired` operation bitmask trace exactly aligns with the final declared execution set at seal-time.
- **Duplicate Prevention:** Enforces idempotency by failing dynamically if an operation attempts to fire twice in a single run limit.

### OCEL 2.0 Export
When utilized alongside the standard library (`std`) or through `crates/bcinr-powl-receipt/src/pm_bridge.rs`, these execution traces natively serialize into the IEEE CPS 2023 **OCEL 2.0 JSON** format. It models artifacts like `PowlRun` and `PowlOp` as first-class objects linked accurately to operational events (`"ocel:events"` and `"ocel:objects"`), effectively bridging deterministic execution directly with standard process mining workflows.

## 2. BLAKE3 Cryptographic Causal Receipts
To mathematically assure execution integrity, run logs are cryptographically sealed into sequential BLAKE3 causal receipt chains. This provides an irrefutable proof that the system evaluated rules correctly, concurrently, and sequentially.

### Off-Hot-Path Hashing
- Hot-path execution strictly evaluates logic and rapidly pushes an `EventWorkItem` to a `LockFreeMpmcRing`, preserving latency.
- Cryptographic hashing is deferred completely to a separate `ReceiptWorker` (`crates/bcinr-powl/src/receipt_worker.rs`), draining the lock-free ring buffer within a bounded processing window.
- Critically, the `ReceiptWorker` evaluates occurrences against a `ConcurrencyGuardTable`. Runs exhibiting jointly inadmissible concurrency events are strictly refused sealing, guaranteeing causal integrity.

### Rolling Chains & `OcelCausalFrame`
The `bcinr-powl-receipt` crate codifies an `OcelCausalFrame`—an exactly 128-byte (aligned to 64-byte cache boundaries) robust structure containing:
- Execution and node identifiers (`instruction_id`, `node_kind`)
- Execution outcome parameters (`denial` via `DenialPolarity`)
- Packed context indices (`obj_refs` modeling types and IDs)
- Wall-clock timestamp metrics 

These sequences forge the `OcelCausalReceipt` representing the invariant rolling chain:
```text
chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))
```
Every receipt entry explicitly encodes the hashed predecessor, sealing in topological ordering and verifying chronological order flawlessly into a 57-byte portable log entry.

## Summary
By harmonizing strict process conformance (via OCEL 2.0 logic bounds) and cryptographic integrity (via branchless BLAKE3 hashing), the `bcinr` deterministic substrate certifies an unconditionally provable history while satisfying stringent zero-allocation, branchless execution constraints.
