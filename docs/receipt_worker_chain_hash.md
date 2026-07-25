# ReceiptWorker Cryptographic Chain Hashing

Based on a detailed analysis of the BCINR codebase, there are a few important corrections to make regarding the assumptions in the research request. Specifically, the `ReceiptWorker` does **not** use the `ChaChaSponge` for chain hashing, nor does it directly hash `OcelCausalFrame` objects. Instead, the architecture utilizes **BLAKE3** for cryptographic chaining across two distinct layers.

Here is the exact mechanism of how causal order of execution is mathematically proven.

## 1. The ReceiptWorker Chaining Process (BLAKE3, not ChaChaSponge)

The `ReceiptWorker` (located in `crates/bcinr-powl/src/receipt_worker.rs`) processes `EventWorkItem`s drained from the `LockFreeMpmcRing` and accumulates them by `run_id`. 

When a run is completed and deemed admissible, the worker seals a 57-byte receipt. The binding of the causal chain is performed using **BLAKE3**, not `ChaChaSponge`. The worker binds the `prev_chain_hash` to the new receipt data in `ReceiptWorker::build_entry` through a streaming BLAKE3 hash:

```rust
let mut h = blake3::Hasher::new();
h.update(&self.prev_chain_hash);           // 32 bytes: The causal link to the previous receipt
h.update(&run_id.to_le_bytes());           // 8 bytes (LE)
h.update(&op_trace.to_le_bytes());         // 8 bytes (LE)
h.update(&[topo_tag]);                     // 1 byte
let chain_hash = *h.finalize().as_bytes();
```

This ensures that every executed run is mathematically bound to the chronological order of all previously executed runs, creating an unbreakable causal chain. `BLAKE3` is executed off the hot path during the `drain()` budget window.

## 2. OcelCausalFrame Binding

`OcelCausalFrame`s are handled by a separate pipeline: the `OcelCausalReceipt` (located in `crates/bcinr-powl-receipt/src/causal_receipt.rs`). 

When an `OcelCausalFrame` is bumped/emitted from the `OcelEmitArena`, it is chained to mathematically prove the strict order of manufacturing steps. Similar to the `ReceiptWorker`, it utilizes a rolling **BLAKE3** hash:

1. **Canonical Serialization**: The 128-byte `OcelCausalFrame` is serialized into a strict 99-byte little-endian format (extracting fields like `instruction_id`, `fired_mask`, `denial`, `obj_refs`, etc., and omitting the internal padding).
2. **Rolling Hash**: The `OcelCausalReceipt::chain` method binds the new frame using a two-part fold:
   ```rust
   let mut h = blake3::Hasher::new();
   h.update(&self.chain_hash); // The prior hash
   h.update(&frame_bytes);     // The 99 canonical bytes
   self.chain_hash = *h.finalize().as_bytes();
   ```

*(Note: The generalized hash folding pattern is also implemented in `crates/bcinr-powl-receipt/src/chain.rs` as `fold(prior_hash, canonical_bytes)`).*

## 3. The Actual Role of ChaChaSponge

The `ChaChaSponge` (`crates/bcinr-logic/src/patterns/chacha_sponge.rs`) *is* a branchless ($CC=1$) cryptographic primitive implemented in the codebase (performing an 8-round permutation). However, its purpose is for deterministic entropy generation and cryptographic state mixing on the Substrate (as documented in `docs/deterministic_entropy_generation.md`), not for the `ReceiptWorker`'s chronological event logs or the OCEL frame chaining.

## Conclusion

The mathematical proof of causal execution order is guaranteed because every new state hash explicitly incorporates the output of the previous state hash as its first input. Modifying any historical event or reordering the execution trace would completely alter all subsequent hashes. This provides a completely deterministic and mathematically certifiable chain, built atop **BLAKE3**.
