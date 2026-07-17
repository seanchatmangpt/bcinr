# DEEP TRACE D — RECEIPTS: SEMANTIC EDGE ARCHAEOLOGY

## 1. Trace Analysis: SchedulerDecision -> seal -> hash -> verify -> replay

### The Current Edge
1. **`tick_and_seal_execution_receipt`**:
   - Computes `ready_mask` via a dry-run `scheduler_tick(tape, &mut preview)`.
   - Computes `fired_set` via `scheduler_tick_guarded(tape, state, selector, guards)`.
   - Hashes `(tick, ready_mask, fired_mask)` into `scheduler_decision_digest`.
   - Calls `seal_execution_receipt`, passing `scheduler_decision_digest` opaquely, along with explicit `fired` and `completed_after` EventSets.

2. **`seal_execution_receipt`**:
   - Checks `guards.admits(&fired)`.
   - DOES NOT check if `fired` is a subset of `ready` because `ready` is not provided (only its digest).
   - Folds `scheduler_decision_digest` into the `canonical_bytes` hash.

3. **`verify_execution_receipt`**:
   - Takes `ExecutionReceipt` and `ConcurrencyGuardTable`.
   - Checks `guards.admits(&receipt.fired)`.
   - Recomputes `canonical_bytes` by blindly trusting `receipt.scheduler_decision_digest`.
   - DOES NOT verify `scheduler_decision_digest` against `fired` or `ready`, because `ready` is opaque.

4. **Replay**:
   - If a verifier has the `prior state` + `compiled model` (tape), they *can* reconstruct `ReadySet` by running `scheduler_tick`. They can then reconstruct `scheduler_decision_digest` and ensure it matches the receipt.
   - **Conclusion on Reconstruction**: `ReadySet` **CAN** be reconstructed from `receipt + compiled model + prior state`. 

### The Broken Edge
Although a *stateful* replay can reconstruct `ReadySet` and verify the `scheduler_decision_digest`, the receipt itself is structurally deficient for *stateless* validation. 
Because `ExecutionReceipt` commits to `ready_mask` only through a hash (`scheduler_decision_digest`), the fundamental scheduling invariant — `fired ⊆ ready` — is completely hidden from the receipt.
A malicious or faulty prover could hand-craft an `ExecutionReceipt` where `fired` contains ops that are not actually ready, as long as `guards.admits(&fired)` holds. The `verify_execution_receipt` function will happily validate this receipt because it cannot "look inside" the `scheduler_decision_digest` to check the subset property. 

## 2. Minimal Explicit Evidence & Migration

To repair the stateless verification boundary, the receipt must carry `ReadySet` explicitly, allowing `verify_execution_receipt` to assert `fired ⊆ ready` without requiring a full stateful replay.

**Minimal Explicit Evidence Field**:
```rust
// In ExecutionReceipt
pub ready: EventSet,
```

**Receipt-Version / Hash Migration**:
1. Add `pub ready: EventSet` to `ExecutionReceipt`.
2. Remove `pub scheduler_decision_digest: Digest` (it is redundant if `tick`, `ready`, and `fired` are explicit).
3. Update `canonical_bytes` to serialize `ready` instead of `scheduler_decision_digest`:
   ```rust
   push_event_set(&mut buf, ready);
   // replacing buf.extend_from_slice(scheduler_decision_digest.as_bytes());
   ```
4. Update `seal_execution_receipt` and `verify_execution_receipt` to include the check:
   ```rust
   if !fired.is_subset_of(&ready) {
       return Err(ExecutionIntegrityError::FiredNotSubsetOfReady { ready, fired });
   }
   ```

---

## 3. EDGE CARDS

### EDGE CARD: Opaque Scheduler Decision
- **Source Node**: `SchedulerDecision`
- **Target Node**: `ExecutionReceipt` / `seal_execution_receipt`
- **Fault Type**: Hidden Invariant / Opaque Commitment
- **Description**: `ExecutionReceipt` commits to `ready_mask` only through the opaque `scheduler_decision_digest`. This strips the semantic relationship between `ready` and `fired` from the receipt, preventing stateless verifiers (like `verify_execution_receipt`) from checking that `fired ⊆ ready`.
- **Proof of Vulnerability**: A hand-crafted receipt with a valid hash chain and a `fired` set that passes `guards.admits()` will be accepted by `verify_execution_receipt`, even if `fired` includes ops that were not ready. The verifier blindly includes the provided `scheduler_decision_digest` in the hash computation.
- **Resolution**: Make `ready: EventSet` an explicit field on `ExecutionReceipt`. Replace the opaque `scheduler_decision_digest` in the hash chain with the explicit serialization of `ready`. Enforce `fired.is_subset_of(&ready)` in both `seal` and `verify` phases.
