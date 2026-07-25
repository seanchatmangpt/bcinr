# Cryptographic BLAKE3 Receipt Chains in BCINR

The BCINR codebase utilizes cryptographic BLAKE3 hash chains to establish deterministic, tamper-evident audit trails (receipts) of causal execution steps, compilation bounds, and scheduler decisions. These chains ensure execution integrity by proving that computations occurred exactly as claimed and were governed by the proper safety constraints.

## 1. Core Chain Mechanics
The fundamental building block for receipt chaining is defined in `crates/bcinr-powl-receipt/src/chain.rs`. It provides a shared `fold` operation that securely links the previous state to the new state using BLAKE3:

```text
chain_hash(t+1) = BLAKE3(chain_hash(t) || canonical_bytes(t+1))
```

This discipline is applied to various execution traces:

### A. Causal Execution (OcelCausalReceipt)
Defined in `causal_receipt.rs`, this receipt tracks ordered sequences of manufacturing steps (`OcelCausalFrame`). 
* The genesis hash initializes to the BLAKE3 hash of 32 zero bytes.
* Each frame serializes 99 bytes of state (including instruction IDs, denial polarities, and object references) alongside the `prior_hash`, allowing a rolling validation of causal history.

### B. Scheduler Execution (ExecutionReceipt)
Defined in `execution.rs`, the `ExecutionReceipt` attests to a single tick of the `bcinr_powl` scheduler, specifically which operations were admitted (`fired` set) and completed.
* **Guard Table Binding:** The receipt folds in a `guards_digest` (a structural hash of the `ConcurrencyGuardTable`). This guarantees that verification is performed against the exact same ruleset used during execution, eliminating vulnerabilities where an attacker could verify against a weaker, disjoint guard table.
* **Integrity Enforcement:** The `verify_execution_receipt` function guards against three fatal integrity violations:
  1. `HashMismatch`: The receipt was tampered with or hand-assembled.
  2. `InadmissibleFiredSet`: The executed set violated concurrency constraints.
  3. `GuardsMismatch`: The guard table supplied for verification differs from the one used to seal the receipt.

## 2. Receipt DAG Schema 
Beyond linear chains, BCINR explores extending receipts into a Merkle DAG (Detailed in `03_RECEIPT_DAG_SCHEMA.md`). 
A multi-parent receipt incorporates a sorted set of predecessor receipts, providing fan-in for parallel validation tasks (e.g., `R_n = BLAKE3(event_n, {R_p1, ..., R_pk}, outputs_n)`). This allows `generation`, `verification`, and `admission` receipts to securely roll up into a single terminal `ReleaseArtifact` digest.

## 3. Ensuring Integrity under The Radon Law (Branchless Execution)
Because BCINR is a deterministic computing substrate, verifying execution receipts must not introduce timing side-channels or heap allocations. The codebase proposes two critical innovations (`DBRH` and `ZA-BRVP`) to bring receipt verification into compliance with the BCINR Radon Law ($CC=1$, zero allocation):

### Direct Bitmask Receipt Hashing (DBRH)
Currently, serializing an `EventSet` requires traversing active bits—a variable-time loop that leaks data density via timing side-channels and allocates memory. **DBRH** proposes hashing the raw 64-byte `[u64; 8]` bitmask array directly. This removes all loops and conditional jumps, yielding a perfectly constant-time serialization process.

### Zero-Allocation Branchless Validation (ZA-BRVP)
Traditional validation uses short-circuiting conditional branches (e.g., `if hash != expected { return Err }`) and allocates vectors (`Vec<u8>`) to build canonical byte arrays. **ZA-BRVP** redesigns this by:
* **Streaming Hashes:** Pushing data chunks directly into `blake3::Hasher::update()` without intermediate heap allocations.
* **Bit-Parallel Evaluation:** Running all validation checks (hash matching, admissibility checks) using bitwise masks (e.g., `is_eq = (diff == 0) as u64; mask = 0u64.wrapping_sub(is_eq)`).
* **Branchless Priority Encoding:** Aggregating the masks into a single branchless status word. 

By eliminating early returns and variable loops, the verification pipeline guarantees a fixed execution duration independent of the receipt's validity, physically eliminating timing-based attacks on execution certificates.
