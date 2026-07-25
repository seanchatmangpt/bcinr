# Fixed-Width Receipt Architecture (Rule 11 & AcceptedOutcomeReceipt)

Based on the BCINR substrate's `AGENTS.md` and documentation, **Rule 11 (The ReceiptSound Law)** mandates a strict conjunctive gate for adaptive mutation. Persistent state can only mutate when five mathematical proofs evaluate true simultaneously, one of which is the `AcceptedOutcomeReceipt` (also called the Yield Proof). 

## Rule 11: The ReceiptSound Law
Adaptive mutation requires all of:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. **`AcceptedOutcomeReceipt`**: Cryptographically guarantees that any mutation to the adaptive state is backed by a verified telemetry event/outcome. It ensures the runtime cannot mutate persistent state based on speculative, untracked, or "unwitnessed" operations.
5. `CertifiedLearningMode`

Under the **Radon Law ($CC=1$)** and **Zero-Allocation Boundary**, the absence or invalidity of an `AcceptedOutcomeReceipt` mathematically evaluates the admission mask ($m_{\mathrm{admitted}}$) to `0`. This mechanically blocks state mutation via constant-time, bit-level selection ($x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t$), freezing learning while allowing deterministic selection to continue seamlessly.

## Fixed C-ABI Structure (No String Allocations)
In compliance with the `#![no_std]` and zero heap allocation laws, receipts like `AcceptedOutcomeReceipt` cannot use allocated `String` fields for data like `request_hash`, `timing_result`, or `selected_tool`. Based on identical architectures in the codebase (such as `OcelCausalFrame` and `CertifiedModeSwitch`), these properties are encoded into fixed-width C-ABI structures.

An `AcceptedOutcomeReceipt` or similar struct models its properties as follows:

- **`request_hash` (Fixed Digest)**: 
  Instead of a hex string, hashes are stored as fixed-size byte arrays or integer digests. For instance, `[u8; 32]` is used for BLAKE3 chain hashes (as seen in `OcelCausalReceipt`), or a `u64` digest (as seen in `CertifiedModeSwitch`'s `target_mode_digest`).

- **`timing_result` (Scalar Integers)**: 
  Execution timing is tracked as primitive integers rather than dynamic `Duration` or formatted time strings. For example, `OcelCausalFrame` uses a simple `ts_ns: u64` representing wall-clock time in nanoseconds.

- **`selected_tool` (Packed References & Indices)**: 
  String names for tools or activities are prohibited in the hot path. They are replaced by fixed-width identifiers or interned indices mapped via the slow rail. For instance:
  - `PackedObjRef(u32)`: A transparent `u32` wrapping a type index (high 8 bits) and object ID (low 24 bits).
  - `activity_idx: u16`: An index into an activity table.
  - `node_kind: u8`: A classifier byte representing the tool/node kind.

### Conceptual Fixed-Width Layout
An equivalent `AcceptedOutcomeReceipt` maps exactly to cache-line-friendly C-ABI memory layouts without pointers to the heap:

```rust
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct AcceptedOutcomeReceipt {
    /// Replaces an allocated string `request_hash`
    pub request_hash: [u8; 32],
    
    /// Replaces a formatted `timing_result` struct or string
    pub timing_result_ns: u64,
    
    /// Replaces `selected_tool: String`
    pub selected_tool_id: u16,
    
    /// The actual verified yield / outcome metric
    pub observed_yield: u64,
    
    /// Internal padding to maintain alignment
    pub pad: [u8; 14], 
}
```

This representation guarantees that the MAPE-K loop's Observe and Infer phases execute within strictly bounded $O(1)$ stack space and $O(1)$ time complexity, executing perfectly in alignment with the substrate's branchless and deterministic mandates.
